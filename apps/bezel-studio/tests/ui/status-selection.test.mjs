import { test } from 'node:test';
import assert from 'node:assert/strict';
import { selectionStatus } from '../../src/status-selection.js';

const t = (key, params = {}) => `${key}${Object.keys(params).length ? JSON.stringify(params) : ''}`;
const cpu = { id: 1, name: 'CPU', frame: { x: 90, y: 300, width: 300, height: 300 } };
const gpu = { id: 2, name: 'GPU', frame: { x: 400, y: 320, width: 80, height: 40 }, rotation: 15.4 };

test('nothing selected invites a click or a drag', () => {
  assert.deepEqual(selectionStatus({ elements: [cpu], selection: [] }, t), { empty: true, element: null, title: 'shell.noSelection', figures: [], snap: null, hint: null });
});

test('one object: its name, position, size and a turn when it has one; several: their common box', () => {
  const one = selectionStatus({ elements: [cpu, gpu], selection: [1] }, t);
  assert.equal(one.title, 'CPU');
  assert.equal(one.element, cpu);
  assert.deepEqual(one.figures, ['shell.position{"x":"90","y":"300"}', 'shell.size{"width":"300","height":"300"}']);
  assert.deepEqual(selectionStatus({ elements: [cpu, gpu], selection: [2] }, t).figures.at(-1), 'framing.degrees{"degrees":"15"}');
  const both = selectionStatus({ elements: [cpu, gpu], selection: [1, 2] }, t, v => `${v}!`);
  assert.equal(both.title, 'layers.count{"count":2}');
  assert.equal(both.element, null);
  assert.deepEqual(both.figures, ['shell.position{"x":"90!","y":"300!"}', 'shell.size{"width":"390!","height":"300!"}']);
});

test('a gesture says what it does, whether it snapped to a centre or an edge, and a useful key', () => {
  const move = selectionStatus({ elements: [cpu], selection: [1], gesture: { mode: 'move', guides: [{ axis: 'x', position: 240 }] } }, t);
  assert.equal(move.title, 'shell.gesture.move{"name":"CPU"}');
  assert.equal(move.snap, 'shell.snapped.centre');
  assert.equal(move.hint, 'shell.gestureHint.move');
  assert.equal(selectionStatus({ elements: [cpu], selection: [1], gesture: { mode: 'resize', guides: [{ axis: 'y', position: 600 }] } }, t).snap, 'shell.snapped.edge');
  assert.equal(selectionStatus({ elements: [cpu], selection: [1], gesture: { mode: 'move', guides: [] } }, t).snap, null);
  assert.equal(selectionStatus({ elements: [cpu], selection: [1], gesture: { mode: 'move' } }, t).snap, null);
  const turn = selectionStatus({ elements: [cpu], selection: [1], gesture: { mode: 'rotate', guides: [{ axis: 'x', position: 240 }] } }, t);
  assert.equal(turn.snap, null, 'a turn does not snap to guides');
  assert.equal(turn.hint, 'shell.gestureHint.rotate');
});
