import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  TRANSITION_DEFAULT_MS, TRANSITION_RANGE, TRANSITION_STEP_MS, TRANSITION_MODES, defaultTransition, demoSampleDue, demoTransitionDue,
  ease, easedDemoValue, easesValue, formatMs, transitionLength, transitionOf, transitionPatch, inertiaAt,
} from '../../src/value-transition.js';
import { createWidget, WIDGETS } from '../../src/editor/widgets.js';
import { commands } from '../../src/editor/store.js';
import { createDemoBackend } from '../../src/demo-backend.js';

const box = { x: 0, y: 0, width: 100, height: 20 };
const binding = { key: 'cpu.usage', min: 0, max: 100 };
const bar = (valueTransition) => ({ id: 1, name: 'b', frame: box, opacity: 1, visible: true, locked: false, ...(valueTransition ? { valueTransition } : {}), kind: { type: 'bar', binding, direction: 'leftToRight', fill: '#ffffffff' } });
const smooth = (durationMs = 400) => ({ mode: 'smooth', durationMs });

test('objects that show a value ease; others do not', () => {
  assert.ok(easesValue(bar()));
  assert.ok(easesValue({ kind: { type: 'ring', binding } }));
  assert.ok(easesValue({ kind: { type: 'needle', binding } }));
  assert.ok(easesValue({ kind: { type: 'text', content: { type: 'sensor', key: 'cpu.usage' } } }));
  assert.ok(!easesValue({ kind: { type: 'text', content: { type: 'static', text: 'x' } } }));
  assert.ok(!easesValue({ kind: { type: 'graph', binding } }), 'a graph draws its history');
  assert.ok(!easesValue({ isGroup: true, kind: { type: 'shape' } }));
  assert.ok(!easesValue({ card: {}, kind: { type: 'shape' } }));
  assert.ok(!easesValue(null));
});

test('a theme without a transition reads as none and keeps the default duration', () => {
  assert.deepEqual(transitionOf(bar()), { mode: 'none', durationMs: TRANSITION_DEFAULT_MS });
  assert.deepEqual(transitionOf(bar(smooth(700))), { mode: 'smooth', durationMs: 700 });
  assert.deepEqual(transitionOf(bar({ mode: 'none', durationMs: 900 })), { mode: 'none', durationMs: 900 });
  assert.deepEqual(transitionOf(bar({ mode: 'bounce', durationMs: -3 })), { mode: 'none', durationMs: TRANSITION_DEFAULT_MS });
  assert.equal(transitionLength(bar()), null);
  assert.equal(transitionLength(bar({ mode: 'none', durationMs: 400 })), null);
  assert.equal(transitionLength(bar(smooth(0))), null);
  assert.equal(transitionLength(bar(smooth(400))), 400);
  assert.equal(transitionLength(bar(smooth(10))), 50, 'the backend bounds');
  assert.equal(transitionLength(bar(smooth(60_000))), 5000);
});

test('a patch always carries both fields within the inspector range', () => {
  assert.deepEqual(transitionPatch(bar(), { mode: 'smooth' }), { mode: 'smooth', durationMs: TRANSITION_DEFAULT_MS });
  assert.deepEqual(transitionPatch(bar(smooth(600)), { mode: 'none' }), { mode: 'none', durationMs: 600 }, 'kept for later');
  assert.deepEqual(transitionPatch(bar(smooth()), { durationMs: 99_999 }), { mode: 'smooth', durationMs: TRANSITION_RANGE.max });
  assert.deepEqual(transitionPatch(bar(smooth()), { durationMs: 1 }), { mode: 'smooth', durationMs: TRANSITION_RANGE.min });
  assert.deepEqual(transitionPatch(bar(smooth()), { durationMs: 412.6 }), { mode: 'smooth', durationMs: 413 });
  assert.deepEqual(transitionPatch(bar(smooth()), { durationMs: 'x' }), { mode: 'smooth', durationMs: TRANSITION_DEFAULT_MS });
  assert.deepEqual(transitionPatch(bar(smooth()), { mode: 'bounce' }), { mode: 'none', durationMs: 400 });
  assert.deepEqual(TRANSITION_MODES, ['none', 'smooth', 'inertia', 'count', 'roll']);
  assert.equal(formatMs(400), '400 ms');
});

