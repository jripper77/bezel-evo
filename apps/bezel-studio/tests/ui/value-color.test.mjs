import { test } from 'node:test';
import assert from 'node:assert/strict';
import { MAX_COLOR_THRESHOLDS, supportsValueColor, colorAt, defaultValueColor, addColorThreshold, demoColorValue, withValueColor } from '../../src/value-color.js';
import { demoSensorValue, DEMO_SENSORS, demoValue } from '../../src/demo-sensors.js';
import { createWidget } from '../../src/editor/widgets.js';
import { moveRule } from '../../src/card-rules.js';

const rule = { base: '#00ff00ff', thresholds: [{ value: 80, color: '#ff0000ff' }, { value: 60, color: '#ff8800ff' }] };
const element = widget => ({ id: 1, ...createWidget(widget, { width: 480, height: 1920 }) });
test('inclusive steps, unsorted rows, ties and unavailable readings match native rules', () => {
  for (const [value, color] of [[null, rule.base], [NaN, rule.base], [Infinity, rule.base], [59.99, rule.base], [60, '#ff8800ff'], [79.99, '#ff8800ff'], [80, '#ff0000ff'], [100, '#ff0000ff']]) assert.equal(colorAt(rule, value), color);
  const tie = { ...rule, thresholds: [...rule.thresholds, { value: 80, color: rule.base }] };
  assert.equal(colorAt(tie, 80), rule.base);
  assert.equal(colorAt({ ...rule, thresholds: moveRule(tie.thresholds, 2, 0) }, 80), '#ff0000ff');
  assert.equal(colorAt({ ...rule, thresholds: [{ value: NaN, color: 'invalid' }] }, 0), rule.base);
});
test('only supported kinds have value colours, including SVG icons and unbound text', () => {
  for (const widget of ['text', 'value', 'shape', 'ring', 'bar', 'needle', 'graph']) assert.ok(supportsValueColor(element(widget)), widget);
  for (const widget of ['clock', 'weather', 'player', 'card', 'image']) assert.equal(supportsValueColor(element(widget)), false, widget);
  assert.equal(supportsValueColor(null), false);
  assert.equal(supportsValueColor({ ...element('shape'), isGroup: true }), false);
  assert.equal(supportsValueColor({ kind: { type: 'shape', videoWindow: true } }), false);
  assert.ok(supportsValueColor({ kind: { type: 'image', asset: 'icon.SVG' } }));
});
test('base inherits existing solid, gradient or icon colour; new rows are bounded', () => {
  const bar = element('bar');
  assert.deepEqual(defaultValueColor(bar, 'ignored'), { base: bar.kind.fill, thresholds: [] });
  assert.equal(defaultValueColor({ kind: { type: 'shape', fill: { stops: [[0, rule.base]] } } }, 'cpu.usage').base, rule.base);
  assert.equal(defaultValueColor({ kind: { type: 'ring', binding: { key: 'cpu.usage' }, fill: { start: rule.base } } }).base, rule.base);
  assert.equal(defaultValueColor(element('image'), 'cpu.usage', rule.base).base, rule.base);
  assert.equal(defaultValueColor(element('image'), 'cpu.usage').sensor, 'cpu.usage');
  assert.equal(Object.hasOwn(defaultValueColor(element('shape'), ''), 'sensor'), false, 'no catalogue yet: the rule uses its base colour');
  let r = { base: rule.base, thresholds: [] };
  r = addColorThreshold(r); assert.deepEqual(r.thresholds, [{ value: 0, color: rule.base }]);
  r = addColorThreshold(r); assert.equal(r.thresholds[1].value, 10);
  while (r.thresholds.length < MAX_COLOR_THRESHOLDS) r = addColorThreshold(r);
  assert.equal(addColorThreshold(r), r);
});
test('demo resolves the drawn value and changes only the supported paint', () => {
  const plain = element('bar'); assert.equal(withValueColor(plain, 80), plain);
  const clock = { ...element('clock'), colorByValue: rule }; assert.equal(withValueColor(clock, 80), clock);
  for (const widget of ['text', 'value', 'shape', 'ring', 'bar', 'needle', 'graph']) {
    const e = { ...element(widget), colorByValue: rule }, resolved = withValueColor(e, 80);
    const k = resolved.kind;
    const paint = k.type === 'text' ? k.style.paint : ['shape', 'ring', 'bar'].includes(k.type) ? k.fill : k.color;
    assert.equal(paint, '#ff0000ff', widget);
    assert.deepEqual(k.track, e.kind.track);
    assert.equal(e.colorByValue, rule);
  }
  const icon = withValueColor({ kind: { type: 'image', asset: 'icon.svg' }, colorByValue: rule }, null);
  assert.equal(icon.kind.color, rule.base);
  assert.equal(demoColorValue(element('value'), .75, () => 0), 75);
  const bar = element('bar'); bar.kind.binding = { key: 'cpu.temperature', min: 20, max: 100 };
  assert.equal(demoColorValue(bar, .5, () => 0), 60);
  assert.equal(demoColorValue({ ...element('shape'), colorByValue: { sensor: 'cpu.usage' } }, .5, key => key === 'cpu.usage' ? 92 : null), 92);
  assert.equal(demoColorValue(element('shape'), .5, key => key ?? null), null);
});
test('demo colour sensors reuse the sampled catalogue and unavailable means null', () => {
  assert.equal(demoSensorValue('missing', 1), null);
  // Available memory is what used memory leaves (T-9d), so the two always agree.
  DEMO_SENSORS.forEach(([key, , , , base, swing], i) => assert.equal(demoSensorValue(key, 2),
    key === 'memory.available.percent' ? 100 - demoSensorValue('memory.percent', 2) : demoValue(base, swing, 2, i)));
});
