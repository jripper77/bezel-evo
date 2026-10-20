// Smooth and inertial value transitions, the
// UI's side: what an element's `valueTransition` says, the patch a control
// makes, and the demo's mirror of the runtime (crates/bezel-core
// domain::transition): cubic easing or damped motion, a frame every 50 ms at most while a
// value moves, none once it lands, and a snap when motion is reduced.

/** Duration of the transition of an object made in the Studio, ms. */
export const TRANSITION_DEFAULT_MS = 400;
/** The shortest and longest transition the backend keeps, ms. */
export const TRANSITION_BOUNDS = Object.freeze({ min: 50, max: 5000 });
/** The durations the inspector offers, ms. */
export const TRANSITION_RANGE = Object.freeze({ min: 100, max: 2000, step: 50 });
/** Shortest time between two frames drawn while a value moves, ms (20 fps). */
export const TRANSITION_STEP_MS = 50;
/** The modes, in the inspector's order; `count` and `roll` are for numbers (D-2026-10-20-studio-ux-v2-05). */
export const TRANSITION_MODES = Object.freeze(['none', 'smooth', 'inertia', 'count', 'roll']);
/** How many readings a value can be averaged over («Media su»); 1 is off. */
export const AVERAGE_RANGE = Object.freeze({ min: 1, max: 30 });

/** The transition of an object made in the Studio. */
export const defaultTransition = () => ({ mode: 'smooth', durationMs: TRANSITION_DEFAULT_MS });

/** Whether `element` shows a value that can ease: a bar, a ring, a needle or a sensor text. */
export function easesValue(element) {
  const k = element?.kind;
  if (!k || element.isGroup || element.card) return false;
  if (k.type === 'text') return k.content?.type === 'sensor';
  return ['bar', 'ring', 'needle'].includes(k.type);
}

/**
 * The element's transition as the inspector shows it: `none` when absent
 * (a theme written before it), with the duration it keeps for when it is
 * turned on.
 * @returns {{mode: 'none'|'smooth'|'inertia', durationMs: number}}
 */
export function transitionOf(element) {
  const t = element?.valueTransition;
  const mode = t?.mode === 'inertia' || (t?.mode === 'smooth' && t.inertia) ? 'inertia' : ['smooth', 'count', 'roll'].includes(t?.mode) ? t.mode : 'none';
  const ms = Number(t?.durationMs);
  return { mode, durationMs: Number.isFinite(ms) && ms > 0 ? ms : TRANSITION_DEFAULT_MS };
}

/**
 * The whole `valueTransition` after a change of `mode` or `durationMs`
 * (always both fields: a merge never leaves a smooth transition without
 * its duration). The duration is held within the inspector's range.
 */
export function transitionPatch(element, change) {
  const next = { ...transitionOf(element), average: averageOf(element), ...change };
  const mode = TRANSITION_MODES.includes(next.mode) ? next.mode : 'none';
  const ms = Math.round(Number(next.durationMs));
  const durationMs = Number.isFinite(ms) ? Math.min(TRANSITION_RANGE.max, Math.max(TRANSITION_RANGE.min, ms)) : TRANSITION_DEFAULT_MS;
  const average = clampAverage(next.average);
  return { mode: mode === 'inertia' ? 'smooth' : mode, durationMs,
    ...(mode === 'inertia' || element?.valueTransition?.inertia ? { inertia: mode === 'inertia' } : {}),
    ...(average > 1 ? { average } : {}) };
}

const clampAverage = (n) => {
  const v = Math.round(Number(n));
  return Number.isFinite(v) ? Math.min(AVERAGE_RANGE.max, Math.max(AVERAGE_RANGE.min, v)) : 1;
};

/** Readings the shown value is the mean of (1: each reading as it is), like the backend's. */
export function averageOf(element) {
  return clampAverage(element?.valueTransition?.average ?? 1);
}

/** How long a change lasts, ms, like the backend's; null when it jumps. */
export function transitionLength(element) {
  const t = element?.valueTransition;
  if (!['smooth', 'inertia', 'count', 'roll'].includes(t?.mode) || !(t.durationMs > 0)) return null;
  return Math.min(TRANSITION_BOUNDS.max, Math.max(TRANSITION_BOUNDS.min, t.durationMs));
}

/** Ease-out cubic: the share done at `progress` (0..1); the end outside it. */
export function ease(progress) {
  if (Number.isNaN(progress)) return 1;
  const p = Math.min(1, Math.max(0, progress));
  return 1 - (1 - p) ** 3;
}

/** Critically damped motion, with position and velocity continuous on retarget. */
export function inertiaAt(from, to, velocity, elapsed, length) {
  if (!(length > 0) || elapsed >= length) return { value: to, velocity: 0 };
  const time = Math.max(0, elapsed), omega = 8 / length;
  const displacement = from - to, b = velocity + omega * displacement, decay = Math.exp(-omega * time);
  return { value: to + (displacement + b * time) * decay, velocity: (velocity - omega * b * time) * decay };
}

/** Milliseconds from the last sample (every `every` s) to `t` s, to the microsecond. */
const sinceSample = (t, every) => Math.round((t - Math.floor(t / every) * every) * 1e6) / 1e3;

/** `n` written as milliseconds ("400 ms"), as the card's transition writes it. */
export const formatMs = (n) => `${n} ms`;

