// Gauge paint controls share the same field builders and theme paint model.
import { atPath, pathPatch } from '../../inspector-edit.js';
import { el } from '../dom.js';
import { gradientField, iconChoice, numberField, swatchChip } from '../fields.js';

export function gaugeFillFields(ctx, element, { path = ['kind', 'fill'] } = {}) {
  const { t, update, colorField, store } = ctx, fill = atPath(element, path);
  const ring = element.kind.type === 'ring', arc = fill?.scale === 'arc', linear = Boolean(fill?.stops);
  const first = arc ? fill.start : linear ? fill.stops[0]?.[1] ?? '#00000000' : fill;
  const last = arc ? fill.end : linear ? fill.stops.at(-1)?.[1] ?? first : first;
  const replace = paint => store.dispatch('patchElements', { respectLocks: false, patches: [{ id: element.id, patch: pathPatch(path, null) }, { id: element.id, patch: pathPatch(path, paint) }] });
  const set = paint => update(element.id, pathPatch(path, paint));
  const modes = [['solid', t('ring.solid'), null, { leading: swatchChip(first), text: t('insp.solid') }], ['gradient', t('insp.gradient'), null, { leading: swatchChip(first, '', arc || linear ? last : null) }], ...(ring && linear ? [['linear', t('ring.importedLinear')]] : [])];
  const color = (index, value) => {
    if (arc) set({ ...fill, [index === 0 ? 'start' : 'end']: value });
    else if (linear) set({ ...fill, stops: fill.stops.map((stop, i) => i === (index === 0 ? 0 : fill.stops.length - 1) ? [stop[0], value] : stop) });
    else set(value);
  };
  return [el('div', { class: 'paint-fields' }, [
    iconChoice(t(ring ? 'ring.fillMode' : 'inspector.fill'), arc || linear && !ring ? 'gradient' : linear ? 'linear' : 'solid', modes, mode => {
      if (mode === 'solid') replace(first);
      else if (mode === 'gradient') replace(ring ? { scale: 'arc', start: first, end: last, transition: 50 } : { angle: 0, stops: [[0, first], [1, last]] });
    }, { wide: true, caption: t('inspector.fill') }),
    arc || linear ? gradientField(
      colorField(t('ring.startColor'), first, value => color(0, value), { swatchOnly: true }),
      colorField(t('ring.endColor'), last, value => color(1, value), { swatchOnly: true }),
      { stops: arc ? [[0, first], [1, last]] : fill.stops, midpoint: arc ? fill.transition ?? 50 : null,
        label: t('ring.transition'), onMidpoint: transition => set({ ...fill, transition: Math.max(0, Math.min(100, transition)) }) })
      : colorField(t('inspector.fill'), first, value => color(0, value), { hideLabel: true }),
    linear && numberField(t('shape.fadeAngle'), fill.angle, angle => set({ ...fill, angle }), { step: 'any', unit: '°' }),
  ])];
}
