import { gaugeFillFields } from './paint.js';
import { describeSections } from './sections.js';
// Existing bar controls, isolated from inspector navigation.
import { propertySection as section, iconChoice } from '../fields.js';
import { cornerValue } from '../../corner-radii.js';

export function panels(ctx) {
  const { t, update, sourceSection, trackFields, segmentsFields, cornersSection, transitionSection } = ctx;
  function barForm(e) {
    const k = e.kind;
    const data = [sourceSection(e, k.binding.key, (key) => ({ kind: { binding: { key } } })), transitionSection(e)];
    const look = [section('style', t('inspector.style'), [
      iconChoice(t('common.direction'), k.direction, ['leftToRight', 'rightToLeft', 'bottomToTop', 'topToBottom'].map((d, i) => [d, t(`dir.${d}`), ['arrowRight', 'arrowLeft', 'up', 'down'][i]]), direction => update(e.id, { kind: { direction } }), { wide: true, iconOnly: true }),
    ]), section('color', t('inspector.color'), gaugeFillFields(ctx, e)), section('track', t('inspector.track'), trackFields(e), { caption: true }),
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
