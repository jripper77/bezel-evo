import test from 'node:test';
import assert from 'node:assert/strict';
import {
  PLAYER_PARTS, PROGRESS_DEFAULTS, barGeometry, barHeight, blockMetrics, blockRows, clockTime, coverFrame, coverShare, fitPlayer, knobDiameter,
  partPatch, partsOf, placeBlock, playerText, playerTimes, progressBlocks, progressOf, progressRadius, showsTime, widestTimes,
} from '../../src/player-parts.js';

const media = { title: 'Track', artist: 'Artist', source: 'Spotify', position: 84, duration: 210 };
/** A player as saved before the parts existed. */
const legacy = { type: 'player', showCover: true, showProgress: true, showSource: false };
/** A label measure: 7 px per character, 17 px lines. */
const label = (text) => ({ width: text.length * 7, height: 17 });

test('a player saved before the parts shows what it showed', () => {
  assert.deepEqual(partsOf(legacy), { cover: true, title: true, artist: true, time: true, progress: true, source: false });
  assert.deepEqual(playerText(legacy, media), { lines: ['Track', 'Artist', '1:24 / 3:30'], times: null, progress: 0.4 });
  assert.equal(showsTime({ ...legacy, showProgress: false }), false, 'times followed the bar');
  assert.deepEqual(progressOf(legacy), PROGRESS_DEFAULTS);
  assert.deepEqual(progressOf({ progressStyle: { knob: true } }), { ...PROGRESS_DEFAULTS, knob: true });
  assert.deepEqual(PLAYER_PARTS, ['cover', 'title', 'artist', 'time', 'progress', 'source']);
});

test('each part turns on and off alone; the bar stops pulling the times', () => {
  assert.deepEqual(partPatch(legacy, 'title'), { showTitle: false });
  assert.deepEqual(partPatch({ ...legacy, showArtist: false }, 'artist'), { showArtist: true });
  assert.deepEqual(partPatch(legacy, 'time'), { showTime: false });
  assert.deepEqual(partPatch(legacy, 'progress'), { showProgress: false, showTime: true });
  assert.deepEqual(partPatch({ ...legacy, showTime: false }, 'progress'), { showProgress: false });
  assert.deepEqual(partPatch(legacy, 'cover'), { showCover: false });
  assert.deepEqual(partPatch(legacy, 'source'), { showSource: true });
  const titleOnly = { ...legacy, showArtist: false, showProgress: false, showTime: false };
  assert.deepEqual(playerText(titleOnly, media).lines, ['Track']);
  const artistTime = { ...legacy, showTitle: false, showProgress: false, showTime: true, showSource: true };
  assert.deepEqual(playerText(artistTime, media).lines, ['Artist', 'Spotify', '1:24 / 3:30']);
  assert.deepEqual(playerText(artistTime, { ...media, duration: 0 }).lines, ['Artist', 'Spotify'], 'no times without a duration');
});

test('times: formats, widest and placement', () => {
  const cases = [['elapsedTotal', ['1:24', '3:30']], ['elapsedRemaining', ['1:24', '-2:06']], ['elapsed', ['1:24', null]], ['remaining', [null, '-2:06']], ['total', [null, '3:30']]];
  for (const [timeFormat, times] of cases) assert.deepEqual(playerTimes({ timeFormat }, media), times);
  assert.deepEqual(widestTimes({ timeFormat: 'elapsedRemaining' }, media), ['3:30', '-3:30']);
  assert.deepEqual(widestTimes({ timeFormat: 'remaining' }, media), [null, '-3:30']);
  assert.equal(clockTime(Number.NaN), '0:00');
  assert.equal(clockTime(-2), '0:00');
  assert.equal(clockTime(4500), '75:00');
  const sides = { ...legacy, showTitle: false, showArtist: false, timeFormat: 'remaining', progressStyle: { times: 'sides' } };
  assert.deepEqual(playerText(sides, media), { lines: [], times: [null, '-2:06'], progress: 0.4 });
  assert.deepEqual(playerText({ ...sides, showProgress: false, showTime: true }, media).lines, ['-2:06'], 'no bar: times are text');
});

test('bar geometry: radius, knob, blocks', () => {
  const s = { ...PROGRESS_DEFAULTS };
  assert.deepEqual([progressRadius(s), knobDiameter(s), barHeight(s)], [2, 0, 4]);
  assert.deepEqual(progressBlocks(s, 100), [[0, 100]]);
  assert.deepEqual(progressBlocks(s, 0), []);
  assert.deepEqual([knobDiameter({ ...s, knob: true }), barHeight({ ...s, knob: true })], [10, 10]);
  assert.equal(progressRadius({ ...s, ends: 'square' }), 0);
  const blocks = progressBlocks({ ...s, pattern: 'segments', segments: 4 }, 100);
  assert.equal(blocks.length, 4);
  assert.equal(blocks[3][1], 100);
  assert.ok(Math.abs(blocks[1][0] - blocks[0][1] - 3) < 1e-9);
  const dots = progressBlocks({ ...s, pattern: 'dots' }, 100);
  assert.equal(dots.length, 16);
  assert.ok(dots.every(([a, b]) => Math.abs(b - a - 4) < 1e-9));
  assert.deepEqual(progressBlocks({ ...s, pattern: 'dots' }, 3), [[0, 3]]);
  assert.deepEqual(progressBlocks({ ...s, pattern: 'segments', segments: 200 }, 10), [[0, 10]]);
});

