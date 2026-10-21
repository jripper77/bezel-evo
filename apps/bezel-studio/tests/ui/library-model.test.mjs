import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  WIDGET_COUNT, WIDGET_GROUPS, countLayers, createHistory, filterLayers, layerMeta, matchesQuery, mediaTotals,
  sensorDropTarget, sharedSource, sparkPath, updateSeconds, widgetGroups,
} from '../../src/library-model.js';
import { createWidget, WIDGETS } from '../../src/editor/widgets.js';
import { layerGroups } from '../../src/editor/layers.js';

const LABELS = { text: 'Testo', value: 'Valore', clock: 'Orologio', weather: 'Meteo', ring: 'Anello', bar: 'Barra', needle: 'Lancetta', graph: 'Grafico', image: 'Immagine', player: 'Media player', spectrum: 'Spettro audio', shape: 'Forma', card: 'Card', group: 'Gruppo' };
const labelOf = (w) => LABELS[w];

test('the widget groups hold every widget once, plus the Group tile', () => {
  const all = WIDGET_GROUPS.flatMap((g) => g.widgets);
  assert.deepEqual(WIDGET_GROUPS.map((g) => g.id), ['values', 'timeMedia', 'textShapes', 'containers']);
  assert.deepEqual([...all].sort(), [...WIDGETS, 'group'].sort());
  assert.equal(new Set(all).size, all.length);
  assert.equal(WIDGET_COUNT, 14);
});

test('the widget search matches names and ids, ignoring case and accents, and drops empty groups', () => {
  assert.deepEqual(widgetGroups(labelOf).map((g) => g.widgets.length), [5, 4, 3, 2]);
  assert.deepEqual(widgetGroups(labelOf, ' BARRA '), [{ id: 'values', widgets: ['bar'] }]);
  assert.deepEqual(widgetGroups(labelOf, 'player'), [{ id: 'timeMedia', widgets: ['player'] }]);
  assert.deepEqual(widgetGroups(labelOf, 'media pl'), [{ id: 'timeMedia', widgets: ['player'] }]);
  assert.deepEqual(widgetGroups(labelOf, 'nothing'), []);
  assert.ok(matchesQuery('meteo', 'Météo'));
  assert.ok(matchesQuery('', 'anything'));
  assert.ok(!matchesQuery('cpu gpu', 'cpu only'));
});

test('the history keeps the last numbers of each sensor and when samples came', () => {
  const h = createHistory(3);
  h.push({ a: { value: 1 }, b: { unavailable: 'no' } }, 1000);
  h.push({ a: { value: 2 }, b: { value: 5 } }, 2000);
  h.push({ a: { value: Number.NaN } }, null);
  h.push({ a: { value: 3 } }, 3000);
  h.push({ a: { value: 4 } }, 4000);
  h.push(null, 5000);
  assert.deepEqual(h.values('a'), [2, 3, 4]);
  assert.deepEqual(h.values('b'), [5]);
  assert.deepEqual(h.values('missing'), []);
  assert.deepEqual(h.times(), [3000, 4000, 5000]);
});

test('the update rate is the median gap between samples, in whole seconds', () => {
  assert.equal(updateSeconds([]), null);
  assert.equal(updateSeconds([1000]), null);
  assert.equal(updateSeconds([1000, 1000]), null);
  assert.equal(updateSeconds([0, 1000, 2100, 3000]), 1);
  assert.equal(updateSeconds([0, 2000, 4000, 9000]), 2);
  assert.equal(updateSeconds([0, 300]), 1, 'never under a second');
  assert.equal(updateSeconds([0, 1000, 3000, 6000]), 2, 'an even count averages the middle gaps');
});

test('a sparkline spans its box; a flat one runs through the middle', () => {
  assert.equal(sparkPath([], 60), null);
  assert.equal(sparkPath([5, Number.NaN], 60), null);
  assert.equal(sparkPath([0, 10], 60), 'M0 18 L60 2');
  assert.equal(sparkPath([0, 5, 10], 40, 10, 0), 'M0 10 L20 5 L40 0');
  assert.equal(sparkPath([7, 7, 7], 60), 'M0 10 L30 10 L60 10');
});

test('a sensor group names its source only when all of it shares one', () => {
  assert.equal(sharedSource([{ source: 'hwmon k10temp' }, { source: 'hwmon k10temp' }]), 'hwmon k10temp');
  assert.equal(sharedSource([{ source: 'a' }, { source: 'b' }]), null);
  assert.equal(sharedSource([{}]), null);
});

