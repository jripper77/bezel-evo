import { describeSections } from './sections.js';
import { el } from '../dom.js';
import { fieldGrid, prefixedField, numberField, iconChoice, imageField, toggleField, iconButton, propertySection as section } from '../fields.js';
import { ICONS } from '../icons.js';
import { MAX_ZONES, TRACKS, SCALE_TICKS, SCALE_NUMBERS, NEEDLE_SHAPES, PIVOT_CAPS, lookOf } from '../../needle-look.js';
import { textStyle } from '../../editor/widgets.js';
import { textStyleFields, overflowField, valueFormatFields } from './text.js';

export function panels(ctx) {
  const { t, update, colorField, sourceSection, transitionSection, angleFields, element: e, assets } = ctx;
  const k = e.kind, look = lookOf(k.look);
  const images = assets.filter(asset => asset.kind === 'image' || /\.(png|jpe?g|webp|bmp|svg)$/i.test(asset.ref)).map(asset => [asset.ref, asset.name ?? asset.ref]);
  if (k.asset && !images.some(([ref]) => ref === k.asset)) images.push([k.asset, k.asset]);
  const set = patch => update(e.id, { kind: { look: patch } });
  const choice = (label, key, values) => iconChoice(t(`insp.needle.${label}`), look[key], values.map(value => [value, t(`insp.needle.${key}.${value}`)]), value => set({ [key]: value }), { wide: true });
  const number = (label, key, min, max, unit = 'px') => numberField(t(`insp.needle.${label}`), look[key], value => set({ [key]: Math.max(min, Math.min(max, value)) }), { min, max, unit });
  const center = look.center;
  const centerFields = center ? [
    ...valueFormatFields(ctx, e, center, center => set({ center })),
    ...textStyleFields(ctx, e, { style: center.style, setStyle: style => set({ center: { style } }), parts: true, scope: 'needle-center' }),
    overflowField(ctx, e, { style: center.style, setStyle: style => set({ center: { style } }) }),
  ] : [];
  const zoneRows = look.zones.map((zone, index) => {
    const change = patch => set({ zones: look.zones.map((z, i) => i === index ? { ...z, ...patch } : z) });
    return el('div', { class: 'needle-zone', role: 'group', 'aria-label': t('insp.needle.zone', { number: index + 1 }) }, [
      fieldGrid([
        numberField(t('insp.needle.zoneFrom', { number: index + 1 }), zone.from, from => change({ from }), { step: 'any' }),
        numberField(t('insp.needle.zoneTo', { number: index + 1 }), zone.to, to => change({ to }), { step: 'any' }),
      ]),
      colorField(t('insp.needle.zoneColor', { number: index + 1 }), zone.color, color => change({ color })),
      iconButton(t('insp.needle.removeZone', { number: index + 1 }), ICONS.trash, () => set({ zones: look.zones.filter((_, i) => i !== index) })),
    ]);
  });
  return {
    data: [sourceSection(e, k.binding.key, key => ({ kind: { binding: { key } } })), transitionSection(e),
      section('peak', t('insp.needle.peak'), [
        toggleField(t('insp.needle.peak'), look.peakSeconds != null, on => set({ peakSeconds: on ? 10 : null })),
        look.peakSeconds != null && number('peakDuration', 'peakSeconds', 1, 3600, 's'),
      ])],
    look: [
      section('arc', t('insp.arc'), [angleFields(e), number('arcWidth', 'arcWidth', 0, 200), choice('arcCap', 'arcCap', ['round', 'butt'])]),
      section('scale-marks', t('insp.needle.ticks'), [choice('ticks', 'ticks', SCALE_TICKS), choice('numbers', 'numbers', SCALE_NUMBERS), number('numberGap', 'numberGap', 0, 200)]),
      section('needle-shape', t('insp.needle.hand'), [
        choice('hand', 'shape', NEEDLE_SHAPES),
        fieldGrid([number('length', 'length', 10, 150, '%'), number('tail', 'tail', 0, 100, '%')]),
        numberField(t('inspector.lineWidth'), k.width, width => update(e.id, { kind: { width: Math.max(1, width) } }), { min: 1, unit: 'px' }),
        fieldGrid([
          prefixedField(t('insp.xPrefix'), t('inspector.pivotX'), Math.round((k.pivot?.[0] ?? 0.5) * 100), value => update(e.id, { kind: { pivot: [Math.min(100, Math.max(0, value)) / 100, k.pivot?.[1] ?? 0.5] } }), { min: 0, max: 100, unit: '%' }),
          prefixedField(t('insp.yPrefix'), t('inspector.pivotY'), Math.round((k.pivot?.[1] ?? 0.5) * 100), value => update(e.id, { kind: { pivot: [k.pivot?.[0] ?? 0.5, Math.min(100, Math.max(0, value)) / 100] } }), { min: 0, max: 100, unit: '%' }),
        ]),
        choice('cap', 'cap', PIVOT_CAPS), look.cap !== 'none' && number('capDiameter', 'capDiameter', 0, 200),
      ]),
      section('font', t('insp.needle.center'), [
        toggleField(t('insp.needle.center'), Boolean(center), on => set({ center: on ? { style: textStyle(24, 'center'), format: { showUnit: true }, prefix: '', suffix: '' } : null })),
        ...centerFields,
      ]),
      section('picture', t('insp.needle.image'), [imageField(t('insp.needle.image'), k.asset ?? '', [['', t('fade.none')], ...images], asset => update(e.id, { kind: { asset: asset || null } }))]),
      section('color', t('inspector.color'), [
        colorField(t('insp.needle.hand'), k.color, color => update(e.id, { kind: { color } })),
        choice('track', 'track', TRACKS), ...zoneRows,
        el('button', { type: 'button', class: 'text-button', text: t('insp.needle.addZone'), disabled: look.zones.length >= MAX_ZONES, onclick: () => set({ zones: [...look.zones, { from: k.binding.min, to: k.binding.max, color: k.color }] }) }),
        toggleField(t('insp.needle.colorFromZone'), look.colorFromZone, colorFromZone => set({ colorFromZone })),
        colorField(t('insp.needle.marks'), look.marks, marks => set({ marks })),
        look.cap !== 'none' && colorField(t('insp.needle.cap'), look.capColor, capColor => set({ capColor })),
      ]),
    ],
  };
}

export function sections(ctx) { return describeSections(ctx, ctx.completePanels(panels(ctx))); }
