import { describeSections } from './sections.js';
// Existing player controls, isolated from inspector navigation.
import { el } from '../dom.js';
import { ICONS } from '../icons.js';
import { dashedHint, fieldGrid, numberField, prefixedField, iconChoice, propertySection as section, iconButton, selectField, textField } from '../fields.js';
import { PLAYER_PARTS, PROGRESS_DEFAULTS, PROGRESS_PATTERNS, PROGRESS_SEGMENTS, PROGRESS_THICKNESS, TIME_FORMATS, TIME_PLACEMENTS, partPatch, partsOf, progressOf } from '../../player-parts.js';
import { ART, PART_TILES, TIME_FORMAT_KEYS, TIME_PLACEMENT_KEYS, PATTERN_KEYS } from './art.js';

export function panels(ctx) {
  const { t, update, colorField, fontFields, coverCorners, appChipGroup, manualSource, optionTiles, customSource } = ctx;
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
      section('stopped', t('insp.stopped'), [
        iconChoice(t('insp.stopped'), c.hideWhenStopped ? 'hide' : 'text', [['hide', t('insp.stoppedHide'), 'eyeOff'], ['text', t('insp.stoppedText'), 'text']], value => set({ hideWhenStopped: value === 'hide' }), { wide: true }),
        !c.hideWhenStopped && textField(t('player.empty'), c.emptyText ?? '', emptyText => set({ emptyText: emptyText.slice(0, 512) })),
      ], { heading: false }),
      parts.time ? section('time', t('insp.partTime'), [
        selectField(t('insp.timeFormat'), c.timeFormat ?? 'elapsedTotal', TIME_FORMATS.map((f) => [f, t(TIME_FORMAT_KEYS[f])]), (timeFormat) => set({ timeFormat })),
      ]) : null,
      dashedHint([t('insp.playerHint')], { iconName: 'info' }),
    ];
    const tiles = PLAYER_PARTS.map((part) => ({ text: t(PART_TILES[part][0]), art: PART_TILES[part][1], pressed: parts[part], onClick: () => set(partPatch(c, part)) }));
    const partTiles = optionTiles(t('insp.show'), tiles); partTiles.classList.add('player-part-tiles');
    const look = [
      section('show', t('insp.show'), [partTiles], { caption: true }),
      parts.cover ? section('spacing', t('insp.cover'), [
        coverCorners(e),
        prefixedField('', t('player.coverGap'), c.coverGap ?? s.size * 0.3, v => set({ coverGap: Math.max(0, v) }), { min: 0, step: 'any', unit: 'px', iconName: 'spacing' }),
      ], { caption: true }) : null,
      parts.cover ? section('cover-fade', t('insp.player.coverFade'), ctx.fadeFields(e, { value: c.coverFade, set: coverFade => set({ coverFade }) }), { caption: true }) : null,
      parts.progress ? progressSection(e, parts) : null,
      section('font', t('inspector.font'), fontFields(e, { compact: true })),
    ];
    const font = look.find(node => node?.dataset.section === 'font');
    const body = font.querySelector('.property-section-body');
    const alignment = body.querySelector(`[role="group"][aria-label="${CSS.escape(t('inspector.align'))}"]`)?.closest('.fld-grid');
    if (alignment) look.find(node => node?.dataset.section === 'show').querySelector('.property-section-body').append(alignment);
    const textPaint = body.querySelector(':scope > .color-field');
    const colors = [textPaint];
    if (parts.progress) {
      const progress = progressOf(c), textColor = typeof s.paint === 'string' ? s.paint : PROGRESS_DEFAULTS.track;
      colors.push(colorField(t('insp.player.progressFill'), progress.fill ?? textColor, fill => set({ progressStyle: { fill } })), colorField(t('insp.player.progressTrack'), progress.track, track => set({ progressStyle: { track } })));
    }
    look.push(section('color', t('inspector.color'), colors));
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
    return section('progress', t('insp.progressBar'), [
      parts.time && iconChoice(t('insp.timesPlace'), style.times, TIME_PLACEMENTS.map((p, i) => [p, t(TIME_PLACEMENT_KEYS[i]), ['text', 'alignCenterX', 'alignTop'][i]]), (times) => times !== style.times && set({ times }), { wide: true }),
      iconChoice(t('inspector.style'), style.pattern, PROGRESS_PATTERNS.map(value => [value, t(PATTERN_KEYS[value]), ART[value]]), pattern => pattern !== style.pattern && set({ pattern }), { wide: true, caption: t('insp.progressBar') }),
      fieldGrid([
        prefixedField('', t('insp.thickness'), style.thickness, thickness => set({ thickness: Math.max(PROGRESS_THICKNESS.min, Math.min(PROGRESS_THICKNESS.max, thickness)) }), { ...PROGRESS_THICKNESS, unit: 'px', iconName: 'strokeWidth' }),
        el('div', { class: 'bound-field' }, [iconChoice(t('insp.ends'), style.ends, [['round', t('inspector.roundCaps'), 'ends-round'], ['square', t('insp.flatEnds'), 'ends-flat']], ends => ends !== style.ends && set({ ends }), { iconOnly: true, bare: true }),
        iconButton(t('insp.progressKnob'), ICONS['progress-knob'], () => set({ knob: !style.knob }), { 'aria-pressed': String(style.knob) })]),
      ]),
      style.pattern === 'segments' && numberField(t('inspector.count'), style.segments,
        (v) => set({ segments: Math.min(PROGRESS_SEGMENTS.max, Math.max(PROGRESS_SEGMENTS.min, Math.round(v))) }), { ...PROGRESS_SEGMENTS, compact: true }),
    ], { heading: false });
  }
  return playerForm(ctx.element);
}

export function sections(ctx) {
  return describeSections(ctx, ctx.completePanels(panels(ctx)));
}
