import { describeSections } from './sections.js';
// Existing image controls, isolated from inspector navigation.
import { readIcon } from '../../icon-catalog.js';
import { el } from '../dom.js';
import { imageField, numberField, propertySection as section, selectField } from '../fields.js';

export function panels(ctx) {
  const { t, update, colorField, editIcon, fadeFields } = ctx;
  function imageForm(e, assets) {
    const k = e.kind;
    const paint = readIcon(assets.find((a) => a.ref === k.asset)?.iconSvg);
    const change = (patch) => editIcon(e.id, k.asset, { ...paint, ...patch });
    const options = assets.filter((a) => a.kind === 'image').map((a) => [a.ref, a.ref.split('/').pop()]);
    if (k.asset && !options.some(([r]) => r === k.asset)) options.push([k.asset, k.asset]);
    const data = [section('content', t('inspector.content'), [
      options.length
        ? imageField(t('common.image'), k.asset, [['', t('inspector.noImage')], ...options], (v) => update(e.id, { kind: { asset: v } }))
        : el('p', { class: 'hint', text: t('inspector.addMediaFirst') }),
    ])];
    const look = [section('appearance', t('inspector.appearance'), [
      selectField(t('inspector.fit'), k.fit, ['fill', 'contain', 'cover', 'none'].map((f) => [f, t(`fit.${f}`)]), (v) => update(e.id, { kind: { fit: v } })),
    ]), section('color', t('inspector.color'), [
      paint && colorField(t('inspector.color'), paint.color, (color) => change({ color }), { live: false }),
      paint?.style === 'outline' && numberField(t('inspector.strokeWidth'), paint.stroke, (stroke) => change({ stroke: Math.min(4, Math.max(0.5, stroke)) }), { min: 0.5, max: 4, step: 0.5, unit: 'px', iconName: 'strokeWidth' }),
    ]), section('shadow', t('icons.shadow'), [
      paint && numberField(t('icons.shadow'), paint.shadow, (shadow) => change({ shadow: Math.min(4, Math.max(0, shadow)) }), { min: 0, max: 4, step: 0.5, unit: 'px' }),
      paint && paint.shadow > 0 && colorField(t('icons.shadowColor'), paint.shadowColor, (shadowColor) => change({ shadowColor }), { live: false }),
    ]), section('transparency', t('fade.mode'), fadeFields(e))];
    return { data, look };
  }
  return imageForm(ctx.element, ctx.assets);
}

export function sections(ctx) {
  return describeSections(ctx, ctx.completePanels(panels(ctx)));
}
