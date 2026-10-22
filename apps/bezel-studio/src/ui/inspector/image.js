import { ART } from './art.js';
import { describeSections } from './sections.js';
// Existing image controls, isolated from inspector navigation.
import { cornerValue } from '../../corner-radii.js';
import { IMAGE_ANCHORS, isAnimatedImage, playbackSpeeds } from '../../media-inspector.js';
import { readIcon } from '../../icon-catalog.js';
import { el } from '../dom.js';
import { numberField, iconChoice, propertySection as section } from '../fields.js';

export function panels(ctx) {
  const { t, update, colorField, editIcon, fadeFields, assetField, optionTiles } = ctx;
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
        ? assetField(t('common.image'), k.asset, [['', t('inspector.noImage')], ...options], v => set({ asset: v }), assets)
        : el('p', { class: 'hint', text: t('inspector.addMediaFirst') }),
    ])];
    const look = [section('appearance', t('inspector.fit'), [
      optionTiles(t('inspector.fit'), ['contain', 'cover', 'fill', ...(k.fit === 'none' ? ['none'] : [])].map(value => ({ text: t(`fit.${value}`), art: ({ contain: ART.repeatContain, cover: ART.repeatCover, fill: ART.repeatStretch, none: ART.repeatCenter })[value], pressed: k.fit === value, onClick: () => set({ fit: value }) }))),
      k.fit === 'cover' && el('div', { class: 'image-anchor' }, [iconChoice(t('insp.image.anchor'), k.anchor ?? 'center', IMAGE_ANCHORS.map(value => [value, t(`insp.image.anchor.${value}`)]), anchor => set({ anchor: anchor === 'center' ? null : anchor }), { iconOnly: true })]),
    ], { caption: true }), ctx.cornersSection(t('insp.cornersTitle'), t('inspector.radius'), cornerValue(k.radius ?? 0, k.corners), patch => set(patch), { box: e.frame }), section('color', t('inspector.color'), [
      paint && colorField(t('inspector.color'), paint.color, (color) => change({ color }), { live: false }),
      paint?.style === 'outline' && numberField(t('inspector.strokeWidth'), paint.stroke, (stroke) => change({ stroke: Math.min(4, Math.max(0.5, stroke)) }), { min: 0.5, max: 4, step: 0.5, unit: 'px', iconName: 'strokeWidth' }),
    ]), section('shadow', t('icons.shadow'), [
      paint && numberField(t('icons.shadow'), paint.shadow, (shadow) => change({ shadow: Math.min(4, Math.max(0, shadow)) }), { min: 0, max: 4, step: 0.5, unit: 'px' }),
      paint && paint.shadow > 0 && colorField(t('icons.shadowColor'), paint.shadowColor, (shadowColor) => change({ shadowColor }), { live: false }),
    ]), section('transparency', t('fade.mode'), fadeFields(e), { heading: false })];
    return { data, look, behavior: animated ? [section('image-playback', t('insp.playback.mode'), [
      iconChoice(t('insp.playback.mode'), k.once ? 'once' : 'loop', [['loop', t('insp.playback.loop'), 'playback-loop'], ['once', t('insp.playback.once'), 'playback-once']], value => set({ once: value === 'once' }), { wide: true }),
      iconChoice(t('insp.playback.speed'), String(k.speed ?? 1), playbackSpeeds(k.speed).map(value => [String(value), `${value}×`]), value => set({ speed: Number(value) === 1 ? null : Number(value) }), { wide: true }),
      el('p', { class: 'hint', text: t('insp.image.gifHint') }),
    ], { heading: false })] : [] };
  }
  return imageForm(ctx.element, ctx.assets);
}

export function sections(ctx) {
  return describeSections(ctx, ctx.completePanels(panels(ctx)));
}
