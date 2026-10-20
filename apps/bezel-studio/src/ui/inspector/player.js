// Existing player controls, isolated from inspector navigation.
import { el } from '../dom.js';
import { dashedHint, fieldGrid, numberField, prefixedField, rangeField, propertySection as section, segmented, selectField, switchRow, textField } from '../fields.js';
import { PLAYER_PARTS, PROGRESS_DEFAULTS, PROGRESS_PATTERNS, PROGRESS_SEGMENTS, PROGRESS_THICKNESS, TIME_FORMATS, TIME_PLACEMENTS, partPatch, partsOf, progressOf } from '../../player-parts.js';
import { ART, PART_TILES, TIME_FORMAT_KEYS, TIME_PLACEMENT_KEYS, PATTERN_KEYS } from './art.js';

export function panels(ctx) {
  const { t, update, colorField, fontFields, coverCorners, appChipGroup, manualSource, optionTiles, choiceCard, customSource } = ctx;
  function playerForm(e) {
    const c = e.kind.content;
    const s = e.kind.style;
    const set = (patch) => update(e.id, { kind: { content: patch } });
    const parts = partsOf(c);
    const { group, chips, custom } = appChipGroup(e);
    const data = [
      section('apps', t('insp.apps'), [
        group,
        el('div', { class: 'player-source' }, customSource || custom !== null ? [manualSource(e)] : []),
        el('p', { class: 'hint player-apps-hint', text: t(chips.length ? 'insp.appsHint' : 'insp.appsNone') }),
      ]),
      section('stopped', t('insp.stopped'), [el('div', { class: 'choice-cards', role: 'radiogroup', 'aria-label': t('insp.stopped') }, [
        choiceCard(`player-stopped-${e.id}`, Boolean(c.hideWhenStopped), t('insp.stoppedHide'), t('insp.stoppedHideNote'), () => set({ hideWhenStopped: true })),
        choiceCard(`player-stopped-${e.id}`, !c.hideWhenStopped, t('insp.stoppedText'), t('insp.stoppedTextNote'), () => set({ hideWhenStopped: false }),
          el('div', { class: 'choice-card-extra' }, [textField(t('player.empty'), c.emptyText ?? '', (emptyText) => set({ emptyText: emptyText.slice(0, 512) }))])),
      ])]),
      parts.time ? section('time', t('insp.partTime'), [
        selectField(t('insp.timeFormat'), c.timeFormat ?? 'elapsedTotal', TIME_FORMATS.map((f) => [f, t(TIME_FORMAT_KEYS[f])]), (timeFormat) => set({ timeFormat })),
      ]) : null,
      dashedHint([t('insp.playerHint')], { iconName: 'info' }),
    ];
    const tiles = PLAYER_PARTS.map((part) => ({ text: t(PART_TILES[part][0]), art: PART_TILES[part][1], pressed: parts[part], onClick: () => set(partPatch(c, part)) }));
    const look = [
      section('show', t('insp.show'), [optionTiles(t('insp.show'), tiles)]),
      parts.cover ? coverCorners(e) : null,
      parts.progress ? progressSection(e, parts) : null,
      section('spacing', t('insp.spacing'), [
        fieldGrid([
          prefixedField(t('insp.gap'), t('player.coverGap'), c.coverGap ?? s.size * 0.3, (v) => set({ coverGap: Math.max(0, v) }), { min: 0, step: 'any', unit: 'px', iconName: 'spacing' }),
          prefixedField(t('insp.textSize'), t('inspector.size'), s.size, (v) => update(e.id, { kind: { style: { size: Math.max(4, v) } } }), { min: 4, max: 800, unit: 'px', iconName: 'textSize' }),
        ]),
        el('p', { class: 'hint', text: t('insp.gapHint') }),
      ]),
      section('font', t('inspector.font'), fontFields(e, { size: false })),
    ];
    return { data: data.filter(Boolean), look };
  }

  /**
   * The player's progress bar (D-2026-10-16-studio-redesign-fidelity-26):
   * colours, thickness, ends, knob, pattern and, with the times on, where
   * they sit. Absent fields draw the bar as before it had options.
   */
  function progressSection(e, parts) {
    const c = e.kind.content;
    const style = progressOf(c);
    const set = (patch) => update(e.id, { kind: { content: { progressStyle: patch } } });
    const textColor = typeof e.kind.style.paint === 'string' ? e.kind.style.paint : PROGRESS_DEFAULTS.track;
    const pattern = (value) => ({ text: t(PATTERN_KEYS[value]), art: ART[value], pressed: style.pattern === value, onClick: () => style.pattern !== value && set({ pattern: value }) });
    return section('progress', t('insp.progressBar'), [
      colorField(t('inspector.fill'), style.fill ?? textColor, (fill) => set({ fill })),
      colorField(t('inspector.track'), style.track, (track) => set({ track })),
      rangeField(t('insp.thickness'), style.thickness, (thickness) => set({ thickness }), { ...PROGRESS_THICKNESS, format: (v) => t('insp.px', { value: v }) }),
      switchRow(t('inspector.roundCaps'), style.ends !== 'square', (on) => set({ ends: on ? 'round' : 'square' })),
      switchRow(t('insp.progressKnob'), style.knob, (knob) => set({ knob })),
      optionTiles(t('inspector.style'), PROGRESS_PATTERNS.map(pattern)),
      style.pattern === 'segments' && numberField(t('inspector.count'), style.segments,
        (v) => set({ segments: Math.min(PROGRESS_SEGMENTS.max, Math.max(PROGRESS_SEGMENTS.min, Math.round(v))) }), PROGRESS_SEGMENTS),
      parts.time && segmented(t('insp.timesPlace'), style.times, TIME_PLACEMENTS.map((p, i) => [p, t(TIME_PLACEMENT_KEYS[i])]), (times) => times !== style.times && set({ times })),
    ]);
  }
  return playerForm(ctx.element);
}
