import { test } from 'node:test';
import assert from 'node:assert/strict';
import { fadeLine, fadeHandlePatch } from '../../src/shape-fade.js';
test('linear transparency spans non-square boxes at horizontal, vertical and diagonal angles', () => {
  const f = { x: 10, y: 20, width: 80, height: 40 };
  const close = (actual, expected) => actual.forEach((x, i) => assert.ok(Math.abs(x - expected[i]) < 1e-8));
  close(fadeLine(f, 0), [10, 40, 90, 40]);
  close(fadeLine(f, 90), [50, 20, 50, 60]);
  close(fadeLine(f, 180), [90, 40, 10, 40]);
  const d = fadeLine(f, 45);
  close([(d[0] + d[2]) / 2, (d[1] + d[3]) / 2], [50, 40]);
});

test('fade handles turn linear fades and clamp radial centres without changing their extent', () => {
  assert.deepEqual(fadeHandlePatch({}, { x: 1, y: .5 }), { angle: 0 });
  assert.deepEqual(fadeHandlePatch({}, { x: .5, y: 1 }), { angle: 90 });
  assert.deepEqual(fadeHandlePatch({}, { x: 0, y: .5 }), { angle: 180 });
  assert.deepEqual(fadeHandlePatch({}, { x: .5, y: 0 }), { angle: -90 });
  assert.deepEqual(fadeHandlePatch({ radial: [.5, .5, 1.4] }, { x: -.2, y: 1.2 }), { radial: [0, 1, 1.4] });
  assert.deepEqual(fadeHandlePatch({}, { x: .5, y: .5 }), {});
  assert.deepEqual(fadeHandlePatch({}, { x: NaN, y: 0 }), {});
});
