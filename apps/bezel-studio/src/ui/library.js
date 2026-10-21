// The left library: widgets, sensors, layers, themes, media (the theme's and
// the GIF collection) and screen, each a tab panel (artboards Main, Sensors,
// Layers, Media, Themes). Widgets and sensors drag onto the canvas. Every
// visual object comes from the primitives (ui/fields.js, ui/icons.js,
// base.css); what the panels compute lives in library-model.js.
import { el, icon } from './dom.js';
import { ICONS } from './icons.js';
import { accordionSection, chipButton, dashedHint, kindIcon, searchField, screenThumb, sectionTitle, selectField } from './fields.js';
import { createSectionMemory } from '../inspector-sections.js';
import { alignItems } from './object-menu.js';
import { openMenuAt } from './menu.js';
import { makeDraggable } from './dragdrop.js';
import { defaultRange, widgetOf, WIDGETS } from '../editor/widgets.js';
import { canGroup } from '../editor/cards.js';
import { backgroundOf, fileNameOf, mediaItems, moves, videoFacts } from '../editor/background.js';
import { warningText } from '../messages.js';
import { formatBytes, wireSubtabs } from './storage.js';
import { SHOW_ALL, axisOf, countText, emptyState, filterThemes, rememberedFilter, screenCount, scopeIn, screenLabel, searchThemes, thumbnailKey } from '../theme-filter.js';
import { createIconPicker } from './icon-picker.js';
import { filtersToggle } from './filters-toggle.js';
import { themePreview } from '../theme-previews.js';
import { missingSensors, recentThemes, themeUses } from '../gallery-model.js';
import { createThemesDialog } from './themes-dialog.js';
import { layerGroups } from '../editor/layers.js';
import { wireLayerDrag } from './layer-drag.js';
import { SENSOR_CATEGORY_ICONS } from '../sensor-picker-model.js';
import { WIDGET_GROUPS, assetUses, countLayers, createHistory, groupHeadline, filterLayers, layerMeta, mediaTotals, sensorDropTarget, sharedSource, sparkPath, updateSeconds, widgetGroups } from '../library-model.js';
import { libreStatus } from '../libre-status.js';
import { matchesSensor, probeOf, readingState } from '../sensor-probes.js';
import { timeAgo } from '../relative-time.js';

export { axisOf, filtersToggle };

const CATEGORY_ORDER = ['cpu', 'gpu', 'memory', 'disk', 'network', 'board', 'system'];

/** The "show every category" choice of the Sensors panel's hardware filter. */
export const ALL_CATEGORIES = 'all';

const categoryRank = (c) => (CATEGORY_ORDER.includes(c) ? CATEGORY_ORDER.indexOf(c) : CATEGORY_ORDER.length);

/** The categories the catalog has, in display order: the hardware filter's chips. */
export function sensorCategories(catalog) {
  return [...new Set(catalog.map((s) => s.category))].sort((a, b) => categoryRank(a) - categoryRank(b) || a.localeCompare(b));
}

/**
 * Groups catalog entries by category, in display order, filtered by text
 * and, unless `category` is ALL_CATEGORIES, by one category.
 */
export function groupSensors(catalog, filter = '', category = ALL_CATEGORIES, options = {}) {
  const q = filter.trim().toLowerCase();
  const groups = new Map();
  for (const s of catalog) {
    if (!matchesSensor(s, { ...options, catalog })) continue;
    if (category !== ALL_CATEGORIES && s.category !== category) continue;
    if (q && !`${s.label} ${s.key}`.toLowerCase().includes(q)) continue;
    if (!groups.has(s.category)) groups.set(s.category, []);
    groups.get(s.category).push(s);
  }
  return [...groups.entries()].sort((a, b) => categoryRank(a[0]) - categoryRank(b[0]) || a[0].localeCompare(b[0]));
}

/**
 * The keys of the sensors the list shows: those the filter keeps while the
 * list is open (its tab selected, the window visible), none otherwise. The
 * app measures them with the theme's; `net.ping` sends packets only while
 * something shown uses it (D-2026-09-30-release-polish-11).
 */
export function catalogKeysInView(catalog, filter, open, category = ALL_CATEGORIES, options = {}) {
  if (!open) return [];
  // Read the whole probe/category/search scope even when a state filter hides
  // rows, so failed readings can recover and healthy readings can later fail.
  return groupSensors(catalog, filter, category, { probe: options.probe }).flatMap(([, items]) => items.map((s) => s.key));
}

const SVG = 'http://www.w3.org/2000/svg';
/** A cut mark no translation contains, to put a key cap inside a sentence. */
const CUT = '\u0001';

/**
 * @param {object} deps
 * @param {{scope?: string|null, axis?: string}|null} [deps.themeFilter] the Themes tab's filter as remembered
 * @param {object|null} [deps.bridge] the backend, for what the panels read themselves
 */
