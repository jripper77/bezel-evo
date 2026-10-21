import { describeSections } from './sections.js';
// Existing image controls, isolated from inspector navigation.
import { cornerValue } from '../../corner-radii.js';
import { IMAGE_ANCHORS, isAnimatedImage, playbackSpeeds } from '../../media-inspector.js';
import { readIcon } from '../../icon-catalog.js';
import { el } from '../dom.js';
import { imageField, numberField, iconChoice, propertySection as section } from '../fields.js';

export function panels(ctx) {
  const { t, update, colorField, editIcon, fadeFields } = ctx;
  function imageForm(e, assets) {
    const k = e.kind;
    const animated = isAnimatedImage(k.asset, assets);
    const set = patch => update(e.id, { kind: patch });
    const paint = readIcon(assets.find((a) => a.ref === k.asset)?.iconSvg);
    const change = (patch) => editIcon(e.id, k.asset, { ...paint, ...patch });
    const options = assets.filter((a) => a.kind === 'image').map((a) => [a.ref, a.ref.split('/').pop()]);
    if (k.asset && !options.some(([r]) => r === k.asset)) options.push([k.asset, k.asset]);
    const data = [section('content', t('inspector.content'), [
      options.length
        ? imageField(t('common.image'), k.asset, [['', t('inspector.noImage')], ...options], (v) => update(e.id, { kind: { asset: v } }))
        : el('p', { class: 'hint', text: t('inspector.addMediaFirst') }),
      el('button', { type: 'button', class: 'text-button', text: t('insp.image.media'), onclick: () => ctx.root.ownerDocument.querySelector('#tab-media')?.click() }),
    ])];
    const look = [section('appearance', t('inspector.appearance'), [
      iconChoice(t('inspector.fit'), k.fit, ['contain', 'cover', 'fill', ...(k.fit === 'none' ? ['none'] : [])].map(value => [value, t(`fit.${value}`)]), fit => set({ fit }), { wide: true }),
      k.fit === 'cover' && el('div', { class: 'image-anchor' }, [iconChoice(t('insp.image.anchor'), k.anchor ?? 'center', IMAGE_ANCHORS.map(value => [value, t(`insp.image.anchor.${value}`)]), anchor => set({ anchor: anchor === 'center' ? null : anchor }), { iconOnly: true })]),
    ]), ctx.cornersSection(t('insp.cornersTitle'), t('inspector.radius'), cornerValue(k.radius ?? 0, k.corners), patch => set(patch), { box: e.frame }), section('color', t('inspector.color'), [
      paint && colorField(t('inspector.color'), paint.color, (color) => change({ color }), { live: false }),
      paint?.style === 'outline' && numberField(t('inspector.strokeWidth'), paint.stroke, (stroke) => change({ stroke: Math.min(4, Math.max(0.5, stroke)) }), { min: 0.5, max: 4, step: 0.5, unit: 'px', iconName: 'strokeWidth' }),
    ]), section('shadow', t('icons.shadow'), [
      paint && numberField(t('icons.shadow'), paint.shadow, (shadow) => change({ shadow: Math.min(4, Math.max(0, shadow)) }), { min: 0, max: 4, step: 0.5, unit: 'px' }),
      paint && paint.shadow > 0 && colorField(t('icons.shadowColor'), paint.shadowColor, (shadowColor) => change({ shadowColor }), { live: false }),
    ]), section('transparency', t('fade.mode'), fadeFields(e))];
    return { data, look, behavior: animated ? [section('image-playback', t('insp.playback.mode'), [
      iconChoice(t('insp.playback.mode'), k.once ? 'once' : 'loop', [['loop', t('insp.playback.loop')], ['once', t('insp.playback.once')]], value => set({ once: value === 'once' }), { wide: true }),
      iconChoice(t('insp.playback.speed'), String(k.speed ?? 1), playbackSpeeds(k.speed).map(value => [String(value), `${value}×`]), value => set({ speed: Number(value) === 1 ? null : Number(value) }), { wide: true }),
      el('p', { class: 'hint', text: t('insp.image.gifHint') }),
    ])] : [] };
  }
  return imageForm(ctx.element, ctx.assets);
}

export function sections(ctx) {
  return describeSections(ctx, ctx.completePanels(panels(ctx)));
}
