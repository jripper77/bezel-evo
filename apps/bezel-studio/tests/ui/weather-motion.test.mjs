import { test } from 'node:test';
import assert from 'node:assert/strict';
import { ICON_FADE_MS, ICON_MOTIONS, ICON_SPEED, REST, amplitudeOf, iconPose, iconStep, soft } from '../../src/weather-motion.js';
import { sunCenter, weatherIconParts, weatherIconPaths } from '../../src/weather-icon.js';
import { createWidget } from '../../src/editor/widgets.js';

const PERIOD = [168, 168, 7, 840, 1, 3, 3, 4];

const close = (a, b) => {
  const near = (x, y) => Math.abs(x - y) < 1e-3;
  const rays = ((a.rays - b.rays) % 360 + 360) % 360;
  return (near(rays, 0) || near(rays, 360)) && near(a.disc, b.disc) && near(a.cloud, b.cloud)
    && a.strips.every((s, i) => near(s[0], b.strips[i][0]) && near(s[1], b.strips[i][1]))
    && a.drops.every((d, i) => d.every((v, j) => near(v, b.drops[i][j])))
    && near(a.bolt, b.bolt) && near(a.whole, b.whole) && near(a.halo, b.halo) && near(a.flash, b.flash);
};

test('every movement repeats without a jump; still and broken times rest', () => {
  for (let family = 0; family < 8; family++) {
    for (const speed of [0.5, 1, 2]) {
      for (const t of [0, 0.37, 1.9, 2.5]) assert.ok(close(iconPose(family, t, 1, speed), iconPose(family, t + PERIOD[family] / speed, 1, speed)), `${family} ${speed} ${t}`);
    }
    assert.equal(iconPose(family, 3, 0), REST);
    assert.equal(iconPose(family, NaN, 1), REST);
  }
});

test('the movements follow the specification, like the backend', () => {
  assert.ok(Math.abs(iconPose(0, 6, 1).rays - 90) < 1e-3);
  assert.ok(Math.abs(iconPose(0, 2, 1).disc - 0.12) < 1e-4);
  assert.ok(Math.abs(iconPose(0, 2, 0.5).halo - 0.25) < 1e-4);
  assert.ok(Math.abs(iconPose(1, 3.5, 1).cloud - 1) < 1e-4);
  assert.ok(Math.abs(iconPose(2, 1.75, 1, 2).cloud - 3) < 1e-4, 'twice as fast');
  const fog = iconPose(3, 2.5, 1);
  assert.ok(Math.abs(fog.strips[0][0] - 4) < 1e-3 && Math.abs(fog.strips[0][1] - 1) < 1e-3);
  assert.ok(Math.abs(iconPose(4, 0.5, 1).drops[0][1] - 2) < 1e-3);
  const snow = iconPose(5, 1.5, 1);
  assert.ok(Math.abs(snow.drops[0][0] - 2) < 1e-3 && Math.abs(snow.drops[0][1] - 2) < 1e-3);
  assert.ok(Math.abs(iconPose(6, 1.8, 1).bolt - 1) < 1e-3 && Math.abs(iconPose(6, 1.95, 1).bolt - 0.2) < 1e-3);
  assert.ok(Math.abs(iconPose(6, 1.8, 1).flash - 0.12) < 1e-3);
  assert.ok(Math.abs(iconPose(7, 2, 1).whole - 0.85) < 1e-3);
  assert.ok(Math.abs(iconPose(4, -0.5, 1).drops[0][1] - 2) < 1e-3, 'times before the start loop too');
  assert.ok(Math.abs(soft(0.5) - 0.5) < 1e-3 && soft(0) === 0 && soft(1) === 1);
  assert.deepEqual([amplitudeOf('still'), amplitudeOf('gentle'), amplitudeOf('lively'), amplitudeOf(undefined)], [0, 0.5, 1, 0]);
  assert.deepEqual([iconStep('still'), iconStep('gentle'), iconStep('lively')], [null, 50, 33]);
  assert.deepEqual(ICON_MOTIONS, ['still', 'gentle', 'lively']);
  assert.deepEqual([ICON_SPEED.min, ICON_SPEED.max, ICON_FADE_MS], [0.5, 2, 300]);
});

test('icons are made of parts that join into the icon at rest', () => {
  assert.deepEqual(weatherIconParts(0).map((p) => p.role), ['disc', 'rays']);
  assert.deepEqual(weatherIconParts(45).map((p) => `${p.role}${p.index}`), ['cloud0', 'strip0', 'strip1']);
  assert.deepEqual(weatherIconParts(61).map((p) => p.role), ['cloud', 'drop', 'drop', 'drop']);
  assert.deepEqual(weatherIconParts(95).map((p) => p.role), ['cloud', 'bolt']);
  assert.deepEqual(weatherIconParts(999).map((p) => p.role), ['cross']);
  assert.equal(weatherIconPaths(61, true).length, 2, 'drops are one component at rest');
  assert.deepEqual(sunCenter(true), [12, 9]);
  assert.deepEqual(sunCenter(false), [12, 10]);
});

test('a new weather object moves gently', () => {
  assert.equal(createWidget('weather', { width: 480, height: 320 }).kind.content.iconMotion, 'gentle');
});
