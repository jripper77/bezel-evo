import { test } from 'node:test';
import assert from 'node:assert/strict';
import { blurMask, drawTextShadow } from '../../src/text-shadow.js';

test('the demo and native filter share transparent-edge rounding', () => {
  assert.deepEqual([...blurMask([0,0,0,0,255,0,0,0,0], 3, 3, 1)], Array(9).fill(28));
  assert.deepEqual([...blurMask([255], 1, 1, 1)], [28]);
  assert.deepEqual([...blurMask([12,255,0,60], 2, 2, 0)], [12,255,0,60]);
  // A larger, asymmetric input compared to a deliberately simple reference.
  const data = Uint8Array.from({ length: 35 }, (_, i) => i * 37 % 256);
  const pass = (source, axis, r) => Uint8Array.from(source, (_, i) => {
    const x = i % 7, y = Math.floor(i / 7);
    let sum = 0;
    for (let offset = -r; offset <= r; offset++) {
      const xx = x + (axis === 'x' ? offset : 0), yy = y + (axis === 'y' ? offset : 0);
      if (xx >= 0 && xx < 7 && yy >= 0 && yy < 5) sum += source[yy * 7 + xx];
    }
    return Math.floor(sum / (r * 2 + 1));
  });
  for (const r of [0,1,3,12,64]) assert.deepEqual(blurMask(data, 7, 5, r), pass(pass(data, 'x', r), 'y', r));
});

test('a disabled shadow performs no canvas work or extra drawing', () => {
  const ctx = new Proxy({}, { get() { assert.fail('no canvas access'); } });
  drawTextShadow(ctx, {}, { width: 20, height: 20 }, () => assert.fail('no extra drawing'));
  drawTextShadow(ctx, { shadow: {} }, { width: 0, height: 20 }, () => assert.fail('no extra drawing'));
});
