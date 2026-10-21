// Gauge paint controls share the same field builders and theme paint model.
import { iconChoice, numberField } from '../fields.js';

export function gaugeFillFields(ctx, element) {
  const { t, update, colorField, store } = ctx, fill = element.kind.fill;
  const ring = element.kind.type === 'ring', arc = fill?.scale === 'arc', linear = Boolean(fill?.stops);
  const first = arc ? fill.start : linear ? fill.stops[0]?.[1] ?? '#00000000' : fill;
  const last = arc ? fill.end : linear ? fill.stops.at(-1)?.[1] ?? first : first;
  const replace = paint => store.dispatch('patchElements', { respectLocks: false, patches: [{ id: element.id, patch: { kind: { fill: null } } }, { id: element.id, patch: { kind: { fill: paint } } }] });
  const set = paint => update(element.id, { kind: { fill: paint } });
  const modes = [['solid', t('ring.solid')], ['gradient', t('insp.gradient')], ...(ring && linear ? [['linear', t('ring.importedLinear')]] : [])];
  const color = (index, value) => {
    if (arc) set({ ...fill, [index === 0 ? 'start' : 'end']: value });
    else if (linear) set({ ...fill, stops: fill.stops.map((stop, i) => i === (index === 0 ? 0 : fill.stops.length - 1) ? [stop[0], value] : stop) });
    else set(value);
  };
  return [
    iconChoice(t('ring.fillMode'), arc || linear && !ring ? 'gradient' : linear ? 'linear' : 'solid', modes, mode => {
      if (mode === 'solid') replace(first);
      else if (mode === 'gradient') replace(ring ? { scale: 'arc', start: first, end: last, transition: 50 } : { angle: 0, stops: [[0, first], [1, last]] });
    }, { wide: true }),
    colorField(t(arc || linear ? 'ring.startColor' : 'inspector.fill'), first, value => color(0, value)),
    (arc || linear) && colorField(t('ring.endColor'), last, value => color(1, value)),
    arc && numberField(t('ring.transition'), fill.transition ?? 50, transition => set({ ...fill, transition: Math.max(0, Math.min(100, transition)) }), { min: 0, max: 100, unit: '%' }),
    linear && numberField(t('shape.fadeAngle'), fill.angle, angle => set({ ...fill, angle }), { step: 'any', unit: '°' }),
  ];
}