export function createLibrary({ store, canvas, stage, t, pickers, locale = () => 'en', themeFilter = null, bridge, actions }) {
  const $ = (id) => document.getElementById(id);
  let catalog = [];
  let readings = {};
  let editing = null;
  // The hardware the Sensors panel shows (one category or all of them).
  let sensorProbe = 'all';
  let sensorStatus = 'all';
  let sensorError = false;
  const collapsedLayers = new Set();
  const history = createHistory();
  // What each panel shows, to draw it again in another language.
  let themes = [];
  let media = [];
  let importReport = null;
  let screenArgs = [[], null];
  let widgetQuery = '';
  let layerQuery = '';
  let layerType = 'all';
  let themeQuery = '';

  /** A search box (the primitive) in `slot`, relabelled when the language changes. */
  const searches = [];
  function mountSearch(slot, labelKey, onInput, { id, placeholderKey = labelKey } = {}) {
    const box = searchField(t(labelKey), onInput, { id, placeholder: t(placeholderKey) });
    const input = box.querySelector('input');
    searches.push({ input, labelKey, placeholderKey });
    $(slot).replaceChildren(box);
    return input;
  }

  const duplicateTheme = (entry) => actions.duplicateTheme?.(entry.location, t('layers.copyOf', { name: entry.name }));
  const gallery = createThemesDialog({ root: $('panel-themes'), t, openTheme: (entry) => actions.openTheme(entry.location), duplicateTheme, bridge, catalog: () => catalog, readings: () => readings, screen: screenInUse, refresh: () => actions.refreshThemes(), onFacts: () => renderThemes(themes), onSensors: reportShown });
  $('theme-gallery')?.addEventListener('click', () => gallery.show());

  // ---------------------------------------------------------------- tabs --
  const tabs = [...document.querySelectorAll('.tabs [role="tab"]')];
  function selectTab(tab) {
    if (tab?.dataset.tab !== 'themes') gallery.close();
    for (const t2 of tabs) {
      const on = t2 === tab;
      t2.setAttribute('aria-selected', String(on));
      t2.tabIndex = on ? 0 : -1;
      $(t2.getAttribute('aria-controls')).hidden = !on;
    }
    reportShown();
    fetchThumbnails();
  }
  tabs.forEach((tab, i) => {
    tab.addEventListener('click', () => selectTab(tab));
    tab.addEventListener('keydown', (evt) => {
      // A vertical rail: Up/Down move, Left/Right too, Home/End jump.
      const keys = { ArrowDown: i + 1, ArrowRight: i + 1, ArrowUp: i - 1, ArrowLeft: i - 1, Home: 0, End: tabs.length - 1 };
      if (!(evt.key in keys)) return;
      evt.preventDefault();
      const next = tabs[(keys[evt.key] + tabs.length) % tabs.length];
      selectTab(next);
      next.focus();
    });
  });

  const center = () => {
    const { theme, selection } = store.getState();
    const chosen = selection.length === 1 && theme.elements.find(e => e.id === selection[0]);
    const card = chosen?.card ? chosen : theme.elements.find(e => e.id === chosen?.cardMember?.parent);
    if (card) return { x: card.frame.x + card.frame.width / 2, y: card.frame.y + card.frame.height / 2 };
    const c = theme.canvas;
    return { x: c.width / 2, y: c.height / 2 };
  };

  /** `t(key)` with a key cap where its `{key}` is. */
  function withKeyCap(key, cap) {
    const [before, after = ''] = t(key, { key: CUT }).split(CUT);
    return [before, el('kbd', { text: cap }), after];
  }

  // Which groups of the Widgets panel are open (all at first), remembered.
  // Which sensor categories are open (the processor and graphics at first), remembered.
  const sensorMemory = createSectionMemory({ key: 'bezel.sensorGroups', ids: CATEGORY_ORDER, open: ['cpu', 'gpu'] });
  // Whether Media's "In the theme" is open (at first it is), remembered.
  const mediaMemory = createSectionMemory({ key: 'bezel.mediaGroups', ids: ['theme'], open: ['theme'] });
  const widgetMemory = createSectionMemory({ key: 'bezel.widgetGroups', ids: WIDGET_GROUPS.map(g => g.id), open: WIDGET_GROUPS.map(g => g.id) });

  // ------------------------------------------------------------- widgets --
  function widgetTile(w) {
    const label = t(`widget.${w}`);
    const tile = el('button', { type: 'button', class: 'widget-tile', dataset: { widget: w }, 'aria-label': t('library.addWidget', { name: label }) }, [icon(ICONS[w], 22), el('span', { text: label })]);
    makeDraggable(tile, {
      label,
      canvas,
      stage,
      onDrop: (p) => store.dispatch('add', { widget: w, x: p.x, y: p.y }),
      onClick: () => store.dispatch('add', { widget: w, ...center() }),
    });
    return tile;
  }

  /** The Group tile groups the selection (Ctrl G); it says why when it cannot. */
  function syncGroupTile(tile = document.querySelector('.widget-tile[data-widget="group"]')) {
    if (!tile) return;
    const { theme, selection } = store.getState();
    const able = canGroup(theme, selection);
    tile.setAttribute('aria-disabled', String(!able));
    tile.title = able ? '' : t('group.scope');
  }
  store.subscribe?.(() => syncGroupTile());

  function groupTile() {
    const tile = el('button', {
      type: 'button', class: 'widget-tile', dataset: { widget: 'group' }, 'aria-label': t('widgets.groupTile'),
      onclick: () => {
        const { theme, selection } = store.getState();
        if (canGroup(theme, selection)) store.dispatch('groupSelection', { ids: selection });
      },
    }, [icon(ICONS.group, 22), el('span', { text: t('widget.group') })]);
    syncGroupTile(tile);
    return tile;
  }

  function renderWidgets() {
    const groups = widgetGroups((w) => t(`widget.${w}`), widgetQuery);
    $('widget-count').textContent = t('widgets.count', { count: groups.reduce((n, g) => n + g.widgets.length, 0) });
    // A search shows every group it finds open; without one, each as it was left.
    const sections = groups.map((g) => accordionSection({
      id: g.id, title: t(`widgets.group.${g.id}`), icon: null, summary: String(g.widgets.length), keepSummary: true,
      expanded: Boolean(widgetQuery.trim()) || widgetMemory.isOpen('widgets', g.id),
      onToggle: ({ exclusive }) => { widgetMemory.toggle('widgets', g.id, { exclusive }); renderWidgets(); $('widget-groups').querySelector(`[data-inspector-section="${g.id}"] .accordion-toggle`)?.focus(); },
      render: (body) => body.append(el('ul', { class: 'widget-grid' }, g.widgets.map((w) => el('li', {}, [w === 'group' ? groupTile() : widgetTile(w)])))),
    }).element);
    sections.forEach(section => section.classList.add('widget-group'));
    $('widget-groups').replaceChildren(...(groups.length ? sections : [el('p', { class: 'empty-note', text: t('widgets.none') })]));
    $('widget-hint').replaceChildren(dashedHint(withKeyCap('widgets.hint', t('widgets.enterKey'))));
  }

  mountSearch('widget-search', 'widgets.search', (q) => { widgetQuery = q; renderWidgets(); }, { id: 'widget-search-input', placeholderKey: 'widgets.searchPlaceholder' });

  // ------------------------------------------------------------- sensors --
  /**
   * The sparkline's width in px: the `--spark-width` token (tokens.css), which
   * also sizes its column of the sensor row, so the drawing is never scaled.
   */
  let sparkWidth = null;
  function sparkWidthPx() {
    sparkWidth ??= parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--spark-width'));
    return sparkWidth;
  }

  /** A sensor's recent values as a line, coloured by its hardware (library.css). */
  function sparkline(key, category) {
    const width = sparkWidthPx();
    const svg = document.createElementNS(SVG, 'svg');
    svg.setAttribute('class', 'sparkline');
    svg.setAttribute('viewBox', `0 0 ${width} 20`);
    svg.setAttribute('width', String(width));
    svg.setAttribute('height', '20');
    svg.setAttribute('preserveAspectRatio', 'none');
    svg.setAttribute('aria-hidden', 'true');
    svg.dataset.sparkFor = key;
    svg.dataset.category = category;
    svg.append(document.createElementNS(SVG, 'path'));
    drawSpark(svg);
    return svg;
  }

  function drawSpark(svg) {
    const d = sparkPath(history.values(svg.dataset.sparkFor), sparkWidthPx());
    if (d) svg.firstChild.setAttribute('d', d);
    else svg.firstChild.removeAttribute('d');
  }

  const SYSTEM_STATES = new Set(['system.audio.muted', 'system.power.ac', 'system.battery.charging']);
  const sensorValue = (key, r) => SYSTEM_STATES.has(key) && !r?.unavailable && Number.isFinite(r?.value)
    ? t(`sensorsPanel.state.${r.value ? 'on' : 'off'}`) : r?.display ?? '…';
  const sensorValueTitle = (r) => r?.unavailable ? String(r.unavailable) : r?.display ?? '';
  const systemGroup = (key) => key === 'system.volume' || key.startsWith('system.audio.') ? 'audio'
    : key.startsWith('system.battery.') || key.startsWith('system.power.') ? 'power' : 'system';

  function sensorRow(s) {
    const r = sensorError ? { unavailable: t('sensorsPanel.status.unavailable') } : readings[s.key];
    const status = readingState(s, readings, sensorError), probe = probeOf(s, catalog);
    const value = el('span', { class: `value${r?.unavailable ? ' unavailable' : ''}`, dataset: { valueFor: s.key }, text: status === 'unavailable' ? '—' : sensorValue(s.key, r), title: sensorValueTitle(r) });
    // Two lines (D-2026-10-20-studio-ux-v2-14): the whole name and the value,
    // then the key and its sparkline, or why the sensor cannot be read.
    const why = status === 'unavailable'
      ? el('span', { class: 'reason', text: probe === 'LibreHardwareMonitor' ? t('sensorsPanel.needsLibre') : t('shell.memoryUnavailable'), title: sensorValueTitle(r) })
      : el('span', { class: 'key', text: s.key });
    const row = el('button', { type: 'button', class: `sensor-row${s.quantity === 'text' ? ' sensor-row-text' : ''}${s.category === 'system' ? ' sensor-row-system' : ''}`, dataset: { key: s.key, probe, status }, 'aria-label': t('library.addSensor', { name: s.label }) }, [
      el('span', { class: 'label', text: s.label }),
      value,
      why,
      ...(s.quantity === 'text' || s.category === 'system' || status === 'unavailable' ? [] : [sparkline(s.key, s.category)]),
    ]);
    const sensor = { key: s.key, quantity: s.quantity, label: s.label };
    makeDraggable(row, {
      label: s.label,
      canvas,
      stage,
      onDrop: (p) => {
        // Over an object that reads a sensor the drop binds it; elsewhere it adds a value.
        const target = sensorDropTarget(store.getState().theme.elements, p, sensor, defaultRange);
        if (!target) {
          store.dispatch('add', { widget: 'value', x: p.x, y: p.y, sensor });
          return;
        }
        store.dispatch('update', target);
        store.select([target.id]);
      },
      onClick: () => store.dispatch('add', { widget: 'value', ...center(), sensor }),
    });
    return row;
  }

  // What the list shows, told to the app when it changes.
  let reported = null;
  function reportShown() {
    const open = !$('panel-sensors').hidden && document.visibilityState !== 'hidden';
    const keys = gallery.wantedSensors() ?? catalogKeysInView(catalog, sensorSearch.value, open, ALL_CATEGORIES, { probe: sensorProbe });
    if (keys.join(' ') === reported) return;
    reported = keys.join(' ');
    actions.showSensors(keys);
  }
  document.addEventListener('visibilitychange', () => reportShown());

  /** Quick state chips: all, readable, not readable (with how many, within the probe and search). */
  function renderSensorFilter() {
    const scoped = (status) => groupSensors(catalog, sensorSearch.value, ALL_CATEGORIES, { probe: sensorProbe, status, readings, error: sensorError }).reduce((n, [, items]) => n + items.length, 0);
    const missing = scoped('unavailable');
    const chip = (status, label) => chipButton(label, sensorStatus === status, () => { sensorStatus = status; renderSensors(); }, { dataset: { status } });
    $('sensor-filter').setAttribute('aria-label', t('sensorsPanel.status'));
    $('sensor-filter').replaceChildren(
      chip('all', t('collection.filter.all')),
      chip('available', t('sensorsPanel.status.available')),
      chip('unavailable', missing ? t('sensorsPanel.unavailableCount', { count: missing }) : t('sensorsPanel.status.unavailable')),
    );
  }

  /** The values beside a category's title: how many sensors, then its headline readings. */
  const groupSummary = (cat, items) => [String(items.length), ...groupHeadline(cat, items, sensorError ? null : readings)].join(' · ');

  function renderSensors() {
    renderProbeFilters();
    renderLibre();
    reportShown();
    renderSensorFilter();
    const list = $('sensor-list');
    const groups = groupSensors(catalog, sensorSearch.value, ALL_CATEGORIES, { probe: sensorProbe, status: sensorStatus, readings, error: sensorError });
    const shown = groups.reduce((n, [, items]) => n + items.length, 0);
    const resultText = t('sensorsPanel.results', { shown, total: catalog.length });
    if ($('sensor-result-count').textContent !== resultText) $('sensor-result-count').textContent = resultText;
    // Said only while a filter hides some sensors (it stays a live region for screen readers).
    $('sensor-result-count').classList.toggle('visually-hidden', shown === catalog.length);
    renderRate();
    if (groups.length === 0) {
      list.replaceChildren(el('p', { class: 'empty-note', text: t('library.noSensors') }));
      return;
    }
    // A search or a filter shows every category it finds open; otherwise each as it was left.
    const filtered = Boolean(sensorSearch.value.trim()) || sensorStatus !== 'all' || sensorProbe !== 'all';
    list.replaceChildren(...groups.map(([cat, items]) => {
      const section = accordionSection({
        id: cat, title: t(`category.${cat}`), icon: SENSOR_CATEGORY_ICONS[cat] ?? 'monitor', summary: groupSummary(cat, items), keepSummary: true,
        expanded: filtered || !CATEGORY_ORDER.includes(cat) || sensorMemory.isOpen('sensors', cat),
        onToggle: ({ exclusive }) => { sensorMemory.toggle('sensors', cat, { exclusive }); renderSensors(); list.querySelector(`[data-inspector-section="${cat}"] .accordion-toggle`)?.focus(); },
        render: (body) => body.append(...(cat === 'system' ? ['system', 'audio', 'power'].flatMap((group) => {
          const members = items.filter((s) => systemGroup(s.key) === group);
          return members.length ? [el('h4', { class: 'sensor-subgroup', text: t(`sensorsPanel.group.${group}`) }), ...members.map(sensorRow)] : [];
        }) : items.map(sensorRow))),
      }).element;
      section.classList.add('sensor-group');
      section.dataset.category = cat;
      const source = sharedSource(items);
      if (source) section.querySelector('.accordion-toggle').title = source;
      return section;
    }));
  }

  /** Beside the title: how many sensors and, once two samples came, how often they update. */
  function renderRate() {
    const seconds = updateSeconds(history.times());
    const text = [String(catalog.length), seconds ? t('insp.theme.every', { seconds }) : null].filter(Boolean).join(' · ');
    const rate = $('sensor-rate');
    rate.classList.add('count');
    rate.title = seconds ? t('sensorsPanel.every', { seconds }) : '';
    if (rate.textContent !== text) rate.textContent = text;
  }

  // The LibreHardwareMonitor notice, drawn again only when what it says changes.
  let libreShown = null;
  function renderLibre() {
    const health = libreStatus(catalog, readings);
    const missing = (sensorProbe === 'all' || sensorProbe === 'LibreHardwareMonitor') && health && health.state !== 'ok' ? health.failed.join(', ') : '';
    if (missing === libreShown) return;
    libreShown = missing;
    $('sensor-libre').replaceChildren(...(missing ? [el('section', { class: 'notice sensor-libre', 'aria-labelledby': 'sensor-libre-title' }, [
      el('div', { class: 'notice-head' }, [icon(ICONS.warning, 18), el('h2', { id: 'sensor-libre-title', text: t('status.libreMissing', { hardware: missing }) })]),
      el('p', { text: t('sensorsPanel.libreBody') }),
      el('button', { type: 'button', class: 'text-button', text: t('sensorsPanel.startLibre'), onclick: restartLibre }),
    ])] : []));
  }

  /** Share the probe action's progress and duplicate-click guard with the notice. */
  function restartLibre() {
    if (actions.restartLibre) actions.restartLibre();
  }

  /**
   * The readings of a sample. `at` (ms since the epoch) is when it was
   * taken: the sparklines' history and the update rate come from it.
   */
  function updateReadings(next, at = null) {
    const visibleKeys = () => groupSensors(catalog, sensorSearch.value, ALL_CATEGORIES, { probe: sensorProbe, status: sensorStatus, readings, error: sensorError }).flatMap(([, items]) => items.map(s => s.key)).join(' ');
    const before = sensorStatus !== 'all' ? visibleKeys() : null;
    const wasError = sensorError;
    sensorError = false;
    readings = next;
    gallery.sample();
    history.push(next, at);
    // A row that becomes readable or not changes its second line: draw the list again.
    const turned = wasError || [...document.querySelectorAll('.sensor-row[data-key]')].some(row => {
      const sensor = catalog.find(s => s.key === row.dataset.key);
      return sensor && (readingState(sensor, readings) === 'unavailable') !== (row.dataset.status === 'unavailable');
    });
    if (turned || (before !== null && before !== visibleKeys())) { renderSensors(); return; }
    for (const row of document.querySelectorAll('.sensor-row[data-key]')) {
      const sensor = catalog.find(s => s.key === row.dataset.key);
      if (sensor) row.dataset.status = readingState(sensor, readings);
    }
    for (const section of document.querySelectorAll('#sensor-list .sensor-group')) {
      const items = groupSensors(catalog, sensorSearch.value, ALL_CATEGORIES, { probe: sensorProbe, status: sensorStatus, readings }).find(([cat]) => cat === section.dataset.category)?.[1];
      const summary = section.querySelector('.accordion-summary');
      if (items && summary) summary.textContent = groupSummary(section.dataset.category, items);
    }
    for (const span of document.querySelectorAll('[data-value-for]')) {
      const r = readings[span.dataset.valueFor];
      span.textContent = sensorValue(span.dataset.valueFor, r);
      span.title = sensorValueTitle(r);
      span.classList.toggle('unavailable', Boolean(r?.unavailable));
    }
    for (const svg of document.querySelectorAll('svg[data-spark-for]')) drawSpark(svg);
    renderRate();
    renderLibre();
  }

  const sensorSearch = mountSearch('sensor-search-box', 'library.searchSensors', () => renderSensors(), { id: 'sensor-search', placeholderKey: 'sensorsPanel.search' });
  const probeField = selectField(t('sensorsPanel.probe'), 'all', [], value => { sensorProbe = value; renderSensors(); });
  const statusField = selectField(t('sensorsPanel.status'), 'all', [], value => { sensorStatus = value; renderSensors(); });
  const probeSelect = probeField.querySelector('select');
  const statusSelect = statusField.querySelector('select');
  probeSelect.dataset.sensorFilter = 'probe';
  statusSelect.dataset.sensorFilter = 'status';
  $('sensor-probe-filters').replaceChildren(probeField, statusField);
  // Probe and state live behind Filters beside the search; a dot says one is on.
  const { button: filtersButton, show: showFilters } = filtersToggle($('sensor-search-box'), $('sensor-probe-filters'));
  let filterSignature = '';
  function renderProbeFilters() {
    const probes = [...new Set(catalog.map(sensor => probeOf(sensor, catalog)))].sort();
    if (sensorProbe !== 'all' && !probes.includes(sensorProbe)) sensorProbe = 'all';
    const statuses = ['all', 'available', 'unavailable', 'pending'];
    const signature = JSON.stringify([probes, ...statuses.map(s => t(`sensorsPanel.status.${s}`)), t('sensorsPanel.probeUnknown'), t('sensorsPanel.probe')]);
    if (signature !== filterSignature) {
      filterSignature = signature;
      probeField.querySelector('span').textContent = t('sensorsPanel.probe');
      statusField.querySelector('span').textContent = t('sensorsPanel.status');
      probeSelect.replaceChildren(...['all', ...probes].map(value => el('option', { value, text: value === 'all' ? t('sensorsPanel.allProbes') : value === 'unassigned' ? t('sensorsPanel.probeUnknown') : value })));
      statusSelect.replaceChildren(...statuses.map(value => el('option', { value, text: t(`sensorsPanel.status.${value}`) })));
    }
    probeSelect.value = sensorProbe;
    statusSelect.value = sensorStatus;
    filtersButton.title = t('sensorsPanel.filters');
    filtersButton.setAttribute('aria-label', t('sensorsPanel.filters'));
    filtersButton.classList.toggle('filters-on', sensorProbe !== 'all' || sensorStatus === 'pending');
  }

  // -------------------------------------------------------------- layers --
  const layerDrag = wireLayerDrag($('layer-list'), store);
  const objectsText = (n) => (n === 1 ? t('layers.countOne') : t('layers.count', { count: n }));

  /** What a row says beside the name: the sensor it reads, or its kind (a card with its faces). */
  function metaOf(e, widget) {
    const meta = layerMeta(e);
    if (meta.sensor) return el('small', { class: 'layer-meta mono', text: meta.sensor });
    const kind = t(`widget.${widget}`);
    if (meta.faces === undefined) return el('small', { class: 'layer-meta', text: kind });
    const faces = meta.faces === 1 ? t('layersPanel.facesOne') : t('layersPanel.faces', { count: meta.faces });
    return el('small', { class: 'layer-meta', text: `${kind} · ${faces}` });
  }

  function layerName(e, widget, selected) {
    if (editing === e.id) {
      return el('input', {
        type: 'text', value: e.name, 'aria-label': t('common.name'), class: 'layer-name',
        onkeydown: (evt) => {
          if (evt.key === 'Enter') evt.target.blur();
          if (evt.key === 'Escape') { editing = null; renderLayers(); }
        },
        onblur: (evt) => {
          editing = null;
          if (evt.target.value.trim()) store.dispatch('update', { id: e.id, patch: { name: evt.target.value.trim() } });
          else renderLayers();
        },
      });
    }
    return el('button', {
      type: 'button', class: 'layer-name', 'aria-pressed': String(selected),
      onclick: (evt) => {
        const previous = store.getState().selection;
        if (e.cardMember?.face != null) store.dispatch('cardFace', { id: e.cardMember.parent, face: e.cardMember.face });
        store.select(evt.ctrlKey || evt.metaKey ? previous.includes(e.id) ? previous.filter(id => id !== e.id) : [...previous, e.id] : evt.shiftKey ? [...new Set([...previous, e.id])] : [e.id]);
      },
      ondblclick: () => { editing = e.id; renderLayers(); document.querySelector('.layer-list input')?.focus(); },
    }, [el('span', { class: 'layer-label', text: e.name }), metaOf(e, widget)]);
  }

  function foldButton(e) {
    const folded = collapsedLayers.has(e.id);
    return el('button', {
      type: 'button', class: 'icon-button quiet layer-fold', 'aria-label': t(folded ? 'layers.expand' : 'layers.collapse'), 'aria-expanded': String(!folded),
      onclick: () => {
        if (folded) collapsedLayers.delete(e.id);
        else collapsedLayers.add(e.id);
        renderLayers();
      },
    }, [icon(folded ? ICONS.chevronRight : ICONS.chevronDown, 14)]);
  }

  function layerRow(e, index, total, neighbors = null) {
    const selected = store.getState().selection.includes(e.id);
    const widget = widgetOf(e);
    const container = e.card || e.isGroup;
    const btn = (label, paths, onclick, disabled = false, pressed = null, extra = '') => el('button', { type: 'button', class: `icon-button quiet layer-tool${extra}`, title: label, 'aria-label': label, disabled, 'aria-pressed': pressed === null ? null : String(pressed), onclick }, [icon(paths, 16)]);
    // The row keeps room at its end for the flags it shows (a hidden object's eye sits third of four).
    const room = e.visible === false ? 2 : e.locked ? 1 : 0;
    return el('li', { dataset: { elementId: e.id, flags: String(room) }, class: `layer-row${container ? ' layer-container' : ''}${selected ? ' selected' : ''}${e.visible === false ? ' hidden-el' : ''}` }, [
      container ? foldButton(e) : el('span', { class: 'layer-fold', 'aria-hidden': 'true' }),
      el('span', { class: 'layer-kind', 'aria-hidden': 'true' }, [icon(e.isGroup ? ICONS.group : ICONS[widget] ?? ICONS.shape, 16)]),
      layerName(e, widget, selected),
      el('span', { class: 'layer-tools' }, [
        btn(t('layers.up'), ICONS.up, () => store.dispatch('reorder', { id: e.id, index: neighbors ? neighbors.up : index + 1 }), neighbors ? neighbors.up == null : index === total - 1),
        btn(t('layers.down'), ICONS.down, () => store.dispatch('reorder', { id: e.id, index: neighbors ? neighbors.down : index - 1 }), neighbors ? neighbors.down == null : index === 0),
        btn(e.visible === false ? t('common.show') : t('common.hide'), e.visible === false ? ICONS.eyeOff : ICONS.eye, () => store.dispatch('update', { id: e.id, patch: { visible: e.visible === false } }), false, e.visible !== false, e.visible === false ? ' layer-flag' : ''),
        btn(e.locked ? t('common.unlock') : t('common.lock'), e.locked ? ICONS.lock : ICONS.unlock, () => store.dispatch('update', { id: e.id, patch: { locked: !e.locked } }), false, Boolean(e.locked), e.locked ? ' layer-flag' : ''),
      ]),
    ]);
  }

  /** Cards and groups: what "Expand all" and "Collapse all" open and close. */
  const containers = () => store.getState().theme.elements.filter((e) => e.card || e.isGroup).map((e) => e.id);

  function faceHeading(card, face, elements) {
    const active = face !== null && card.card.activeFace === face;
    const label = face === null ? t('cardInsp.base') : card.card.faces[face];
    const heading = face === null
      ? el('span', { class: 'layer-group-title', text: label })
      : el('button', {
        type: 'button', class: 'layer-group-title', text: label,
        'aria-pressed': String(active),
        'aria-current': active ? 'true' : null,
        onclick: () => {
          if (!active) store.dispatch('cardFace', { id: card.id, face });
          else store.select([card.id]);
        },
      });
    const count = objectsText(elements.length);
    return el('div', { class: 'layer-group-heading', dataset: { dropCard: card.id, dropFace: face === null ? 'base' : face } }, [
      icon(face === null ? ICONS.base : ICONS.face, 14),
      heading,
      // The base is always drawn; a face says how many objects it holds.
      el('small', { class: 'layer-meta', text: face === null ? t('layersPanel.alwaysShown') : count, title: face === null ? count : t('layersPanel.faceNumber', { number: face + 1 }) }),
      active && el('span', { class: 'face-badge', title: t('layers.inViewHint'), text: t('layers.inView') }),
    ]);
  }

  function renderNodes(nodes, els) {
    const indexOf = e => els.findIndex(item => item.id === e.id);
    return nodes.map((node, i) => {
      const { element: card, groups, children } = node;
      const row = layerRow(card, indexOf(card), els.length, {
        up: i > 0 ? indexOf(nodes[i - 1].element) : null,
        down: i + 1 < nodes.length ? indexOf(nodes[i + 1].element) : null,
      });
      if (card.isGroup) return el('li', { class: 'layer-card', dataset: { groupId: card.id } }, [
        el('ul', { class: 'layer-list' }, [row]),
        el('ul', { class: 'layer-list layer-children', hidden: !layerQuery.trim() && layerType === 'all' && collapsedLayers.has(card.id) }, renderNodes(children, els)),
      ]);
      if (!groups.length) return row;
      return el('li', { class: 'layer-card', dataset: { cardId: card.id } }, [
        el('ul', { class: 'layer-list' }, [row]),
        el('ul', { class: 'layer-groups', hidden: !layerQuery.trim() && layerType === 'all' && collapsedLayers.has(card.id) }, groups.map(({ face, elements, nodes: members }) => {
          const active = face !== null && card.card.activeFace === face;
          const drop = { dropCard: card.id, dropFace: face === null ? 'base' : face };
          return el('li', { class: `layer-group${active ? ' active-face' : ''}`, dataset: { face: face === null ? 'base' : face } }, [
            faceHeading(card, face, elements),
            el('ul', { class: 'layer-list layer-children' }, members.length ? renderNodes(members, els) : [el('li', { class: 'layer-empty', dataset: drop, text: t('layers.dropHere') })]),
          ]);
        })),
      ]);
    });
  }

  /** Two or more layers selected: how many, Deselect, Group (Ctrl G) and Make card. */
  function renderSelectionBar() {
    const { theme, selection } = store.getState();
    const bar = $('layer-selection');
    bar.hidden = selection.length < 2;
    if (bar.hidden) {
      bar.replaceChildren();
      return;
    }
    const chosen = theme.elements.filter((e) => selection.includes(e.id));
    const groupable = canGroup(theme, selection);
    const cardable = chosen.some((e) => !e.locked && !e.card);
    bar.replaceChildren(
      el('div', { class: 'layer-selection-head' }, [
        el('strong', { text: t('layersPanel.selected', { count: selection.length }) }),
        el('button', { type: 'button', class: 'text-button quiet', text: t('layersPanel.clear'), onclick: () => store.select([]) }),
      ]),
      el('div', { class: 'layer-selection-actions' }, [
        // The shortcut is in the title: three actions share one row (D-14).
        el('button', { type: 'button', class: 'text-button', 'aria-label': t('layersPanel.groupLabel'), title: groupable ? `${t('layersPanel.groupLabel')} (${t('layersPanel.groupKeys')})` : t('group.scope'), 'aria-keyshortcuts': 'Control+G', disabled: !groupable, onclick: () => store.dispatch('groupSelection', { ids: selection }) }, [
          icon(ICONS.group, 16), el('span', { text: t('layersPanel.group') }),
        ]),
        el('button', { type: 'button', class: 'text-button', 'aria-label': t('layersPanel.cardLabel'), disabled: !cardable, onclick: () => store.dispatch('cardFromSelection', { ids: selection }) }, [
          icon(ICONS.card, 16), el('span', { text: t('layersPanel.card') }),
        ]),
        el('button', { type: 'button', class: 'text-button', 'aria-haspopup': 'menu', 'aria-expanded': 'false', disabled: chosen.every((e) => e.locked),
          onclick: (evt) => openMenuAt(evt.currentTarget, { label: t('inspector.alignTools'), items: alignItems({ store, t, ids: selection }) }) }, [
          icon(ICONS.alignLeft, 16), el('span', { text: t('inspector.alignTools') }),
        ]),
      ]),
    );
  }

  function renderLayers() {
    const els = store.getState().theme.elements;
    const list = $('layer-list');
    renderSelectionBar();
    renderLayerTypeFilter();
    const nodes = filterLayers(layerGroups(els), layerQuery, (e) => t(`widget.${widgetOf(e)}`), layerType);
    // How many objects, beside the title; the list reads from the topmost down.
    $('layer-count').textContent = els.length ? objectsText(countLayers(nodes)) : '';
    $('layer-list').setAttribute('aria-label', t('layersPanel.front'));
    if (els.length === 0) {
      list.replaceChildren(el('li', { class: 'empty-note', text: t('layers.empty') }));
      return;
    }
    if (!nodes.length) {
      list.replaceChildren(el('li', { class: 'empty-note', text: t('layersPanel.noMatch') }));
      return;
    }
    list.replaceChildren(el('li', { class: 'layer-root-drop', dataset: { rootDrop: 'true' }, text: t('layers.rootDrop') }), ...renderNodes(nodes, els));
    layerDrag.refresh();
  }

  mountSearch('layer-search', 'layersPanel.filter', (q) => { layerQuery = q; renderLayers(); }, { id: 'layer-filter' });
  // The count joins the title; the hint closes the panel (D-14).
  $('panel-layers').querySelector('.panel-head h2').after(el('span', { class: 'count', id: 'layer-count' }));
  $('layer-selection').after(el('p', { class: 'hint panel-foot', id: 'layer-hint', 'data-i18n': 'layersPanel.hint', text: t('layersPanel.hint') }));
  const layerTypeField = selectField(t('layersPanel.type'), layerType, [], value => { layerType = value; renderLayers(); });
  const layerTypeSelect = layerTypeField.querySelector('select');
  layerTypeSelect.id = 'layer-type-filter';
  layerTypeField.htmlFor = layerTypeSelect.id;
  $('layer-type').replaceChildren(layerTypeField);
  const { button: layerFilters } = filtersToggle($('layer-search'), $('layer-type'));
  let layerTypesSignature = '';
  function renderLayerTypeFilter() {
    const options = [['all', t('layersPanel.allTypes')], ...[...WIDGETS, 'group'].map(type => [type, t(`widget.${type}`)])];
    const signature = JSON.stringify([t('layersPanel.type'), options]);
    if (signature !== layerTypesSignature) {
      layerTypesSignature = signature;
      layerTypeField.querySelector('span').textContent = t('layersPanel.type');
      layerTypeSelect.replaceChildren(...options.map(([value, text]) => el('option', { value, text })));
    }
    layerTypeSelect.value = layerType;
    layerFilters.title = t('layersPanel.filters');
    layerFilters.setAttribute('aria-label', t('layersPanel.filters'));
    layerFilters.classList.toggle('filters-on', layerType !== 'all');
  }
  $('layers-expand').addEventListener('click', () => { collapsedLayers.clear(); renderLayers(); });
  $('layers-collapse').addEventListener('click', () => {
    for (const id of containers()) collapsedLayers.add(id);
    renderLayers();
  });

  // -------------------------------------------------------------- themes --
  // Which themes the gallery lists (remembered), the thumbnails it got (by
  // location and revision; `null`: the theme cannot be drawn) and the mini
  // screens of the cards shown, to fill in as thumbnails arrive.
  let filter = rememberedFilter(themeFilter);
  const thumbnails = new Map();
  const cardScreens = new Map();
  const waiting = [];
  let fetching = false;
  // The screen the gallery filtered for (once listed), to list again when it changes.
  let filteredFor = null;

  /** The screen in use (the one chosen at the top), or `null`. */
  function screenInUse() {
    const [screens, current] = screenArgs;
    return screens.find((s) => s.key === current) ?? null;
  }

  /** What the gallery's filter depends on of `screen`: its key and models. */
  const signature = (screen) => (screen ? `${screen.key} ${screen.models.map((m) => m.id).join(' ')}` : '');

  /** Fills a card's mini screen with its thumbnail, or marks it without one. */
  function paintThumbnail(screen, url) {
    screen.dataset.state = url ? 'ready' : url === null ? 'none' : 'loading';
    screen.style.backgroundImage = url ? `url("${url}")` : '';
    const thumb = screen.parentElement;
    thumb?.querySelector('.no-preview')?.remove();
    if (url === null) thumb?.append(el('span', { class: 'badge no-preview' }, [icon(ICONS.eyeOff, 14), el('span', { text: t('themes.noPreview') })]));
  }

  /** Asks for the missing thumbnails of the cards shown, one at a time, while the tab is open. */
  async function fetchThumbnails() {
    if (fetching) return;
    fetching = true;
    while (waiting.length && !$('panel-themes').hidden) {
      const entry = waiting.shift();
      const key = thumbnailKey(entry);
      if (thumbnails.has(key)) continue;
      const url = bridge ? await themePreview(bridge, entry) : await Promise.resolve(actions.themeThumbnail?.(entry.location)).catch(() => null);
      thumbnails.set(key, url ?? null);
      const screen = cardScreens.get(key);
      if (screen) paintThumbnail(screen, url ?? null);
    }
    fetching = false;
  }

  /** A card's second line: its screen, and when its files last changed. */
  function themeFacts(th) {
    const label = screenLabel(th, locale());
    return Number.isFinite(th.modifiedMs) ? `${label} · ${timeAgo(th.modifiedMs, Date.now(), locale())}` : label;
  }

  function themeCard(th) {
    const axis = axisOf(th);
    const key = thumbnailKey(th);
    const screen = screenThumb({ width: th.canvas.width, height: th.canvas.height });
    screen.classList.add('thumb-screen');
    cardScreens.set(key, screen);
    const thumb = el('span', { class: 'thumb', dataset: { axis }, 'aria-hidden': 'true' }, [screen]);
    paintThumbnail(screen, thumbnails.has(key) ? thumbnails.get(key) : th.thumbnail);
    const tools = gallery.tools(th);
    return el('li', { class: 'theme-item' }, [
      el('button', { type: 'button', class: 'theme-card', dataset: { location: th.location }, onclick: () => { if (!gallery.select(th)) actions.openTheme(th.location); } }, [
        thumb,
        el('strong', { text: th.name }),
        el('small', { class: 'theme-screen', text: themeFacts(th) }),
        el('span', { class: 'card-meta' }, [
          el('span', { class: 'badge' }, [icon(ICONS[axis], 14), t(`axis.${axis}`)]),
          th.bundled && el('span', { class: 'tag', text: t('themes.bundled') }),
          ...themeUses(th, gallery.overview()).filter(use => ['screen', 'profile'].includes(use.kind)).map(use => el('span', { class: 'badge', text: use.kind === 'screen' ? t('themesPanel.onScreen') : use.name })),
          missingSensors(th, catalog).length > 0 && el('span', { class: 'badge theme-missing', text: t('themesPanel.missing') }),
        ]),
      ]),
      tools,
      el('button', {
        type: 'button', class: 'icon-button quiet theme-duplicate', title: t('common.duplicate'), 'aria-label': t('themesPanel.duplicateLabel', { name: th.name }),
        onclick: () => duplicateTheme(th),
      }, [icon(ICONS.copy, 16)]),
    ]);
  }

  /** "New theme": vertical or horizontal, empty, in the size of the screen in use. */
  function newThemeItem() {
    const model = screenInUse()?.models[0];
    const choice = (axis) => el('button', {
      type: 'button', class: 'text-button quiet', id: `theme-new-${axis}`,
      'aria-label': t(axis === 'vertical' ? 'themes.newVertical' : 'themes.newHorizontal'),
      onclick: () => { gallery.close(); actions.newTheme(axis); },
    }, [icon(ICONS[axis], 14), el('span', { text: t(`axis.${axis}`) })]);
    const size = model ? `${Math.min(model.width, model.height)}×${Math.max(model.width, model.height)}` : null;
    return el('li', { class: 'theme-new' }, [
      kindIcon('plus'),
      el('strong', { text: t('themesPanel.new') }),
      size && el('span', { class: 'theme-new-size', text: size }),
      choice('vertical'),
      choice('horizontal'),
      el('button', { type: 'button', class: 'text-button quiet', text: t('themesPanel.fromPreset'), onclick: event => gallery.presets(themes.filter(theme => theme.bundled), event.currentTarget) }),
    ]);
  }

  /** What the gallery shows when the filter keeps no theme: why, and "Show all". */
  function emptyItem({ key, showAll }) {
    return el('li', { class: 'empty-state' }, [
      el('p', { class: 'empty-note', text: t(key) }),
      showAll && el('button', { type: 'button', class: 'text-button show-all', text: t('themes.showAll'), onclick: () => {
        chooseFilter(SHOW_ALL);
        document.querySelector('#theme-scope [data-scope="all"]')?.focus();
      } }),
    ]);
  }

  /** The filter buttons say what is chosen; "For this screen" needs a screen. */
  function syncFilterButtons(screen) {
    const scope = scopeIn(filter, screen);
    for (const button of document.querySelectorAll('#theme-scope [data-scope]')) {
      button.setAttribute('aria-pressed', String(button.dataset.scope === scope));
      if (button.dataset.scope !== 'screen') continue;
      button.disabled = !screen;
      const model = screen?.models.length === 1 ? screen.models[0].name : screen?.key;
      button.title = screen ? t('themes.forScreenHint', { name: model }) : t('themes.forScreenNone');
    }
    for (const button of document.querySelectorAll('#theme-axis [data-axis]')) {
      button.setAttribute('aria-pressed', String(button.dataset.axis === filter.axis));
    }
  }

  /** The head's count: how many the gallery shows, and how many fit the screen in use. */
  function themeCountText(shown, list, screen) {
    const count = countText(shown, list.length);
    if (!count) { $('theme-screen-count').textContent = ''; return ''; }
    const fit = screenCount(list, screen);
    const text = t(count.key, count.params);
    $('theme-screen-count').textContent = fit === null ? '' : ` \u00b7 ${t('themesPanel.forScreen', { count: fit })}`;
    return text;
  }

  function renderThemes(list) {
    themes = list;
    const screen = screenInUse();
    filteredFor = signature(screen);
    syncFilterButtons(screen);
    const filtered = filterThemes(list, filter, screen);
    const shown = recentThemes(searchThemes(filtered, themeQuery, catalog));
    $('theme-count').textContent = themeCountText(shown.length, list, screen);
    const grid = $('theme-grid');
    cardScreens.clear();
    if (!shown.length) {
      grid.replaceChildren(newThemeItem(), emptyItem(filtered.length ? { key: 'themesPanel.noMatch', showAll: false } : emptyState(list, filter, screen)));
      gallery.update(shown);
      return;
    }
    grid.replaceChildren(newThemeItem(), ...shown.map(themeCard));
    waiting.splice(0, waiting.length, ...shown.filter((entry) => !thumbnails.has(thumbnailKey(entry))));
    gallery.update(shown);
    fetchThumbnails();
  }

  /** Changes the filter (what is not given stays), remembers it and lists again. */
  function chooseFilter(next) {
    filter = { ...filter, ...next };
    actions.rememberThemeFilter?.(filter);
    renderThemes(themes);
  }

  /** Lists again when the screen in use changed (another model, or none). */
  function screenChanged() {
    if (filteredFor !== null && signature(screenInUse()) !== filteredFor) renderThemes(themes);
  }

  for (const button of document.querySelectorAll('#theme-scope [data-scope]')) {
    button.addEventListener('click', () => chooseFilter({ scope: button.dataset.scope }));
  }
  for (const button of document.querySelectorAll('#theme-axis [data-axis]')) {
    button.addEventListener('click', () => chooseFilter({ axis: button.dataset.axis }));
  }
  mountSearch('theme-search', 'themesPanel.search', (q) => { themeQuery = q; renderThemes(themes); }, { id: 'theme-search-input' });
  document.querySelector('.theme-filter').id = 'theme-filters';
  const { button: themeFilters } = filtersToggle($('theme-search'), document.querySelector('.theme-filter'));
  themeFilters.title = t('themesPanel.filters'); themeFilters.setAttribute('aria-label', t('themesPanel.filters'));
  const recentLabel = el('span', { class: 'hint', text: t('themesPanel.recent') });
  document.querySelector('.theme-filter').append(recentLabel);

  /** Lists what an import could not map exactly, until dismissed; `null` clears it. */
  function showImportReport(report) {
    importReport = report;
    const root = $('import-report');
    if (!report?.warnings?.length) {
      root.replaceChildren();
      return;
    }
    const count = report.warnings.length;
    const dismiss = el('button', {
      type: 'button', class: 'icon-button', title: t('import.dismiss'), 'aria-label': t('import.dismiss'),
      onclick: () => { showImportReport(null); $('theme-import').focus(); },
    }, [icon(ICONS.close, 16)]);
    root.replaceChildren(el('section', { class: 'notice', 'aria-labelledby': 'import-report-title' }, [
      el('div', { class: 'notice-head' }, [icon(ICONS.warning, 18), el('h2', { id: 'import-report-title', text: t('import.title') }), dismiss]),
      el('p', { text: count === 1 ? t('import.summaryOne', { name: report.name }) : t('import.summary', { name: report.name, count }) }),
      el('ul', {}, report.warnings.map((w) => el('li', { text: warningText(t, w) }))),
    ]));
  }

  $('theme-open').addEventListener('click', () => actions.refreshThemes());
  $('theme-import').addEventListener('click', () => actions.importTheme());
  $('theme-save-as').addEventListener('click', () => actions.saveThemeAs());

  // --------------------------------------------------------------- media --
  /** A file's badge: Video, else its format (`PNG`, `GIF`). */
  function kindOf(a) {
    if (a.kind === 'video') return t('common.video');
    const name = fileNameOf(a.ref);
    const ext = name.includes('.') ? name.split('.').pop().toUpperCase() : '';
    return ext || (moves(a) ? t('collection.kind.gif') : t('media.kindImage'));
  }

  /**
   * One picture or video of the theme. A video, or an animated GIF, shows
   * its poster, its kind, play time and size, and becomes a video
   * background; a picture becomes a picture background or an image element.
   */
  /** "In use: background, CPU +2": where the theme uses a file, from its real references. */
  function usesText(uses) {
    const names = uses.map((u) => (u.background ? t('common.background').toLocaleLowerCase(locale()) : u.name));
    const shown = names.slice(0, 2).join(', ');
    return t('mediaPanel.usedBy', { where: names.length > 2 ? `${shown} +${names.length - 2}` : shown });
  }

  function mediaItem(a, assets) {
    const moving = moves(a);
    const poster = moving && a.poster ? assets.find((p) => p.ref === a.poster)?.dataUrl : null;
    const preview = poster ?? a.dataUrl ?? null;
    const label = a.kind === 'video' ? t('common.video') : moving ? t('media.animatedGif') : kindOf(a);
    const details = moving ? [label, ...videoFacts(a, (n) => formatBytes(n, locale()))].join(' · ')
      : [label, Number.isFinite(a.bytes) ? formatBytes(a.bytes, locale()) : null].filter(Boolean).join(' · ');
    const uses = assetUses(store.getState().theme, a.ref);
    const action = (text, paths, onclick, extra = '') => el('button', { type: 'button', class: `icon-button quiet${extra}`, title: text, 'aria-label': text, onclick }, [icon(paths, 16)]);
    // Every file says where it is used; only an unused one can go (D-2026-10-20-studio-ux-v2-14).
    return el('li', { class: `media-item${uses.length ? ' in-use' : ''}`, dataset: { ref: a.ref, uses: String(uses.length) } }, [
      el('span', { class: 'thumb', style: preview ? { backgroundImage: `url(${preview})` } : {} }, [
        preview ? null : icon(a.kind === 'video' ? ICONS.video : ICONS.image, 22),
        uses.length ? el('span', { class: 'tag media-uses', text: usesText(uses) }) : null,
      ]),
      el('span', { class: 'media-name' }, [
        el('span', { text: fileNameOf(a.ref) }),
        details ? el('small', { text: details }) : null,
      ]),
      el('div', { class: 'actions' }, [
        moving && action(t('common.preview'), ICONS.play, () => actions.previewVideo(a)),
        action(t('collection.useBackground'), ICONS.background, () => store.dispatch('setTheme', { patch: { background: backgroundOf(a) } })),
        a.kind === 'image' && action(t('media.addImage'), ICONS.place, () => {
          const { x, y } = center();
          store.dispatch('add', { widget: 'image', x, y });
          const id = store.getState().selection[0];
          store.dispatch('update', { id, patch: { kind: { asset: a.ref } } });
        }),
        uses.length ? null : action(t('mediaPanel.remove', { name: fileNameOf(a.ref) }), ICONS.trash, () => actions.removeAsset(a), ' media-remove'),
      ]),
    ]);
  }

  /** Beside the title: how many files and their size; "In the theme" folds with its count. */
  function mediaHead(items) {
    const { count, bytes } = mediaTotals(items);
    const files = count === 1 ? t('media.countOne') : count ? t('media.count', { count }) : null;
    $('media-count').textContent = files && bytes !== null ? `${files} · ${formatBytes(bytes, locale())}` : files ?? '';
    const section = accordionSection({
      id: 'theme', title: t('media.inTheme'), icon: 'image', summary: String(count), keepSummary: true,
      expanded: mediaMemory.isOpen('media', 'theme'),
      onToggle: () => { mediaMemory.toggle('media', 'theme'); renderMedia(media); $('media-head').querySelector('.accordion-toggle')?.focus(); },
      render: (body) => body.append($('media-list'), mediaHint),
    }).element;
    section.classList.add('media-group');
    $('media-head').replaceChildren(section);
  }
  const mediaHint = el('p', { class: 'hint media-hint', 'data-i18n': 'mediaPanel.usesHint', text: t('mediaPanel.usesHint') });

  function renderMedia(assets) {
    media = assets;
    const list = $('media-list');
    const items = mediaItems(assets);
    mediaHead(items);
    usesSignature = signatureOfUses();
    if (!items.length) {
      list.replaceChildren(el('li', { class: 'empty-note', text: t('media.empty') }));
      return;
    }
    list.replaceChildren(...items.map((a) => mediaItem(a, assets)));
  }

  // An edit that changes where a file is used redraws the list (its badge, its trash).
  let usesSignature = '';
  const signatureOfUses = () => JSON.stringify(mediaItems(media ?? []).map((a) => assetUses(store.getState().theme, a.ref)));
  store.subscribe?.(() => {
    if (!media || signatureOfUses() === usesSignature) return;
    renderMedia(media);
  });

  /** While a file is added (a poster may take seconds), the add buttons wait. */
  function setAdding(on) {
    $('media-add').disabled = on;
    $('media-add-video').disabled = on;
    mediaPlus.disabled = on;
    $('media-status').hidden = !on;
    $('panel-media').setAttribute('aria-busy', String(on));
  }

  $('media-add').addEventListener('click', () => actions.addImage());
  $('media-add-video').addEventListener('click', () => actions.addVideo());
  // The compact drop area (D-14): one + offers a picture or a video, the two
  // existing commands; the files are still dropped anywhere on the area.
  const mediaPlus = el('button', { type: 'button', class: 'icon-button media-plus', 'aria-haspopup': 'menu', 'aria-expanded': 'false',
    onclick: (evt) => openMenuAt(evt.currentTarget, { label: t('mediaPanel.choose'), items: [
      { label: t('media.add'), run: () => actions.addImage() },
      { label: t('media.addVideo'), run: () => actions.addVideo() },
    ] }) }, [icon(ICONS.plus, 18)]);
  $('panel-media').querySelector('.media-drop').append(mediaPlus);
  $('panel-media').querySelector('.panel-head h2').after(el('span', { class: 'count', id: 'media-count' }));
  // "From the collection" and "Icons" lead to their tabs.
  const mediaLink = (subtab, iconName, title, summary) => el('button', { type: 'button', class: 'media-link', dataset: { subtab }, onclick: () => $(`subtab-media-${subtab}`).click() }, [
    icon(ICONS[iconName], 16), el('span', { class: 'media-link-title', text: title }), el('span', { class: 'media-link-summary', text: summary }), icon(ICONS.chevronRight, 12),
  ]);
  function renderMediaLinks() {
    $('media-theme').querySelector('.media-links')?.remove();
    $('media-theme').append(el('div', { class: 'media-links' }, [
      mediaLink('collection', 'grid', t('mediaPanel.fromCollection'), t('mediaPanel.collectionSummary')),
      mediaLink('icons', 'star', t('media.icons'), t('mediaPanel.iconsSummary')),
    ]));
    mediaPlus.title = t('mediaPanel.choose');
    mediaPlus.setAttribute('aria-label', t('mediaPanel.choose'));
  }
  renderMediaLinks();
  // Each origin with its icon (board LibraryPanelsNew); the text moves into a span the translations fill.
  for (const [subtab, name] of [['theme', 'image'], ['collection', 'grid'], ['icons', 'star']]) {
    const tabButton = $(`subtab-media-${subtab}`);
    if (!tabButton?.dataset.i18n) continue;
    tabButton.replaceChildren(icon(ICONS[name], 14), el('span', { 'data-i18n': tabButton.dataset.i18n, text: t(tabButton.dataset.i18n) }));
    delete tabButton.dataset.i18n;
  }

  // "This theme" | "Collection" (D-2026-10-01-gif-sticker-search-5): the
  // collection is where GIFs and stickers are searched on KLIPY.
  const iconPicker = createIconPicker({ store, canvas, stage, t, pickers, add: actions.addIcon });
  wireSubtabs($('panel-media').querySelector('.subtabs'), (name) => {
    if (name === 'icons') void iconPicker.show();
    actions.mediaSubtab?.(name);
  });
  $('gif-search-open').addEventListener('click', () => actions.searchGifs());

  // -------------------------------------------------------------- screen --
  /** The screens listed and the one in use: the Themes tab filters by it. */
  function setScreens(screens, current) {
    screenArgs = [screens, current];
    screenChanged();
  }

  return {
    renderWidgets,
    setCatalog(next) {
      catalog = next;
      renderSensors();
      renderThemes(themes);
      libreShown = null;
      renderLibre();
    },
    updateReadings,
    setSensorError(error) { sensorError = Boolean(error); renderSensors(); },
    openSensors({ probe = 'all', status = 'all' } = {}) {
      sensorProbe = probe;
      sensorStatus = status;
      sensorSearch.value = '';
      // A filtered opening shows its filters.
      showFilters(probe !== 'all' || status !== 'all');
      renderSensors();
      selectTab(tabs.find(tab => tab.dataset.tab === 'sensors'));
      probeSelect.focus();
    },
    renderLayers,
    renderThemes,
    openThemes() { selectTab(tabs.find(tab => tab.dataset.tab === 'themes')); return gallery.show(); },
    closeThemes: gallery.close,
    showImportReport,
    renderMedia,
    setAdding,
    setScreens,
    selectTab: (name) => selectTab(tabs.find((x) => x.dataset.tab === name)),
    /** The UI's language changed: every panel is drawn again. */
    retranslate() {
      for (const { input, labelKey, placeholderKey } of searches) {
        input.setAttribute('aria-label', t(labelKey));
        input.placeholder = t(placeholderKey);
      }
      gallery.retranslate();
      recentLabel.textContent = t('themesPanel.recent');
      themeFilters.title = t('themesPanel.filters'); themeFilters.setAttribute('aria-label', t('themesPanel.filters'));
      iconPicker.retranslate();
      renderWidgets();
      renderSensors();
      renderRate();
      libreShown = null;
      renderLibre();
      renderLayers();
      renderThemes(themes);
      showImportReport(importReport);
      renderMedia(media);
    },
  };
}
