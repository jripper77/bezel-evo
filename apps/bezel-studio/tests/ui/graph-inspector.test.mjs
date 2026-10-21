import { test } from 'node:test';
import assert from 'node:assert/strict';
import { graphSeriesOf, graphStyleOf, graphSeriesPatch, graphStylePatch } from '../../src/graph-inspector.js';
import { merge } from '../../src/editor/store.js';
import { drawVisualization } from '../../src/demo-visualization.js';

test('series count and each graph style combine without losing the lower source', () => {
  let e = { kind: { type: 'graph', style: 'line' } };
  assert.equal(graphStyleOf(e), 'line'); assert.equal(graphSeriesOf(e), 'single');
  e = merge(e, graphSeriesPatch(e, 'mirrored'));
  e.visualization.secondary = { key: 'net.up', min: 0, max: 500 };
  for (const style of ['line', 'area', 'bars', 'rounded']) {
    e = merge(e, graphStylePatch(e, style));
    assert.equal(graphStyleOf(e), style); assert.equal(graphSeriesOf(e), 'mirrored');
    assert.equal(e.visualization.secondary.key, 'net.up');
  }
  e = merge(e, graphSeriesPatch(e, 'single'));
  assert.equal(graphStyleOf(e), 'rounded'); assert.equal(graphSeriesOf(e), 'single');
  e = merge(e, graphSeriesPatch(e, 'mirrored'));
  assert.equal(e.visualization.secondary.key, 'net.up');
  // Imported presets use the old rounded geometry by default.
  assert.equal(graphStyleOf({ kind: { style: 'bars' }, visualization: { style: 'mirrored' } }), 'rounded');
});

test('the demo keeps the native series gap and independent rounded geometry', () => {
  const rectangles = [];
  const context = { beginPath() {}, roundRect(...rect) { rectangles.push(rect); }, fill() {} };
  const e = { frame: { x: 0, y: 0, width: 100, height: 80 }, kind: { type: 'graph', style: 'bars', lineWidth: 2, color: '#ffffffff' }, visualization: { style: 'mirrored', roundedBars: false, seriesGap: 12, gap: 3, secondary: { key: 'net.up' }, secondaryColor: '#ff0000ff' } };
  assert.equal(drawVisualization(context, e, 0, [1,1], [1,1]), true);
  assert.deepEqual(rectangles.map(rect => [rect[1], rect[3], rect[4]]), [[0,34,0],[0,34,0],[46,34,0],[46,34,0]]);
  rectangles.length = 0; e.visualization.roundedBars = true;
  drawVisualization(context, e, 0, [1,1], [1,1]);
  assert.ok(rectangles.every(rect => rect[4] > 0));
});
