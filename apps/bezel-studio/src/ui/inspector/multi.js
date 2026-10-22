import { orderField } from './order.js';
import { describeSections } from './sections.js';
// Existing multi-selection tools, without inspector navigation.
import { canGroup } from '../../editor/cards.js';
import { el, icon } from '../dom.js';
import { ICONS } from '../icons.js';
import { propertySection as section, fieldGrid, numberField, objectHeader, chipButton, iconButton, iconChoice } from '../fields.js';
import { commonSelection, selectionPatches, MIXED } from '../../inspector-edit.js';
import { widgetOf } from '../../editor/widgets.js';
import { sensorPicker } from '../sensor-picker.js';
import { createInspectorTrial, TRIAL_FRAME_MS } from '../../inspector-trial.js';
import { createPreviewAnimation } from '../../preview-animation.js';
import { openMenuAt } from '../menu.js';
import { objectMenuItems } from '../object-menu.js';

export function header({ ids, t, store, host, video }) {
  const elements = store.getState().theme.elements.filter(e => ids.includes(e.id));
  const preview = el('div', { class: 'multi-preview', 'aria-hidden': 'true' });
  const motion = () => video.context().motion !== false && !window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const entries = elements.slice(0, 3).map(element => {
    const canvas = el('canvas', { width: 72, height: 72 });
    preview.append(canvas);
    return { element, canvas, trial: createInspectorTrial() };
  });
  let rendering = false;
  const animation = createPreviewAnimation({ request: () => render(), perSecond: 30, enabled: () => preview.isConnected && !document.hidden && motion() });
  const tryButton = iconButton(t('insp.tryObject'), ICONS.play, () => {
    entries.forEach(entry => entry.trial.start(entry.element));
    render();
  }, { class: 'quiet inspector-try', disabled: !motion(), 'aria-pressed': 'false' });
  async function render(initial = false) {
    if (rendering) return;
    if (!initial && (!preview.isConnected || document.hidden || !motion())) {
      entries.forEach(entry => entry.trial.stop()); animation.stop(); tryButton.setAttribute('aria-pressed', 'false'); return;
    }
    rendering = true;
    const started = performance.now();
    try {
      await Promise.all(entries.map(async ({ element, canvas, trial }) => {
        const trialMs = trial.at(element, motion());
        const frame = await host.renderElement(element.id, { max: 72, motion: trialMs !== null, ...(trialMs !== null ? { trialMs } : {}) });
        if (!frame?.width || !frame?.height || !canvas.isConnected) return;
        const source = document.createElement('canvas'); source.width = frame.width; source.height = frame.height;
        source.getContext('2d').putImageData(new ImageData(new Uint8ClampedArray(frame.pixels ?? frame.rgba), frame.width, frame.height), 0, 0);
        const scale = Math.min(72 / frame.width, 72 / frame.height);
        canvas.getContext('2d').clearRect(0, 0, 72, 72);
        canvas.getContext('2d').drawImage(source, (72 - frame.width * scale) / 2, (72 - frame.height * scale) / 2, frame.width * scale, frame.height * scale);
      }));
      const active = entries.some(entry => entry.trial.at(entry.element, motion()) !== null);
      tryButton.setAttribute('aria-pressed', String(active));
      animation.shown({ nextMs: active ? TRIAL_FRAME_MS : null, elapsed: performance.now() - started });
    } catch {
      animation.stop(); entries.forEach(entry => entry.trial.stop()); tryButton.setAttribute('aria-pressed', 'false');
    } finally { rendering = false; }
  }
  render(true);
  return el('div', { class: 'multi-heading' }, [
    objectHeader({
      preview,
      name: el('h2', { text: t('inspector.multi', { count: ids.length }) }),
      subtitle: el('span', { class: 'inspector-sub', text: [...new Set(elements.map(e => t(`widget.${widgetOf(e)}`)))].join(' · ') }),
      tryButton,
      menuButton: iconButton(t('insp.more'), ICONS.more, event => openMenuAt(event.currentTarget, { label: t('menu.title'), items: objectMenuItems({ store, t, fitContent: id => host.fitContent?.(id) }) }), { class: 'quiet object-menu-button', 'aria-haspopup': 'menu' }),
    }),
    el('div', { class: 'chip-group multi-objects', role: 'group', 'aria-label': t('insp.selectedObjects') }, elements.map(e => { const chip = chipButton(e.name, true, () => store.select(ids.filter(id => id !== e.id)), { title: t('layersPanel.clear'), 'aria-description': t('layersPanel.clear') }); chip.append(icon(ICONS.close, 12)); return chip; })),
  ]);
}

