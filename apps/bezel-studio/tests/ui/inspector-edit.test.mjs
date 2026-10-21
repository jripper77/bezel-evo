import { test } from 'node:test';
import assert from 'node:assert/strict';
import { commonValue, commonSelection, selectionPatches, partStylePatch, MIXED } from '../../src/inspector-edit.js';
import { createStore } from '../../src/editor/store.js';
import { createWidget } from '../../src/editor/widgets.js';
import { DEMO_THEME } from '../../src/demo-theme.js';

const made = (widget, id) => ({ ...createWidget(widget, DEMO_THEME.canvas), id, name: widget, frame: { x: 10, y: id * 20, width: 120, height: 40 } });
test('parts inherit independently without losing other overrides', () => {
  const style = { parts: { unit: { color: '#123456ff', baseline: 'super' } } };
  assert.deepEqual(partStylePatch(style, 'unit', { sizePercent: 60 }), { parts: { unit: { color: '#123456ff', baseline: 'super', sizePercent: 60 } } });
  assert.deepEqual(partStylePatch(style, 'unit', null), { parts: { unit: null } });
  assert.deepEqual(partStylePatch(style, 'unknown', {}), {});
  assert.equal(style.parts.unit.sizePercent, undefined);
});
test('multi selection exposes only common capabilities and marks different values', () => {
  const elements = [made('ring', 1), made('bar', 2), made('value', 3)];
  const shared = commonSelection(elements);
  assert.equal(shared.frame.x, 10); assert.equal(shared.frame.y, MIXED);
  assert.equal(shared.sensor, 'cpu.usage'); assert.equal(shared.paint, MIXED);
  assert.equal(commonSelection([...elements, made('image', 4)]).paint, null);
  assert.equal(commonSelection([...elements, made('text', 4)]).sensor, null);
  assert.equal(commonValue([]), MIXED);
  assert.equal(commonValue([null, null]), null);
});
test('different paint paths and bulk geometry are one undo step, respecting locks', () => {
  const elements = [made('ring', 1), made('bar', 2), made('value', 3), { ...made('needle', 4), locked: true }];
  const store = createStore({ ...DEMO_THEME, elements });
  const original = store.getState().theme;
  store.dispatch('patchElements', { patches: selectionPatches(elements, 'paint', '#112233ff') });
  let edited = store.getState().theme.elements;
  assert.equal(edited[0].kind.fill, '#112233ff'); assert.equal(edited[1].kind.fill, '#112233ff'); assert.equal(edited[2].kind.style.paint, '#112233ff');
  assert.equal(edited[3].kind.color, elements[3].kind.color);
  assert.deepEqual(edited[0].kind.binding, elements[0].kind.binding);
  store.undo(); assert.deepEqual(store.getState().theme, original);
  store.dispatch('patchElements', { patches: selectionPatches(elements, 'width', 250) });
  edited = store.getState().theme.elements;
  assert.deepEqual(edited.slice(0, 3).map(e => [e.frame.width, e.frame.height]), [[250,40],[250,40],[250,40]]);
  assert.equal(edited[3].frame.width, 120);
  store.undo(); assert.deepEqual(store.getState().theme, original);
  assert.deepEqual(selectionPatches(elements, 'unknown', 1), []);
  assert.equal(selectionPatches(elements, 'opacity', 2)[0].patch.opacity, 1);
  assert.equal(selectionPatches(elements, 'height', -2)[0].patch.frame.height, 4);
  assert.deepEqual(selectionPatches([elements[2]], 'sensor', 'gpu.usage')[0].patch, { kind: { content: { key: 'gpu.usage' } } });
  store.dispatch('patchElements', { patches: [{ id: 4, patch: { kind: { color: '#abcdef88' } } }], respectLocks: false });
  assert.equal(store.getState().theme.elements[3].kind.color, '#abcdef88', 'explicit inspector property edits keep the existing lock semantics');
});

test('rotation is common only when every selected object can rotate', () => {
  const objects = [made('ring', 1), made('bar', 2)];
  const original = structuredClone(objects);
  assert.equal(commonSelection(objects).rotation, 0);
  assert.equal(commonSelection([objects[0], { ...objects[1], rotation: 30 }]).rotation, MIXED);
  const excluded = [made('card', 3), { ...made('shape', 3), isGroup: true }, { ...made('ring', 3), groupParent: 9 }, { ...made('ring', 3), cardMember: { parent: 9, face: 0 } }, { ...made('shape', 3), kind: { type: 'shape', videoWindow: true } }];
  for (const object of excluded) assert.equal(commonSelection([...objects, object]).rotation, null);
  assert.equal(commonSelection([]).rotation, null);
  assert.deepEqual(objects, original);
});

test('bulk rotation normalizes angles, respects locks, omits zero and undoes once', () => {
  const elements = [made('ring', 1), made('bar', 2), { ...made('needle', 3), locked: true }];
  const store = createStore({ ...DEMO_THEME, elements });
  const original = structuredClone(store.getState().theme);
  store.dispatch('rotateSelection', { ids: [1, 2, 3], rotation: 390 });
  assert.deepEqual(store.getState().theme.elements.map(e => e.rotation ?? 0), [30, 30, 0]);
  store.undo(); assert.deepEqual(store.getState().theme, original);
  store.redo();
  store.dispatch('rotateSelection', { ids: [1, 2], rotation: 360 });
  assert.ok(store.getState().theme.elements.every(e => !Object.hasOwn(e, 'rotation')));
  const withCard = createStore({ ...DEMO_THEME, elements: [...elements, made('card', 4)] });
  const before = structuredClone(withCard.getState().theme);
  withCard.dispatch('rotateSelection', { ids: [1, 4], rotation: 30 });
  assert.deepEqual(withCard.getState().theme, before);
  withCard.dispatch('rotateSelection', { ids: [999], rotation: 30 });
  assert.deepEqual(withCard.getState().theme, before);
});