test('the easing is the backend\'s ease-out cubic', () => {
  assert.equal(ease(0), 0);
  assert.equal(ease(1), 1);
  assert.equal(ease(0.5), 0.875);
  assert.equal(ease(-1), 0);
  assert.equal(ease(3), 1);
  assert.equal(ease(Number.NaN), 1);
});

test('inertia keeps momentum on retarget and stores a compatible optional flag',()=>{
  const setting=transitionPatch(bar(),{mode:'inertia'});
  assert.deepEqual(setting,{mode:'smooth',durationMs:400,inertia:true});
  assert.equal(transitionOf(bar(setting)).mode,'inertia');
  assert.equal(transitionLength(bar(setting)),400);
  assert.deepEqual(transitionPatch(bar(setting),{mode:'smooth'}),{mode:'smooth',durationMs:400,inertia:false});
  const initial=inertiaAt(0,100,0,0,.8);
  assert.deepEqual(initial,{value:0,velocity:0});
  const before=inertiaAt(0,100,0,.1,.8);
  assert.ok(before.value>0 && before.value<33.1,'starts more gently than the cubic response');
  const retarget=inertiaAt(before.value,20,before.velocity,0,.8);
  assert.ok(Math.abs(retarget.value-before.value)<1e-9);
  assert.equal(retarget.velocity,before.velocity);
  assert.deepEqual(inertiaAt(before.value,20,before.velocity,.8,.8),{value:20,velocity:0});
  const e=bar({mode:'smooth',durationMs:800,inertia:true});
  const signal=t=>t<1?0:100;
  assert.ok(easedDemoValue(e,signal,1.1,1)>0);
  assert.equal(easedDemoValue(e,signal,1.8,1),100);
  assert.equal(easedDemoValue(e,signal,1.1,1,false),100);
});

test('objects made in the Studio ease for 400 ms; the rest have no transition', () => {
  const canvas = { width: 480, height: 1920 };
  for (const widget of WIDGETS) {
    const made = createWidget(widget, canvas);
    if (['value', 'bar', 'ring', 'needle'].includes(widget)) assert.deepEqual(made.valueTransition, { mode: 'smooth', durationMs: 400 }, widget);
    else assert.equal(made.valueTransition, undefined, widget);
  }
  assert.notEqual(createWidget('ring', canvas).valueTransition, createWidget('ring', canvas).valueTransition, 'a fresh object each');
  assert.deepEqual(defaultTransition(), { mode: 'smooth', durationMs: 400 });
  const theme = { canvas, elements: [] };
  const added = commands.add(theme, { widget: 'ring', x: 100, y: 100 }).theme.elements[0];
  assert.deepEqual(added.valueTransition, { mode: 'smooth', durationMs: 400 });
  const text = commands.add(theme, { widget: 'text', x: 100, y: 100 }).theme.elements[0];
  assert.equal('valueTransition' in text, false);
  const edited = commands.update({ ...theme, elements: [added] }, { id: added.id, patch: { valueTransition: transitionPatch(added, { mode: 'none' }) } });
  assert.deepEqual(edited.theme.elements[0].valueTransition, { mode: 'none', durationMs: 400 });
});

test('the demo eases from the previous sample to the last one, like the backend', () => {
  const reading = (s) => s; // the sample at s seconds reads s
  const eased = bar(smooth(400));
  assert.equal(easedDemoValue(eased, reading, 10.0, 1), 9, 'from the sample before');
  const near = (a, b) => assert.ok(Math.abs(a - b) < 1e-9, `${a} ≈ ${b}`);
  near(easedDemoValue(eased, reading, 10.2, 1), 9 + ease(0.5));
  assert.equal(easedDemoValue(eased, reading, 10.4, 1), 10, 'landed');
  assert.equal(easedDemoValue(eased, reading, 10.9, 1), 10);
  assert.equal(easedDemoValue(bar(), reading, 10.2, 1), 10, 'no transition: the last sample');
  assert.equal(easedDemoValue(eased, reading, 10.2, 1, false), 10, 'reduced motion: the last sample');
  near(easedDemoValue(eased, reading, 10.2, 0), 9 + ease(0.5));
});

