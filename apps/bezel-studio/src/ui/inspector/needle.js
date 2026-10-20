import { describeSections } from './sections.js';
// Existing needle controls, isolated from inspector navigation.
import { fieldGrid, prefixedField, rangeField, propertySection as section } from '../fields.js';

export function panels(ctx) {
  const { t, update, colorField, sourceSection, transitionSection, angleFields } = ctx;
  function needleForm(e) {
    const k = e.kind;
    const data = [sourceSection(e, k.binding.key, (key) => ({ kind: { binding: { key } } })), transitionSection(e)];
    const look = [section('arc', t('insp.arc'), [
      angleFields(e),
      fieldGrid([
        prefixedField(t('insp.xPrefix'), t('inspector.pivotX'), Math.round((k.pivot?.[0] ?? 0.5) * 100), (v) => update(e.id, { kind: { pivot: [v / 100, k.pivot?.[1] ?? 0.5] } }), { min: 0, max: 100, unit: '%' }),
        prefixedField(t('insp.yPrefix'), t('inspector.pivotY'), Math.round((k.pivot?.[1] ?? 0.5) * 100), (v) => update(e.id, { kind: { pivot: [k.pivot?.[0] ?? 0.5, v / 100] } }), { min: 0, max: 100, unit: '%' }),
      ]),
      rangeField(t('inspector.lineWidth'), k.width, (v) => update(e.id, { kind: { width: Math.max(1, v) } }), { min: 1, max: Math.max(32, k.width), format: (v) => t('insp.px', { value: v }) }),
    ]), section('color', t('inspector.color'), [
      colorField(t('inspector.color'), k.color, (v) => update(e.id, { kind: { color: v } })),
    ])];
    return { data, look };
  }
  return needleForm(ctx.element, ctx.assets);
}

export function sections(ctx) {
  return describeSections(ctx, ctx.completePanels(panels(ctx)));
}
