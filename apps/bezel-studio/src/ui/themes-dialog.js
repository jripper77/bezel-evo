// One live library grid and one thumbnail cache, moved into the gallery while open.
import { el } from './dom.js';
import { ICONS } from './icons.js';
import { iconButton, screenThumb } from './fields.js';
import { openMenuAt } from './menu.js';
import { askChoice } from './dialog.js';
import { catalogKeys, missingSensors, themeUses } from '../gallery-model.js';
import { themePreview } from '../theme-previews.js';
import { errorText, sensorLabel } from '../messages.js';

export function createThemesDialog({ root, t, openTheme, duplicateTheme, bridge, catalog = () => null, readings = () => ({}), screen = () => null, refresh, onFacts = () => {}, onSensors = () => {} }) {
  let selected = null, entries = [], moved = [], opener, busy = false, overview = null, timer, lastFacts;
  const head = el('div', { class: 'dialog-head themes-gallery-head' });
  const filters = el('div', { class: 'themes-gallery-filters' });
  const body = el('div', { class: 'dialog-body themes-gallery-body' }, [filters]);
  const detail = el('aside', { class: 'themes-gallery-detail', 'aria-label': t('themesPanel.details') });
  const message = el('p', { class: 'themes-gallery-message', role: 'status' });
  const closeButton = iconButton(t('dialog.close'), ICONS.close, () => dialog.close());
  const dialog = el('dialog', { class: 'confirm-dialog themes-dialog', 'aria-labelledby': 'themes-gallery-title' }, [head, el('div', { class: 'themes-gallery-split' }, [body, detail]), message]);
  const run = async action => {
    if (busy) return;
    busy = true; paint();
    try { await action(); }
    catch (error) { message.textContent = errorText(t, error); }
    finally { busy = false; paint(); }
  };
  async function facts() {
    try { overview = await bridge.profiles(); if (!overview.status?.preview && message.textContent === t('themesPanel.trialStarted')) message.textContent = ''; const signature = JSON.stringify([overview.config, overview.displays, overview.status?.active?.profile, overview.status?.preview?.themes]); if (dialog.open && signature !== lastFacts) { lastFacts = signature; onFacts(); paint(); } }
    catch { /* Missing runtime status never becomes a guessed active profile. */ }
  }
  function poll() { clearTimeout(timer); if (!dialog.open || document.hidden) return; timer = setTimeout(async () => { await facts(); poll(); }, 2000); }
  const visibility = () => { if (document.hidden) clearTimeout(timer); else { void facts(); poll(); } };
  async function tryTheme(entry) {
    const current = screen();
    if (!current) { message.textContent = t('themesPanel.noScreenTrial'); return; }
    const count = await bridge.tryThemes({ [current.key]: entry.location });
    message.textContent = count ? t('themesPanel.trialStarted') : t('profileRules.ui.noLiveScreen');
    await facts();
  }
  const used = entry => themeUses(entry, overview);
  async function remove(entry) {
    const answer = await askChoice(t, { title: t('themesPanel.deleteTitle', { name: entry.name }), body: t('themesPanel.deleteBody'), actions: [{ id: 'delete', label: t('storage.deleteAction'), kind: 'danger' }] });
    if (answer !== 'delete') return;
    await bridge.deleteTheme(entry.location); await refresh(); await facts();
  }
  function menu(entry, anchor) {
    openMenuAt(anchor, { label: entry.name, items: [
      { label: t('themesPanel.openEditor'), run: () => { dialog.close(); openTheme(entry); } },
      { label: t('common.duplicate'), run: () => void run(() => duplicateTheme(entry)) },
      { label: t('themesPanel.export'), disabled: !bridge.exportTheme, run: () => void run(() => bridge.exportTheme(entry.location)) },
      { label: t('storage.deleteAction'), disabled: entry.bundled || !overview || used(entry).length > 0, run: () => void run(() => remove(entry)) },
    ] });
  }
  function tools(entry) {
    const more = iconButton(t('themesPanel.more', { name: entry.name }), ICONS.more, () => menu(entry, more));
    return el('span', { class: 'theme-tools' }, [iconButton(t('themesPanel.try', { name: entry.name }), ICONS.play, () => void run(() => tryTheme(entry))), more]);
  }
  function paint() {
    closeButton.title = t('dialog.close'); closeButton.setAttribute('aria-label', t('dialog.close'));
    if (!dialog.open) return;
    detail.setAttribute('aria-label', t('themesPanel.details')); onSensors();
    for (const card of dialog.querySelectorAll('.theme-card')) card.setAttribute('aria-pressed', String(card.dataset.location === selected?.location));
    if (!selected) { detail.replaceChildren(el('p', { class: 'hint', text: t('themesPanel.selectTheme') })); return; }
    const entry = selected, sensors = catalog() ?? [], missing = missingSensors(entry, catalog()), uses = used(entry);
    const preview = screenThumb(entry.canvas); preview.classList.add('gallery-preview');
    void themePreview(bridge, entry).then(url => { if (url) preview.style.backgroundImage = `url("${url}")`; });
    const action = (text, fn, attrs = {}) => el('button', { type: 'button', class: 'text-button', text, disabled: busy, onclick: () => void run(fn), ...attrs });
    detail.replaceChildren(...[
      el('h3', { text: entry.name }), preview,
      el('p', { class: 'hint', text: t('themesPanel.objects', { count: entry.objects ?? 0, width: entry.canvas.width, height: entry.canvas.height }) }),
      el('h4', { text: t('themesPanel.sensors') }),
      el('ul', { class: 'gallery-sensors' }, (entry.sensors ?? []).map(key => {
        const sensor = sensors.find(s => s.key === key), reading = readings()[key];
        return el('li', { dataset: { missing: String(missing.includes(key)) } }, [el('span', { text: sensor ? sensorLabel(t, sensor) : key }), el('span', { dataset: { reading: key }, text: missing.includes(key) ? t('themesPanel.missing') : (reading?.display ?? '\u2014') })]);
      })),
      !(entry.sensors ?? []).length && el('p', { class: 'hint', text: t('themesPanel.noSensors') }),
      el('h4', { text: t('themesPanel.usedWhere') }),
      ...uses.map(use => el('p', { class: 'gallery-use', text: t(`themesPanel.use.${use.kind}`, { name: use.name }) })),
      !uses.length && el('p', { class: 'hint', text: t('themesPanel.unused') }),
      el('div', { class: 'gallery-actions' }, [
        action(t('themesPanel.openEditor'), () => { dialog.close(); openTheme(entry); }, { class: 'primary-button' }),
        action(t('common.duplicate'), () => duplicateTheme(entry)),
        action(t('themesPanel.export'), () => bridge.exportTheme(entry.location), { disabled: busy || !bridge.exportTheme }),
        action(t('storage.deleteAction'), () => remove(entry), { disabled: busy || entry.bundled || !overview || uses.length > 0, title: entry.bundled ? t('themes.bundled') : uses.length ? t('themesPanel.removeUses') : '' }),
      ]),
    ].filter(Boolean));
  }
  const move = (selector, destination) => {
    const node = root.querySelector(selector), marker = document.createComment('gallery location');
    node.before(marker); moved.push({ node, marker }); destination.append(node);
  };
  async function show() {
    if (dialog.open) return;
    opener = document.activeElement; message.textContent = ''; lastFacts = null;
    move('.panel-head', head); move('.theme-actions', head); head.append(closeButton);
    head.querySelector('h2').id = 'themes-gallery-title';
    move('#theme-search', filters); move('.theme-filter', filters); move('#import-report', body); move('#theme-grid', body);
    root.append(dialog); dialog.showModal();
    selected = entries.find(entry => entry.location === selected?.location) ?? entries[0] ?? null;
    paint(); dialog.querySelector('input[type=search]').focus();
    document.addEventListener('visibilitychange', visibility); await facts(); poll();
  }
  dialog.addEventListener('close', () => {
    clearTimeout(timer); document.removeEventListener('visibilitychange', visibility);
    for (const { node, marker } of moved) marker.replaceWith(node);
    moved = []; for (const card of root.querySelectorAll('.theme-card')) card.removeAttribute('aria-pressed');
    dialog.remove(); onSensors(); if (opener?.isConnected) opener.focus();
  });
  dialog.addEventListener('keydown', event => {
    if (event.key !== 'Escape' || !event.target.matches('input[type=search]')) return; event.preventDefault(); event.stopPropagation(); dialog.close();
  }, true);
  return {
    show, tools, overview: () => overview,
    wantedSensors: () => dialog.open && !document.hidden ? selected ? catalogKeys(selected) : [] : null,
    sample() {
      if (!dialog.open) return;
      for (const node of detail.querySelectorAll('[data-reading]')) {
        if (node.parentElement.dataset.missing === 'true') continue;
        const reading = readings()[node.dataset.reading];
        node.textContent = reading?.display ?? '\u2014';
      }
    },
    presets(list, anchor) { openMenuAt(anchor, { label: t('themesPanel.fromPreset'), items: list.map(entry => ({ label: entry.name, run: () => void run(async () => {
      const copy = await bridge.duplicateTheme(entry.location, t('layers.copyOf', { name: entry.name })); await refresh(); dialog.close(); openTheme(copy);
    }) })) }); },
    select(entry) { if (!dialog.open) return false; selected = entry; paint(); return true; },
    update(shown) { entries = shown; if (dialog.open) { selected = entries.find(entry => entry.location === selected?.location) ?? entries[0] ?? null; paint(); } },
    close: () => dialog.open && dialog.close(), retranslate: paint,
  };
}
