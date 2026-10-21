import { describeSections } from './sections.js';
import { applyTextCase } from '../../text-case.js';
import { CLOCK_PATTERNS, analogOf, formatClock } from '../../clock-format.js';
import { CLOCK_PARTS } from '../../text-layout.js';
import { textStyleFields } from './text.js';
import { el } from '../dom.js';
import { iconChoice, numberField, propertySection as section, selectField, textField, toggleField, chipButton } from '../fields.js';
import { ICONS } from '../icons.js';

export function panels(ctx) {
  const { t, update, element: e, colorField, optionTiles } = ctx;
  const c = e.kind.content, style = e.kind.style;
  const analog = c.analog != null, dial = analogOf(c.analog);
  const set = content => update(e.id, { kind: { content } });
  const setDial = patch => set({ analog: { ...c.analog, ...patch } });
  const choice = (key, values) => iconChoice(t(`insp.clock.${key}`), dial[key], values.map(value => [value, t(`insp.clock.${key}.${value}`)]), value => setDial({ [key]: value }), { wide: true });
  const language = selectField(t('clock.language'), c.language ?? '', [['', t('clock.system')], ['it', t('language.it')], ['en', t('language.en')], ['pt-BR', t('language.pt-BR')]], language => set({ language: language || null }));
  const look = [section('style', t('inspector.style'), [optionTiles(t('inspector.style'), [
    { text: t('insp.clock.digital'), art: ICONS.text, pressed: !analog, onClick: () => analog && set({ analog: null }) },
    { text: t('insp.clock.analog'), art: ICONS.clock, pressed: analog, onClick: () => !analog && set({ analog: {} }) },
  ])])];
  if (analog) {
    look.push(section('clock-dial', t('insp.clock.dial'), [
      choice('ticks', ['none', 'hours', 'minutes']), choice('numbers', ['none', 'quarters', 'all', 'roman']),
      optionTiles(t('insp.clock.hands'), ['hour', 'minute', 'second'].map(hand => ({ text: t(`insp.clockPart.${hand}s`), art: ICONS.clock, pressed: dial[`${hand}Hand`], onClick: () => setDial({ [`${hand}Hand`]: !dial[`${hand}Hand`] }) }))),
      choice('handShape', ['thin', 'rounded', 'arrow']),
      numberField(t('insp.outline'), dial.border, border => setDial({ border: Math.max(0, Math.min(64, border)) }), { min: 0, max: 64, unit: 'px' }),
    ]));
    look.push(section('color', t('inspector.color'), [
      colorField(t('insp.clock.face'), dial.face, face => setDial({ face })),
      colorField(t('insp.clock.marks'), dial.marks, marks => setDial({ marks })),
      ...['hour', 'minute', 'second'].map(hand => colorField(t(`insp.clock.${hand}`), dial[`${hand}Color`], color => setDial({ [`${hand}Color`]: color }))),
    ]));
    const behavior = dial.secondHand ? [section('clock-seconds', t('insp.clock.secondsMotion'), [iconChoice(t('insp.clock.secondsMotion'), dial.sweep ? 'smooth' : 'step', [['step', t('insp.clock.step')], ['smooth', t('insp.transitionSmooth')]], value => setDial({ sweep: value === 'smooth' }), { wide: true })])] : [];
    return { data: [section('content', t('inspector.content'), [choice('date', ['none', 'day', 'dayMonth']), language])], look, behavior };
  }
  const [parts, ...font] = textStyleFields(ctx, e, { parts: true, scope: 'clock', partIds: CLOCK_PARTS, partsLabel: t('insp.clock.parts'), partLabel: part => t(`insp.clockPart.${part}`), setStyle: patch => update(e.id, { kind: { style: patch, ...(patch.casing != null ? { content: { casing: 'normal' } } : {}) } }) });
  look.push(section('clock-parts', t('insp.clock.parts'), [parts]));
  look.push(section('font', t('inspector.font'), font));
  return {
    data: [section('content', t('inspector.content'), [
      selectField(t('clock.format'), CLOCK_PATTERNS.includes(c.pattern) ? c.pattern : 'custom', [...CLOCK_PATTERNS.map(pattern => [pattern, formatClock(pattern, new Date(), c.language, c.casing)]), ['custom', t('clock.custom')]], pattern => {
        if (pattern === 'custom') ctx.root.querySelector('[list="clock-patterns"]')?.focus();
        else set({ pattern });
      }),
      textField(t('inspector.pattern'), c.pattern, pattern => set({ pattern }), { list: 'clock-patterns' }),
      el('datalist', { id: 'clock-patterns' }, CLOCK_PATTERNS.map(pattern => el('option', { value: pattern }))),
      el('div', { class: 'chip-group', role: 'group', 'aria-label': t('insp.clock.insert') }, ['H', 'M', 'S', 'd', 'A', 'B'].map(code => chipButton(`%${code}`, false, () => set({ pattern: c.pattern + `%${code}` })))),
      language,
      el('p', { id: 'clock-preview', class: 'hint', text: t('clock.preview', { value: applyTextCase(formatClock(c.pattern, new Date(), c.language, c.casing), style.casing) }) }),
      el('p', { class: 'hint', text: t('inspector.patternHelp') }),
    ])], look,
    behavior: [section('clock-blink', t('insp.clock.blink'), [
      toggleField(t('insp.clock.blink'), Boolean(style.parts?.separators?.blink), blink => update(e.id, { kind: { style: { parts: { separators: { blink } } } } })),
      el('p', { class: 'hint', text: t('insp.clock.blinkHint') }),
    ])],
  };
}

export function sections(ctx) { return describeSections(ctx, ctx.completePanels(panels(ctx))); }
