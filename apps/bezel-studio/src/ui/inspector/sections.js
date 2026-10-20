import { SECTION_ORDER, summaryOf } from '../../inspector-sections.js';
import { sectionFor, summaryParts } from '../../inspector-model.js';

const ICONS = { layout: 'section-layout', data: 'data', shape: 'shape', text: 'text', color: 'drop', behavior: 'behav', faces: 'layers', motion: 'play', triggers: 'sliders' };

/** All kinds share the order and descriptor contract; no controls are discarded. */
export function describeSections(ctx, panels) {
  const groups = new Map();
  for (const [panel, nodes] of Object.entries(panels)) {
    for (const node of nodes.filter(Boolean)) {
      const id = ['faces', 'motion', 'triggers'].includes(panel) ? panel : sectionFor(node.dataset?.section, panel === 'look' ? 'behavior' : panel);
      if (!groups.has(id)) groups.set(id, []);
      groups.get(id).push(node);
    }
  }
  return SECTION_ORDER.filter(id => groups.has(id)).map(id => ({
    id, title: ctx.t(({ data: 'inspector.tab.data', shape: 'common.shape', text: 'common.text', color: 'inspector.color', faces: 'card.tab.faces', motion: 'card.tab.motion' })[id] ?? `insp.section.${id}`), icon: ICONS[id],
    summary: element => {
      const locale = ctx.video?.context().locale ?? 'en';
      const parts = summaryOf(summaryParts(id, element, locale).map(part => part?.key ? ctx.t(part.key) : part), { locale });
      if (parts) return parts;
      // Native selected options and pressed choices already have localized labels.
      return groups.get(id).flatMap(node => [...node.querySelectorAll('select, button[aria-pressed="true"], input[type="text"], [data-color]')])
        .map(node => node.tagName === 'SELECT' ? node.selectedOptions[0]?.textContent : node.tagName === 'INPUT' ? node.value : node.dataset.color ?? node.getAttribute('aria-label') ?? node.textContent)
        .filter(Boolean).slice(0, 3).join(' · ') || (ctx.ids ? ctx.t('inspector.multi', { count: ctx.ids.length }) : ctx.t('fade.none'));
    },
    render: body => body.append(...groups.get(id)),
  }));
}
