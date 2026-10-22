// Shared text and styled-part editors, also usable by the clock and the
// needle's centre value. All model edits still go through the caller.
import { el } from '../dom.js';
import { fontPicker, numberField, prefixedField, iconChoice, partsRow, toggleField, iconButton, fieldGrid, textField, selectField, propertySection as section } from '../fields.js';
import { ICONS } from '../icons.js';
import { VALUE_PARTS, OVERFLOWS } from '../../text-layout.js';
import { partStylePatch } from '../../inspector-edit.js';
import { partPieces } from '../../editor/handles.js';

const selected = new WeakMap();
const partMemory = store => {
  if (!selected.has(store)) selected.set(store, new Map());
  return selected.get(store);
};

/** Picks `part` in the parts row of element `id` (`scope`: value or clock), as a click on it would. */
export function pickPart(store, id, scope, part) {
  partMemory(store).set(`${id}:${scope}`, part);
}

export function valueFormatFields(ctx, element, content, setContent) {
  const { t } = ctx, format = content.format ?? {};
  const decimals = format.decimals;
  const values = [...new Set(['auto', '0', '1', '2', '3', ...(decimals != null ? [String(decimals)] : [])])];
  return [
    fieldGrid([
      textField(t('inspector.prefix'), content.prefix ?? '', prefix => setContent({ prefix })),
      textField(t('inspector.suffix'), content.suffix ?? '', suffix => setContent({ suffix })),
    ]),
    fieldGrid([
      selectField(t('inspector.decimals'), decimals == null ? 'auto' : String(decimals), values.map(value => [value, value === 'auto' ? t('inspector.decimalsAuto') : value]), value => setContent({ format: { decimals: value === 'auto' ? null : Number(value) } })),
      toggleField(t('inspector.showUnit'), format.showUnit !== false, showUnit => setContent({ format: { showUnit } })),
    ]),
    ctx.sensors.catalog().find(sensor => sensor.key === (content.key ?? element.kind.binding?.key))?.quantity === 'celsius' && toggleField(t('inspector.fahrenheit'), Boolean(format.fahrenheit), fahrenheit => setContent({ format: { fahrenheit } })),
  ];
}

