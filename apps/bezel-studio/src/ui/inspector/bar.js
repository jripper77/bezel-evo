import { describeSections } from './sections.js';
// Existing bar controls, isolated from inspector navigation.
import { propertySection as section, selectField } from '../fields.js';
import { cornerValue } from '../../corner-radii.js';

export function panels(ctx) {
  const { t, update, colorField, sourceSection, trackFields, segmentsFields, cornersSection, transitionSection } = ctx;
  function barForm(e) {
    const k = e.kind;
    const data = [sourceSection(e, k.binding.key, (key) => ({ kind: { binding: { key } } })), transitionSection(e)];
    const look = [section('style', t('inspector.style'), [
      selectField(t('common.direction'), k.direction, ['leftToRight', 'rightToLeft', 'bottomToTop', 'topToBottom'].map((d) => [d, t(`dir.${d}`)]), (v) => update(e.id, { kind: { direction: v } })),
    ]), section('color', t('inspector.color'), [
      colorField(t('inspector.fill'), typeof k.fill === 'string' ? k.fill : '#38bdf8ff', (v) => update(e.id, { kind: { fill: v } })),
    ]), section('track', t('inspector.track'), trackFields(e)),
    section('segments', t('inspector.segments'), segmentsFields(e)),
    cornersSection(t('insp.cornersTitle'), t('inspector.radius'), cornerValue(k.radius ?? 0), ({ radius }) => update(e.id, { kind: { radius } }),
      { individual: false, box: e.frame })];
    return { data, look };
  }
  return barForm(ctx.element, ctx.assets);
}

export function sections(ctx) {
  return describeSections(ctx, ctx.completePanels(panels(ctx)));
}
