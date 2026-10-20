import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  BLINK_HALF_MS, CAP_HEIGHT, ELLIPSIS, SCROLL_GAP, SCROLL_SPEED, SCROLL_STEP_MS, blinkDue, blinks, fitText, laidInPieces,
  layoutText, measureText, partStyle, pieceFace, separatorsOff,
} from '../../src/text-layout.js';

// A proportional font: "1" and "." are narrow, every other character 0.6 em.
const measure = (text, face) => [...text].reduce((w, ch) => w + ('1.'.includes(ch) ? 0.3 : 0.6) * face.size, 0);
const style = (more = {}) => ({ size: 10, font: { family: 'Inter', weight: 400 }, align: 'left', valign: 'top', ...more });
const plain = (text, more = {}) => ({ text, part: null, hidden: false, wheels: null, ...more });
const box = { x: 0, y: 0, width: 100, height: 40 };

test('which texts are laid out in pieces', () => {
  assert.ok(laidInPieces({ content: { type: 'sensor' }, style: {} }));
  assert.ok(laidInPieces({ content: { type: 'clock' }, style: {} }));
  assert.ok(laidInPieces({ content: { type: 'weather' }, style: {} }));
  assert.ok(laidInPieces({ content: { type: 'static' }, style: { overflow: 'scroll' } }));
  assert.ok(!laidInPieces({ content: { type: 'static' }, style: {} }));
  assert.ok(!laidInPieces({ content: { type: 'player' }, style: {} }));
  assert.ok(!laidInPieces(null));
});

test('digits are as wide as the widest one; without tabular digits they keep their width', () => {
  const width = (text, tabular) => layoutText([plain(text)], style(), measure, { tabular })[0].width;
  assert.equal(width('11', true), width('88', true));
  assert.ok(width('11', false) < width('88', false));
  assert.equal(width('a1b', true), 18, 'a, a digit slot of 6, b');
  const [line] = layoutText([plain('1x')], style(), measure);
  const [one] = line.frags[0].cells;
  assert.deepEqual([one.slot, one.w, one.digit], [6, 3, 0], 'the narrow digit is centred in a wider slot');
});

test('parts take their own face, size, colour and raised baseline', () => {
  assert.equal(partStyle(style(), 'unit'), null);
  assert.equal(partStyle(style({ parts: { unit: { baseline: 'base' } } }), 'unit'), null, 'nothing changes');
  assert.equal(partStyle(style({ parts: { separators: { blink: true } } }), 'separators'), null, 'blinking alone is not a style');
  const unit = { sizePercent: 50, color: '#ff0000ff', baseline: 'super', font: 'IBM Plex Mono', weight: 700 };
  assert.deepEqual(partStyle(style({ parts: { unit } }), 'unit'), unit);
  const face = pieceFace(style(), unit);
  assert.deepEqual(face, { family: 'IBM Plex Mono', weight: 700, italic: false, size: 5, color: '#ff0000ff', raise: 5 * CAP_HEIGHT });
  assert.equal(pieceFace(style(), { sizePercent: 9000 }).size, 40, 'at most 400 %');
  assert.equal(pieceFace(style(), { sizePercent: 1 }).size, 1, 'at least 10 %');
  assert.equal(pieceFace({}, null).size, 24, 'the default size');
  assert.equal(pieceFace(style(), null, 0.5).size, 5);

  const [line] = layoutText([plain('36'), plain('%', { part: { sizePercent: 50, baseline: 'super' } })], style(), measure);
  assert.equal(line.frags.length, 2);
  assert.equal(line.width, 12 + 3);
  assert.equal(line.height, line.frags[0].height, 'the number sets the line');
  assert.equal(line.frags[1].raise, 3.5);
});

test('lines, empty lines and empty texts', () => {
  const lines = layoutText([plain('a\n\nb')], style(), measure);
  assert.equal(lines.length, 3);
  assert.equal(lines[1].frags.length, 0);
  assert.equal(lines[1].height, 12, 'an empty line keeps the text height');
  assert.equal(layoutText([plain('')], style(), measure), null);
  assert.equal(measureText([plain('')], style(), measure), null);
  assert.deepEqual(measureText([plain('ab\nabc')], style(), measure), { width: 18, height: 24 });
});