test('the demo asks for a frame every 50 ms while a shown value moves, none after', () => {
  const elements = [bar(smooth(400)), { ...bar(smooth(1000)), id: 2, visible: false }];
  const shown = (e) => e.visible;
  assert.equal(demoTransitionDue(elements, 10.0, 1, { shown }), TRANSITION_STEP_MS);
  assert.equal(demoTransitionDue(elements, 10.38, 1, { shown }), 20, 'until it lands');
  assert.equal(demoTransitionDue(elements, 10.4, 1, { shown }), null, 'settled: no frames');
  assert.equal(demoTransitionDue(elements, 10.5, 1), TRANSITION_STEP_MS, 'the hidden one counts when shown');
  assert.equal(demoTransitionDue(elements, 10.1, 1, { motion: false }), null, 'reduced motion');
  assert.equal(demoTransitionDue([bar()], 10.1, 1), null, 'no transition');
  assert.equal(demoTransitionDue([bar(smooth(400))], 10.1, 0), TRANSITION_STEP_MS);
  assert.equal(demoTransitionDue([bar(smooth(400))], 10.3999, 1), 1, 'at least 1 ms');
});

test('a demo preview that eases wakes at the next sample, where a transition starts', () => {
  const shown = (e) => e.visible;
  assert.equal(demoSampleDue([bar(smooth(400))], 10.4, 1), 600);
  assert.equal(demoSampleDue([bar(smooth(400))], 10.4, 0), 600, 'a refresh of 0 reads as 1 s');
  assert.equal(demoSampleDue([bar(smooth(400))], 12.5, 5), 2500);
  assert.equal(demoSampleDue([bar(smooth(400))], 10.9999999, 1), 1, 'at least 1 ms');
  assert.equal(demoSampleDue([bar()], 10.4, 1), null, 'nothing eases');
  assert.equal(demoSampleDue([{ ...bar(smooth(400)), visible: false }], 10.4, 1, { shown }), null, 'hidden');
  assert.equal(demoSampleDue([bar(smooth(400))], 10.4, 1, { motion: false }), null, 'reduced motion');
});

/** A stand-in for the browser's OffscreenCanvas that draws nothing. */
class BlankCanvas {
  constructor(width, height) { Object.assign(this, { width, height }); }
  getContext() {
    const { width, height } = this;
    const ctx = { getImageData: () => ({ data: new Uint8ClampedArray(width * height * 4) }) };
    return new Proxy(ctx, { get: (t, k) => (k in t ? t[k] : () => {}) });
  }
}

test('the demo backend times its frames and counts those of moving values (fps measure)', async (t) => {
  const saved = globalThis.OffscreenCanvas;
  globalThis.OffscreenCanvas = BlankCanvas;
  t.after(() => { globalThis.OffscreenCanvas = saved; });
  let ms = 1_000_000;
  const counts = [];
  const demo = createDemoBackend('turing88', { now: () => ms / 1000 }, { onTransitionFrame: (n) => counts.push(n) });
  const { theme } = await demo.session();
  const eased = { ...theme, refreshSeconds: 1, elements: [bar(smooth(400))] };
  // Follow the frames the backend asks for over one second, as the preview does.
  const drawn = [];
  while (ms < 1_001_000) {
    const frame = await demo.render(eased);
    drawn.push(ms);
    ms = frame.nextMs === null ? 1_001_000 : ms + frame.nextMs;
  }
  const moving = drawn.filter((at) => at < 1_000_400);
  const fps = moving.length / 0.4;
  assert.equal(fps, 1000 / TRANSITION_STEP_MS, `${fps} frames/s while easing`);
  assert.equal(counts.at(-1), moving.length, 'each moving frame counted');
  assert.equal(drawn.length, moving.length + 1, 'one landing frame, then the next sample');

  ms = 1_000_100;
  assert.equal((await demo.render(eased, { motion: false })).nextMs, null, 'reduced motion: no frames');
  assert.equal((await demo.render({ ...eased, elements: [bar()] })).nextMs, null, 'no transition: no frames');
  const alone = await demo.renderElement(eased, 1, { max: 64 });
  assert.equal(alone.nextMs, TRANSITION_STEP_MS, 'the element preview moves too');
  assert.equal((await demo.renderElement(eased, 1, { max: 64, motion: false })).nextMs, null);
  ms = 1_000_500;
  assert.equal((await demo.render(eased)).nextMs, 500, 'settled: the next sample');
});

