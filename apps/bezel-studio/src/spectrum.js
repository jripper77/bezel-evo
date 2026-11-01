// Spectrum configuration shared by the palette, inspector and explicit demo.
import { DANCERS } from './dancer-pixel.js';
export const SPECTRUM_DEFAULTS = Object.freeze({ player: null, bands: 32, low: 60, high: 16000, motion: 'balanced', peaks: true, demo: false, thickness: 12, gap: 3, corners: Object.freeze([2,2,2,2]), centre: true, style: 'mirrored', block: 6, stroke: 2, fill: true, inner: 40, gain: 1, audio: 'player', dancer: 'robot', peakHold: 0.35, unlit: null, waves: true });
/** Drawing styles (D-AS-11); `mirrored` is the original D-AS-4 look. */
export const SPECTRUM_STYLES = Object.freeze(['mirrored', 'bars', 'blocks', 'line', 'radial', 'neon', 'dancer']);
/** Seconds a band takes to fall in each motion (bezel-core `Motion::times`). */
const FALL = Object.freeze({ smooth: 0.9, balanced: 0.45, snappy: 0.18 });
/** How a neon spectrum glows and waves, like the backend's (bezel-render spectrum.rs). */
export const NEON = Object.freeze({
  waves: 7, periods: 3, speed: 1.2, tips: 0.1,
  halos: Object.freeze([[2.4, 0.07], [1.2, 0.1], [0.5, 0.16]]),
});
/** Spacing of the past moments the demo's peak caps look back at, seconds. */
const DEMO_PEAK_STEP = 0.05;
export function spectrumOf(content) { return { ...SPECTRUM_DEFAULTS, ...content }; }
export function followedPlayer(theme, content) {
  const players = theme.elements.filter(e => e.kind.type === 'text' && e.kind.content.type === 'player');
  return content.player == null ? (players.length === 1 ? players[0] : null) : players.find(e => e.id === content.player) ?? null;
}
export function spectrumPatch(content, patch) {
  const c = { ...spectrumOf(content), ...patch };
  const clamp = (n, low, high, fallback) => Number.isFinite(n) ? Math.max(low, Math.min(high, n)) : fallback;
  c.bands = Math.round(clamp(c.bands, 8, 64, 32));
  c.low = clamp(c.low, 20, 23999, 60); c.high = clamp(c.high, c.low + 1, 24000, 16000);
  c.thickness = clamp(c.thickness, 1, 256, 12); c.gap = clamp(c.gap, 0, 128, 3);
  if (!['smooth','balanced','snappy'].includes(c.motion)) c.motion = 'balanced';
  if (!SPECTRUM_STYLES.includes(c.style)) c.style = 'mirrored';
  if (!['player', 'system'].includes(c.audio)) c.audio = 'player';
  if (!DANCERS.includes(c.dancer)) c.dancer = 'robot';
  c.block = clamp(c.block, 1, 64, 6); c.stroke = clamp(c.stroke, 1, 32, 2); c.inner = clamp(c.inner, 0, 90, 40); c.gain = clamp(c.gain, 0.1, 4, 1);
  c.fill = c.fill !== false; c.waves = c.waves !== false;
  c.peakHold = clamp(c.peakHold, 0, 10, 0.35);
  c.unlit = typeof c.unlit === 'string' && /^#[0-9a-f]{6}([0-9a-f]{2})?$/i.test(c.unlit) ? c.unlit : null;
  c.corners = (Array.isArray(c.corners) && c.corners.length === 4 ? c.corners : SPECTRUM_DEFAULTS.corners).map(v => clamp(v, 0, 256, 2));
  return c;
}
const demoLevel = (i, seconds) => 0.45 + 0.25 * Math.sin(seconds * 2 + i * 0.37);
export function demoSpectrum(content, seconds) {
  const c = spectrumOf(content);
  return Array.from({ length: c.bands }, (_, i) => demoLevel(i, seconds));
}
/** The demo's peak caps, like the backend's (`Spectrum::demo_levels`): each holds the highest level for `peakHold`, then falls. */
export function demoPeaks(content, seconds) {
  const c = spectrumOf(content), fall = FALL[c.motion] ?? FALL.balanced;
  const steps = Math.ceil((c.peakHold + fall) / DEMO_PEAK_STEP);
  return Array.from({ length: c.bands }, (_, i) => {
    let peak = 0;
    for (let k = 0; k <= steps; k++) {
      const ago = k * DEMO_PEAK_STEP;
      peak = Math.max(peak, demoLevel(i, seconds - ago) - Math.max(0, ago - c.peakHold) / fall);
    }
    return Math.min(1, peak);
  });
}
