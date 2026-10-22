// A card's trigger rules as the sentences the Triggers tab shows ("When an
// app is in front <app> show <face>"), and the edits on them. Pure: no DOM,
// no store. The theme's rule is `{source, app, face, priority,
// returnSeconds, returnFace}` (bezel-themes validates the same bounds).

/** The conditions a rule can wait for, in the order the sentence offers them. */
export const TRIGGER_SOURCES = Object.freeze(['foreground', 'process', 'processClosed', 'mediaPlaying', 'timer']);

/** The rules a card can have, and the longest app name. */
export const MAX_RULES = 33; // A legacy Card can have 32 app rules plus rotation.
export const MAX_APP = 160;

/** The quick choices of the rotation interval, seconds. */
export const ROTATION_PRESETS = Object.freeze([5, 10, 15, 30, 60]);

/** The bounds of the rotation interval, seconds. */
export const ROTATION_MIN = 5;
export const ROTATION_MAX = 3600;

/**
 * The app a rule falls back to when it needs one (every condition but
 * music) and the PC reported none yet: the default rules always had.
 */
export const FALLBACK_APP = 'Spotify';

const clamp = (value, min, max) => Math.max(min, Math.min(max, Math.round(Number(value) || 0)));

/** A rule's priority within 0–100. */
export const clampPriority = (value) => clamp(value, 0, 100);

/** A rule's wait before returning, within 0–300 s. */
export const clampReturn = (value) => clamp(value, 0, 300);

/** A rotation interval within 5–3600 s. */
export const clampRotation = (value) => clamp(value, ROTATION_MIN, ROTATION_MAX);

/** Whether a condition is about music or a video (any app may play it: the app is optional). */
export const isMedia = (source) => source === 'mediaPlaying';

/**
 * The parts of a rule's sentence, in order: words (`when`, `in`, `show`)
 * and the tokens the user picks (`source`, `app`, `face`).
 * @returns {{word?: 'when'|'in'|'show', token?: 'source'|'app'|'face'}[]}
 */
export function sentenceOf(rule) {
  if (rule.source === 'timer') return [{ token: 'source' }, { token: 'interval' }, { token: 'unit' }, { word: 'thenShow' }, { token: 'face' }];
  return [
    { word: 'when' }, { token: 'source' },
    ...(isMedia(rule.source) ? [{ word: 'in' }] : []),
    { token: 'app' },
    { word: 'show' }, { token: 'face' },
  ];
}

/**
 * The apps a rule can name, from the last sample's activity: the one in
 * front first, then the running ones (music: the media sessions' sources),
 * the rule's own app kept; no duplicates, at most `limit`.
 * @param {{foreground?: string|null, applications?: string[], media?: {source: string}[]}|null} activity
 */
export function appSuggestions(activity, source, current = '', limit = 40) {
  const names = isMedia(source)
    ? (activity?.media ?? []).map((m) => m.source)
    : [activity?.foreground, ...(activity?.applications ?? [])];
  const seen = new Set();
  const out = [];
  for (const name of [current, ...names]) {
    const clean = typeof name === 'string' ? name.trim() : '';
    const key = clean.toLowerCase();
    if (!clean || seen.has(key) || out.length >= limit) continue;
    seen.add(key);
    out.push(clean);
  }
  return out;
}

/** The app a rule that needs one gets: its own, the one in front, the first running, or the fallback. */
function appFor(rule, activity) {
  if (rule.app?.trim()) return rule.app;
  return activity?.foreground || activity?.applications?.[0] || activity?.media?.[0]?.source || FALLBACK_APP;
}

/** A rule waiting for another condition; one that needs an app gets one. */
export function withSource(rule, source, activity = null) {
  if (!TRIGGER_SOURCES.includes(source)) return rule;
  if (source === 'timer') return { ...rule, source, app: '', intervalSeconds: rule.intervalSeconds ?? 10, nextFace: rule.nextFace ?? true, returnSeconds: 0, returnFace: null };
  const { intervalSeconds, nextFace, ...rest } = rule;
  rule = rest;
  return { ...rule, source, app: isMedia(source) ? rule.app ?? '' : appFor(rule, activity) };
}

/** A rule naming another app: trimmed to 160 characters; empty only for music. */
export function withApp(rule, app) {
  const name = Array.from(String(app ?? '').trim()).slice(0, MAX_APP).join('');
  if (!name && !isMedia(rule.source)) return rule;
  return { ...rule, app: name };
}

/** What every rule returns to: `'previous'`, a face, `'mixed'` when they differ, null without rules. */
export function commonReturn(rules) {
  rules = rules.filter(r => r.source !== 'timer');
  if (!rules.length) return null;
  const values = new Set(rules.map((r) => (r.returnFace == null ? 'previous' : r.returnFace)));
  return values.size === 1 ? [...values][0] : 'mixed';
}

/** Every rule returning to `value` (`'previous'` or a face index). */
export function withReturn(rules, value) {
  const returnFace = value === 'previous' ? null : value;
  return rules.map((r) => r.source === 'timer' ? r : ({ ...r, returnFace }));
}

/** A new rule: music in any app shows the face in view, returning like the others. */
export function newRule(card) {
  const back = commonReturn(card.triggers ?? []);
  return {
    source: 'mediaPlaying', app: '', face: card.activeFace, priority: 0, returnSeconds: 3,
    returnFace: typeof back === 'number' ? back : null,
  };
}

/** The rules with the one at `from` moved to `to` (same list when either is out of range). */
export function moveRule(rules, from, to) {
  if (from === to || from < 0 || to < 0 || from >= rules.length || to >= rules.length) return rules;
  const next = [...rules];
  const [rule] = next.splice(from, 1);
  next.splice(to, 0, rule);
  return next;
}

/** Where a dragged rule lands: the index of the first rule whose middle is below `y`, among `mids` (the others' middles, top to bottom). */
export function dropIndex(mids, y) {
  const i = mids.findIndex((mid) => y < mid);
  return i === -1 ? mids.length : i;
}

// Rotation presets and graph windows share the same duration formatting.
export { durationParts } from './inspector-helpers.js';

/** Timed rules keep a baseline face; app overrides pause their schedule. */
export function newTimedRule(card) { return withSource(newRule(card), 'timer'); }
export function timedParts(rule) {
  const seconds = clampRotation(rule.intervalSeconds);
  return seconds >= 60 && seconds % 60 === 0 ? { value: seconds / 60, unit: 'min' } : { value: seconds, unit: 's' };
}
export function timedSeconds(value, unit) { return clampRotation(Number(value) * (unit === 'min' ? 60 : 1)); }

/** Matching of the demo's measured desktop activity, like the core. */
export function triggerMatches(rule, activity) {
  const name = value => String(value ?? '').trim().toLowerCase().replace(/\.exe$/, '');
  const app = name(rule.app);
  if (rule.source === 'mediaPlaying') return (activity?.media ?? []).some(m => m.playing && (!app || m.source.toLowerCase().includes(app)));
  if (rule.source === 'foreground') return !!activity?.foreground && name(activity.foreground) === app;
  if (!activity?.available || !['process','processClosed'].includes(rule.source)) return false;
  const exists = (activity.applications ?? []).some(a => name(a) === app);
  return rule.source === 'process' ? exists : !exists;
}
