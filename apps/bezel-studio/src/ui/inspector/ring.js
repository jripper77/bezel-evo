import { gaugeFillFields } from './paint.js';
import { describeSections } from './sections.js';
// Existing ring controls, isolated from inspector navigation.
import { el } from '../dom.js';
import { propertySection as section, numberField, iconChoice, toggleField as switchRow } from '../fields.js';

export function panels(ctx) {
  const { t, update, sourceSection, trackFields, segmentsFields, transitionSection, angleFields, arcPreview } = ctx;

  function ringForm(e) {
    const k = e.kind;
    const data = [
      sourceSection(e, k.binding.key, (key) => ({ kind: { binding: { key } } })),
      section('arc', t('insp.arc'), [
        el('div', { class: 'arc-row' }, [arcPreview(k), angleFields(e)]),
        numberField(t('insp.thickness'), k.thickness, (v) => update(e.id, { kind: { thickness: Math.max(1, v) } }), { min: 1, max: Math.max(64, k.thickness), unit: 'px' }),
        iconChoice(t('common.direction'), k.clockwise !== false ? 'clockwise' : 'counterclockwise', [['clockwise', t('inspector.clockwise'), 'rotate'], ['counterclockwise', t('insp.counterclockwise'), 'undo']], value => update(e.id, { kind: { clockwise: value === 'clockwise' } }), { wide: true }),
        iconChoice(t('insp.ends'), k.roundCaps ? 'round' : 'flat', [['flat', t('insp.flatEnds')], ['round', t('inspector.roundCaps')]], value => update(e.id, { kind: { roundCaps: value === 'round' } }), { wide: true }),
      ]),
      transitionSection(e),
    ];
    const look = [
      section('color', t('inspector.color'), gaugeFillFields(ctx, e)),
      section('track', t('inspector.track'), trackFields(e), { caption: true }),
      section('segments', t('inspector.segments'), segmentsFields(e)),
      section('preview', t('common.preview'), [
        switchRow(t('ring.testFull'), Boolean(k.testFull), (v) => update(e.id, { kind: { testFull: v } })),
      ]),
    ];
    return { data, look };
  }
  return ringForm(ctx.element, ctx.assets);
}

export function sections(ctx) {
  return describeSections(ctx, ctx.completePanels(panels(ctx)));
}
