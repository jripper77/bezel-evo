import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { PIXEL_PALETTE, PIXEL_SPRITES, cells, drawPixelWeather, pixelSprites, rayFrame, steps } from '../../src/weather-pixel.js';
import { REST, iconPose } from '../../src/weather-motion.js';

const rust = readFileSync(new URL('../../../../crates/bezel-render/src/weather_pixel.rs', import.meta.url), 'utf8');

test('the sprites and the palette are the backend\'s', () => {
  const sprites = rust.slice(rust.indexOf('const SPRITES'), rust.indexOf('/// The sprite called'));
  const parsed = Object.fromEntries([...sprites.matchAll(/\(\s*"([a-z-]+)",\s*&\[([^\]]*)\]/g)]
    .map(([, name, rows]) => [name, [...rows.matchAll(/"([^"]*)"/g)].map(m => m[1])]));
  assert.deepEqual(parsed, PIXEL_SPRITES);
  const hex = (n) => Number(n).toString(16).padStart(2, '0');
  const palette = Object.fromEntries([...rust.matchAll(/\('(.)', rgb\((\d+), (\d+), (\d+)\)\)/g)].map(([, c, r, g, b]) => [c, `#${hex(r)}${hex(g)}${hex(b)}`]));
  assert.deepEqual(palette, PIXEL_PALETTE);
});

test('the pose snaps to the grid like the backend', () => {
  assert.equal(cells(8), 2); assert.equal(cells(-6), -1); assert.equal(cells(NaN), 0);
  assert.equal(steps(0.3), 0.5); assert.equal(steps(0.2), 0); assert.equal(steps(0.8), 1);
  assert.equal(rayFrame(0), 0); assert.equal(rayFrame(30), 1); assert.equal(rayFrame(-10), 1);
  assert.deepEqual(pixelSprites(61, REST).map(s => s.name), ['cloud', 'drop', 'drop', 'drop']);
  const frames = new Set(Array.from({ length: 48 }, (_, i) => pixelSprites(0, iconPose(0, i * 0.5, 1, 1))[1].name));
  assert.deepEqual([...frames].sort(), ['rays-a', 'rays-b']);
});

test('cells land on whole pixels in the centered grid', () => {
  const rects = [];
  const ctx = { save() {}, restore() {}, set fillStyle(v) { this.colour = v; }, globalAlpha: 1, fillRect: (x, y, w, h) => rects.push([x, y, w, h]) };
  drawPixelWeather(ctx, 10.4, 0, 40, 0, REST);
  assert.ok(rects.length > 40);
  assert.ok(rects.every(([x, y, w, h]) => Number.isInteger(x) && Number.isInteger(y) && w === 2 && h === 2));
  assert.equal(Math.min(...rects.map(r => r[0])), 14 + 2, 'the 32-pixel grid centered in 40');
});
