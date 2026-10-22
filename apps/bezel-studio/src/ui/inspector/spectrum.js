import { gaugeFillFields } from './paint.js';
import { ART } from './art.js';
import { ICONS } from '../icons.js';
import { describeSections } from './sections.js';
// Existing spectrum controls, isolated from inspector navigation.
import { SPECTRUM_STYLES, followedPlayer, spectrumOf, spectrumPatch } from '../../spectrum.js';
import { el } from '../dom.js';
import { numberField, prefixedField, iconChoice, fieldGrid, propertySection as section, segmented, selectField, toggleField as switchRow } from '../fields.js';

export function panels(ctx) {
  const { t, store, update, colorField, barCorners } = ctx;
  /** The media player (artboards PlayerData and PlayerLook). */
  function spectrumForm(e) {
    const c = spectrumOf(e.kind.content), theme = store.getState().theme;
    const set = patch => update(e.id, { kind: { content: spectrumPatch(c, patch) } });
    const players = theme.elements.filter(item => item.kind.type === 'text' && item.kind.content.type === 'player');
    const selected = followedPlayer(theme, c);
    const options = [['', t('spectrum.automatic')], ...players.map(item => [String(item.id), item.name])];
    if (c.player != null && !players.some(p => p.id === c.player)) options.push([String(c.player), t('spectrum.playerMissing')]);
    const STYLE_LABELS = { mirrored: 'spectrum.mirrored', bars: 'graph.bars', blocks: 'spectrum.blocks', line: 'graph.line', radial: 'fade.radial' };
    // Fields that do not apply to the chosen style are hidden, never disabled.
    const line = c.style === 'line';
    const baseLabel = { mirrored: 'spectrum.centre', radial: 'spectrum.ring' }[c.style] ?? 'spectrum.baseline';
    return {
      data: [section('spectrum-source', t('spectrum.source'), [
        // D-AS-13: the followed Player's app, or everything the PC plays.
        iconChoice(t('spectrum.source'), c.audio, [['player', t('spectrum.audioPlayer'), 'music'], ['system', t('spectrum.audioSystem'), 'monitor']], audio => set({ audio }), { bare: true, wide: true }),
        ...(c.audio === 'system' ? [el('p', { class: 'hint', text: t('spectrum.systemHint') })] : [
          selectField(t('spectrum.follow'), c.player == null ? '' : String(c.player), options, v => set({ player: v ? Number(v) : null })),
          !selected && el('p', { class: 'hint', role: 'status', text: t('spectrum.playerMissing') }),
          el('p', { class: 'hint', text: t('spectrum.browser') }),
        ]),
        segmented(t('common.preview'), c.demo ? 'demo' : 'live', [['live',t('spectrum.live')],['demo',t('spectrum.demo')]], v => set({ demo: v === 'demo' })),
        c.demo && el('p', { class: 'hint', role: 'status', text: t('spectrum.demoHint') }),
        el('p', { class: 'hint', text: t('spectrum.enableHint') }),
      ]), section('spectrum-bands', t('spectrum.bands'), [
        fieldGrid([
          prefixedField(t('insp.countPrefix'), t('spectrum.bands'), c.bands, bands => set({ bands }), { min: 8, max: 64, step: 1 }),
          prefixedField(t('insp.minPrefix'), t('spectrum.low'), c.low, low => set({ low }), { min: 20, max: 23999, step: 1, unit: 'Hz' }),
          prefixedField(t('insp.maxPrefix'), t('spectrum.high'), c.high, high => set({ high }), { min: c.low + 1, max: 24000, step: 1, unit: 'Hz' }),
        ], { columns: 3, lead: true }),
        numberField(t('spectrum.gain'), c.gain, gain => set({ gain }), { min: 0.1, max: 4, step: 0.1, unit: '×', iconName: 'gain', compact: true }),
        el('p', { class: 'hint', text: t('spectrum.gainHint') }),
      ])],
      look: [section('spectrum-paint', t('inspector.color'), gaugeFillFields(ctx, e, { path: ['kind', 'style', 'paint'] })), section('spectrum-bars', t('spectrum.style'), [
        ctx.optionTiles(t('spectrum.style'), SPECTRUM_STYLES.map(value => ({ text: t(STYLE_LABELS[value]), caption: value === 'mirrored' ? t('insp.spectrum.mirrorShort') : value === 'blocks' ? t('insp.spectrum.ledShort') : t(STYLE_LABELS[value]), art: ART[value] ?? (value === 'radial' ? ICONS.ring : ART.bars), pressed: c.style === value, onClick: () => set({ style: value }) }))),
        !line && fieldGrid([numberField(t('spectrum.thickness'), c.thickness, thickness => set({ thickness }), { min: 1, max: 256, unit: 'px', iconName: 'width', compact: true }),
        !line && numberField(t('spectrum.gap'), c.gap, gap => set({ gap }), { min: 0, max: 128, unit: 'px', iconName: 'spacing', compact: true })]),
        c.style === 'blocks' && numberField(t('spectrum.block'), c.block, block => set({ block }), { min: 1, max: 64, unit: 'px', iconName: 'height' }),
        c.style === 'blocks' && el('p', { class: 'hint', text: t('spectrum.blockColors') }),
        line && numberField(t('spectrum.stroke'), c.stroke, stroke => set({ stroke }), { min: 1, max: 32, unit: 'px', iconName: 'strokeWidth' }),
        line && switchRow(t('spectrum.fill'), c.fill, fill => set({ fill })),
        c.style === 'radial' && numberField(t('spectrum.inner'), c.inner, inner => set({ inner }), { min: 0, max: 90 }),
        c.style !== 'blocks' && switchRow(t(baseLabel), c.centre, centre => set({ centre })),
        !line && switchRow(t('spectrum.peaks'), c.peaks, peaks => set({ peaks })),
      ], { caption: true }), !line && barCorners(e.id, c, set, e.frame)],
      behavior: [section('spectrum-motion', t('insp.spectrum.motion'), [iconChoice(t('insp.spectrum.motion'), c.motion, ['smooth', 'balanced', 'snappy'].map(value => [value, t(`spectrum.${value}`), `spectrum-${value}`]), motion => set({ motion }), { wide: true, bare: true })], { caption: true })],
    };
  }
  return spectrumForm(ctx.element, ctx.assets);
}

export function sections(ctx) {
  return describeSections(ctx, ctx.completePanels(panels(ctx)));
}
