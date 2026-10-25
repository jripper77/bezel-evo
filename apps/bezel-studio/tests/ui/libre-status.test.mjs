import test from 'node:test';
import assert from 'node:assert/strict';
import { hardwareRows, libreStatus, sensorChip } from '../../src/libre-status.js';

const catalog = [
  { key: 'cpu.temperature', category: 'cpu', label: 'CPU Package', source: 'LibreHardwareMonitor', quantity: 'celsius' },
  { key: 'lhm.psu.corsair.0.voltage.3', category: 'board', label: 'Corsair Input', source: 'LibreHardwareMonitor', quantity: 'volts' },
  { key: 'lhm.psu.corsair.0.power.14', category: 'board', label: 'Corsair Total Output', source: 'LibreHardwareMonitor', quantity: 'watts' },
];

test('a working CPU cannot hide a failed Corsair reporting a synthetic zero', () => {
  const health = libreStatus(catalog, {
    'cpu.temperature': { value: 53 },
    'lhm.psu.corsair.0.voltage.3': { unavailable: 'No reply' },
    'lhm.psu.corsair.0.power.14': { value: 0 },
  });
  assert.equal(health.state, 'partial');
  assert.deepEqual(health.failed, ['Corsair Input']);
  assert.deepEqual(health.hardware, [{ id: 'cpu', name: 'CPU Package', category: 'cpu', ok: true }, { id: 'lhm.psu.corsair.0', name: 'Corsair Input', category: 'board', ok: false }]);
});

test('missing/stale readings fail; legitimate zero measurements are healthy', () => {
  assert.equal(libreStatus(catalog, null).state, 'error');
  assert.equal(libreStatus(catalog, {
    'cpu.temperature': { value: 53, unavailable: 'Stale' },
    'lhm.psu.corsair.0.voltage.3': { value: Number.NaN },
  }).state, 'error');
  assert.equal(libreStatus(catalog, {
    'cpu.temperature': { value: 0 },
    'lhm.psu.corsair.0.voltage.3': { value: 230 },
  }).state, 'ok');
  assert.equal(libreStatus([], {}), null);
});

test('the sensors chip: green with the time, amber with the count, red, restarting', () => {
  assert.deepEqual(sensorChip({ health: null, sampleMillis: 3.4 }), { state: 'ok', key: 'sensorChip.ok', params: { ms: 3 }, quickRestart: false });
  assert.deepEqual(sensorChip({ health: { state: 'ok', failed: [] } }).params, { ms: 0 });
  assert.deepEqual(sensorChip({ health: { state: 'partial', failed: ['PSU'] } }), { state: 'partial', key: 'sensorChip.missingOne', params: { count: 1 }, quickRestart: false });
  assert.equal(sensorChip({ health: { state: 'partial', failed: ['PSU', 'Fans'] } }).key, 'sensorChip.missingMany');
  // Libre stopped: the restart comes beside the chip; without Libre there is none to restart.
  assert.deepEqual(sensorChip({ health: { state: 'error', failed: ['PSU'] } }), { state: 'error', key: 'sensorChip.stopped', params: {}, quickRestart: true });
  assert.equal(sensorChip({ health: null, error: new Error('x') }).quickRestart, false);
  assert.equal(sensorChip({ health: { state: 'ok', failed: [] }, error: new Error('x') }).state, 'error');
  assert.equal(sensorChip({ health: { state: 'error', failed: [] }, restarting: true }).state, 'restarting');
});

test('with counts the chip says how many sensors read of all, amber while some do not', () => {
  assert.deepEqual(sensorChip({ health: null, counts: { available: 21, unavailable: 0, pending: 0 }, sampleMillis: 3.4 }), { state: 'ok', key: 'sensorChip.active', params: { available: 21, total: 21, ms: 3 }, quickRestart: false });
  assert.deepEqual(sensorChip({ health: { state: 'partial', failed: ['PSU'] }, counts: { available: 20, unavailable: 1, pending: 0 } }), { state: 'partial', key: 'sensorChip.active', params: { available: 20, total: 21, ms: 0 }, quickRestart: false });
  assert.equal(sensorChip({ health: { state: 'error', failed: ['PSU'] }, counts: { available: 0, unavailable: 3, pending: 0 } }).state, 'error');
  assert.equal(sensorChip({ health: null, counts: { available: 0, unavailable: 0, pending: 4 } }).key, 'sensorChip.pending');
});

