import { gaugeFillFields } from './paint.js';
import { describeSections } from './sections.js';
// Existing ring controls, isolated from inspector navigation.
import { propertySection as section, fieldGrid, numberField, iconChoice, toggleField as switchRow } from '../fields.js';

export function panels(ctx) {
  const { t, update, sourceSection, trackFields, segmentsFields, transitionSection, angleFields, arcPreview } = ctx;

  function ringForm(e) {
    const k = e.kind;
    const data = [
      sourceSection(e, k.binding.key, (key) => ({ kind: { binding: { key } } })),
      section('arc', t('insp.arc'), [
        angleFields(e),
        fieldGrid([
          numberField(t('insp.thickness'), k.thickness, v => update(e.id, { kind: { thickness: Math.max(1, v) } }), { min: 1, max: Math.max(64, k.thickness), unit: 'px', iconName: 'strokeWidth', compact: true }),
          iconChoice(t('common.direction'), k.clockwise !== false ? 'clockwise' : 'counterclockwise', [['clockwise', t('inspector.clockwise'), 'rotate'], ['counterclockwise', t('insp.counterclockwise'), 'undo']], value => update(e.id, { kind: { clockwise: value === 'clockwise' } }), { iconOnly: true, wide: true }),
        ]),
        fieldGrid([
          iconChoice(t('insp.ends'), k.roundCaps ? 'round' : 'flat', [['round', t('inspector.roundCaps'), 'ends-round'], ['flat', t('insp.flatEnds'), 'ends-flat']], value => update(e.id, { kind: { roundCaps: value === 'round' } }), { iconOnly: true }),
          !['matrix', 'pixel'].includes(e.visualization?.style) && segmentsFields(e)[0],
        ].filter(Boolean), { lead: true }),
      ], { caption: true, tool: arcPreview(k) }),
      transitionSection(e),
    ];
    const look = [
      section('color', t('inspector.color'), gaugeFillFields(ctx, e)),
      section('track', t('inspector.track'), trackFields(e), { caption: true }),
      section('preview', t('common.preview'), [
        switchRow(t('ring.testFull'), Boolean(k.testFull), (v) => update(e.id, { kind: { testFull: v } })),
      ], { heading: false }),
    ];
    return { data, look };
  }
  return ringForm(ctx.element, ctx.assets);
}

export function sections(ctx) {
  return describeSections(ctx, ctx.completePanels(panels(ctx)));
}
