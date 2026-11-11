import { test } from 'node:test';
import assert from 'node:assert/strict';
import { PEAK_LIMIT, PREVIEW_MAX_SCALE, PREVIEW_MAX_SIDE, createPeakWatch, createPreviewScheduler, fitPreview, previewOffset, previewSide } from '../../src/element-preview.js';

test('visible preview content is centred without changing pixels or frame dimensions', () => {
  const rgba = new Uint8ClampedArray(10 * 6 * 4);
  rgba[(1 * 10) * 4 + 3] = 255;
  rgba[(3 * 10 + 2) * 4 + 3] = 1;
  // Invisible RGB data must not affect the visible bounds.
  rgba[(5 * 10 + 9) * 4] = 255;
  const before = rgba.slice();
  const frame = { width: 10, height: 6, rgba };
  assert.deepEqual(previewOffset(frame), { x: 3.5, y: 0.5 });
  assert.deepEqual(rgba, before);
  assert.equal(frame.width, 10); assert.equal(frame.height, 6);
});

test('centred, blank and missing previews need no translation', () => {
  assert.deepEqual(previewOffset(null), { x: 0, y: 0 });
  assert.deepEqual(previewOffset({ width: 0, height: 0, pixels: [] }), { x: 0, y: 0 });
  const pixels = new Uint8ClampedArray(4 * 4 * 4);
  assert.deepEqual(previewOffset({ width: 4, height: 4, pixels }), { x: 0, y: 0 });
  pixels[(1 * 4 + 1) * 4 + 3] = 255;
  pixels[(2 * 4 + 2) * 4 + 3] = 255;
  assert.deepEqual(previewOffset({ width: 4, height: 4, pixels }), { x: 0, y: 0 });
});

test('the preview fits the box, keeps its shape and enlarges at most four times', () => {
  assert.equal(PREVIEW_MAX_SCALE, 4);
  assert.deepEqual(fitPreview({ width: 400, height: 200 }, { width: 260, height: 120 }), { width: 240, height: 120 });
  assert.deepEqual(fitPreview({ width: 20, height: 10 }, { width: 260, height: 120 }), { width: 80, height: 40 });
  // Device pixels: a frame asked at 2x is drawn at half its pixel size.
  assert.deepEqual(fitPreview({ width: 40, height: 20 }, { width: 260, height: 120 }, 2), { width: 80, height: 40 });
  assert.deepEqual(fitPreview({ width: 1000, height: 1 }, { width: 100, height: 100 }), { width: 100, height: 1 });
});

test('nothing is drawn without a frame, with an empty frame or an empty box', () => {
  assert.equal(fitPreview(null, { width: 10, height: 10 }), null);
  assert.equal(fitPreview({ width: 0, height: 10 }, { width: 10, height: 10 }), null);
  assert.equal(fitPreview({ width: 10, height: 10 }, { width: 0, height: 10 }), null);
});

test('the side asked for follows the box and the device ratio, within bounds', () => {
  assert.equal(PREVIEW_MAX_SIDE, 512);
  assert.equal(previewSide({ width: 260, height: 120 }), 260);
  assert.equal(previewSide({ width: 260, height: 120 }, 1.5), 390);
  assert.equal(previewSide({ width: 400, height: 120 }, 2), 512);
  assert.equal(previewSide({ width: 2, height: 2 }), 16);
  assert.equal(previewSide({ width: Number.NaN, height: Number.NaN }), 512);
});

/** A render the test answers by hand. */
function manual() {
  const calls = [];
  const render = (key) => new Promise((resolve, reject) => calls.push({ key, resolve, reject }));
  return { calls, render };
}
const tick = () => new Promise((r) => setTimeout(r, 0));

test('requests while a render runs are coalesced into one, the last one', async () => {
  const { calls, render } = manual();
  const shown = [];
  const scheduler = createPreviewScheduler({ render, show: (frame, key) => shown.push([frame, key]) });
  scheduler.request(1);
  assert.equal(scheduler.busy(), true);
  await tick();
  scheduler.request(2);
  scheduler.request(3);
  scheduler.request(4);
  assert.equal(calls.length, 1);
  calls[0].resolve('a');
  await tick();
  // The first answer was overtaken: it is dropped, and only the last request runs.
  assert.deepEqual(shown, []);
  assert.deepEqual(calls.map((c) => c.key), [1, 4]);
  calls[1].resolve('d');
  await tick();
  assert.deepEqual(shown, [['d', 4]]);
  assert.equal(scheduler.busy(), false);
});

test('a burst of twenty requests asks for two renders at most', async () => {
  let renders = 0;
  const shown = [];
  const scheduler = createPreviewScheduler({ render: async (key) => { renders += 1; await tick(); return key; }, show: (frame) => shown.push(frame) });
  for (let i = 0; i < 20; i += 1) scheduler.request(i);
  for (let i = 0; i < 5; i += 1) await tick();
  assert.equal(renders, 1);
  assert.deepEqual(shown, [19]);
});

