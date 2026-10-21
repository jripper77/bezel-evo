// One sensor selector: the existing source card outside, searchable dialog inside.
import { el, icon } from './dom.js';
import { ICONS } from './icons.js';
import { listCard, searchField } from './fields.js';
import { openPopover, isOpenBy } from './popover.js';
import { sensorChoices, sensorChoiceReading, SENSOR_CATEGORY_ICONS } from '../sensor-picker-model.js';

let activePicker = null;
let sequence = 0;
export const refreshSensorPicker = () => activePicker?.refresh();
export const closeSensorPicker = () => activePicker?.close('dismiss');

export function sensorPicker({ t, key, catalog, readings, onChange, label, numeric = false, scope = label, summary = null }) {
  const entry = catalog().find(s => s.key === key);
  const card = listCard({ icon: SENSOR_CATEGORY_ICONS[entry?.category] ?? 'sensors', title: entry?.label ?? (key || t('insp.sensorPickerChoose')), subtitle: key || null, value: readings()?.[key]?.display ?? '…', label });
  if (summary) {
    card.querySelector('.list-card-title').textContent = summary.title;
    card.querySelector('.list-card-value').textContent = summary.value;
  }
  card.setAttribute('role', 'combobox');
  card.setAttribute('aria-haspopup', 'dialog');
  card.setAttribute('aria-expanded', 'false');
  card.dataset.sensorKey = key;
  if (!summary) card.querySelector('.list-card-value').dataset.liveKey = key;
  const wrap = el('div', { class: 'source-card', dataset: { focusScope: `sensor-${scope}` } }, [card]);

  function open() {
    if (isOpenBy(card)) { activePicker?.close('dismiss'); return; }
    let close;
    const title = el('strong', { text: label });
    const dismiss = el('button', { type: 'button', class: 'icon-button quiet', 'aria-label': t('dialog.close'), onclick: () => { close('escape'); card.focus(); } }, [icon(ICONS.close, 16)]);
    const list = el('div', { class: 'sensor-picker-list', id: `sensor-picker-${++sequence}`, role: 'listbox', 'aria-label': label });
    const count = el('span', { class: 'hint sensor-picker-count', role: 'status' });
    const search = searchField(t('insp.sensorPickerSearch'), () => render(), { placeholder: t('insp.sensorPickerPlaceholder') });
    const input = search.querySelector('input');
    input.setAttribute('aria-controls', list.id);
    const panel = el('div', { class: 'sensor-picker-popover', id: `dialog-${list.id}` }, [el('div', { class: 'sensor-picker-head' }, [title, dismiss]), search, count, list]);
    card.setAttribute('aria-controls', panel.id);
    const probeName = probe => probe === 'unassigned' ? t('sensorsPanel.probeUnknown') : probe;
    const categoryName = category => ['cpu', 'gpu', 'memory', 'disk', 'network', 'board', 'system'].includes(category) ? t(`category.${category}`) : t('common.other');
    const statusText = state => t(`sensorsPanel.status.${state}`);
    let lastReadings;
    function refresh(force = false) {
      if (!card.isConnected) { close('dismiss'); return; }
      const values = readings();
      if (!force && values === lastReadings) return;
      lastReadings = values;
      for (const row of list.querySelectorAll('[role="option"]')) {
        const reading = sensorChoiceReading({ key: row.dataset.key, quantity: row.dataset.quantity }, values);
        row.dataset.state = reading.state;
        row.querySelector('.status-dot').dataset.state = reading.state === 'available' ? 'ok' : reading.state === 'pending' ? 'restarting' : 'error';
        row.querySelector('.sensor-choice-value').textContent = reading.state === 'available' ? reading.value : statusText(reading.state);
        row.title = [row.dataset.name, row.dataset.key, statusText(reading.state), reading.reason].filter(Boolean).join('\n');
      }
    }
    function choose(sensor) {
      close('done');
      card.focus({ preventScroll: true });
      if (sensor.key !== key) onChange(sensor.key);
    }
    function render() {
      const groups = sensorChoices(catalog(), input.value, { numeric, categoryLabel: categoryName, probeLabel: probeName });
      const total = groups.reduce((n, [, rows]) => n + rows.length, 0);
      count.textContent = t(total === 1 ? 'sensorChip.sensorsOne' : 'sensorChip.sensorsMany', { count: total });
      list.replaceChildren(...groups.map(([category, rows]) => el('div', { role: 'group', 'aria-label': categoryName(category) }, [
        el('div', { class: 'sensor-picker-category', 'aria-hidden': 'true', text: categoryName(category) }),
        ...rows.map(sensor => el('button', { type: 'button', class: 'sensor-choice', role: 'option', 'aria-selected': String(sensor.key === key), tabIndex: -1,
          dataset: { key: sensor.key, quantity: sensor.quantity ?? '', name: sensor.label }, onclick: () => choose(sensor) }, [
          el('span', { class: 'status-dot', 'aria-hidden': 'true' }),
          el('span', { class: 'sensor-choice-text' }, [
            el('strong', { text: sensor.label }), el('span', { class: 'mono', text: sensor.key }),
            el('span', { class: 'sensor-choice-probe', text: probeName(sensor.probe) }),
          ]),
          el('span', { class: 'sensor-choice-value' }),
        ])),
      ])));
      if (!groups.length) list.append(el('p', { class: 'hint', text: t('insp.sensorPickerEmpty') }));
      const first = list.querySelector('[aria-selected="true"]') ?? list.querySelector('[role="option"]');
      if (first) first.tabIndex = 0;
      refresh(true);
    }
    const options = () => [...list.querySelectorAll('[role="option"]')];
    panel.addEventListener('keydown', event => {
      const rows = options(), index = rows.indexOf(document.activeElement);
      if (event.key === 'Enter' && event.target === input && rows.length) { event.preventDefault(); rows[0].click(); return; }
      let target;
      if (['ArrowDown', 'ArrowUp'].includes(event.key)) target = index < 0 ? (event.key === 'ArrowDown' ? 0 : rows.length - 1) : (index + (event.key === 'ArrowDown' ? 1 : -1) + rows.length) % rows.length;
      else if (index >= 0 && ['Home', 'End'].includes(event.key)) target = event.key === 'Home' ? 0 : rows.length - 1;
      else return;
      event.preventDefault();
      if (rows[target]) { rows.forEach(row => { row.tabIndex = -1; }); rows[target].tabIndex = 0; rows[target].focus(); }
    });
    close = openPopover({ node: panel, anchor: card, label, onClose: () => { activePicker = null; card.removeAttribute('aria-controls'); } });
    activePicker = { close, refresh, node: panel };
    panel.addEventListener('focusout', event => {
      // Native pointer focus can pass through body between blur and focus.
      // Keep the option mounted until its click when focus stays in the popup.
      if (event.relatedTarget && (panel.contains(event.relatedTarget) || event.relatedTarget === card)) return;
      queueMicrotask(() => {
        if (activePicker?.node === panel && !panel.contains(document.activeElement) && document.activeElement !== card) close('dismiss');
      });
    });
    render();
    list.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: 'nearest' });
    input.focus();
  }
  card.addEventListener('click', open);
  card.addEventListener('keydown', event => { if (event.key === 'ArrowDown' || event.key === 'ArrowUp') { event.preventDefault(); open(); } });
  return wrap;
}
