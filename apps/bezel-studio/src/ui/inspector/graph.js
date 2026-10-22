import { describeSections } from './sections.js';
import { graphSeriesOf, graphStyleOf, graphSeriesPatch, graphStylePatch, graphBarWidth, graphThicknessPatch } from '../../graph-inspector.js';
// Existing graph controls, isolated from inspector navigation.
import { el, svgEl as node } from '../dom.js';
import { chipButton, detailsField, fieldGrid, numberField, iconButton, iconChoice, propertySection as section, segmented, selectField, sentence } from '../fields.js';
import { joinColor, splitColor } from '../../color-math.js';
import { sentenceParts } from '../../sentence.js';
import { DURATION_UNITS, INTERVAL_UNITS, TIME_UNITS, graphRuler, graphSentence, graphTimingPatch, graphWindowRange, quickDurations } from '../../inspector-helpers.js';
import { ICONS } from '../icons.js';
import { ART } from './art.js';

const precisionViews = new WeakMap();

export function panels(ctx) {
  const { store, t, update, colorField, rangeFields, sourceSection, timingOf, graphWindowMs, durationText, optionTiles } = ctx;
  if (!precisionViews.has(store)) precisionViews.set(store, new Map());
  const precision = precisionViews.get(store);
  /**
   * The tokens of a time in the window sentence: its number and the select
   * of its unit (words agreeing with the number), `onChange(seconds)` when
   * either changes. Built from numberField and selectField; the sentence
   * hides their labels, so each control carries its name.
   */
  function timeTokens(shown, units, label, unitLabel, onChange) {
    const number = numberField(label, shown.value, (v) => onChange(v * TIME_UNITS[shown.unit]), { min: 0, step: 'any' });
    const unit = selectField(unitLabel, shown.unit, units.map((u) => [u, t(`insp.unitWord.${u}.${shown.plural}`)]), (u) => onChange(shown.value * TIME_UNITS[u]));
    number.querySelector('input').setAttribute('aria-label', label);
    unit.querySelector('select').setAttribute('aria-label', unitLabel);
    unit.querySelector('select').classList.add('sentence-unit');
    return { number, unit };
  }

  /**
   * The graph's window as one sentence (D-2026-10-15-studio-redesign-fidelity-17):
   * "Show the last [1] [hour], with one point every [30] [seconds]", its
   * word order from the locale's template.
   */
  function windowSentence(timing, setTiming) {
    const shown = graphSentence(timing);
    const duration = timeTokens(shown.duration, DURATION_UNITS, t('insp.durationLabel'), t('insp.durationUnit'), (d) => setTiming(d, timing.interval));
    const interval = timeTokens(shown.interval, INTERVAL_UNITS, t('insp.intervalLabel'), t('insp.intervalUnit'), (i) => setTiming(timing.duration, i));
    const slots = { last: t(shown.last), duration: duration.number, durationUnit: duration.unit, interval: interval.number, intervalUnit: interval.unit };
    return sentence(sentenceParts(t('insp.windowSentence')).map((part) => part.word ?? slots[part.slot]), { class: 'graph-window' });
  }

  /** The time ruler under the sentence: from −duration to now, a mark per point (thinned out past a few dozen), how many. */
  function timeRuler(timing) {
    const line = node('svg', { class: 'graph-ruler-line', viewBox: '0 0 100 10', preserveAspectRatio: 'none', 'aria-hidden': 'true' });
    line.append(
      node('line', { class: 'graph-ruler-base', x1: 0, y1: 9.5, x2: 100, y2: 9.5 }),
      ...graphRuler(timing.points).ticks.map((x) => node('line', { class: 'graph-ruler-tick', x1: x * 100, y1: 3, x2: x * 100, y2: 9.5 })),
    );
    return el('div', { class: 'graph-ruler' }, [line, el('div', { class: 'graph-ruler-labels inspector-preview-axis' }, [
      el('span', { 'aria-hidden': 'true', text: t('insp.ago', { duration: durationText(timing.duration) }) }),
      el('span', { class: 'graph-points', 'aria-live': 'polite', text: t('insp.points', { points: timing.points }) }),
      el('span', { 'aria-hidden': 'true', text: t('insp.now') }),
    ])]);
  }

  /** The graph's window: quick durations, the sentence of its duration and point interval, the ruler of what they make. */
  function graphWindow(e) {
    const timing = timingOf(e);
    const setTiming = (duration, interval) => {
      const { history, intervalSeconds } = graphTimingPatch(duration, interval);
      update(e.id, { kind: { history, intervalSeconds } });
    };
    const range = graphWindowRange(timing.interval);
    return section('window', t('insp.window'), [
      el('div', { class: 'chip-group', role: 'group', 'aria-label': t('insp.quickDurations') },
        quickDurations(timing.interval, timing.points).map((d) => chipButton(durationText(d.seconds), d.pressed, () => setTiming(d.seconds, timing.interval), { disabled: d.disabled }))),
      detailsField(t('insp.graphPrecision', { interval: durationText(timing.interval), points: timing.points }), [
        windowSentence(timing, setTiming), timeRuler(timing),
        el('p', { class: 'hint graph-range', text: t('insp.windowRange', { min: durationText(range.min), max: durationText(range.max) }) }),
        timing.own ? null : el('p', { class: 'hint', text: t('insp.followsRefresh', { seconds: timing.interval }) }),
        el('p', { class: 'hint', text: t('insp.historyHint') }),
      ], { open: precision.get(e.id) ?? false, onToggle: open => precision.set(e.id, open) }),
    ], { caption: true });
  }

  function graphForm(e) {
    const k = e.kind;
    // Data (artboard GraphData): what it plots, its vertical scale and how far back it looks.
    const data = [
      section('series', t('insp.graphSeries'), [iconChoice(t('insp.graphSeries'), graphSeriesOf(e), [['single', t('insp.graphSingle'), 'graph-single'], ['mirrored', t('insp.graphMirrored'), 'graph-mirrored']], series => update(e.id, graphSeriesPatch(e, series)), { wide: true, bare: true })], { caption: true }),
      sourceSection(e, k.binding.key, (key) => ({ kind: { binding: { key } } }), { range: false }),
      section('scale', t('insp.scale'), [
        segmented(t('insp.scale'), k.autoscale ? 'auto' : 'fixed', [['auto', t('insp.automatic')], ['fixed', t('insp.fixed')]], (v) => update(e.id, { kind: { autoscale: v === 'auto' } }), { bare: true }),
        rangeFields(e.id, k.binding, { muted: Boolean(k.autoscale) }),
        k.autoscale ? el('p', { class: 'hint' }, [t('insp.autoscaleHint'), ' ', t('insp.peak'), ' ', el('span', { class: 'mono graph-peak', dataset: { peakKey: k.binding.key, windowMs: String(graphWindowMs(e)) }, text: '—' })]) : null,
      ], { caption: true }),
      graphWindow(e),
    ];
    // Look (artboard GraphLook): the style, the color, the fill of an area and the line.
    const style = (value, text) => ({ text, art: ART[value === 'rounded' ? 'bars' : value], pressed: graphStyleOf(e) === value, onClick: () => graphStyleOf(e) !== value && update(e.id, graphStylePatch(e, value)) });
    const { rgb } = splitColor(k.color);
    const fillAlpha = typeof k.fill === 'string' ? splitColor(k.fill).alpha : 25;
    const look = [
      section('style', t('inspector.style'), [
        optionTiles(t('inspector.style'), [style('line', t('graph.line')), style('area', t('graph.area')), style('bars', t('graph.bars')), style('rounded', t('insp.graphRounded'))]),
        fieldGrid([el('div', { class: 'bound-field' }, [numberField(t('insp.thickness'), k.style === 'bars' ? Math.round(graphBarWidth(e) * 100) / 100 : k.lineWidth, width => update(e.id, graphThicknessPatch(e, width)), { min: 1, step: 0.5, unit: 'px', compact: true }), k.style === 'bars' && e.visualization?.barWidth != null && iconButton(t('insp.graphAutoWidth'), ICONS.undo, () => update(e.id, graphThicknessPatch(e, null)))]),
        graphSeriesOf(e) === 'mirrored' && numberField(t('insp.graphSeriesGap'), e.visualization.seriesGap ?? 2, seriesGap => update(e.id, { visualization: { seriesGap: Math.min(32, Math.max(0, seriesGap)) } }), { min: 0, max: 32, unit: 'px', compact: true }),
        ].filter(Boolean)),
        e.visualization && e.visualization.barWidth == null && ['rounded', 'bars'].includes(graphStyleOf(e)) && numberField(t('visual.gap'), e.visualization.gap, gap => update(e.id, { visualization: { gap: Math.max(0, Math.min(32, gap)) } }), { min: 0, max: 32, unit: 'px' }),
      ]),
      section('color', t('inspector.color'), [colorField(t('inspector.color'), k.color, (v) => update(e.id, { kind: { color: v } }))]),
      k.style === 'area' ? section('fill', t('insp.areaFill'), [fieldGrid([
        numberField(t('insp.areaFill'), fillAlpha, value => update(e.id, { kind: { fill: joinColor(rgb, Math.max(0, Math.min(100, value))) } }), { min: 0, max: 100, unit: '%', iconName: 'graph-single', compact: true }),
      ])], { heading: false }) : null,
    ];
    return { data, look };
  }
  return graphForm(ctx.element, ctx.assets);
}

export function sections(ctx) {
  return describeSections(ctx, ctx.completePanels(panels(ctx)));
}