test('a long line shrinks, ends with an ellipsis or scrolls; one that fits is left alone', () => {
  const long = [plain('abcdefghijklmnopqrstuvwxyz')];
  const narrow = { ...box, width: 60 };
  const none = fitText(long, style(), narrow, measure);
  assert.equal(none.lines[0].width, 156);
  assert.deepEqual(none.starts, [[0]]);
  assert.equal(none.dueMs, null);

  const shrunk = fitText(long, style({ overflow: 'shrink' }), narrow, measure);
  assert.ok(shrunk.lines[0].width <= 60.001 && shrunk.lines[0].width > 55);

  const cut = fitText(long, style({ overflow: 'ellipsis' }), narrow, measure);
  const frags = cut.lines[0].frags;
  assert.equal(frags[frags.length - 1].cells[0].text, ELLIPSIS);
  assert.ok(cut.lines[0].width <= 60);
  assert.equal(frags[0].cells[0].text, 'abcdefghi');
  const digits = fitText([plain('12345678901234')], style({ overflow: 'ellipsis' }), { ...box, width: 30 }, measure);
  assert.ok(digits.lines[0].width <= 30);
  const tiny = fitText([plain('abc'), plain('def', { part: { color: '#ff0000ff' } })], style({ overflow: 'ellipsis' }), { ...box, width: 7 }, measure);
  assert.deepEqual(tiny.lines[0].frags.map((f) => f.cells.map((c) => c.text).join('')), [ELLIPSIS]);
  const coloured = fitText([plain('abc'), plain('defghijk', { part: { color: '#ff0000ff' } })], style({ overflow: 'ellipsis' }), { ...box, width: 40 }, measure);
  assert.equal(coloured.lines[0].frags.at(-1).face.color, '#ff0000ff', 'the ellipsis takes the colour of the last part');

  const scroll = (seconds, motion = true) => fitText(long, style({ overflow: 'scroll' }), narrow, measure, { seconds, motion });
  const turn = 156 + 10 * SCROLL_GAP;
  assert.deepEqual(scroll(0).starts, [[0, turn]]);
  assert.equal(scroll(0).dueMs, SCROLL_STEP_MS);
  assert.deepEqual(scroll(1).starts, [[-SCROLL_SPEED, turn - SCROLL_SPEED]]);
  assert.equal(scroll(turn / SCROLL_SPEED).starts[0][0] + 0, 0, 'round again');
  assert.deepEqual(scroll(1, false).starts, [[0, turn]], 'reduced motion holds the start');
  assert.equal(scroll(1, false).dueMs, null);

  const short = fitText([plain('abc')], style({ overflow: 'scroll', align: 'right', valign: 'bottom' }), box, measure);
  assert.deepEqual(short.starts, [[82]]);
  assert.equal(short.top, 28);
  assert.equal(short.dueMs, null);
  const centred = fitText([plain('abc')], style({ align: 'center', valign: 'middle' }), box, measure);
  assert.deepEqual([centred.starts[0][0], centred.top], [41, 14]);
  assert.equal(fitText([plain('')], style(), box, measure), null);
});

test('separators blink in the second half of every second, only with motion', () => {
  const s = style({ parts: { separators: { blink: true } } });
  assert.ok(blinks(s) && !blinks(style()));
  assert.ok(!separatorsOff(s, 200, true));
  assert.ok(separatorsOff(s, 600, true));
  assert.ok(separatorsOff(s, 1999, true));
  assert.ok(!separatorsOff(s, 600, false));
  assert.ok(!separatorsOff(style(), 600, true));
  assert.equal(blinkDue(s, 200, true), 300);
  assert.equal(blinkDue(s, 600, true), 400);
  assert.equal(blinkDue(s, 1000, true), BLINK_HALF_MS);
  assert.equal(blinkDue(s, 600, false), null);
  assert.equal(blinkDue(style(), 600, true), null);
});

test('rolling digits are cells of their own even without tabular digits', () => {
  const [line] = layoutText([plain('41', { wheels: [[3, 0.5], [9, 0.5]] })], style(), measure, { tabular: false });
  assert.deepEqual(line.frags[0].cells.map((c) => [c.text, c.digit]), [['4', 0], ['1', 1]]);
  assert.deepEqual(line.frags[0].wheels, [[3, 0.5], [9, 0.5]]);
});
