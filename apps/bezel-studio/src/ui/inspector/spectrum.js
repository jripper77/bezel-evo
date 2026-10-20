// Existing spectrum controls, isolated from inspector navigation.
import { SPECTRUM_STYLES, followedPlayer, spectrumOf, spectrumPatch } from '../../spectrum.js';
import { el } from '../dom.js';
import { numberField, propertySection as section, segmented, selectField, switchRow } from '../fields.js';

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
    const paint = e.kind.style.paint, gradient = typeof paint === 'object';
    const start = gradient ? paint.stops?.[0]?.[1] ?? '#38bdf8ff' : paint;
    const end = gradient ? paint.stops?.at(-1)?.[1] ?? '#f97316ff' : '#f97316ff';
    const setPaint = (mode, a = start, b = end) => update(e.id, { kind: { style: { paint: mode === 'gradient' ? { angle: 90, stops: [[0,a],[1,b]] } : a } } });
    const STYLE_LABELS = { mirrored: 'spectrum.mirrored', bars: 'graph.bars', blocks: 'spectrum.blocks', line: 'graph.line', radial: 'fade.radial' };
    // Fields that do not apply to the chosen style are hidden, never disabled.
    const line = c.style === 'line';
    const baseLabel = { mirrored: 'spectrum.centre', radial: 'spectrum.ring' }[c.style] ?? 'spectrum.baseline';
    return {
      data: [section('spectrum-source', t('spectrum.source'), [
        // D-AS-13: the followed Player's app, or everything the PC plays.
        segmented(t('spectrum.source'), c.audio, [['player', t('spectrum.audioPlayer')], ['system', t('spectrum.audioSystem')]], audio => set({ audio }), { bare: true }),
        ...(c.audio === 'system' ? [el('p', { class: 'hint', text: t('spectrum.systemHint') })] : [
          selectField(t('spectrum.follow'), c.player == null ? '' : String(c.player), options, v => set({ player: v ? Number(v) : null })),
          !selected && el('p', { class: 'hint', role: 'status', text: t('spectrum.playerMissing') }),
          el('p', { class: 'hint', text: t('spectrum.browser') }),
        ]),
        segmented(t('common.preview'), c.demo ? 'demo' : 'live', [['live',t('spectrum.live')],['demo',t('spectrum.demo')]], v => set({ demo: v === 'demo' })),
        c.demo && el('p', { class: 'hint', role: 'status', text: t('spectrum.demoHint') }),
        el('p', { class: 'hint', text: t('spectrum.enableHint') }),
      ]), section('spectrum-bands', t('spectrum.bands'), [
        numberField(t('spectrum.bands'), c.bands, bands => set({ bands }), { min: 8, max: 64, step: 1, iconName: 'count' }),
        numberField(t('spectrum.low'), c.low, low => set({ low }), { min: 20, max: 23999, step: 1, unit: 'Hz', iconName: 'frequency' }),
        numberField(t('spectrum.high'), c.high, high => set({ high }), { min: c.low + 1, max: 24000, step: 1, unit: 'Hz', iconName: 'frequency' }),
        numberField(t('spectrum.gain'), c.gain, gain => set({ gain }), { min: 0.1, max: 4, step: 0.1, unit: '×', iconName: 'gain' }),
        el('p', { class: 'hint', text: t('spectrum.gainHint') }),
      ])],
      look: [section('spectrum-paint', t('inspector.color'), [
        selectField(t('inspector.fill'), gradient ? 'gradient' : 'solid', [['solid', t('ring.solid')],['gradient',t('spectrum.gradient')]], mode => setPaint(mode)),
        colorField(t('inspector.color'), start, v => setPaint(gradient ? 'gradient' : 'solid', v)),
        gradient && colorField(t('ring.endColor'), end, v => setPaint('gradient', start, v)),
      ]), section('spectrum-bars', t('graph.bars'), [
        selectField(t('spectrum.style'), c.style, SPECTRUM_STYLES.map(v => [v, t(STYLE_LABELS[v])]), style => set({ style })),
        !line && numberField(t('spectrum.thickness'), c.thickness, thickness => set({ thickness }), { min: 1, max: 256, unit: 'px', iconName: 'width' }),
        !line && numberField(t('spectrum.gap'), c.gap, gap => set({ gap }), { min: 0, max: 128, unit: 'px', iconName: 'spacing' }),
        c.style === 'blocks' && numberField(t('spectrum.block'), c.block, block => set({ block }), { min: 1, max: 64, unit: 'px', iconName: 'height' }),
        c.style === 'blocks' && el('p', { class: 'hint', text: t('spectrum.blockColors') }),
        line && numberField(t('spectrum.stroke'), c.stroke, stroke => set({ stroke }), { min: 1, max: 32, unit: 'px', iconName: 'strokeWidth' }),
        line && switchRow(t('spectrum.fill'), c.fill, fill => set({ fill })),
        c.style === 'radial' && numberField(t('spectrum.inner'), c.inner, inner => set({ inner }), { min: 0, max: 90 }),
        c.style !== 'blocks' && switchRow(t(baseLabel), c.centre, centre => set({ centre })),
        !line && switchRow(t('spectrum.peaks'), c.peaks, peaks => set({ peaks })),
        segmented(t('card.tab.motion'), c.motion, ['smooth','balanced','snappy'].map(v => [v,t(`spectrum.${v}`)]), motion => set({ motion })),
      ]), !line && barCorners(e.id, c, set, e.frame)],
    };
  }
  return spectrumForm(ctx.element, ctx.assets);
}
