// Presentation of the backend's already scoped/aggregated byte counts.
// Decimal MB, rounded only at display time, like memory::megabytes in Rust.
export function memoryStatus(report, t) {
  const value = (bytes) => bytes === null || bytes === undefined ? t('shell.memoryUnavailable') : t('shell.memoryMB', { value: Math.round(bytes / 1_000_000) });
  const demo = report?.demo === true;
  const amount = value(report?.totalBytes);
  const label = t('shell.memoryChip', { value: amount });
  const note = t(demo ? 'shell.memoryDemo' : 'shell.memoryMethod');
  const processes = (report?.processes ?? []).map(p => ({ name: t(`shell.memoryRole.${p.role}`), pid: p.pid, value: value(p.bytes) }));
  const rows = processes.map(p => t('shell.memoryProcess', p));
  return { label: demo ? `${label} · ${t('shell.memoryDemoBadge')}` : label, amount, note, rows, processes, title: [label, note, ...rows].join('\n'), demo };
}

/** One frame at the preview's 30 frames a second: a render longer than this is late. */
export const FRAME_BUDGET_MS = 33;
/** How long the Bezel chip remembers render times. */
export const RENDER_WINDOW_MS = 60_000;

/** Render times of the last minute (`at` in ms), oldest first. */
export function createRenderHistory(windowMs = RENDER_WINDOW_MS) {
  let points = [];
  return {
    push(ms, at) {
      if (!Number.isFinite(ms) || !Number.isFinite(at)) return;
      points = [...points.filter(p => at - p.at < windowMs), { ms, at }];
    },
    values: () => points.map(p => p.ms),
    last: () => points.at(-1)?.ms ?? null,
  };
}

/**
 * The Bezel chip (D-2026-10-20-studio-ux-v2-11): the last render time and the
 * memory, yellow when a frame took longer than its budget; and what its
 * dialog says. `lastMs` null: nothing rendered yet.
 */
export function bezelStatus(lastMs, values, report, t) {
  const memory = memoryStatus(report, t);
  const ms = lastMs === null ? '–' : String(Math.round(lastMs));
  const slow = lastMs !== null && lastMs > FRAME_BUDGET_MS;
  const longest = values.length ? Math.round(Math.max(...values)) : null;
  return {
    label: t('shell.bezelChip', { ms, memory: memory.amount }),
    tone: slow ? 'warn' : 'ok',
    render: t('shell.bezelRender', { ms, limit: FRAME_BUDGET_MS }),
    longest,
    memory,
    diagnostics: [
      t('shell.bezelTitle'),
      t('shell.bezelRender', { ms, limit: FRAME_BUDGET_MS }),
      longest === null ? null : t('shell.bezelLongest', { ms: longest }),
      memory.label,
      ...memory.rows,
    ].filter(Boolean).join('\n'),
  };
}