test('count and roll are number transitions with a length; the average is kept only above 1', async () => {
  const { AVERAGE_RANGE, averageOf } = await import('../../src/value-transition.js');
  for (const mode of ['count', 'roll']) {
    assert.deepEqual(transitionOf(bar({ mode, durationMs: 300 })), { mode, durationMs: 300 });
    assert.equal(transitionLength(bar({ mode, durationMs: 300 })), 300);
    assert.deepEqual(transitionPatch(bar(), { mode }), { mode, durationMs: TRANSITION_DEFAULT_MS });
  }
  assert.ok(TRANSITION_MODES.includes('count') && TRANSITION_MODES.includes('roll'));
  assert.equal(averageOf(bar()), 1);
  assert.equal(averageOf(bar({ mode: 'none', durationMs: 400, average: 3 })), 3);
  assert.equal(averageOf(bar({ mode: 'none', durationMs: 400, average: 99 })), AVERAGE_RANGE.max);
  assert.deepEqual(transitionPatch(bar(), { average: 3 }), { mode: 'none', durationMs: TRANSITION_DEFAULT_MS, average: 3 });
  assert.deepEqual(transitionPatch(bar({ mode: 'count', durationMs: 400, average: 3 }), { durationMs: 500 }), { mode: 'count', durationMs: 500, average: 3 }, 'kept');
  assert.deepEqual(transitionPatch(bar({ mode: 'count', durationMs: 400, average: 3 }), { average: 1 }), { mode: 'count', durationMs: 400 }, '1 is omitted');
  assert.deepEqual(transitionPatch(bar(), { average: 'x' }), { mode: 'none', durationMs: TRANSITION_DEFAULT_MS });
});

test('the demo counts evenly, averages its samples and rolls like the backend', async () => {
  const { demoRoll, rollWheels } = await import('../../src/value-transition.js');
  const step = (s) => (s < 10 ? 10 : 50);
  const counting = bar({ mode: 'count', durationMs: 400 });
  assert.equal(easedDemoValue(counting, step, 10.2, 1), 30, 'half way, evenly');
  assert.equal(easedDemoValue(bar(smooth(400)), step, 10.2, 1), 45, 'eased');
  const averaged = bar({ mode: 'none', durationMs: 400, average: 3 });
  const seq = (s) => [30, 60, 90, 0][Math.floor(s)] ?? NaN;
  assert.equal(easedDemoValue(averaged, seq, 2.5, 1), 60);
  assert.equal(easedDemoValue(averaged, seq, 3.5, 1), 50, 'the oldest drops');
  assert.equal(easedDemoValue(averaged, seq, 0.5, 1), 30, 'fewer readings at the start');
  assert.ok(Number.isNaN(easedDemoValue(averaged, () => NaN, 1, 1)));

  const rolling = bar({ mode: 'roll', durationMs: 400 });
  assert.deepEqual(demoRoll(rolling, step, 10.2, 1), { from: 10, to: 50, progress: 0.875 });
  assert.equal(demoRoll(rolling, step, 10.5, 1), null, 'landed');
  assert.equal(demoRoll(rolling, step, 10.2, 1, false), null, 'reduced motion');
  assert.equal(demoRoll(bar(smooth(400)), step, 10.2, 1), null);
  assert.equal(demoRoll(bar({ mode: 'roll', durationMs: 0 }), step, 10.2, 1), null);
  assert.equal(demoRoll(rolling, (s) => 5, 10.2, 1), null, 'nothing changes');

  assert.deepEqual(rollWheels({ from: 36, to: 41, progress: 0 }, 0, 2), [[3, 0], [6, 0]]);
  assert.deepEqual(rollWheels({ from: 36, to: 41, progress: 1 }, 0, 2), [[4, 0], [1, 0]]);
  assert.deepEqual(rollWheels({ from: 36, to: 41, progress: 0.7 }, 0, 2), [[3, 0.5], [9, 0.5]], '9 carries');
  assert.deepEqual(rollWheels({ from: 36, to: 41, progress: 0.2 }, 0, 2)[0], [3, 0]);
  assert.deepEqual(rollWheels({ from: 4.7, to: 4.72, progress: 0.5 }, 2, 3), [[4, 0], [7, 0], [1, 0]]);
  assert.deepEqual(rollWheels({ from: -5, to: -7, progress: 1 }, 0, 1), [[7, 0]]);
  assert.deepEqual(rollWheels({ from: NaN, to: 3, progress: 1 }, 0, 1), [[3, 0]]);
  assert.deepEqual(rollWheels({ from: 99, to: 100, progress: 0 }, 0, 3), [[0, 0], [9, 0], [9, 0]]);
});
