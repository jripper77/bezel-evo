// Existing ring controls, isolated from inspector navigation.
import { el } from '../dom.js';
import { rangeField, propertySection as section, selectField, switchRow } from '../fields.js';

export function panels(ctx) {
  const { t, update, colorField, sourceSection, trackFields, segmentsFields, transitionSection, angleFields, arcPreview } = ctx;
  function ringFillFields(e) {
    const p = e.kind.fill;
    const arc = p?.scale === 'arc';
    const linear = typeof p === 'object' && Boolean(p?.stops);
    const start = arc ? p.start : typeof p === 'string' ? p : p?.stops?.[0]?.[1] ?? '#38bdf8ff';
    const end = arc ? p.end : p?.stops?.at(-1)?.[1] ?? '#f97316ff';
    const modes = [['solid', t('ring.solid')], ['arc', t('ring.arc')]];
    if (linear) modes.push(['linear', t('ring.importedLinear')]);
    const setArc = (patch) => update(e.id, { kind: { fill: { scale: 'arc', start, end, transition: p?.transition ?? 50, ...patch } } });
    return [
      selectField(t('ring.fillMode'), arc ? 'arc' : linear ? 'linear' : 'solid', modes, (v) => {
        if (v === 'arc') setArc({});
        else if (v === 'solid') update(e.id, { kind: { fill: start } });
      }),
      ...(arc ? [
        colorField(t('ring.startColor'), start, (v) => setArc({ start: v })),
        colorField(t('ring.endColor'), end, (v) => setArc({ end: v })),
        rangeField(t('ring.transition'), p.transition ?? 50, (v) => setArc({ transition: v }), { min: 0, max: 100, step: 1, format: (v) => `${v}%` }),
        el('p', { class: 'hint', text: t('ring.transitionHint') }),
      ] : [!linear && colorField(t('inspector.fill'), start, (v) => update(e.id, { kind: { fill: v } }))]),
    ];
  }

  function ringForm(e) {
    const k = e.kind;
    const data = [
      sourceSection(e, k.binding.key, (key) => ({ kind: { binding: { key } } })),
      section('arc', t('insp.arc'), [
        el('div', { class: 'arc-row' }, [arcPreview(k), angleFields(e)]),
        rangeField(t('insp.thickness'), k.thickness, (v) => update(e.id, { kind: { thickness: Math.max(1, v) } }), { min: 1, max: Math.max(64, k.thickness), format: (v) => t('insp.px', { value: v }) }),
        switchRow(t('inspector.clockwise'), k.clockwise !== false, (v) => update(e.id, { kind: { clockwise: v } })),
        switchRow(t('inspector.roundCaps'), Boolean(k.roundCaps), (v) => update(e.id, { kind: { roundCaps: v } })),
      ]),
      transitionSection(e),
    ];
    const look = [
      section('color', t('inspector.color'), ringFillFields(e)),
      section('track', t('inspector.track'), trackFields(e)),
      section('segments', t('inspector.segments'), segmentsFields(e)),
      section('preview', t('common.preview'), [
        switchRow(t('ring.testFull'), Boolean(k.testFull), (v) => update(e.id, { kind: { testFull: v } })),
      ]),
    ];
    return { data, look };
  }
  return ringForm(ctx.element, ctx.assets);
}
