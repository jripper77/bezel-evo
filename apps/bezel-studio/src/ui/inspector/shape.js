import { gaugeFillFields } from './paint.js';
import { describeSections } from './sections.js';
// Existing shape controls, isolated from inspector navigation.
import { el } from '../dom.js';
import { ICONS } from '../icons.js';
import { fieldAction, iconButton, fieldGrid, numberField, prefixedField, iconChoice, propertySection as section } from '../fields.js';
import { cornerValue } from '../../corner-radii.js';
import { ART, IMAGE_REPEATS, REPEAT_KEYS, NATURAL_REPEATS } from './art.js';

export function panels(ctx) {
  const { t, update, colorField, video, cornersSection, fadeFields, button, assetField } = ctx;
  function shapePicture(e, assets) {
    const k = e.kind;
    const image = k.image ?? null;
    const set = (patch) => update(e.id, { kind: { image: patch } });
    const options = assets.filter((a) => a.kind === 'image').map((a) => [a.ref, a.ref.split('/').pop()]);
    if (image?.asset && !options.some(([r]) => r === image.asset)) options.push([image.asset, image.asset]);
    const pick = assetField(t('common.image'), image?.asset ?? '', [['', t('inspector.noImage')], ...options], (asset) => {
      if (!asset) set(null);
      else if (image) set({ asset });
      else set({ asset, repeat: 'cover', scale: 1, offsetX: 0, offsetY: 0, opacity: 1 });
    }, assets);
    if (!image) return section('picture', t('common.image'), [pick], { heading: false });
    const repeat = IMAGE_REPEATS.includes(image.repeat) ? image.repeat : 'cover';
    const art = (value) => ART[`repeat${value[0].toUpperCase()}${value.slice(1)}`];
    const natural = NATURAL_REPEATS.includes(repeat);
    return section('picture', t('common.image'), [
      pick,
      iconChoice(t('shape.repeat'), repeat, IMAGE_REPEATS.map(value => [value, t(REPEAT_KEYS[value]), art(value)]), repeat => set({ repeat }), { wide: true, iconOnly: true }),
      natural && fieldGrid([
        prefixedField('', t('shape.imageScale'), Math.round((image.scale ?? 1) * 100), value => set({ scale: Math.max(5, Math.min(1000, value)) / 100 }), { min: 5, max: 1000, unit: '%' }),
        prefixedField(t('insp.xPrefix'), t('shape.imageOffsetX'), image.offsetX ?? 0, offsetX => set({ offsetX })),
        prefixedField(t('insp.yPrefix'), t('shape.imageOffsetY'), image.offsetY ?? 0, offsetY => set({ offsetY })),
      ], { columns: 3 }),
      numberField(t('shape.imageOpacity'), Math.round((image.opacity ?? 1) * 100), value => set({ opacity: Math.max(0, Math.min(100, value)) / 100 }), { min: 0, max: 100, unit: '%', compact: true }),
      el('p', { class: 'hint', text: t('shape.imageHint') }),
    ], { heading: false });
  }

  function shapeForm(e, assets = []) {
    const k = e.kind;
    return { look: [section('shape', t('common.shape'), [
      fieldAction(iconChoice(t('insp.shape.silhouette'), k.shape, [['rect', t('shape.rect'), 'shape-rect'], ['ellipse', t('shape.ellipse'), 'shape-ellipse']], shape => update(e.id, { kind: { shape } }), { wide: true }), iconButton(t('shape.videoWindow'), ICONS.video, () => update(e.id, { kind: { videoWindow: !k.videoWindow } }), { 'aria-pressed': String(Boolean(k.videoWindow)) })),
    ], { heading: false }), section('color', t('inspector.color'), [
      k.videoWindow ? el('div', {}, [el('p', { class: 'hint', text: t('shape.videoWindowHint') }), button(t('deviceVideo.choose'), () => video.openStorage())]) : el('div', {}, gaugeFillFields(ctx, e)),
    ]), k.videoWindow ? null : shapePicture(e, assets),
    section('transparency', t('fade.mode'), fadeFields(e), { heading: false }),
    section('outline', t('insp.outline'), [fieldGrid([
      numberField(t('inspector.strokeWidth'), k.strokeWidth ?? 0, v => update(e.id, { kind: { strokeWidth: Math.max(0, v), stroke: v > 0 ? k.stroke ?? '#ffffffff' : null } }), { min: 0, unit: 'px', iconName: 'strokeWidth', compact: true }),
      (k.strokeWidth ?? 0) > 0 && colorField(t('inspector.stroke'), k.stroke ?? '#ffffffff', stroke => update(e.id, { kind: { stroke } }), { hideLabel: true }),
    ].filter(Boolean))], { heading: false }), k.shape === 'rect' ? cornersSection(t('insp.cornersTitle'), t('inspector.radius'), cornerValue(k.radius ?? 0, k.corners),
      ({ radius, corners }) => update(e.id, { kind: { radius, corners } }), { box: e.frame }) : null] };
  }
  return shapeForm(ctx.element, ctx.assets);
}

export function sections(ctx) {
  return describeSections(ctx, ctx.completePanels(panels(ctx)));
}
