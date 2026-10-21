// Compact sensor health: one row per probe, with linked availability counts.
// Details live in the filtered Sensors panel. Restart affects only Libre.
import { el, icon } from './dom.js';
import { ICONS } from './icons.js';
import { libreStatus, sensorChip } from '../libre-status.js';
import { canRestartProbe, probeRows, probeTotals } from '../sensor-probes.js';
import { timeAgo } from '../relative-time.js';

/** The pill's tone of each state (`.pill[data-tone]`). */
const TONES = Object.freeze({ ok: 'ok', partial: 'warn', error: 'danger', pending: 'info', restarting: 'info' });

/**
 * @param {{
 *   t: (k: string, p?: object) => string,
 *   bridge: { restartLibre: () => Promise<unknown> },
 *   notify: (message: string) => void,
 *   fail: (error: unknown) => void,
 *   root: HTMLElement,
 *   onOpenSensors: (filters?: {probe?: string, status?: string}) => void,
 * }} deps `root` is `#status-libre`, the chip's wrapper
 * @returns {{ update: (sample: {health: {state: string, failed: string[], hardware?: {name: string, category: string|null, ok: boolean}[]} | null, sampleMillis?: number, sampledAt?: number, readings?: object|null, catalog?: object[], error?: unknown}) => void, close: () => void, restartLibre: () => Promise<void> }}
 */
