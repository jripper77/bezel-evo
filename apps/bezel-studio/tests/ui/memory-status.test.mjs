import { test } from 'node:test';
import assert from 'node:assert/strict';
import { FRAME_BUDGET_MS, RENDER_WINDOW_MS, bezelStatus, createRenderHistory, memoryStatus } from '../../src/memory-status.js';
import { screenStatus } from '../../src/screen-status.js';
import { translator } from '../../src/i18n/index.js';
import { createBridge } from '../../src/bridge.js';
import { createDemoBackend } from '../../src/demo-backend.js';

for (const locale of ['en', 'pt-BR', 'it']) {
  test(`memory presentation uses backend totals and distinguishes unavailable from zero (${locale})`, () => {
    const t = translator(locale);
    const missing = memoryStatus(null, t);
    assert.equal(missing.label, t('shell.memoryChip', { value: t('shell.memoryUnavailable') }));
    assert.deepEqual(missing.rows, []);
    assert.equal(missing.demo, false);
    assert.equal(missing.note, t('shell.memoryMethod'));
    const measured = memoryStatus({ totalBytes: 312_000_000, processes: [{ pid: 8, role: 'studio', bytes: 7_500_000 }, { pid: 9, role: 'webview', bytes: null }] }, t);
    assert.equal(measured.label, t('shell.memoryChip', { value: t('shell.memoryMB', { value: 312 }) }));
    assert.equal(measured.rows[0], t('shell.memoryProcess', { name: t('shell.memoryRole.studio'), pid: 8, value: t('shell.memoryMB', { value: 8 }) }));
    assert.ok(measured.rows[1].includes(t('shell.memoryUnavailable')));
    assert.ok(measured.title.includes(measured.rows.join('\n')));
    assert.equal(memoryStatus({ totalBytes: 0 }, t).label, t('shell.memoryChip', { value: t('shell.memoryMB', { value: 0 }) }));
    assert.ok(memoryStatus({}, t).title.includes(t('shell.memoryUnavailable')));
  });
}

test('memory bridge exposes only the read-only command and passes its report through', async () => {
  const report = { totalBytes: null, processes: [] };
  const calls = [];
  const bridge = createBridge({ location: { hostname: 'localhost', search: '' }, __TAURI__: { core: { invoke: async (...args) => { calls.push(args); return report; } } } });
  assert.deepEqual(await bridge.memoryUsage(), report);
  assert.deepEqual(calls, [['memory_usage']]);
});

test('demo memory is explicitly marked, aggregates its processes, and returns independent reports', async () => {
  const demo = createDemoBackend('turing88');
  const report = await demo.memoryUsage();
  assert.equal(report.demo, true);
  assert.equal(report.totalBytes, 312_000_000);
  assert.equal(report.processes.reduce((total, p) => total + p.bytes, 0), report.totalBytes);
  assert.equal(new Set(report.processes.map(p => p.pid)).size, report.processes.length);
  assert.deepEqual([...new Set(report.processes.map(p => p.role))].sort(), ['light', 'sensors', 'studio', 'webview']);
  const t = translator('it');
  const status = memoryStatus(report, t);
  assert.ok(status.label.endsWith(t('shell.memoryDemoBadge')));
  assert.equal(status.note, t('shell.memoryDemo'));
  report.processes[0].bytes = 0;
  assert.equal((await demo.memoryUsage()).processes[0].bytes, 90_000_000);
});

test('render times keep the last minute only and skip what is not a time', () => {
  const history = createRenderHistory();
  assert.equal(history.last(), null);
  history.push(10, 0); history.push(NaN, 10); history.push(12, 'x');
  history.push(20, RENDER_WINDOW_MS - 1);
  assert.deepEqual(history.values(), [10, 20]);
  history.push(30, RENDER_WINDOW_MS);
  assert.deepEqual(history.values(), [20, 30], 'a point a minute old leaves');
  assert.equal(history.last(), 30);
});

test('the Bezel chip joins the render time and the memory, yellow past the frame budget', () => {
  const t = translator('it');
  const report = { totalBytes: 182_000_000, processes: [{ role: 'studio', pid: 1, bytes: 96_000_000 }] };
  const fine = bezelStatus(11.4, [9, 11.4], report, t);
  assert.equal(fine.label, t('shell.bezelChip', { ms: '11', memory: t('shell.memoryMB', { value: 182 }) }));
  assert.equal(fine.tone, 'ok');
  assert.equal(fine.render, t('shell.bezelRender', { ms: '11', limit: FRAME_BUDGET_MS }));
  assert.equal(fine.longest, 11);
  assert.deepEqual(fine.memory.processes, [{ name: t('shell.memoryRole.studio'), pid: 1, value: t('shell.memoryMB', { value: 96 }) }]);
  assert.match(fine.diagnostics, /182 MB/);
  assert.equal(bezelStatus(FRAME_BUDGET_MS + 5, [38], report, t).tone, 'warn');
  const none = bezelStatus(null, [], null, t);
  assert.equal(none.tone, 'ok');
  assert.equal(none.longest, null);
  assert.equal(none.label, t('shell.bezelChip', { ms: '–', memory: t('shell.memoryUnavailable') }));
});

test('the Screen chip is green and counts frames while live, grey otherwise, yellow on trouble', () => {
  const t = translator('it');
  const base = { name: 'Turing 8,8″', port: 'COM5', size: { width: 480, height: 1920 }, turned: true, theme: 'Monitor', profile: null };
  const live = screenStatus({ ...base, live: true, fps: 29.6, message: 'In diretta' }, t);
  assert.equal(live.label, t('shell.screenChipLive', { name: base.name, fps: 30 }));
  assert.equal(live.tone, 'ok');
  assert.deepEqual(live.rows.map(r => r.value), [t('shell.screenUsb', { port: 'COM5' }), t('shell.screenTurned', { size: '480 × 1920', degrees: 180 }), t('shell.fpsValue', { fps: 30 }), 'Monitor']);
  assert.equal(live.title, `${live.label}
In diretta`);
  const idle = screenStatus({ ...base, turned: false, live: false, fps: 30, profile: 'Gioco' }, t);
  assert.equal(idle.label, t('shell.screenChipIdle', { name: base.name }));
  assert.equal(idle.tone, '');
  assert.deepEqual(idle.rows.map(r => r.value), [t('shell.screenUsb', { port: 'COM5' }), '480 × 1920', 'Gioco'], 'no frames nor theme while not live');
  assert.equal(screenStatus({ ...base, live: true, fps: null, problem: true }, t).label, base.name);
  assert.equal(screenStatus({ ...base, live: true, problem: true }, t).tone, 'warn');
  assert.deepEqual(screenStatus({ name: 'X', live: false }, t).rows, []);
  assert.deepEqual(screenStatus(null, t), { present: false, label: t('shell.noScreenChip'), tone: '', title: t('shell.noScreenChip'), rows: [] });
  // Finding screens failed: no screen, yellow, and why.
  const failed = screenStatus(null, t, { message: 'permission denied', problem: true });
  assert.equal(failed.tone, 'warn');
  assert.equal(failed.title, `${t('shell.noScreenChip')}
permission denied`);
  assert.equal(screenStatus({ ...base, live: false }, t, { message: 'Riconnessione…', problem: true }).title, `${t('shell.screenChipIdle', { name: base.name })}
Riconnessione…`);
});
