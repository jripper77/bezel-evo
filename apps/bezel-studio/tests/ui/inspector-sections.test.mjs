import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createSectionMemory, SECTION_ORDER, SECTION_STORAGE_KEY, summaryOf } from '../../src/inspector-sections.js';

function storageWith(value = null) {
  const entries = new Map([[SECTION_STORAGE_KEY, value]]);
  return { getItem: key => entries.get(key), setItem: (key, next) => entries.set(key, next) };
}

test('section order is fixed, unique and immutable, including Card sections', () => {
  assert.deepEqual(SECTION_ORDER, ['layout', 'data', 'shape', 'text', 'color', 'behavior', 'faces', 'motion', 'triggers']);
  assert.equal(new Set(SECTION_ORDER).size, SECTION_ORDER.length);
  assert.ok(Object.isFrozen(SECTION_ORDER));
});

test('sections toggle independently, per kind, without changing other widgets', () => {
  const memory = createSectionMemory({ storage: null });
  assert.equal(memory.isOpen('ring', 'data'), true);
  assert.equal(memory.isOpen('ring', 'color'), false);
  assert.equal(memory.isOpen('card', 'faces'), true);
  assert.equal(memory.toggle('ring', 'color'), true);
  assert.equal(memory.isOpen('ring', 'data'), true);
  assert.equal(memory.isOpen('bar', 'color'), false);
  assert.equal(memory.toggle('ring', 'data'), false);
  assert.equal(memory.isOpen('ring', 'color'), true);
  assert.equal(memory.toggle('ring', 'color'), false);
  assert.equal(memory.isOpen('ring', 'color'), false);
  assert.equal(memory.isOpen('ring', 'missing'), false);
  assert.equal(memory.toggle('ring', 'missing'), false);
});

test('Alt+click opens only its section, including when that section was already open', () => {
  const memory = createSectionMemory({ storage: null });
  memory.toggle('ring', 'color');
  assert.equal(memory.toggle('ring', 'color', { exclusive: true }), true);
  assert.deepEqual(SECTION_ORDER.filter(id => memory.isOpen('ring', id)), ['color']);
  memory.toggle('ring', 'layout', { exclusive: true });
  assert.deepEqual(SECTION_ORDER.filter(id => memory.isOpen('ring', id)), ['layout']);
  assert.equal(memory.isOpen('bar', 'data'), true);
});

test('disclosures survive a new session and tolerate hostile widget names', () => {
  const storage = storageWith();
  const memory = createSectionMemory({ storage });
  memory.toggle('ring', 'data');
  memory.toggle('ring', 'shape');
  memory.toggle('__proto__', 'layout');
  memory.toggle('constructor', 'color');
  const next = createSectionMemory({ storage });
  assert.equal(next.isOpen('ring', 'data'), false);
  assert.equal(next.isOpen('ring', 'shape'), true);
  assert.equal(next.isOpen('__proto__', 'layout'), true);
  assert.equal(next.isOpen('constructor', 'color'), true);
  assert.equal(next.isOpen('bar', 'shape'), false);
});

test('corrupt and obsolete preferences never prevent editing', () => {
  for (const value of ['not json', 'null', '[]', 'true', '42', '"text"']) {
    const memory = createSectionMemory({ storage: storageWith(value) });
    assert.equal(memory.isOpen('ring', 'data'), true);
    assert.equal(memory.toggle('ring', 'shape'), true);
  }
  const storage = storageWith(JSON.stringify({ ring: { color: true, data: 'false', obsolete: true }, bar: [], clock: null, weather: false }));
  const memory = createSectionMemory({ storage });
  assert.equal(memory.isOpen('ring', 'color'), true);
  assert.equal(memory.isOpen('ring', 'data'), true);
  assert.equal(memory.isOpen('ring', 'obsolete'), false);
  assert.equal(memory.isOpen('clock', 'data'), true);
});

test('denied storage retains working session memory', () => {
  const storage = { getItem() { throw Error('denied'); }, setItem() { throw Error('quota'); } };
  const memory = createSectionMemory({ storage });
  assert.equal(memory.toggle('ring', 'shape'), true);
  assert.equal(memory.isOpen('ring', 'shape'), true);
  assert.equal(memory.toggle('ring', 'shape'), false);
});

test('default storage and a denied global storage getter are both supported', () => {
  const previous = Object.getOwnPropertyDescriptor(globalThis, 'localStorage');
  try {
    Object.defineProperty(globalThis, 'localStorage', { configurable: true, value: storageWith() });
    createSectionMemory().toggle('value', 'text');
    assert.equal(createSectionMemory().isOpen('value', 'text'), true);
    Object.defineProperty(globalThis, 'localStorage', { configurable: true, get() { throw Error('denied'); } });
    assert.equal(createSectionMemory().toggle('ring', 'text'), true);
    Object.defineProperty(globalThis, 'localStorage', { configurable: true, value: undefined });
    assert.equal(createSectionMemory().toggle('ring', 'text'), true);
  } finally {
    if (previous) Object.defineProperty(globalThis, 'localStorage', previous);
    else delete globalThis.localStorage;
  }
});

test('an explicit preference key stays isolated', () => {
  const storage = storageWith();
  createSectionMemory({ storage, key: 'custom' }).toggle('ring', 'shape');
  assert.equal(createSectionMemory({ storage, key: 'custom' }).isOpen('ring', 'shape'), true);
  assert.equal(createSectionMemory({ storage }).isOpen('ring', 'shape'), false);
});

test('summaries use the language, units and actual values, with no missing readouts', () => {
  assert.equal(summaryOf([-135, { value: 270, unit: '°' }, { value: 24, unit: 'px' }]), '-135 · 270° · 24 px');
  assert.equal(summaryOf(['IBM Plex Mono', { value: 22, unit: 'px' }, 'centro']), 'IBM Plex Mono · 22 px · centro');
  assert.equal(summaryOf([12345.5, { value: .25, unit: 's' }, { value: 50, unit: '%' }], { locale: 'it' }), '12.345,5 · 0,25 s · 50%');
  assert.equal(summaryOf([1234.5], { locale: 'pt-BR' }), '1.234,5');
  assert.equal(summaryOf([0, { value: 1 }, '', null, undefined, false, NaN, Infinity, {}, { value: NaN }]), '0 · 1');
  assert.equal(summaryOf([], { separator: ' / ' }), '');
  assert.equal(summaryOf([1, 2], { separator: ' / ' }), '1 / 2');
});