export function createSensorStatus({ t, bridge, notify, fail, root, onOpenSensors }) {
  const wrap = root;
  const toggle = document.getElementById('status-libre-toggle');
  const dot = document.getElementById('status-libre-dot');
  const label = document.getElementById('status-libre-label');
  const popover = document.getElementById('libre-popover');
  const list = document.getElementById('libre-hardware');
  // The chip is the pill primitive; the old timing pill gives its place to it.
  toggle.className = 'pill sensor-chip';
  const spinner = icon(ICONS.spinner, 13);
  spinner.classList.add('sensor-spinner');
  toggle.insertBefore(spinner, label);
  popover.querySelector('.libre-popover-close').classList.add('quiet');
  const lastReading = el('span', { class: 'libre-popover-age mono' });
  popover.querySelector('.libre-popover-actions').prepend(lastReading);

  let restarting = false;
  // The last sample as it was read: what the chip and the dialog draw.
  let last = { health: null, error: null, sampleMillis: 0, sampledAt: null, readings: null, catalog: [] };

  const isOpen = () => !popover.hidden;
  const locale = () => document.documentElement.lang || 'en';

  function place() {
    const box = toggle.getBoundingClientRect();
    const right = Math.max(8, window.innerWidth - box.right);
    popover.style.right = `${right}px`;
    popover.style.bottom = `${window.innerHeight - box.top + 8}px`;
  }

  function open() {
    render();
    popover.hidden = false;
    toggle.setAttribute('aria-expanded', 'true');
    place();
    popover.querySelector('.libre-popover-close').focus();
  }

  function close({ refocus = false } = {}) {
    if (!isOpen()) return;
    popover.hidden = true;
    toggle.setAttribute('aria-expanded', 'false');
    if (refocus) toggle.focus();
  }

  function openSensors(filters) { close(); onOpenSensors(filters); }
  function countButton(probe, status, count) {
    return el('button', { type: 'button', class: 'probe-count', dataset: { probe, status, count },
      text: t(`sensorChip.count.${status}`, { count }), onclick: () => openSensors({ probe, status }) });
  }
  function row(probe) {
    const name = probe.id === 'unassigned' ? t('sensorsPanel.probeUnknown') : probe.id;
    return el('li', { class: 'probe-row', dataset: { probe: probe.id, state: probe.state } }, [
      el('button', { type: 'button', class: 'probe-main', dataset: { probe: probe.id, status: 'all' }, onclick: () => openSensors({ probe: probe.id }), }, [
        el('span', { class: 'status-dot', dataset: { state: probe.state === 'pending' ? 'restarting' : probe.state }, 'aria-hidden': 'true' }),
        el('span', { class: 'probe-name' }, [el('strong', { text: name }), el('span', { class: 'hint', text: t(probe.total === 1 ? 'sensorChip.sensorsOne' : 'sensorChip.sensorsMany', { count: probe.total }) })]),
        el('span', { class: 'probe-state', text: t(probe.state === 'pending' ? 'sensorsPanel.status.pending' : `sensorChip.probeState.${probe.state}`) }),
      ]),
      el('div', { class: 'probe-counts' }, [
        countButton(probe.id, 'available', probe.available), countButton(probe.id, 'unavailable', probe.unavailable), ...(probe.pending ? [countButton(probe.id, 'pending', probe.pending)] : []),
        ...(canRestartProbe(probe.id) ? [el('button', { type: 'button', class: 'text-button probe-restart',
          dataset: { probe: probe.id, status: 'restart' }, title: t('sensorChip.restartProbe', { name }),
          'aria-label': t('sensorChip.restartProbe', { name }), onclick: () => restart(probe.id) }, [icon(ICONS.refresh, 14), el('span', { text: t('sensorChip.restart') })])] : []),
      ]),
    ]);
  }

  /** The state shown: the chip's, with the health kept while the reader restarts. */
  const chip = () => sensorChip({ health: last.health, counts: probeTotals(probeRows(last.catalog, last.readings, last.error)), error: last.error, restarting, sampleMillis: last.sampleMillis });

  function render() {
    const state = chip();
    const probes = probeRows(last.catalog, last.readings, last.error);
    const total = probeTotals(probes);
    document.getElementById('libre-popover-dot').dataset.state = state.state;
    // "Sensors · 20 of 21 active" (board StatusBarNew).
    document.getElementById('libre-popover-title').textContent = total.total ? t('sensorChip.activeTitle', { available: total.available, total: total.total }) : t('sensorChip.probesTitle');
    list.setAttribute('aria-label', t('sensorChip.probes'));
    const signature = JSON.stringify([probes, locale()]);
    if (list.dataset.signature !== signature) {
      const focused = list.contains(document.activeElement) ? { ...document.activeElement.dataset } : null;
      list.dataset.signature = signature;
      list.replaceChildren(...probes.map(row));
      if (focused?.probe) list.querySelector(`button[data-probe="${CSS.escape(focused.probe)}"][data-status="${CSS.escape(focused.status)}"]`)?.focus();
    }
    const note = document.getElementById('libre-popover-note');
    note.hidden = false;
    note.classList.add('probe-summary');
    const totalsSignature = JSON.stringify([total, locale()]);
    if (note.dataset.signature !== totalsSignature) {
      const focusedStatus = note.contains(document.activeElement) ? document.activeElement.dataset.status : null;
      note.dataset.signature = totalsSignature;
      note.replaceChildren(el('strong', { text: t('sensorChip.managed', { count: total.total }) }),
        ...['available', 'unavailable', 'pending'].filter(status => status !== 'pending' || total.pending).map(status => countButton('all', status, total[status])));
      if (focusedStatus) note.querySelector(`button[data-status="${CSS.escape(focusedStatus)}"]`)?.focus();
    }
    lastReading.textContent = last.sampledAt === null ? '' : t('sensorChip.lastReading', { ago: timeAgo(last.sampledAt, Date.now(), locale()) });
    for (const button of list.querySelectorAll('.probe-restart')) {
      button.disabled = restarting;
      button.setAttribute('aria-busy', String(restarting));
      button.querySelector('span').textContent = t(restarting ? 'sensorChip.restarting' : 'sensorChip.restart');
    }
  }

  /** The chip, its dot, its title and the restarts. */
  function paint() {
    const state = chip();
    wrap.hidden = false;
    toggle.dataset.tone = TONES[state.state];
    toggle.dataset.state = state.state;
    dot.dataset.state = state.state;
    label.textContent = t(state.key, state.params);
    toggle.title = label.textContent;
    if (isOpen()) render();
  }

  async function restart(probe) {
    if (!canRestartProbe(probe) || restarting) return;
    restarting = true;
    // While the reader restarts there are no readings: the dialog keeps the last ones.
    paint();
    try {
      await bridge.restartLibre();
      notify(t('status.libreRestarted'));
    } catch (e) {
      fail(e);
    } finally {
      restarting = false;
      paint();
    }
  }

  toggle.addEventListener('click', () => (isOpen() ? close() : open()));
  popover.querySelector('.libre-popover-close').addEventListener('click', () => close({ refocus: true }));
  document.getElementById('libre-open-sensors').addEventListener('click', () => {
    close();
    onOpenSensors();
  });
  // Escape closes it from inside, from the chip, or when the focus was lost
  // (the restart button turns disabled while the reader restarts).
  document.addEventListener('keydown', (e) => {
    if (e.key !== 'Escape' || !isOpen()) return;
    const at = document.activeElement;
    if (at && at !== document.body && !popover.contains(at) && at !== toggle) return;
    e.preventDefault();
    e.stopPropagation();
    close({ refocus: true });
  }, true);
  document.addEventListener('pointerdown', (e) => {
    if (isOpen() && !popover.contains(e.target) && !toggle.contains(e.target)) close();
  });
  window.addEventListener('resize', () => isOpen() && place());

  return {
    /** A sample was read (or failed: `error`): the chip and the dialog follow it. */
    update({ health, sampleMillis = 0, sampledAt = null, readings = null, catalog: entries, error = null }) {
      const catalog = entries ?? last.catalog;
      last = {
        health: health === undefined ? libreStatus(catalog, error ? null : readings) : health,
        error,
        sampleMillis,
        // A failed sample keeps when the last one came, and its readings.
        sampledAt: error ? last.sampledAt : sampledAt ?? Date.now(),
        readings: error ? last.readings : readings,
        catalog,
      };
      paint();
    },
    close,
    restartLibre: () => restart('LibreHardwareMonitor'),
  };
}
