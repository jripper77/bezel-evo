import { test } from 'node:test';
import assert from 'node:assert/strict';
import { BIG_STEP, PANELS, STAGE_MIN, STEP, STORAGE_KEY, clampWidth, fitWidths, initialWidths, readWidths, widthAfterDrag, widthAfterKey, writeWidths } from '../../src/panel-sizes.js';

/** A localStorage stand-in. */
function memory(initial = {}) {
  const data = new Map(Object.entries(initial));
  return { getItem: (k) => (data.has(k) ? data.get(k) : null), setItem: (k, v) => data.set(k, String(v)), data };
}

test('each panel starts at its width and stays between its limits', () => {
  assert.deepEqual(initialWidths(), { library: 272, inspector: 360 });
  for (const [panel, { min, max, initial }] of Object.entries(PANELS)) {
    assert.ok(min < initial && initial < max, panel);
    assert.equal(clampWidth(panel, min - 50), min);
    assert.equal(clampWidth(panel, max + 50), max);
    assert.equal(clampWidth(panel, initial + 0.6), initial + 1);
    assert.equal(clampWidth(panel, 'wide'), initial);
    assert.equal(clampWidth(panel, undefined), initial);
    assert.equal(clampWidth(panel, Number.NaN), initial);
  }
});

test('the arrow towards the stage widens a panel, Shift by a bigger step, Home and End go to the limits', () => {
  assert.equal(widthAfterKey('library', 300, 'ArrowRight'), 300 + STEP);
  assert.equal(widthAfterKey('library', 300, 'ArrowLeft'), 300 - STEP);
  assert.equal(widthAfterKey('library', 300, 'ArrowRight', true), 300 + BIG_STEP);
  assert.equal(widthAfterKey('inspector', 300, 'ArrowLeft'), 300 + STEP);
  assert.equal(widthAfterKey('inspector', 360, 'ArrowRight', true), 360 - BIG_STEP);
  assert.equal(widthAfterKey('library', 300, 'Home'), PANELS.library.min);
  assert.equal(widthAfterKey('inspector', 300, 'End'), PANELS.inspector.max);
  assert.equal(widthAfterKey('library', PANELS.library.max, 'ArrowRight'), PANELS.library.max);
  assert.equal(widthAfterKey('library', 300, 'Enter'), null);
});

test('dragging a separator follows the pointer within the limits', () => {
  assert.equal(widthAfterDrag('library', 272, 40), 312);
  assert.equal(widthAfterDrag('inspector', 320, -40), 360);
  assert.equal(widthAfterDrag('inspector', 320, 400), PANELS.inspector.min);
  assert.equal(widthAfterDrag('library', 272, 900), PANELS.library.max);
});

test('the widths are remembered within their limits, and bad storage gives the starting ones', () => {
  const storage = memory();
  assert.deepEqual(readWidths(storage), initialWidths());
  writeWidths(storage, { library: 300, inspector: 9999 });
  assert.equal(storage.data.get(STORAGE_KEY), JSON.stringify({ library: 300, inspector: PANELS.inspector.max }));
  assert.deepEqual(readWidths(storage), { library: 300, inspector: PANELS.inspector.max });
  assert.deepEqual(readWidths(memory({ [STORAGE_KEY]: '{oops' })), initialWidths());
  assert.deepEqual(readWidths(memory({ [STORAGE_KEY]: '7' })), initialWidths());
  assert.deepEqual(readWidths(memory({ [STORAGE_KEY]: '{"library":"x"}' })), initialWidths());
  assert.deepEqual(readWidths(null), initialWidths());
  const refusing = { getItem: () => { throw new Error('denied'); }, setItem: () => { throw new Error('denied'); } };
  assert.deepEqual(readWidths(refusing), initialWidths());
  assert.doesNotThrow(() => writeWidths(refusing, initialWidths()));
});

test('a narrow window takes width from the inspector first, then the library, never below their minimum', () => {
  const widths = { library: 400, inspector: 400 };
  assert.deepEqual(fitWidths(widths, 400 + 400 + STAGE_MIN), widths);
  assert.deepEqual(fitWidths(widths, 400 + 400 + STAGE_MIN - 50), { library: 400, inspector: 350 });
  assert.deepEqual(fitWidths(widths, 400 + PANELS.inspector.min + STAGE_MIN - 30), { library: 370, inspector: PANELS.inspector.min });
  assert.deepEqual(fitWidths(widths, 100), { library: PANELS.library.min, inspector: PANELS.inspector.min });
});

test('the new default does not replace a saved inspector width', () => {
  assert.deepEqual(readWidths(memory({ [STORAGE_KEY]: '{"library":272,"inspector":320}' })), { library: 272, inspector: 320 });
});
