// Shared text and styled-part editors, also usable by the clock and the
// needle's centre value. All model edits still go through the caller.
import { el } from '../dom.js';
import { fontPicker, numberField, iconChoice, partsRow, toggleField, iconButton, fieldGrid, textField, selectField, propertySection as section } from '../fields.js';
import { ICONS } from '../icons.js';
import { VALUE_PARTS, OVERFLOWS } from '../../text-layout.js';
import { partStylePatch } from '../../inspector-edit.js';

const selected = new WeakMap();

export function valueFormatFields(ctx, element, content, setContent) {
  const { t } = ctx, format = content.format ?? {};
  const decimals = format.decimals;
  const values = [...new Set(['auto', '0', '1', '2', '3', ...(decimals != null ? [String(decimals)] : [])])];
  return [
    fieldGrid([
      textField(t('inspector.prefix'), content.prefix ?? '', prefix => setContent({ prefix })),
      textField(t('inspector.suffix'), content.suffix ?? '', suffix => setContent({ suffix })),
    ]),
    selectField(t('inspector.decimals'), decimals == null ? 'auto' : String(decimals), values.map(value => [value, value === 'auto' ? t('inspector.decimalsAuto') : value]), value => setContent({ format: { decimals: value === 'auto' ? null : Number(value) } })),
    toggleField(t('inspector.showUnit'), format.showUnit !== false, showUnit => setContent({ format: { showUnit } })),
    ctx.sensors.catalog().find(sensor => sensor.key === (content.key ?? element.kind.binding?.key))?.quantity === 'celsius' && toggleField(t('inspector.fahrenheit'), Boolean(format.fahrenheit), fahrenheit => setContent({ format: { fahrenheit } })),
  ];
}

export function textStyleFields(ctx, element, { style = element.kind.style, setStyle = patch => ctx.update(element.id, { kind: { style: patch } }), parts = element.kind.content?.type === 'sensor', scope = 'value', tabular = true } = {}) {
  const { t } = ctx;
  const fields = ctx.fontFields(element, { style, setStyle, compact: true });
  if (parts) {
    if (!selected.has(ctx.store)) selected.set(ctx.store, new Map());
    const memory = selected.get(ctx.store), key = `${element.id}:${scope}`;
    const current = memory.get(key) ?? 'number';
    const editPart = part => {
      const value = style.parts?.[part] ?? {};
      const patch = change => setStyle(partStylePatch(style, part, change));
      const { texts, source } = ctx.pickers.font(ctx.sampleText(element));
      const font = fontPicker({ ...texts, family: t('insp.partFont'), face: t('insp.partFace') }, { family: value.font ?? style.font.family, weight: value.weight ?? style.font.weight, italic: style.font.italic, asset: value.font ? value.fontAsset ?? null : style.font.asset }, font => patch({ font: font.family, fontAsset: font.asset ?? null, weight: font.weight }), source, { faceSample: t('insp.faceSample'), fixedItalic: Boolean(style.font.italic) });
      const face = font.lastElementChild;
      return el('div', { class: 'part-editor', role: 'group', 'aria-label': t('insp.partEditor', { part: t(`insp.valuePart.${part}`) }) }, [
        el('div', { class: 'part-editor-head' }, [el('strong', { text: t(`insp.valuePart.${part}`) }), iconButton(t('insp.inheritPart'), ICONS.undo, () => patch(null))]),
        font,
        fieldGrid([
          ctx.colorField(t('insp.partColor'), value.color ?? style.paint, color => patch({ color })),
          numberField(t('insp.partSize'), value.sizePercent, sizePercent => patch({ sizePercent: sizePercent === null ? null : Math.max(10, Math.min(400, sizePercent)) }), { min: 10, max: 400, unit: '%', nullable: true }),
        ]),
        fieldGrid([face, iconChoice(t('insp.partBaseline'), value.baseline ?? 'base', [['base', t('insp.baseline')], ['super', t('insp.superscript')]], baseline => patch({ baseline: baseline === 'base' ? null : baseline }), { wide: true })]),
        el('p', { class: 'hint', text: t('insp.partInheritanceHint') }),
      ]);
    };
    fields.unshift(partsRow(t('insp.valueParts'), VALUE_PARTS.map(part => [part, t(`insp.valuePart.${part}`)]), current,
      part => { memory.set(key, part); ctx.redraw(); }, editPart));
  }
  if (tabular) fields.push(toggleField(t('insp.tabularDigits'), style.tabularDigits !== false, tabularDigits => setStyle({ tabularDigits }), { iconName: 'fixed-digits' }));
  return fields;
}

export function overflowField(ctx, element, { style = element.kind.style, setStyle = patch => ctx.update(element.id, { kind: { style: patch } }) } = {}) {
  const { t } = ctx;
  // Imported legacy text without a mode keeps its original clipping until edited.
  const modes = style.overflow == null ? ['legacy', ...OVERFLOWS] : OVERFLOWS;
  return iconChoice(t('insp.textOverflow'), style.overflow ?? 'legacy', modes.map(mode => [mode, t(`insp.overflow.${mode}`)]), overflow => setStyle({ overflow: overflow === 'legacy' ? null : overflow }), { wide: true });
}

export function shadowSection(ctx, element) {
  const { t, update } = ctx, shadow = element.kind.style.shadow;
  const set = shadow => update(element.id, { kind: { style: { shadow } } });
  return section('shadow', t('insp.textShadow'), [
    toggleField(t('insp.textShadow'), Boolean(shadow), enabled => set(enabled ? { color: '#00000099', blur: 8 } : null), { iconName: 'shadow' }),
    shadow && ctx.colorField(t('insp.shadowColor'), shadow.color, color => set({ ...shadow, color })),
    shadow && numberField(t('insp.shadowBlur'), shadow.blur, blur => set({ ...shadow, blur: Math.max(0, Math.min(64, blur)) }), { min: 0, max: 64, unit: 'px' }),
  ]);
}