const frame = (x, y, width = 100, height = 100) => ({ x, y, width, height });
const ring = (id, extra = {}) => ({ id, name: `R${id}`, frame: frame(0, 0), kind: { type: 'ring', binding: { key: 'cpu.usage', min: 0, max: 100 } }, ...extra });
const value = (id, extra = {}) => ({ id, name: `V${id}`, frame: frame(0, 0), kind: { type: 'text', content: { type: 'sensor', key: 'cpu.usage' } }, ...extra });
const range = () => ({ min: 20, max: 100 });
const sensor = { key: 'gpu.temperature', quantity: 'celsius' };

test('a sensor dropped on an object that reads one binds the topmost there', () => {
  const els = [ring(1), value(2)];
  assert.deepEqual(sensorDropTarget(els, { x: 50, y: 50 }, sensor, range), { id: 2, patch: { kind: { content: { key: 'gpu.temperature' } } } });
  assert.deepEqual(sensorDropTarget([ring(1)], { x: 50, y: 50 }, sensor, range), { id: 1, patch: { kind: { binding: { key: 'gpu.temperature', min: 20, max: 100 } } } });
  assert.equal(sensorDropTarget(els, { x: 150, y: 50 }, sensor, range), null, 'outside every object');
});

test('a sensor drop skips hidden, locked, off-face and sensorless objects', () => {
  const text = { id: 3, name: 'T', frame: frame(0, 0), kind: { type: 'text', content: { type: 'static', text: 'x' } } };
  assert.equal(sensorDropTarget([text], { x: 5, y: 5 }, sensor, range), null);
  assert.equal(sensorDropTarget([ring(1, { visible: false })], { x: 5, y: 5 }, sensor, range), null);
  assert.equal(sensorDropTarget([ring(1, { locked: true })], { x: 5, y: 5 }, sensor, range), null);
  const group = { id: 9, isGroup: true, locked: true, frame: frame(0, 0), kind: { type: 'shape' } };
  assert.equal(sensorDropTarget([group, ring(1, { groupParent: 9 })], { x: 5, y: 5 }, sensor, range), null, 'locked by its group');
  const card = { id: 7, card: { faces: ['A', 'B'], activeFace: 0 }, frame: frame(0, 0), kind: { type: 'shape' } };
  assert.equal(sensorDropTarget([card, ring(1, { cardMember: { parent: 7, face: 1 } })], { x: 5, y: 5 }, sensor, range), null, 'on a face not shown');
  assert.equal(sensorDropTarget([card, ring(1, { cardMember: { parent: 7, face: 0 } })], { x: 5, y: 5 }, sensor, range).id, 1);
  assert.equal(sensorDropTarget([card, ring(1, { cardMember: { parent: 7, face: null } })], { x: 5, y: 5 }, sensor, range).id, 1, 'on the shared base');
});

test('a layer row names the sensor it reads, or a card with its faces', () => {
  assert.deepEqual(layerMeta(ring(1)), { sensor: 'cpu.usage' });
  assert.deepEqual(layerMeta({ id: 2, card: { faces: ['A', 'B', 'C'] }, kind: { type: 'shape' } }), { faces: 3 });
  assert.deepEqual(layerMeta({ id: 3, isGroup: true, kind: { type: 'shape' } }), {});
  assert.deepEqual(layerMeta({ id: 4, kind: { type: 'shape' } }), {});
});