test('block metrics and rows place the times beside or above the bar', () => {
  const bar = { ...legacy, progressStyle: { thickness: 6 } };
  assert.equal(blockMetrics(bar, media, { lines: [], times: null, progress: null }, label, 14), null);
  assert.deepEqual(blockMetrics(bar, media, playerText(bar, media), label, 14), { height: 6, left: 0, right: 0, line: 0, gap: 0, minWidth: 0 });
  const sides = { ...bar, progressStyle: { thickness: 6, times: 'sides' } };
  const shown = playerText(sides, media);
  const m = blockMetrics(sides, media, shown, label, 10);
  assert.deepEqual(m, { height: 17, left: 28, right: 28, line: 17, gap: 4, minWidth: 28 + 28 + 8 + 30 });
  const block = placeBlock({ x: 0, y: 0, width: 200, height: 100 }, m.height, true, 'middle');
  assert.deepEqual(block, { x: 0, y: 41.5, width: 200, height: 17 });
  const rows = blockRows(sides, block, m, shown);
  assert.deepEqual(rows.row, { x: 32, y: 47, width: 136, height: 6 });
  assert.deepEqual(rows.labels.map((l) => [l.text, l.align, l.box.x]), [['1:24', 'left', 0], ['3:30', 'right', 172]]);
  const above = { ...bar, progressStyle: { thickness: 6, times: 'above', knob: true } };
  const ma = blockMetrics(above, media, playerText(above, media), label, 10);
  assert.equal(ma.height, 17 + 2 + 12);
  assert.equal(ma.minWidth, 28 + 28 + 4);
  const ra = blockRows(above, { x: 0, y: 0, width: 200, height: ma.height }, ma, playerText(above, media));
  assert.deepEqual(ra.row, { x: 0, y: 19, width: 200, height: 12 });
  assert.deepEqual(ra.labels.map((l) => l.valign), ['top', 'top']);
  const plain = blockRows(bar, block, m, playerText(bar, media));
  assert.equal(plain.row, block);
  assert.deepEqual(placeBlock({ x: 0, y: 0, width: 10, height: 100 }, 6, false, 'top').y, 94, 'under the text');
  assert.deepEqual(placeBlock({ x: 0, y: 0, width: 10, height: 100 }, 6, true, 'top').y, 0);
  assert.deepEqual(placeBlock({ x: 0, y: 0, width: 10, height: 100 }, 6, true, 'bottom').y, 94);
  const g = barGeometry(above, ra.row, 0);
  assert.deepEqual(g.bar, { x: 0, y: 22, width: 200, height: 6 });
  assert.deepEqual(g.knob, { cx: 6, cy: 25, d: 12 }, 'the knob stays inside the bar');
  assert.equal(g.filled, 0);
  assert.equal(barGeometry(bar, { x: 0, y: 0, width: 0, height: 6 }, 0.5).knob, null);
});

test('fit measures the visible parts only', () => {
  const old = { x: 0, y: 0, width: 220, height: 104 };
  const none = { ...legacy, showCover: false, showTitle: false, showArtist: false, showProgress: false, showTime: false };
  const text = { width: 50, height: 17 };
  const title = { ...none, showTitle: true };
  assert.deepEqual(fitPlayer(title, playerText(title, media), text, null, old, 14), { width: 50, height: 17 });
  assert.equal(fitPlayer(none, playerText(none, media), null, null, old, 14), null);
  const bar = { ...none, showProgress: true };
  const shownBar = playerText(bar, media);
  const m = blockMetrics(bar, media, shownBar, label, 14);
  assert.deepEqual(fitPlayer(bar, shownBar, null, m, old, 14), { width: 220, height: 4 });
  const titled = { ...bar, showTitle: true };
  assert.deepEqual(fitPlayer(titled, playerText(titled, media), text, m, old, 14), { width: 50, height: 25 });
  assert.equal(fitPlayer(titled, playerText(titled, media), null, m, old, 14), null, 'text that cannot be measured');
  const cover = { ...none, showCover: true };
  assert.deepEqual(fitPlayer(cover, playerText(cover, media), null, null, old, 14), { width: 104, height: 104 });
  const coverBar = { ...bar, showCover: true };
  assert.deepEqual(fitPlayer(coverBar, shownBar, null, m, old, 14), { width: 220, height: 104 });
  const full = playerText(legacy, media);
  assert.deepEqual(fitPlayer(legacy, full, { width: 93, height: 51 }, { height: 4, minWidth: 0 }, old, 14), { width: Math.ceil(Math.max(93 + 59 + 4.2, 59 / 0.4)), height: 59 });
  assert.deepEqual(coverShare(cover, playerText(cover, media), 14), { fraction: 1, gap: 0 });
  assert.deepEqual(coverShare({ ...legacy, coverGap: 9 }, full, 14), { fraction: 0.4, gap: 9 });
});

test('the cover frame is the square both renderers draw at the top left', () => {
  const frame = { x: 10, y: 20, width: 200, height: 100 };
  assert.deepEqual(coverFrame(frame, { ...legacy, coverGap: 9 }, 14), { x: 10, y: 20, width: 80, height: 80, gap: 9, alone: false });
  assert.deepEqual(coverFrame({ ...frame, width: 100 }, { ...legacy, coverGap: 95 }, 14), { x: 10, y: 20, width: 5, height: 5, gap: 95, alone: false });
  assert.deepEqual(coverFrame({ ...frame, width: 4 }, { ...legacy, coverGap: 9 }, 14).gap, 4);
  const alone = { type: 'player', showCover: true, showTitle: false, showArtist: false, showTime: false, showProgress: false };
  assert.deepEqual(coverFrame(frame, alone, 14), { x: 10, y: 20, width: 100, height: 100, gap: 0, alone: true });
  // A session without a duration shows no bar or times: a titleless cover is then alone.
  assert.equal(coverFrame(frame, { ...alone, showProgress: true }, 14, { ...media, duration: 0 }).alone, true);
  assert.equal(coverFrame(frame, { ...legacy, showCover: false }, 14), null);
});