/**
 * The demo's value of an element at `t` seconds. The demo samples every
 * `refresh` seconds: `read(s)` is the reading of the sample taken at
 * `s`. An element with a transition eases from the previous sample to the
 * last one for its length after it; without one (or with `motion` off) it
 * shows the last sample; one that averages shows the mean of its last
 * `average` samples.
 * @param {(seconds: number) => number} read
 */
export function easedDemoValue(element, read, t, refresh, motion = true) {
  const every = refresh > 0 ? refresh : 1;
  const fractionAt = averagedReading(element, read, every);
  const sampled = Math.floor(t / every) * every;
  const to = fractionAt(sampled);
  const length = motion ? transitionLength(element) : null;
  if (length === null) return to;
  if (element.valueTransition?.mode === 'inertia' || element.valueTransition?.inertia) {
    // A deterministic bounded history keeps the browser preview independent of
    // render frequency, including readings arriving before settling completes.
    const seconds = length / 1000;
    const count = Math.min(128, Math.ceil(seconds / every) * 4 + 4);
    let start = sampled - count * every;
    let from = fractionAt(start), target = from, velocity = 0;
    for (let n = count - 1; n >= 0; n--) {
      const at = sampled - n * every, current = fractionAt(at);
      if (!Number.isFinite(target) || !Number.isFinite(current)) { start = at; velocity = 0; from = current; target = current; continue; }
      if (current !== target) {
        const state = inertiaAt(from, target, velocity, at - start, seconds);
        from = state.value; velocity = state.velocity; target = current; start = at;
      }
    }
    return inertiaAt(from, target, velocity, t - start, seconds).value;
  }
  const since = sinceSample(t, every);
  if (since >= length) return to;
  const from = fractionAt(sampled - every);
  const share = element.valueTransition?.mode === 'count' ? Math.min(1, since / length) : ease(since / length);
  return from + (to - from) * share;
}

/** `read` averaged over the element's last readings, one sample `every` s apart. */
function averagedReading(element, read, every) {
  const n = averageOf(element);
  if (n <= 1) return read;
  return (s) => {
    const values = [];
    for (let k = 0; k < n; k++) {
      const v = read(s - k * every);
      if (!Number.isFinite(v)) break;
      values.push(v);
    }
    return values.length ? values.reduce((a, b) => a + b, 0) / values.length : read(s);
  };
}

/**
 * The demo's rolling number of `element` at `t` seconds, like the backend's
 * `Roll`: `{from, to, progress}` while a `roll` transition moves, else null.
 */
export function demoRoll(element, read, t, refresh, motion = true) {
  if (element?.valueTransition?.mode !== 'roll' || !motion) return null;
  const length = transitionLength(element);
  if (length === null) return null;
  const every = refresh > 0 ? refresh : 1;
  const fractionAt = averagedReading(element, read, every);
  const sampled = Math.floor(t / every) * every;
  const since = sinceSample(t, every);
  const from = fractionAt(sampled - every), to = fractionAt(sampled);
  if (since >= length || from === to || !Number.isFinite(from) || !Number.isFinite(to)) return null;
  return { from, to, progress: ease(since / length) };
}

/**
 * The wheels of the `digits` digit columns of a number with `decimals`
 * decimals, left to right, like the backend's `Roll::wheels`: `[digit,
 * turn]`, the units column turning evenly from the rounded old number to the
 * rounded new one and a column turning only while every column to its right
 * shows 9 (an odometer).
 */
export function rollWheels({ from, to, progress }, decimals, digits) {
  const scale = 10 ** Math.min(9, decimals);
  const units = (v) => Number.isFinite(v) ? Math.round(Math.abs(v) * scale) : 0;
  const a = units(from), b = units(to);
  const at = a + (b - a) * Math.min(1, Math.max(0, progress));
  const out = [];
  let turn = at - Math.trunc(at), column = Math.trunc(at);
  for (let k = 0; k < digits; k++) {
    const digit = column % 10;
    out.push([digit, turn]);
    if (digit !== 9) turn = 0;
    column = Math.trunc(column / 10);
  }
  return out.reverse();
}

/**
 * When the demo's next sample is due, ms from `t` seconds, for a preview
 * that eases a shown element (a transition starts there), like the
 * backend's; null without one or with reduced motion.
 * @param {(e: object) => boolean} shown whether the element is drawn
 */
export function demoSampleDue(elements, t, refresh, { motion = true, shown = () => true } = {}) {
  if (!motion || !elements.some((e) => easesValue(e) && shown(e) && transitionLength(e) !== null)) return null;
  const every = refresh > 0 ? refresh : 1;
  return Math.max(1, every * 1000 - sinceSample(t, every));
}

/**
 * When the demo's next frame is due for its moving values, ms from `t`
 * seconds: a step later (or when the last one lands, if sooner) while a
 * shown element eases; null when none moves or motion is reduced.
 * @param {(e: object) => boolean} shown whether the element is drawn
 */
export function demoTransitionDue(elements, t, refresh, { motion = true, shown = () => true } = {}) {
  if (!motion) return null;
  const since = sinceSample(t, refresh > 0 ? refresh : 1);
  const left = elements
    .filter((e) => easesValue(e) && shown(e))
    .map(transitionLength)
    .filter((length) => length !== null && since < length)
    .map((length) => length - since);
  if (!left.length) return null;
  return Math.max(1, Math.min(TRANSITION_STEP_MS, ...left));
}
