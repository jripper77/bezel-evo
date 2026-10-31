import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { DANCERS, DANCE_POSES, DANCER_PALETTE, MOVES, BEAT_HELD, JUMP_STRENGTH, SILENCE, DEMO_TEMPO, dancePose, dancerGrid, demoBeat, drawDancer } from '../../src/dancer-pixel.js';
import { SPECTRUM_STYLES, spectrumPatch } from '../../src/spectrum.js';

const fixture = JSON.parse(readFileSync(new URL('./fixtures/dancers.json', import.meta.url), 'utf8'));
const rust = (path) => readFileSync(new URL(`../../../../crates/${path}`, import.meta.url), 'utf8');

test('every pose of every dancer is the designed one, like the backend', () => {
  assert.deepEqual(Object.keys(fixture), [...DANCERS]);
  for (const dancer of DANCERS) DANCE_POSES.forEach((pose, i) => assert.deepEqual(dancerGrid(dancer, pose), fixture[dancer][i], `${dancer} ${pose}`));
  const hex = (n) => Number(n).toString(16).padStart(2, '0');
  const palette = Object.fromEntries([...rust('bezel-render/src/dancer.rs').matchAll(/\('(.)', rgb\((\d+), (\d+), (\d+)\)\)/g)].map(([, c, r, g, b]) => [c, `#${hex(r)}${hex(g)}${hex(b)}`]));
  assert.deepEqual(palette, DANCER_PALETTE);
});

test('the pose follows the beat like the backend: a move, a jump, down between beats, still in silence', () => {
  const core = rust('bezel-core/src/domain/spectrum.rs');
  const constant = (name) => Number(core.match(new RegExp(`pub const ${name}: f32 = ([\\d.]+);`))[1]);
  assert.deepEqual([BEAT_HELD, JUMP_STRENGTH, SILENCE, DEMO_TEMPO], ['BEAT_HELD', 'JUMP_STRENGTH', 'SILENCE', 'DEMO_TEMPO'].map(constant));
  assert.deepEqual([...core.slice(core.indexOf('pub const MOVES'), core.indexOf('];', core.indexOf('pub const MOVES'))).matchAll(/DancePose::(\w+)/g)].map(m => m[1][0].toLowerCase() + m[1].slice(1)), [...MOVES]);
  const loud = [0.5, 0.5];
  assert.equal(dancePose({ levels: loud, beat: { count: 1, since: 0.1, strength: 0.6 } }), 'stepLeft');
  assert.equal(dancePose({ levels: loud, beat: { count: 1, since: 0.1, strength: 0.9 } }), 'jump');
  assert.equal(dancePose({ levels: loud, beat: { count: 1, since: 0.5, strength: 0.9 } }), 'down');
  assert.equal(dancePose({ levels: loud, beat: null }), 'down');
  assert.equal(dancePose({ levels: [0.01], beat: { count: 1, since: 0.1, strength: 0.6 } }), 'still');
  assert.equal(dancePose({ playing: false, levels: loud }), 'still');
  assert.deepEqual(demoBeat(0), { count: 0, since: 0, strength: 0.6 });
  assert.equal(demoBeat(1.6).count, 3); assert.ok(Math.abs(demoBeat(1.6).since - 0.1) < 1e-9);
  assert.equal(demoBeat(3.6).strength, 1);
});

test('a dancer is drawn on whole pixels standing on the bottom, and the spectrum keeps its choice', () => {
  const rects = [];
  const ctx = { set fillStyle(v) { this.colour = v; }, fillRect: (x, y, w, h) => rects.push([x, y, w, h]) };
  drawDancer(ctx, { x: 10.4, y: 0, width: 100, height: 60 }, 'cat', 'still');
  assert.ok(rects.length > 50 && rects.every(([x, y, w, h]) => Number.isInteger(x) && Number.isInteger(y) && w === 2 && h === 2));
  assert.equal(Math.max(...rects.map(r => r[1])), 60 - 2, 'the shadow lies on the bottom row');
  drawDancer(ctx, { x: 0, y: 0, width: 0, height: 10 }, 'cat', 'still');
  assert.ok(SPECTRUM_STYLES.includes('dancer'));
  assert.equal(spectrumPatch({}, { style: 'dancer', dancer: 'cat' }).dancer, 'cat');
  assert.equal(spectrumPatch({}, { dancer: 'dragon' }).dancer, 'robot');
});
