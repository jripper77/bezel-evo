// The extended needle (D-2026-10-20-studio-ux-v2-08), the UI's side: the
// look with every absent field at the backend's default and the pure
// geometry and colours the demo draws with, mirroring
// crates/bezel-core/src/domain/needle.rs and crates/bezel-render/src/needle_gauge.rs.

/** Most colour zones on an arc. */
export const MAX_ZONES = 8;
export const TRACKS = Object.freeze(['single', 'zones', 'gradient']);
export const SCALE_TICKS = Object.freeze(['none', 'major', 'majorMinor']);
export const SCALE_NUMBERS = Object.freeze(['none', 'ends', 'halves', 'major']);
export const NEEDLE_SHAPES = Object.freeze(['thin', 'tapered', 'arrow', 'dot']);
export const PIVOT_CAPS = Object.freeze(['none', 'filled', 'ring']);

/** A needle's look with every absent field at its default (`{}` is the default look). */
export function lookOf(look) {
  return {
    arcWidth: 5, arcCap: 'round', track: 'single', zones: [], colorFromZone: false, ticks: 'majorMinor', numbers: 'halves',
    numberGap: 8, marks: '#aeb5bdff', shape: 'tapered', length: 92, tail: 12, cap: 'ring', capDiameter: 9, capColor: '#15191cff',
    center: null, peakSeconds: null, ...(look ?? {}),
  };
}

/** Share of the scale `value` stands at, within 0..1 (0 for an empty range). */
export function share(binding, value) {
  const span = binding.max - binding.min;
  if (!Number.isFinite(span) || span === 0 || !Number.isFinite(value)) return 0;
  return Math.min(1, Math.max(0, (value - binding.min) / span));
}

/** The zone `value` is in (the last listed wins). */
export function zoneAt(look, value) {
  return [...lookOf(look).zones.slice(0, MAX_ZONES)].reverse().find((z) => value >= Math.min(z.from, z.to) && value <= Math.max(z.from, z.to)) ?? null;
}

/** The needle's colour at `value`: its zone's when it takes it. */
export function needleColor(look, value, color) {
  if (!lookOf(look).colorFromZone || !Number.isFinite(value)) return color;
  return zoneAt(look, value)?.color ?? color;
}

const hex = (c) => { const h = c.replace('#', ''); const n = (i) => parseInt(h.slice(i, i + 2), 16); return [n(0), n(2), n(4), h.length >= 8 ? n(6) : 255]; };
const toHex = (rgba) => `#${rgba.map((v) => Math.round(v).toString(16).padStart(2, '0')).join('')}`;

/** The single arc's colour: the marks' at 30 %. */
export function faint(look) {
  const [r, g, b, a] = hex(lookOf(look).marks);
  return toHex([r, g, b, Math.round(a * 0.3)]);
}

/** The gradient's colour at `at` of the scale: zone colours at their middles, blended, held at the ends. */
export function gradientAt(look, binding, at) {
  const stops = lookOf(look).zones.slice(0, MAX_ZONES).map((z) => [share(binding, (z.from + z.to) / 2), z.color]).sort((a, b) => a[0] - b[0]);
  if (!stops.length) return faint(look);
  if (at <= stops[0][0]) return stops[0][1];
  for (let i = 1; i < stops.length; i++) {
    const [a, ca] = stops[i - 1], [b, cb] = stops[i];
    if (at <= b) {
      const t = b > a ? (at - a) / (b - a) : 1, x = hex(ca), y = hex(cb);
      return toHex(x.map((v, j) => v + (y[j] - v) * t));
    }
  }
  return stops[stops.length - 1][1];
}

/** Shares where ticks stand: `{ major, minor }`. */
export function tickShares(look) {
  const { ticks } = lookOf(look);
  const major = Array.from({ length: 11 }, (_, i) => i / 10);
  if (ticks === 'none') return { major: [], minor: [] };
  if (ticks === 'major') return { major, minor: [] };
  return { major, minor: Array.from({ length: 51 }, (_, i) => i).filter((i) => i % 5 !== 0).map((i) => i / 50) };
}

/** Shares that carry a number. */
export function numberShares(look) {
  const { numbers } = lookOf(look);
  if (numbers === 'ends') return [0, 1];
  if (numbers === 'halves') return [0, 0.5, 1];
  if (numbers === 'major') return Array.from({ length: 11 }, (_, i) => i / 10);
  return [];
}

/** A scale number for `language`, like the backend's: whole as it is, else up to two decimals, a decimal comma in it and pt. */
export function scaleNumber(value, language = 'en') {
  if (!Number.isFinite(value)) return '—';
  const rounded = Math.round(value * 100) / 100;
  let text = Number.isInteger(rounded) ? rounded.toFixed(0) : rounded.toFixed(2).replace(/0+$/, '');
  if (text === '-0') text = '0';
  return /^(it|pt)/i.test(language) ? text.replace('.', ',') : text;
}

/** The largest arc round the pivot of `frame` that stays `margin` inside it, like the backend's `Gauge::fit`. */
export function fitGauge(frame, pivot, start, sweep, margin) {
  const px = frame.x + pivot[0] * frame.width, py = frame.y + pivot[1] * frame.height;
  const s = Math.max(-360, Math.min(360, sweep));
  const angles = [start, start + s];
  for (const c of [0, 90, 180, 270, 360, -90, -180, -270, -360]) {
    if (s >= 0 ? c > start && c < start + s : c < start && c > start + s) angles.push(c);
  }
  const room = (space, reach) => (reach > 1e-6 ? Math.max(0, space - margin) / reach : Infinity);
  const radius = Math.min(...angles.map((a) => {
    const r = a * Math.PI / 180, ux = Math.sin(r), uy = -Math.cos(r);
    return Math.min(room(px - frame.x, -ux), room(frame.x + frame.width - px, ux), room(py - frame.y, -uy), room(frame.y + frame.height - py, uy));
  }));
  return { pivot: [px, py], radius: Number.isFinite(radius) ? radius : 0, start, sweep: s };
}

/** The demo's peak mark at `t` s: the highest sample of the last `seconds` (one sample every `every` s); null without. */
export function demoPeak(look, read, t, every, seconds = lookOf(look).peakSeconds) {
  if (!(seconds > 0)) return null;
  const step = every > 0 ? every : 1;
  const last = Math.floor(t / step) * step;
  let high = -Infinity;
  for (let s = last; s > t - seconds - 1e-9 && s >= last - 3600; s -= step) {
    const v = read(s);
    if (Number.isFinite(v)) high = Math.max(high, v);
  }
  return Number.isFinite(high) ? high : null;
}
