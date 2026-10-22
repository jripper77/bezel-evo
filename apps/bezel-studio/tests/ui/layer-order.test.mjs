import { test } from 'node:test';
import assert from 'node:assert/strict';
import { orderLayers, layerOrderAvailability } from '../../src/editor/layers.js';
import { createStore } from '../../src/editor/store.js';
import { DEMO_THEME } from '../../src/demo-theme.js';
const e = (id, extra = {}) => ({ ...structuredClone(DEMO_THEME.elements[0]), id, name: `Object ${id}`, locked: false, ...extra });
const theme = elements => ({ ...structuredClone(DEMO_THEME), elements });
const ids = t => t.elements.map(e => e.id);

test('a multi-selection moves one sibling, preserving both selected and unselected order, with one undo', () => {
  const original = theme([1,2,3,4,5,6].map(n => e(n)));
  const s = createStore(original); s.select([2,3,5]);
  s.dispatch('stepSelection', { ids: [2,3,5], step: 1 });
  assert.deepEqual(ids(s.getState().theme), [1,4,2,3,6,5]);
  assert.deepEqual(s.getState().selection, [2,3,5]);
  s.undo(); assert.deepEqual(ids(s.getState().theme), [1,2,3,4,5,6]);
  assert.equal(s.canUndo(), false);
  s.redo(); assert.deepEqual(ids(s.getState().theme), [1,4,2,3,6,5]);
  assert.deepEqual(ids(orderLayers(original, [2,3,5], { step: -1 })), [2,3,1,5,4,6]);
});

test('edges, empty selections and invalid directions are no-ops with accurate availability', () => {
  const t = theme([e(1), e(2), e(3)]);
  assert.equal(orderLayers(t, [3], { step: 1 }), t);
  assert.equal(orderLayers(t, [1], { step: -1 }), t);
  assert.equal(orderLayers(t, [1,2,3], { front: true }), t);
  assert.equal(orderLayers(t, [99], { step: 1 }), t);
  assert.equal(orderLayers(t, [1], { step: 0 }), t);
  assert.deepEqual(layerOrderAvailability(t, [3]), { front: false, forward: false, backward: true, back: true });
  assert.deepEqual(layerOrderAvailability(t, [1,2,3]), { front: false, forward: false, backward: false, back: false });
  assert.deepEqual(ids(orderLayers(t, [1,3], { front: true })), [2,1,3]);
  assert.deepEqual(ids(orderLayers(t, [1,3], { front: false })), [1,3,2]);
});

test('groups and Cards travel with descendants, selected descendants are not moved twice', () => {
  const t = theme([e(1, { isGroup: true }), e(2, { groupParent: 1 }), e(3, { groupParent: 1 }), e(4), e(5)]);
  assert.deepEqual(ids(orderLayers(t, [1,2], { step: 1 })), [4,1,2,3,5]);
  assert.deepEqual(ids(orderLayers(t, [1], { front: true })), [4,5,1,2,3]);
  assert.deepEqual(ids(orderLayers(t, [2], { front: true })), [1,3,2,4,5]);
  const card = theme([e(10, { card: { faces: ['A','B'], activeFace: 0 } }), e(11, { cardMember: { parent: 10, face: 0 } }), e(12), e(13)]);
  assert.deepEqual(ids(orderLayers(card, [10,11], { step: 1 })), [12,10,11,13]);
});

test('Card members stay among siblings of the same face; a single-member face cannot move', () => {
  const t = theme([e(1, { card: { faces: ['A','B'], activeFace: 0 } }), e(2, { cardMember: { parent: 1, face: 0 } }), e(3, { cardMember: { parent: 1, face: 1 } }), e(4, { cardMember: { parent: 1, face: 0 } }), e(5)]);
  const result = orderLayers(t, [2], { step: 1 });
  assert.deepEqual(ids(result), [1,3,4,2,5]);
  assert.deepEqual(result.elements.find(e => e.id === 2).cardMember, { parent: 1, face: 0 });
  assert.equal(orderLayers(t, [3], { front: true }), t);
  assert.equal(orderLayers(t, [4], { step: 1 }), t);
});

test('locked objects and locked containers cannot reorder; unlocked selections still work', () => {
  const t = theme([e(1, { isGroup: true, locked: true }), e(2, { groupParent: 1 }), e(3, { groupParent: 1 }), e(4, { locked: true }), e(5), e(6)]);
  for (const selection of [[1], [2], [2,3], [4]]) {
    assert.equal(orderLayers(t, selection, { front: true }), t);
    assert.equal(orderLayers(t, selection, { step: -1 }), t);
  }
  const result = orderLayers(t, [2,5], { step: 1 });
  assert.deepEqual(ids(result), [1,2,3,4,6,5]);
  for (const item of result.elements) assert.equal(item, t.elements.find(e => e.id === item.id));
});