test('a failed render is told, unless a later request overtook it', async () => {
  const { calls, render } = manual();
  const failed = [];
  const scheduler = createPreviewScheduler({ render, show: () => {}, fail: (error, key) => failed.push([error, key]) });
  scheduler.request(1);
  await tick();
  calls[0].reject('boom');
  await tick();
  assert.deepEqual(failed, [['boom', 1]]);
  scheduler.request(2);
  await tick();
  scheduler.request(3);
  calls[1].reject('late');
  await tick();
  assert.deepEqual(failed, [['boom', 1]]);
  calls[2].reject('last');
  await tick();
  assert.deepEqual(failed, [['boom', 1], ['last', 3]]);
  // The default failure handler says nothing.
  const quiet = createPreviewScheduler({ render: () => Promise.reject(new Error('x')), show: () => {} });
  quiet.request(1);
  await tick();
  assert.equal(quiet.busy(), false);
});

test('cancel drops the running answer; a new request is shown', async () => {
  const { calls, render } = manual();
  const shown = [];
  const scheduler = createPreviewScheduler({ render, show: (frame) => shown.push(frame) });
  scheduler.request(1);
  await tick();
  scheduler.cancel();
  calls[0].resolve('stale');
  await tick();
  assert.deepEqual(shown, []);
  scheduler.request(2);
  await tick();
  calls[1].resolve('fresh');
  await tick();
  assert.deepEqual(shown, ['fresh']);
});

test('cancel before the scheduled render starts performs no backend work', async () => {
  const { calls, render } = manual();
  const scheduler = createPreviewScheduler({ render, show: () => assert.fail('cancelled frame') });
  scheduler.request(1);
  scheduler.cancel();
  await tick();
  assert.equal(calls.length, 0);
  assert.equal(scheduler.busy(), false);
});

test('a new selection during cancellation waits for the running render and drops its answer', async () => {
  const { calls, render } = manual();
  const shown = [];
  const scheduler = createPreviewScheduler({ render, show: (frame, key) => shown.push([frame, key]) });
  scheduler.request(1);
  await tick();
  scheduler.request(2);
  scheduler.cancel();
  scheduler.request(3);
  assert.equal(calls.length, 1);
  calls[0].resolve('old');
  await tick();
  assert.deepEqual(calls.map((call) => call.key), [1, 3]);
  assert.deepEqual(shown, []);
  calls[1].resolve('new');
  await tick();
  assert.deepEqual(shown, [['new', 3]]);
});

test('the peak is the highest reading of the sensor within the window', () => {
  const watch = createPeakWatch();
  watch.observe('net.down', { value: 3, display: '3 MB/s' }, 1000);
  watch.observe('net.down', { value: 9, display: '9 MB/s' }, 2000);
  watch.observe('net.down', { value: 4, display: '4 MB/s' }, 3000);
  assert.deepEqual(watch.peak('net.down', 10_000, 3000), { at: 2000, value: 9, display: '9 MB/s' });
  assert.deepEqual(watch.peak('net.down', 999, 3000), { at: 3000, value: 4, display: '4 MB/s' });
  assert.equal(watch.peak('cpu.usage', 10_000, 3000), null);
});

test('the range of one or two sensors spans their lowest and highest readings in the window', () => {
  const watch = createPeakWatch();
  watch.observe('down', { value: 40, display: '40' }, 1000);
  watch.observe('up', { value: 2, display: '2' }, 1000);
  watch.observe('down', { value: 10, display: '10' }, 2000);
  watch.observe('up', { value: 90, display: '90' }, 2000);
  assert.deepEqual([watch.range(['down'], 10_000, 2000).min.value, watch.range(['down'], 10_000, 2000).max.value], [10, 40]);
  assert.deepEqual([watch.range(['down', 'up'], 10_000, 2000).min.value, watch.range(['down', 'up'], 10_000, 2000).max.value], [2, 90]);
  assert.equal(watch.range(['down'], 500, 2000).max.value, 10);
  assert.equal(watch.range(['nothing'], 10_000, 2000), null);
});

test('the same sample twice, unavailable readings and other sensors do not count', () => {
  const watch = createPeakWatch();
  watch.observe('a', { value: 5 }, 10);
  watch.observe('a', { value: 50 }, 10);
  watch.observe('a', { unavailable: 'no', display: '—' }, 20);
  watch.observe('a', null, 30);
  watch.observe('a', { value: Number.NaN }, 40);
  assert.deepEqual(watch.peak('a', 100, 50), { at: 10, value: 5, display: '5' });
  // Two sensors at once (a mirrored graph); a third forgets the least recent.
  watch.observe('b', { value: 1, display: '1' }, 60);
  assert.equal(watch.peak('a', 100, 60).value, 5);
  assert.equal(watch.peak('b', 100, 60).value, 1);
  watch.observe('c', { value: 2, display: '2' }, 70);
  assert.equal(watch.peak('a', 100, 70), null);
  assert.equal(watch.peak('b', 100, 70).value, 1);
});

test('the peak keeps at most its limit of readings', () => {
  assert.equal(PEAK_LIMIT, 3600);
  const watch = createPeakWatch(3);
  for (const [at, value] of [[1, 100], [2, 1], [3, 2], [4, 3]]) watch.observe('k', { value, display: String(value) }, at);
  assert.equal(watch.peak('k', 1000, 4).value, 3);
});
