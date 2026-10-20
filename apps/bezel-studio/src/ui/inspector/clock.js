import { describeSections } from './sections.js';
// Existing clock controls, isolated from inspector navigation.
import { applyTextCase, TEXT_CASES } from '../../text-case.js';
import { CLOCK_PATTERNS, formatClock } from '../../clock-format.js';
import { el } from '../dom.js';
import { selectField, textField } from '../fields.js';
import { panels as textPanels } from './value.js';

export function panels(ctx) {
  const { t, update, root } = ctx;
  function fields(e) {
    const c = e.kind.content, s = e.kind.style;
    const nodes = [], weatherAppearance = [];
    if (c.type === 'clock') {
      nodes.push(selectField(t('clock.language'), c.language ?? '', [['', t('clock.system')], ['it', t('language.it')], ['en', t('language.en')], ['pt-BR', t('language.pt-BR')]], (language) => update(e.id, { kind: { content: { language: language || null } } })));
      nodes.push(selectField(t('clock.format'), CLOCK_PATTERNS.includes(c.pattern) ? c.pattern : 'custom', [...CLOCK_PATTERNS.map((pattern) => [pattern, formatClock(pattern, new Date(), c.language, c.casing)]), ['custom', t('clock.custom')]], (pattern) => {
        if (pattern === 'custom') root.querySelector('[list="clock-patterns"]')?.focus();
        else update(e.id, { kind: { content: { pattern } } });
      }));
      nodes.push(selectField(t('clock.casing'), e.kind.style.casing ?? c.casing ?? 'normal', TEXT_CASES.map((value) => [value, t(`textCase.${value}`)]), (casing) => update(e.id, { kind: { content: { casing: 'normal' }, style: { casing } } })));
      nodes.push(textField(t('inspector.pattern'), c.pattern, (v) => update(e.id, { kind: { content: { pattern: v } } }), { list: 'clock-patterns' }));
      nodes.push(el('datalist', { id: 'clock-patterns' }, CLOCK_PATTERNS.map((p) => el('option', { value: p }))));
      nodes.push(el('p', { id: 'clock-preview', class: 'hint', text: t('clock.preview', { value: applyTextCase(formatClock(c.pattern, new Date(), c.language, c.casing), e.kind.style.casing) }) }));
      nodes.push(el('p', { class: 'hint', text: t('inspector.patternHelp') }));
    }
    return { nodes, weatherAppearance };
  }
  return textPanels({ ...ctx, extra: fields(ctx.element) });
}

export function sections(ctx) {
  return describeSections(ctx, ctx.completePanels(panels(ctx)));
}