test('a row per hardware: sources, the best value and the Libre groups missing', () => {
  const all = [
    { key: 'cpu.usage', category: 'cpu', source: 'sysinfo', quantity: 'percent' },
    ...catalog,
    { key: 'net.down', category: 'network', source: 'sysinfo', quantity: 'bytesPerSecond' },
    { key: 'disk.temp', category: 'disk', source: 'hwmon', quantity: 'celsius' },
  ];
  const readings = {
    'cpu.usage': { value: 30, display: '30%' },
    'cpu.temperature': { value: 53, display: '53 °C' },
    'lhm.psu.corsair.0.voltage.3': { unavailable: 'No reply' },
    'lhm.psu.corsair.0.power.14': { value: 0, display: '0 W' },
    'net.down': { value: 12 },
  };
  const rows = hardwareRows(all, readings, libreStatus(all, readings));
  assert.deepEqual(rows.map((r) => r.category), ['cpu', 'board', 'network', 'disk']);
  assert.deepEqual(rows[0], { category: 'cpu', sources: ['sysinfo', 'LibreHardwareMonitor'], count: rows[0].count, value: '53 °C', missing: [] });
  assert.ok(rows[0].count > 0);
  // The synthetic zero still reads, but the group it belongs to is missing.
  assert.deepEqual(rows[1], { category: 'board', sources: ['LibreHardwareMonitor'], count: rows[1].count, value: '0 W', missing: ['Corsair Input'] });
  assert.equal(rows[2].value, '12');
  assert.equal(rows[3].value, null);
  assert.deepEqual(hardwareRows(all, null, null).map((r) => r.value), [null, null, null, null]);
});


test('separate physical devices of one category keep their own readings and warnings', () => {
  const sensors = [
    { key: 'lhm.board.first.0.temperature.1', category: 'board', label: 'Board temperature', source: 'LibreHardwareMonitor', quantity: 'celsius' },
    { key: 'lhm.psu.corsair.0.voltage.3', category: 'board', label: 'First PSU', source: 'LibreHardwareMonitor', quantity: 'volts' },
    { key: 'lhm.psu.corsair.1.voltage.3', category: 'board', label: 'Second PSU', source: 'LibreHardwareMonitor', quantity: 'volts' },
  ];
  const readings = {
    [sensors[0].key]: { value: 40, display: '40 C' },
    [sensors[1].key]: { unavailable: 'No reply' },
    [sensors[2].key]: { value: 230, display: '230 V' },
  };
  const health = libreStatus(sensors, readings);
  assert.deepEqual(health.failed, ['First PSU']);
  const rows = hardwareRows(sensors, readings, health);
  assert.equal(rows.length, 3);
  assert.deepEqual(rows.map((row) => row.value), ['40 C', null, '230 V']);
  assert.deepEqual(rows.map((row) => row.missing), [[], ['First PSU'], []]);
  assert.equal(rows[1].name, 'First PSU');
  assert.equal(rows[2].name, 'Second PSU');
});


test('identically named hardware uses physical ids to place its missing reading', () => {
  const sensors = [0, 1].map((index) => ({
    key: `lhm.psu.corsair.${index}.voltage.3`, category: 'board', label: 'Corsair Input', source: 'LibreHardwareMonitor', quantity: 'volts',
  }));
  const readings = { [sensors[0].key]: { value: 230, display: '230 V' } };
  const health = libreStatus(sensors, readings);
  assert.deepEqual(health.hardware.map((hardware) => hardware.id), ['lhm.psu.corsair.0', 'lhm.psu.corsair.1']);
  assert.deepEqual(hardwareRows(sensors, readings, health).map((row) => row.missing), [[], ['Corsair Input']]);
});

test('the popover names the reading program, never the raw sensor path', async () => {
  const { sourceProgram, hardwareRows } = await import('../../src/libre-status.js');
  assert.equal(sourceProgram('LibreHardwareMonitor Voltage /lpc/nct6687dr/0/voltage/11'), 'LibreHardwareMonitor');
  assert.equal(sourceProgram('sysinfo'), 'sysinfo');
  assert.equal(sourceProgram('Intel PresentMon (on demand)'), 'PresentMon');
  assert.equal(sourceProgram('none'), null);
  assert.equal(sourceProgram(''), null);
  const catalog = [
    { key: 'a', category: 'board', source: 'LibreHardwareMonitor Fan /lpc/nct6687dr/0/fan/0', quantity: 'rpm', label: 'Fan 1' },
    { key: 'b', category: 'board', source: 'LibreHardwareMonitor Voltage /lpc/nct6687dr/0/voltage/1', quantity: 'volts', label: 'Vcore' },
    { key: 'c', category: 'system', source: 'none', quantity: 'seconds' },
  ];
  const rows = hardwareRows(catalog, {}, null);
  assert.deepEqual(rows.map(r => [r.category, r.sources, r.count]), [['board', ['LibreHardwareMonitor'], 2], ['system', [], 1]]);
});
