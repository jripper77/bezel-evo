import { test } from 'node:test';
import assert from 'node:assert/strict';
import { gaugeBlocks, barSpan } from '../../src/gauge-layout.js';
test('native blocks have gaps between bars and a closing gap on full rings', () => {
  assert.deepEqual(gaugeBlocks(64, { count: 4, gap: 4 }), [[0,13],[17,30],[34,47],[51,64]]);
  assert.deepEqual(gaugeBlocks(360, { count: 4, gap: 10 }, true).at(-1), [270,350]);
  assert.deepEqual(gaugeBlocks(64, { count: 10, gap: 20 }), [[0,64]]);
  assert.deepEqual(gaugeBlocks(8, { count: 0, gap: NaN }), [[0,8]]);
  assert.deepEqual(gaugeBlocks(10), [[0,10]]);
});
test('each bar direction fills from its own edge', () => {
  const frame = { x: 10, y: 20, width: 100, height: 50 };
  assert.deepEqual(barSpan(frame, 'leftToRight', 0, 25), { x:10,y:20,width:25,height:50 });
  assert.deepEqual(barSpan(frame, 'rightToLeft', 0, 25), { x:85,y:20,width:25,height:50 });
  assert.deepEqual(barSpan(frame, 'topToBottom', 0, 25), { x:10,y:20,width:100,height:25 });
  assert.deepEqual(barSpan(frame, 'bottomToTop', 0, 25), { x:10,y:45,width:100,height:25 });
});
