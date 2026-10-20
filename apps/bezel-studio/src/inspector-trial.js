// The inspector's transient Try clock. Same timings and inputs as
// bezel-runtime::inspector_trial; nothing here is written into the document.
export const TRIAL_MS = 3000;
export const WEATHER_TRIAL_MS = 8000;
export const TRIAL_FRAME_MS = 33;
export const TRIAL_WEATHER_CODES = Object.freeze([0, 2, 3, 45, 61, 71, 95, 999]);

export const trialDuration = element => element?.kind?.content?.type === 'weather' ? WEATHER_TRIAL_MS : TRIAL_MS;

export function trialAt(element, elapsedMs, motion = true) {
  if (!element || !motion || !Number.isInteger(elapsedMs) || elapsedMs < 0 || elapsedMs >= trialDuration(element)) return null;
  return {
    elementId: element.id,
    elapsedMs,
    pulse: (1 - Math.cos(2 * Math.PI * elapsedMs / TRIAL_MS)) / 2,
    weatherCode: TRIAL_WEATHER_CODES[Math.floor(elapsedMs / 1000)],
    clockOffsetSeconds: Math.floor(elapsedMs * 86400 / TRIAL_MS),
  };
}

/** One Try per inspector. Selection changes, completion and cancellation reset it. */
export function createInspectorTrial({ now = () => performance.now() } = {}) {
  let active = null;
  return {
    start(element) { active = { id: element.id, start: now() }; },
    stop() { active = null; },
    at(element, motion = true) {
      if (!active) return null;
      if (active.id !== element?.id) { active = null; return null; }
      const elapsedMs = Math.max(0, Math.floor(now() - active.start));
      if (!trialAt(element, elapsedMs, motion)) { active = null; return null; }
      return elapsedMs;
    },
  };
}
