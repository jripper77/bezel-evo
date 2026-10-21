import test from 'node:test';
import assert from 'node:assert/strict';
import { renderApprox } from '../../src/demo-render.js';
import { createWidget } from '../../src/editor/widgets.js';

// Record actual canvas strokes: geometry is compared to native's 64-unit dial.
test('demo-render uses native hand lengths, colours, widths and an unchanged seconds tail', t => {
  let strokes = [], path = [];
  const saved = globalThis.OffscreenCanvas;
  globalThis.OffscreenCanvas = class {
    constructor(width, height) { Object.assign(this, { width, height }); }
    getContext() {
      const ctx = { beginPath() { path = []; }, moveTo(...p) { path.push(p); }, lineTo(...p) { path.push(p); },
        stroke() { strokes.push({ points: path.map(p => [...p]), width: ctx.lineWidth, color: ctx.strokeStyle }); },
        getImageData: () => ({ data: new Uint8ClampedArray(this.width * this.height * 4) }) };
      return new Proxy(ctx, { get: (target, key) => key in target ? target[key] : () => {} });
    }
  };
  t.after(() => { globalThis.OffscreenCanvas = saved; });
  const canvas = { width: 256, height: 256 }, clock = createWidget('clock', canvas);
  clock.frame = { x: 0, y: 0, ...canvas };
  const render = analog => {
    strokes = []; clock.kind.content.analog = { ticks: 'none', border: 0, ...analog };
    renderApprox({ canvas, background: { type: 'color', color: '#000000ff' }, elements: [clock] }, new Date(2026, 9, 21, 3, 0, 0).getTime() / 1000);
    return structuredClone(strokes);
  };
  const original = render({}); assert.equal(original.length, 3);
  const near = (a, b) => assert.ok(Math.abs(a - b) < 1e-6, `${a} != ${b}`);
  for (const [index, length] of [11.7, 20.9, 23].entries()) {
    const tip = original[index].points[1]; near(Math.hypot(tip[0] - 128, tip[1] - 128), length * 4);
    const hand = ['hour', 'minute', 'second'][index], shorter = render({ [`${hand}Length`]: .5 });
    near(Math.hypot(shorter[index].points[1][0] - 128, shorter[index].points[1][1] - 128), length * 2);
    assert.deepEqual(shorter[index].points[0], original[index].points[0]);
    assert.equal(shorter[index].width, original[index].width);
    assert.equal(shorter[index].color, original[index].color);
    for (let other = 0; other < 3; other++) if (other !== index) assert.deepEqual(shorter[other], original[other]);
    assert.deepEqual(render({ [`${hand}Length`]: 1 }), original, 'explicit original lengths draw identical strokes');
  }
});