function commonPanels({ ids, t, store, colorField, sensors, host }) {
  const elements = store.getState().theme.elements.filter(e => ids.includes(e.id));
  const shared = commonSelection(elements);
  const edit = (field, value) => {
    if (value === null) return;
    if (field === 'rotation') store.dispatch('rotateSelection', { ids, rotation: value });
    else store.dispatch('patchElements', { patches: selectionPatches(elements, field, value) });
  };
  const numeric = (key, label, value, scale = 1) => {
    const field = numberField(t(label), value === MIXED ? null : value * scale, value => edit(key, value === null ? null : value / scale), { nullable: true, compact: true, ...(key in shared.frame ? { prefix: t(`insp.${({ x: 'xPrefix', y: 'yPrefix', width: 'widthPrefix', height: 'heightPrefix' })[key]}`) } : {}), unit: key === 'opacity' ? '%' : key === 'rotation' ? '°' : 'px', ...(key === 'opacity' ? { min: 0, max: 100 } : {}) });
    if (value === MIXED) field.querySelector('input').placeholder = t('insp.mixed');
    return field;
  };
  let paint = null;
  if (shared.paint !== null) {
    paint = colorField(t('insp.mainColor'), shared.paint === MIXED ? '#ffffffff' : shared.paint, value => edit('paint', value));
    if (shared.paint === MIXED) {
      paint.classList.add('is-mixed');
      paint.querySelector('.color-button-value').textContent = t('insp.mixed');
      paint.querySelector('.swatch')?.style.setProperty('--swatch', 'transparent');
    }
  }
  return {
    geometry: section('position', t('inspector.position'), [
      fieldGrid(['x', 'y'].map(key => numeric(key, `inspector.${key}`, shared.frame[key]))),
      fieldGrid(['width', 'height'].map(key => numeric(key, `inspector.${key}`, shared.frame[key]))),
      shared.rotation !== null && numeric('rotation', 'insp.rotation', shared.rotation),
    ], { heading: false }),
    data: [section('content', t('insp.commonProperties'), [
      shared.sensor !== null && sensorPicker({ t, key: shared.sensor === MIXED ? '' : shared.sensor, label: t('inspector.sensor'), catalog: () => sensors.catalog(), readings: () => host.lastSample()?.readings,
        summary: shared.sensor === MIXED ? { title: t('insp.mixedSensors', { count: new Set(elements.map(e => e.kind.binding?.key ?? e.kind.content?.key)).size }), value: t('insp.mixed') } : null,
        onChange: key => edit('sensor', key) }),
      paint,
      numeric('opacity', 'inspector.opacity', shared.opacity, 100),
      el('p', { class: 'hint', text: t('insp.commonHint') }),
    ])],
  };
}


export function panel({ ids, t, store, ALIGN_EDGES }) {
  function multiForm(ids) {
    return [
      el('h2', { text: t('inspector.multi', { count: ids.length }) }),
      section('position', t('inspector.alignTools'), [
        iconChoice(t('inspector.alignTools'), null, ALIGN_EDGES.map(([edge, name]) => [edge, t(`align.${edge}`), name]), edge => store.dispatch('align', { ids, edge }), { wide: true, iconOnly: true, actions: true }),
        iconChoice(t('insp.distribute'), null, [['x', t('align.distributeX'), 'distributeX', { disabled: ids.length < 3, text: t('axis.horizontal') }], ['y', t('align.distributeY'), 'distributeY', { disabled: ids.length < 3, text: t('axis.vertical') }]], axis => store.dispatch('distribute', { ids, axis }), { wide: true, actions: true }),
      ]),
      section('membership', t('inspector.membership'), [
        iconChoice(t('inspector.membership'), null, [
          ['group', t('group.create'), 'group', { disabled: !canGroup(store.getState().theme, ids), text: t('layersPanel.group'), description: t('group.scope') }],
          ...(store.getState().theme.elements.some(e => ids.includes(e.id) && e.isGroup) ? [['ungroup', t('group.ungroup'), 'group']] : []),
          ['card', t('card.group'), 'card', { text: t('layersPanel.card') }],
        ], command => store.dispatch(({ group: 'groupSelection', ungroup: 'ungroup', card: 'cardFromSelection' })[command], { ids }), { wide: true, bare: true, actions: true }),
      ], { heading: false }),
      orderField({ store, t, ids }),
    ];
  }
  return multiForm(ids);
}

export function sections(ctx) {
  const shared = commonPanels(ctx);
  return describeSections(ctx, { layout: [shared.geometry, ...panel(ctx).slice(1)], data: shared.data }).map(descriptor => descriptor.id === 'data' ? { ...descriptor, title: ctx.t('insp.commonProperties') } : descriptor);
}
