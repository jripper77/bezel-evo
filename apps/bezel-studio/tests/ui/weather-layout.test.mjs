import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { WEATHER_LAYOUT, layoutPatch, moveLine, weatherKeys, weatherLayout, weatherSegments, weatherText } from '../../src/weather-format.js';
import { partStylePatch } from '../../src/inspector-edit.js';
import { partPieces } from '../../src/editor/handles.js';
import { createStore } from '../../src/editor/store.js';
import { DEMO_THEME } from '../../src/demo-theme.js';

const roma = { type: 'weather', city: 'Roma', latitude: 41.9, longitude: 12.5, language: 'it' };

test('without a layout the weather reads as it always did', () => {
  assert.equal(weatherText(roma, 20, 3), 'Roma\n20°C\nNuvoloso');
  assert.equal(weatherLayout({ ...roma, layout: null }), WEATHER_LAYOUT);
});

test('the label, the order, hidden and joined lines shape the text like the backend', () => {
  const content = { ...roma, label: ' Casa ', layout: [{ field: 'temperature' }, { field: 'condition', inline: true }, { field: 'city' }] };
  assert.deepEqual(weatherSegments(content, 20.4, 0), [
    { field: 'temperature', text: '20°C' }, { field: null, text: ' ' },
    { field: 'condition', text: 'Sereno' }, { field: null, text: '\n' }, { field: 'city', text: 'Casa' },
  ]);
  const first = { ...content, layout: [{ field: 'temperature', hidden: true }, ...content.layout.slice(1)] };
  assert.equal(weatherText(first, 20, 0), 'Sereno\nCasa', 'a first field written inline has nothing to follow');
  assert.equal(weatherText({ ...content, label: '  ' }, 20, 0).endsWith('Roma'), true, 'a blank label shows the city');
});

test('the inspector saves no layout for the plain one and moves lines', () => {
  assert.equal(layoutPatch(WEATHER_LAYOUT.map(line => ({ ...line, hidden: false }))), null);
  const moved = moveLine(WEATHER_LAYOUT, 2, 0);
  assert.deepEqual(moved.map(line => line.field), ['condition', 'city', 'temperature']);
  assert.deepEqual(layoutPatch([{ ...moved[0], inline: true }, { ...moved[1], hidden: false }, { ...moved[2], hidden: true }]),
    [{ field: 'condition', inline: true }, { field: 'city' }, { field: 'temperature', hidden: true }]);
  assert.deepEqual(moveLine(WEATHER_LAYOUT, 0, 1).map(line => line.field), ['temperature', 'city', 'condition']);
});

test('weather keys match the backend and parts are styled one by one', () => {
  assert.deepEqual(weatherKeys(roma), ['weather.41.900000:12.500000.temperature', 'weather.41.900000:12.500000.code']);
  assert.match(readFileSync(new URL('../../../../crates/bezel-core/src/domain/sensor.rs', import.meta.url), 'utf8'), /weather\.\{latitude:\.6\}:\{longitude:\.6\}/);
  assert.deepEqual(partStylePatch({}, 'temperature', { color: '#ff0000ff' }), { parts: { temperature: { color: '#ff0000ff' } } });
  const element = { kind: { type: 'text', content: { ...roma, layout: [{ field: 'city', hidden: true }, { field: 'temperature' }, { field: 'condition', inline: true }] }, style: { parts: { temperature: { weight: 700 } } } } };
  const { scope, pieces } = partPieces(element, { weather: { temperature: 11, code: 61 } });
  assert.equal(scope, 'weather');
  assert.deepEqual(pieces.map(piece => [piece.id, piece.text]), [['temperature', '11°C'], [null, ' '], ['condition', 'Pioggia']]);
  assert.deepEqual(pieces[0].part, { weight: 700 });
});

test('a layout survives the store as a whole, and clearing it restores the plain one', () => {
  const store = createStore(DEMO_THEME);
  store.dispatch('add', { widget: 'weather', x: 240, y: 400 });
  const id = store.getState().selection[0];
  const layout = [{ field: 'condition' }, { field: 'city', hidden: true }, { field: 'temperature', inline: true }];
  store.dispatch('update', { id, patch: { kind: { content: { layout, label: 'Casa' } } } });
  const content = () => store.getState().theme.elements.find(e => e.id === id).kind.content;
  assert.deepEqual(content().layout, layout);
  store.dispatch('update', { id, patch: { kind: { content: { layout: moveLine(layout, 2, 0) } } } });
  assert.deepEqual(content().layout.map(line => line.field), ['temperature', 'condition', 'city']);
  store.dispatch('update', { id, patch: { kind: { content: { layout: null } } } });
  assert.equal(weatherLayout(content()), WEATHER_LAYOUT);
});
