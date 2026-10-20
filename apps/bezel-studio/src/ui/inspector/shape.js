// Existing shape controls, isolated from inspector navigation.
import { el } from '../dom.js';
import { numberField, rangeField, propertySection as section, segmented, selectField, switchRow } from '../fields.js';
import { cornerValue } from '../../corner-radii.js';
import { ART, IMAGE_REPEATS, REPEAT_KEYS, NATURAL_REPEATS } from './art.js';

export function panels(ctx) {
  const { t, update, colorField, video, cornersSection, fadeFields, button, optionTiles } = ctx;
  function shapePicture(e, assets) {
    const k = e.kind;
    const image = k.image ?? null;
    const set = (patch) => update(e.id, { kind: { image: patch } });
    const options = assets.filter((a) => a.kind === 'image').map((a) => [a.ref, a.ref.split('/').pop()]);
    if (image?.asset && !options.some(([r]) => r === image.asset)) options.push([image.asset, image.asset]);
    const pick = options.length
      ? selectField(t('common.image'), image?.asset ?? '', [['', t('inspector.noImage')], ...options], (asset) => {
        if (!asset) set(null);
        else if (image) set({ asset });
        else set({ asset, repeat: 'cover', scale: 1, offsetX: 0, offsetY: 0, opacity: 1 });
      })
      : el('p', { class: 'hint', text: t('inspector.addMediaFirst') });
    if (!image) return section('picture', t('common.image'), [pick]);
    const repeat = IMAGE_REPEATS.includes(image.repeat) ? image.repeat : 'cover';
    const art = (value) => ART[`repeat${value[0].toUpperCase()}${value.slice(1)}`];
    const tiles = IMAGE_REPEATS.map((value) => ({ text: t(REPEAT_KEYS[value]), art: art(value), pressed: repeat === value, onClick: () => repeat !== value && set({ repeat: value }) }));
    const natural = NATURAL_REPEATS.includes(repeat);
    return section('picture', t('common.image'), [
      pick,
      optionTiles(t('shape.repeat'), tiles),
      natural && numberField(t('shape.imageScale'), Math.round((image.scale ?? 1) * 100), (v) => set({ scale: Math.max(5, Math.min(1000, v)) / 100 }), { min: 5, max: 1000, unit: '%' }),
      natural && numberField(t('shape.imageOffsetX'), image.offsetX ?? 0, (v) => set({ offsetX: v }), { unit: 'px' }),
      natural && numberField(t('shape.imageOffsetY'), image.offsetY ?? 0, (v) => set({ offsetY: v }), { unit: 'px' }),
      rangeField(t('shape.imageOpacity'), Math.round((image.opacity ?? 1) * 100), (v) => set({ opacity: Math.max(0, Math.min(100, v)) / 100 }), { min: 0, max: 100, format: (v) => `${v}%` }),
      el('p', { class: 'hint', text: t('shape.imageHint') }),
    ]);
  }

  function shapeForm(e, assets = []) {
    const k = e.kind;
    return { look: [section('shape', t('common.shape'), [
      segmented(t('common.shape'), k.shape, [['rect', t('shape.rect')], ['ellipse', t('shape.ellipse')]], (v) => update(e.id, { kind: { shape: v } })),
      switchRow(t('shape.videoWindow'), Boolean(k.videoWindow), (videoWindow) => update(e.id, { kind: { videoWindow } })),
    ]), section('color', t('inspector.color'), [
      k.videoWindow ? el('div', {}, [el('p', { class: 'hint', text: t('shape.videoWindowHint') }), button(t('deviceVideo.choose'), () => video.openStorage())]) : colorField(t('inspector.fill'), typeof k.fill === 'string' ? k.fill : '#1e293bff', (v) => update(e.id, { kind: { fill: v } })),
    ]), k.videoWindow ? null : shapePicture(e, assets),
    section('transparency', t('fade.mode'), fadeFields(e)),
    section('outline', t('insp.outline'), [
      numberField(t('inspector.strokeWidth'), k.strokeWidth ?? 0, (v) => update(e.id, { kind: { strokeWidth: Math.max(0, v), stroke: v > 0 ? (k.stroke ?? '#ffffffff') : null } }), { min: 0, unit: 'px', iconName: 'strokeWidth' }),
      (k.strokeWidth ?? 0) > 0 && colorField(t('inspector.stroke'), k.stroke ?? '#ffffffff', (v) => update(e.id, { kind: { stroke: v } })),
    ]), k.shape === 'rect' ? cornersSection(t('insp.cornersTitle'), t('inspector.radius'), cornerValue(k.radius ?? 0, k.corners),
      ({ radius, corners }) => update(e.id, { kind: { radius, corners } }), { box: e.frame }) : null] };
  }
  return shapeForm(ctx.element, ctx.assets);
}
