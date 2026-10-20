import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  MAX_ZONES, NEEDLE_SHAPES, PIVOT_CAPS, SCALE_NUMBERS, SCALE_TICKS, TRACKS, demoPeak, faint, fitGauge, gradientAt, lookOf,
  needleColor, numberShares, scaleNumber, share, tickShares, zoneAt,
} from '../../src/needle-look.js';

const binding = { key: 'cpu.usage', min: 0, max: 100 };
const zones = [{ from: 0, to: 60, color: '#00ff00ff' }, { from: 60, to: 85, color: '#ffff00ff' }, { from: 85, to: 100, color: '#ff0000ff' }];

test('a look with every absent field at the backend default', () => {
  const d = lookOf(undefined);
  assert.deepEqual([d.arcWidth, d.arcCap, d.track, d.ticks, d.numbers, d.numberGap, d.shape, d.length, d.tail, d.cap, d.capDiameter],
    [5, 'round', 'single', 'majorMinor', 'halves', 8, 'tapered', 92, 12, 'ring', 9]);
  assert.equal(d.center, null);
  assert.equal(d.peakSeconds, null);
  assert.equal(lookOf({ shape: 'dot' }).shape, 'dot');
  assert.equal(MAX_ZONES, 8);
  for (const list of [TRACKS, SCALE_TICKS, SCALE_NUMBERS, NEEDLE_SHAPES, PIVOT_CAPS]) assert.ok(Object.isFrozen(list) && list.length >= 3);
});

test('shares, zones and the needle colour', () => {
  assert.equal(share(binding, 25), 0.25);
  assert.equal(share(binding, 150), 1);
  assert.equal(share(binding, -1), 0);
  assert.equal(share({ min: 1, max: 1 }, 1), 0);
  assert.equal(share(binding, NaN), 0);
  const look = { zones, colorFromZone: true };
  assert.equal(zoneAt(look, 70).color, '#ffff00ff');
  assert.equal(zoneAt(look, 60).color, '#ffff00ff', 'the later zone wins a boundary');
  assert.equal(zoneAt(look, 120), null);
  assert.equal(zoneAt({ zones: [{ from: 50, to: 10, color: '#ff0000ff' }] }, 20).color, '#ff0000ff');
  assert.equal(needleColor(look, 30, '#ffffffff'), '#00ff00ff');
  assert.equal(needleColor(look, 120, '#ffffffff'), '#ffffffff');
  assert.equal(needleColor(look, null, '#ffffffff'), '#ffffffff');
  assert.equal(needleColor({ zones }, 30, '#ffffffff'), '#ffffffff');
});

test('the faint arc and the gradient like the backend', () => {
  assert.equal(faint({}), '#aeb5bd4d');
  assert.equal(gradientAt({}, binding, 0.5), '#aeb5bd4d');
  const look = { zones };
  assert.equal(gradientAt(look, binding, 0), '#00ff00ff');
  assert.equal(gradientAt(look, binding, 1), '#ff0000ff');
  const mid = gradientAt(look, binding, (0.3 + 0.725) / 2);
  assert.ok(['#7fff00ff', '#80ff00ff'].includes(mid), `half green, half yellow: ${mid}`);
  assert.equal(gradientAt({ zones: [zones[0], { ...zones[0], color: '#0000ffff' }] }, binding, 0.31), '#0000ffff', 'stops at one place');
});

test('ticks, numbers and scale numbers', () => {
  assert.deepEqual([tickShares({}).major.length, tickShares({}).minor.length], [11, 40]);
  assert.equal(tickShares({ ticks: 'major' }).minor.length, 0);
  assert.equal(tickShares({ ticks: 'none' }).major.length, 0);
  assert.deepEqual(numberShares({}), [0, 0.5, 1]);
  assert.deepEqual(numberShares({ numbers: 'ends' }), [0, 1]);
  assert.equal(numberShares({ numbers: 'major' }).length, 11);
  assert.deepEqual(numberShares({ numbers: 'none' }), []);
  assert.equal(scaleNumber(50, 'it'), '50');
  assert.equal(scaleNumber(2.5, 'it'), '2,5');
  assert.equal(scaleNumber(2.5, 'en'), '2.5');
  assert.equal(scaleNumber(1 / 3, 'pt-BR'), '0,33');
  assert.equal(scaleNumber(-0.001), '0');
  assert.equal(scaleNumber(NaN), '—');
});

test('the arc is as large as the box allows, like the backend', () => {
  const area = { x: 0, y: 0, width: 200, height: 200 };
  const half = fitGauge(area, [0.5, 0.5], -90, 180, 0);
  assert.deepEqual(half.pivot, [100, 100]);
  assert.ok(Math.abs(half.radius - 100) < 1e-6);
  assert.ok(Math.abs(fitGauge(area, [0.5, 0.9], -90, 180, 5).radius - 95) < 1e-6);
  assert.ok(Math.abs(fitGauge(area, [0.5, 0.9], -120, 240, 0).radius - 40) < 1e-6);
  assert.ok(Math.abs(fitGauge(area, [0.5, 0.9], -45, 90, 0).radius - 141.421) < 0.01);
  assert.ok(Math.abs(fitGauge(area, [0.5, 0.5], 90, -180, 0).radius - 100) < 1e-6);
  assert.equal(fitGauge({ x: 0, y: 0, width: 0, height: 0 }, [0.5, 0.5], 0, 0, 0).radius, 0);
});

test('the demo peak is the highest sample of the last seconds', () => {
  const read = (s) => [10, 80, 40, 30, 20][Math.floor(s)] ?? NaN;
  assert.equal(demoPeak({}, read, 3, 1), null, 'no mark');
  assert.equal(demoPeak({ peakSeconds: 10 }, read, 3.5, 1), 80);
  assert.equal(demoPeak({ peakSeconds: 2 }, read, 4.5, 1), 30, 'the samples at 3 and 4 s');
  assert.equal(demoPeak({ peakSeconds: 2 }, () => NaN, 4.5, 1), null);
  assert.equal(demoPeak({ peakSeconds: 2 }, read, 4.5, 0), 30, 'one sample a second without a refresh');
});