test('the Layers filter keeps matches and the containers that hold them', () => {
  const card = { id: 1, name: 'Metriche', card: { faces: ['A', 'B'], activeFace: 0 }, frame: frame(0, 0), kind: { type: 'shape' } };
  const els = [
    card,
    { ...ring(2), name: 'CPU', cardMember: { parent: 1, face: 0 } },
    { ...value(3), name: 'Upload', kind: { type: 'text', content: { type: 'sensor', key: 'net.up' } }, cardMember: { parent: 1, face: 1 } },
    { id: 4, name: 'Rete', isGroup: true, frame: frame(0, 0), kind: { type: 'shape' } },
    { ...value(5), name: 'Download', kind: { type: 'text', content: { type: 'sensor', key: 'net.down' } }, groupParent: 4 },
    { id: 6, name: 'Sfondo', frame: frame(0, 0), kind: { type: 'image', asset: 'a' } },
  ];
  const kindOf = (e) => (e.kind.type === 'image' ? 'Immagine' : 'Altro');
  const tree = layerGroups(els);
  assert.equal(filterLayers(tree, '', kindOf), tree);
  assert.equal(countLayers(tree), 6);

  const bySensor = filterLayers(tree, 'net.up', kindOf);
  assert.deepEqual(bySensor.map((n) => n.element.id), [1]);
  assert.deepEqual(bySensor[0].groups.map((g) => g.elements.map((e) => e.id)), [[], [], [3]]);
  assert.equal(countLayers(bySensor), 2);

  const inGroup = filterLayers(tree, 'download', kindOf);
  assert.deepEqual(inGroup.map((n) => [n.element.id, n.children.map((c) => c.element.id)]), [[4, [5]]]);

  assert.deepEqual(filterLayers(tree, 'rete', kindOf).map((n) => [n.element.id, n.children.length]), [[4, 1]], 'a matching container keeps all of it');
  assert.deepEqual(filterLayers(tree, 'immagine', kindOf).map((n) => n.element.id), [6], 'by kind');
  assert.deepEqual(filterLayers(tree, 'zzz', kindOf), []);
  const values = filterLayers(tree, '', kindOf, 'value');
  assert.deepEqual(values.map(n => n.element.id), [4, 1]);
  assert.deepEqual(values[0].children.map(n => n.element.id), [5]);
  assert.deepEqual(values[1].groups.map(g => g.elements.map(e => e.id)), [[], [], [3]], 'face indexes and ancestors survive type filtering');
  assert.equal(countLayers(values), 4);
  assert.deepEqual(filterLayers(tree, 'upload', kindOf, 'value').map(n => n.element.id), [1], 'search and type must both match');
  assert.deepEqual(filterLayers(tree, 'upload', kindOf, 'ring'), []);
  assert.deepEqual(filterLayers(tree, '', kindOf, 'image').map(n => n.element.id), [6]);
  assert.deepEqual(filterLayers(tree, '', kindOf, 'group').map(n => [n.element.id, n.children.length]), [[4, 1]], 'a matching container retains its subtree');
  assert.deepEqual(filterLayers(tree, '', kindOf, 'card').map(n => n.element.id), [1]);
  assert.deepEqual(filterLayers(tree, '', kindOf, 'clock'), []);
  assert.equal(countLayers(tree), 6, 'filtering never mutates the original tree');
});

test('type filtering distinguishes every text-backed widget, cards and groups', () => {
  const elements = WIDGETS.map((widget, id) => ({ id, name: widget, ...createWidget(widget, { width: 480, height: 480 }) }));
  elements.push({ id: 100, name: 'Group', isGroup: true, kind: { type: 'shape' } });
  const tree = layerGroups(elements);
  for (const type of [...WIDGETS, 'group']) {
    const matches = filterLayers(tree, '', () => '', type);
    assert.equal(countLayers(matches), 1, type);
    assert.equal(matches[0].element.name.toLowerCase(), type);
  }
});

test('the Media totals add the sizes known', () => {
  assert.deepEqual(mediaTotals([]), { count: 0, bytes: null });
  assert.deepEqual(mediaTotals([{ bytes: 100 }, {}, { bytes: 50 }]), { count: 3, bytes: 150 });
  assert.deepEqual(mediaTotals([{}, { bytes: Number.NaN }]), { count: 2, bytes: null });
});


test('a sensor drop respects both ownership relations and off-face ancestors', () => {
  const group = { id: 8, isGroup: true, frame: frame(0, 0), kind: { type: 'shape' } };
  const card = { id: 7, card: { faces: ['A', 'B'], activeFace: 0 }, frame: frame(0, 0), kind: { type: 'shape' } };
  const member = ring(1, { groupParent: 8, cardMember: { parent: 7, face: 0 } });
  assert.equal(sensorDropTarget([{ ...card, locked: true }, group, member], { x: 5, y: 5 }, sensor, range), null);
  assert.equal(sensorDropTarget([card, { ...group, cardMember: { parent: 7, face: 1 } }, member], { x: 5, y: 5 }, sensor, range), null);
  assert.equal(sensorDropTarget([member], { x: 5, y: 5 }, sensor, range), null, 'an orphan cannot bind');
});
