import test from 'node:test';
import assert from 'node:assert/strict';
import { canRestartProbe, probeOf, probeRows, probeTotals, readingState } from '../../src/sensor-probes.js';
import { sensorChip } from '../../src/libre-status.js';
import { catalogKeysInView, groupSensors } from '../../src/ui/library.js';

const catalog = [
  { key: 'cpu.usage', category: 'cpu', label: 'CPU', source: 'sysinfo', quantity: 'percent' },
  { key: 'system.hostname', category: 'system', label: 'Host', source: 'Windows', quantity: 'text' },
  { key: 'cpu.temperature', category: 'cpu', label: 'Temperature', source: 'LibreHardwareMonitor Temperature /cpu/0', quantity: 'celsius' },
  { key: 'board.fan', category: 'board', label: 'Fan', source: 'LibreHardwareMonitor Fan /board/0', quantity: 'rpm' },
  { key: 'net.ping', category: 'network', label: 'Ping', source: 'Windows', quantity: 'milliseconds' },
];
const readings = { 'cpu.usage': { value: 0 }, 'system.hostname': { text: 'workstation' }, 'cpu.temperature': { value: 45, unavailable: 'stale' }, 'board.fan': { value: 0 } };

test('GPU aliases and unsupported metrics belong to their actual reader, including multiple adapters', () => {
  const gpu = [
    { key: 'gpu.usage', source: 'gpu.0.usage (NVML)', category: 'gpu' },
    { key: 'gpu.voltage', source: 'gpu.0.voltage', category: 'gpu' },
    { key: 'gpu.fan', source: 'gpu.0.fan (NVML)', category: 'gpu' },
    { key: 'gpu.0.usage', source: 'NVML', category: 'gpu' },
    { key: 'gpu.1.usage', source: 'Windows', category: 'gpu' },
    { key: 'gpu.1.temperature.alias', source: 'gpu.1.temperature', category: 'gpu' },
    { key: 'gpu.unknown', source: 'gpu.2.voltage', category: 'gpu' },
  ];
  const values = Object.fromEntries(gpu.map(s => [s.key, s.source.endsWith('voltage') ? { unavailable: 'not supported' } : { value: 0 }]));
  assert.deepEqual(probeRows(gpu, values).map(({ id, total, unavailable }) => ({ id, total, unavailable })), [
    { id: 'NVML', total: 4, unavailable: 1 }, { id: 'Windows', total: 2, unavailable: 0 }, { id: 'unassigned', total: 1, unavailable: 1 },
  ]);
  assert.deepEqual(groupSensors(gpu, '', 'all', { probe: 'NVML', status: 'unavailable', readings: values })[0][1].map(s => s.key), ['gpu.voltage']);
  assert.deepEqual(catalogKeysInView(gpu, '', true, 'all', { probe: 'Windows' }), ['gpu.1.usage', 'gpu.1.temperature.alias']);
  assert.equal(canRestartProbe('LibreHardwareMonitor'), true);
  for (const id of ['Windows', 'NVML', 'unassigned']) assert.equal(canRestartProbe(id), false);
});

test('one row per probe merges Windows sources and Libre device paths, with exact counts', () => {
  const rows = probeRows(catalog, readings);
  assert.deepEqual(rows, [
    { id: 'Windows', total: 3, available: 2, unavailable: 0, pending: 1, state: 'ok' },
    { id: 'LibreHardwareMonitor', total: 2, available: 1, unavailable: 1, pending: 0, state: 'partial' },
  ]);
  assert.deepEqual(probeTotals(rows), { total: 5, available: 3, unavailable: 1, pending: 1 });
  assert.equal(probeOf({ source: 'none' }), 'unassigned');
  assert.equal(probeOf({ source: 'NVML' }), 'NVML');
});

test('zero and text work, absent readings wait, stale/invalid readings fail and sample errors override old values', () => {
  assert.equal(readingState(catalog[0], readings), 'available');
  assert.equal(readingState(catalog[1], readings), 'available');
  assert.equal(readingState(catalog[2], readings), 'unavailable');
  assert.equal(readingState(catalog[4], readings), 'pending');
  assert.equal(readingState(catalog[0], { 'cpu.usage': { value: Infinity } }), 'unavailable');
  assert.equal(readingState(catalog[0], readings, true), 'unavailable');
  assert.equal(probeRows(catalog, null)[0].state, 'pending');
  assert.equal(probeRows(catalog, readings, true)[0].state, 'error');
});

test('probe, state, category and search filters compose and recovered sensors leave the failed view', () => {
  const keys = options => groupSensors(catalog, '', 'all', options).flatMap(([, items]) => items.map(s => s.key));
  assert.deepEqual(keys({ probe: 'LibreHardwareMonitor', status: 'unavailable', readings }), ['cpu.temperature']);
  assert.deepEqual(keys({ status: 'pending', readings }), ['net.ping']);
  assert.deepEqual(groupSensors(catalog, 'Fan', 'cpu', { probe: 'LibreHardwareMonitor' }), []);
  assert.deepEqual(keys({ probe: 'LibreHardwareMonitor', status: 'unavailable', readings: { ...readings, 'cpu.temperature': { value: 44 } } }), []);
  assert.deepEqual(catalogKeysInView(catalog, '', true, 'all', { probe: 'LibreHardwareMonitor', status: 'unavailable', readings }), ['cpu.temperature', 'board.fan'], 'state filtering must not prevent recovery or detection of new failures');
  assert.deepEqual(catalogKeysInView(catalog, '', false, 'all', { probe: 'Windows' }), []);
});

test('status chip counts actual failed sensors and only offers quick restart for stopped Libre', () => {
  const counts = probeTotals(probeRows(catalog, readings));
  // The chip says how many read of all (T-9b): 3 of 5, one of them failed.
  assert.deepEqual(sensorChip({ counts }).params, { available: 3, total: 5, ms: 0 });
  assert.equal(counts.unavailable, 1);
  assert.equal(sensorChip({ counts }).state, 'partial');
  assert.equal(sensorChip({ counts, health: { state: 'error' } }).quickRestart, true);
  assert.equal(sensorChip({ counts: probeTotals(probeRows(catalog, null)) }).state, 'pending');
  assert.equal(sensorChip({ counts: probeTotals(probeRows(catalog, readings, true)) }).state, 'error');
});
