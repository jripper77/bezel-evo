// Moving weather icons (D-2026-10-20-studio-ux-v2-09), mirroring
// crates/bezel-core/src/domain/weather.rs `pose` for the demo: offsets in the
// units of a 64-unit icon, opacities 0..1, periodic for every family.

/** The movements, in the inspector's order. */
export const ICON_MOTIONS = Object.freeze(['still', 'gentle', 'lively']);
/** Allowed speeds. */
export const ICON_SPEED = Object.freeze({ min: 0.5, max: 2 });
/** Cross-fade between two condition icons, ms. */
export const ICON_FADE_MS = 300;

/** Share of the movement of `motion`: 0, 0.5 or 1. */
export const amplitudeOf = (motion) => (motion === 'lively' ? 1 : motion === 'gentle' ? 0.5 : 0);
/** Time between frames while the icon moves, ms; null when still. */
export const iconStep = (motion) => (motion === 'lively' ? 33 : motion === 'gentle' ? 50 : null);

/** The icon at rest. */
export const REST = Object.freeze({
  rays: 0, disc: 0, cloud: 0, strips: [[0, 1], [0, 1], [0, 1], [0, 1]], drops: [[0, 0, 1], [0, 0, 1], [0, 0, 1]], bolt: 1, whole: 1, halo: 0, flash: 0,
});

/** cubic-bezier(0.45, 0, 0.55, 1) at `x`. */
export function soft(x) {
  if (x <= 0) return 0;
  if (x >= 1) return 1;
  const bezier = (t, a, b) => { const u = 1 - t; return 3 * u * u * t * a + 3 * u * t * t * b + t * t * t; };
  let low = 0, high = 1;
  for (let i = 0; i < 24; i++) { const mid = (low + high) / 2; if (bezier(mid, 0.45, 0.55) < x) low = mid; else high = mid; }
  return bezier((low + high) / 2, 0, 1);
}

const phase = (t, period, delay) => { const p = ((t - delay) / period) % 1; return p < 0 ? p + 1 : p; };
const thereAndBack = (p, from, to) => from + (to - from) * soft(p < 0.5 ? p * 2 : 2 - p * 2);
const through = (p, values) => {
  const spans = values.length - 1, at = Math.max(0, Math.min(spans - 1e-6, p * spans)), i = Math.floor(at);
  return values[i] + (values[i + 1] - values[i]) * (at - i);
};
const keyed = (p, keys) => {
  for (let i = 1; i < keys.length; i++) {
    const [a, va] = keys[i - 1], [b, vb] = keys[i];
    if (p <= b) return b > a ? va + (vb - va) * (p - a) / (b - a) : vb;
  }
  return keys[keys.length - 1][1];
};
const breathe = (v, low, high, a) => { const middle = (low + high) / 2; return middle + (v - middle) * a; };

/** The pose of family `family`'s icon `t` seconds into its movement, like the backend's. */
export function iconPose(family, t, amplitude, speed = 1) {
  const a = Math.min(1, Math.max(0, amplitude));
  if (a === 0 || !Number.isFinite(t)) return REST;
  const time = t * Math.min(ICON_SPEED.max, Math.max(ICON_SPEED.min, speed));
  const p = structuredClone(REST);
  const sun = () => {
    p.rays = phase(time, 24, 0) * 360;
    p.disc = thereAndBack(phase(time, 4, 0), 0, 0.12 * a);
    p.halo = breathe(thereAndBack(phase(time, 4, 0), 0.1, 0.3), 0.1, 0.3, a);
  };
  if (family === 0) sun();
  else if (family === 1) { sun(); p.cloud = thereAndBack(phase(time, 7, 0), -3, 1) * a; }
  else if (family === 2) p.cloud = thereAndBack(phase(time, 7, 0), -2, 3) * a;
  else if (family === 3) p.strips = p.strips.map((_, i) => { const at = phase(time, 5 + i, 0.6 * i); return [thereAndBack(at, -4, 4) * a, breathe(thereAndBack(at, 0.35, 1), 0.35, 1, a)]; });
  else if (family === 4) p.drops = p.drops.map((_, i) => { const at = phase(time, 1, 0.33 * i); return [through(at, [0, -2]) * a, through(at, [-4, 8]) * a, breathe(through(at, [0, 1, 0]), 0, 1, a)]; });
  else if (family === 5) p.drops = p.drops.map((_, i) => { const at = phase(time, 3, i); return [through(at, [0, 2, -1]) * a, through(at, [-6, 2, 10]) * a, breathe(through(at, [0, 1, 0]), 0, 1, a)]; });
  else if (family === 6) {
    const bolt = keyed(phase(time, 3, 0), [[0, 0], [0.55, 0], [0.6, 1], [0.65, 0.2], [0.7, 1], [0.8, 0], [1, 0]]);
    p.bolt = breathe(bolt, 0, 1, a); p.flash = 0.12 * bolt * a;
  } else p.whole = breathe(thereAndBack(phase(time, 4, 0), 0.45, 0.85), 0.45, 0.85, a);
  return p;
}