export function textStyleFields(ctx, element, { style = element.kind.style, setStyle = patch => ctx.update(element.id, { kind: { style: patch } }), parts = element.kind.content?.type === 'sensor', scope = 'value', tabular = true, partIds = VALUE_PARTS, partLabel = part => ctx.t(`insp.valuePart.${part}`), partsLabel = ctx.t('insp.valueParts') } = {}) {
  const { t } = ctx;
  const fields = ctx.fontFields(element, { style, setStyle, compact: true });
  if (parts) {
    const memory = partMemory(ctx.store), key = `${element.id}:${scope}`;
    const current = partIds.includes(memory.get(key)) ? memory.get(key) : partIds.includes('number') ? 'number' : partIds[0];
    const editPart = part => {
      const value = style.parts?.[part] ?? {};
      const patch = change => setStyle(partStylePatch(style, part, change));
      const { texts, source } = ctx.pickers.font(ctx.sampleText(element));
      const font = fontPicker({ ...texts, family: t('insp.partFont'), face: t('insp.partFace') }, { family: value.font ?? style.font.family, weight: value.weight ?? style.font.weight, italic: style.font.italic, asset: value.font ? value.fontAsset ?? null : style.font.asset }, font => patch({ font: font.family, fontAsset: font.asset ?? null, weight: font.weight }), source, { faceSample: t('insp.faceSample'), fixedItalic: Boolean(style.font.italic), hideLabels: true });
      const face = font.lastElementChild;
      return el('div', { class: 'part-editor', role: 'group', 'aria-label': t('insp.partEditor', { part: partLabel(part) }) }, [
        el('div', { class: 'part-editor-head' }, [el('strong', { text: partLabel(part) }), el('span', { class: 'hint', text: t('insp.partInheritanceHint') }), iconButton(t('insp.inheritPart'), ICONS.undo, () => patch(null))]),
        font,
        fieldGrid([
          ctx.colorField(t('insp.partColor'), value.color ?? style.paint, color => patch({ color }), { hideLabel: true }),
          prefixedField(t('insp.sizePrefix'), t('insp.partSize'), value.sizePercent, sizePercent => patch({ sizePercent: sizePercent === null ? null : Math.max(10, Math.min(400, sizePercent)) }), { min: 10, max: 400, unit: '%', nullable: true }),
        ], { tail: true }),
        el('div', { class: 'part-editor-face' }, [face, iconChoice(t('insp.partBaseline'), value.baseline ?? 'base', [['base', t('insp.baseline'), 'textBottom'], ['super', t('insp.superscript'), 'textTop']], baseline => patch({ baseline: baseline === 'base' ? null : baseline }), { wide: true, iconOnly: true, bare: true })]),
      ]);
    };
    let lastSample, lastSecond, cached = [];
    const preview = () => {
      const sample = ctx.host.lastSample(), second = Math.floor(Date.now() / 1000);
      if (sample === lastSample && second === lastSecond) return cached;
      lastSample = sample; lastSecond = second;
      const content = scope === 'needle-center' ? { ...element.kind.look.center, type: 'sensor', key: element.kind.binding.key } : element.kind.content;
      const value = { ...element, kind: { type: 'text', content, style } };
      cached = partPieces(value, { reading: sample?.readings?.[content.key], quantity: ctx.sensors.catalog().find(sensor => sensor.key === content.key)?.quantity, date: new Date() })?.pieces ?? [];
      return cached;
    };
    fields.unshift(partsRow(partsLabel, partIds.map(part => [part, partLabel(part)]), current,
      part => { memory.set(key, part); ctx.redraw(); }, editPart, { preview }));
  }
  if (tabular) fields.push(toggleField(t('insp.tabularDigits'), style.tabularDigits !== false, tabularDigits => setStyle({ tabularDigits }), { iconName: 'fixed-digits' }));
  return fields;
}

export function overflowField(ctx, element, { bare = false, style = element.kind.style, setStyle = patch => ctx.update(element.id, { kind: { style: patch } }) } = {}) {
  const { t } = ctx;
  // Imported legacy text without a mode keeps its original clipping until edited.
  const modes = style.overflow == null ? ['legacy', ...OVERFLOWS] : OVERFLOWS;
  return iconChoice(t('insp.textOverflow'), style.overflow ?? 'legacy', modes.map(mode => [mode, t(`insp.overflow.${mode}`)]), overflow => setStyle({ overflow: overflow === 'legacy' ? null : overflow }), { wide: true, bare });
}

export function shadowSection(ctx, element) {
  const { t, update } = ctx, shadow = element.kind.style.shadow;
  const set = shadow => update(element.id, { kind: { style: { shadow } } });
  const color = ctx.colorField(t('insp.shadowColor'), shadow?.color ?? '#00000099', color => set({ ...shadow, color }), { hideLabel: true });
  const blur = prefixedField(t('insp.blurPrefix'), t('insp.shadowBlur'), shadow?.blur ?? 8, blur => set({ ...shadow, blur: Math.max(0, Math.min(64, blur)) }), { min: 0, max: 64, unit: 'px' });
  for (const control of [...color.querySelectorAll('button'), ...blur.querySelectorAll('input')]) control.disabled = !shadow;
  return section('shadow', t('insp.textShadow'), [
    el('div', { class: 'compact-paint-row' }, [
      iconButton(t('insp.textShadow'), ICONS.shadow, () => set(shadow ? null : { color: '#00000099', blur: 8 }), { 'aria-pressed': String(Boolean(shadow)) }), color, blur,
    ]),
  ], { caption: true });
}
