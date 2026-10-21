// The profile manager's choices, apart from the dialog: who decides what the
// screens show, how a profile's rules read, and what removing one changes.
// The saved configuration keeps its shape (automatic, manual, defaultProfile,
// profiles[].rules[]); only how the dialog presents it lives here.

/** Covers painted over the whole screen (crates/bezel-core/src/domain/cover.rs). */
export const PROFILE_EFFECTS = Object.freeze(['none', 'curtain', 'wipe', 'blinds', 'mosaic']);

/** The cover effects that move in a direction (the mosaic has none). */
export const directed = (effect) => effect !== 'none' && effect !== 'mosaic';

export const PROFILE_RULE_SOURCES = Object.freeze(['mediaPlaying', 'foreground', 'process', 'sensor', 'time', 'idle', 'locked']);
/** A fresh rule; changing its source keeps priority and delay in the caller. */
export function newProfileRule(source, catalog = []) {
  const rule = { source, app: '', priority: 10, returnSeconds: 3 };
  if (source === 'sensor') Object.assign(rule, { key: catalog.find(s => s.quantity !== 'text')?.key ?? '', compare: 'above', value: 70, forSeconds: 0 });
  if (source === 'time') Object.assign(rule, { days: [], from: '09:00', to: '18:00' });
  if (source === 'idle') rule.minutes = 5;
  return rule;
}
const within = (value, max, min = 0) => Number.isInteger(value) && value >= min && value <= max;
export function minuteOf(text) {
  if (typeof text !== 'string' || !/^\d{2}:\d{2}$/.test(text)) return null;
  const [hour, minute] = text.split(':').map(Number);
  return hour < 24 && minute < 60 ? hour * 60 + minute : null;
}

/** The same bounds and source-specific requirements as runtime::profiles::Rule. */
export function validProfileRule(rule) {
  const app = rule.app === undefined ? '' : rule.app;
  const days = rule.days === undefined ? [] : rule.days;
  if (!PROFILE_RULE_SOURCES.includes(rule.source) || !within(rule.priority, 100) || !within(rule.returnSeconds, 300)
    || typeof app !== 'string' || app.length > 256) return false;
  switch (rule.source) {
    case 'foreground': case 'process': return Boolean(rule.app?.trim());
    case 'sensor': return typeof rule.key === 'string' && rule.key.length > 0 && !/\s/.test(rule.key)
      && ['above', 'below'].includes(rule.compare) && Number.isFinite(rule.value) && within(rule.forSeconds === undefined ? 0 : rule.forSeconds, 3600);
    case 'time': return Array.isArray(days) && days.length <= 7
      && days.every(day => within(day, 6)) && minuteOf(rule.from) !== null && minuteOf(rule.to) !== null;
    case 'idle': return within(rule.minutes ?? 0, 1440, 1);
    default: return true;
  }
}

/**
 * Who decides: `auto` (the rules, `fallback` when none holds) or `manual`
 * (always `manual`). A `null` profile is the base themes.
 * @returns {{mode: 'auto'|'manual', manual: string|null, fallback: string|null}}
 */
export function modeOf(config) {
  return {
    mode: config.automatic ? 'auto' : 'manual',
    manual: config.automatic ? null : config.manual ?? null,
    fallback: config.defaultProfile ?? null,
  };
}

/**
 * The configuration with `choice` applied: Automatic keeps its fallback and
 * drops the manual pick; Manual with no profile is the former Default.
 * @param {{mode: 'auto'|'manual', manual?: string|null, fallback?: string|null}} choice
 */
export function withMode(config, choice) {
  const known = (id) => (id && config.profiles.some((p) => p.id === id) ? id : null);
  const automatic = choice.mode === 'auto';
  return {
    ...config,
    automatic,
    manual: automatic ? null : known(choice.manual ?? null),
    defaultProfile: known(choice.fallback === undefined ? config.defaultProfile : choice.fallback),
  };
}

/** The configuration without the profile `id`, no choice pointing at it. */
export function withoutProfile(config, id) {
  return {
    ...config,
    profiles: config.profiles.filter((p) => p.id !== id),
    manual: config.manual === id ? null : config.manual,
    defaultProfile: config.defaultProfile === id ? null : config.defaultProfile,
  };
}

/**
 * The first profile the runtime would refuse, so the dialog can show it
 * before saving: no name, no screen theme assigned, a rule that
 * watches no app or with a priority or delay out of range. `null` when all fit.
 */
export function firstIncomplete(config, displays) {
  const bad = config.profiles.find((profile) => !profile.name?.trim()
    || !Object.values(profile.themes).some(Boolean)
    || !['any', 'all'].includes(profile.match === undefined ? 'any' : profile.match)
    || profile.rules.some(rule => !validProfileRule(rule)));
  return bad?.id ?? null;
}

/**
 * The themes ▶ tries, by display key: the selected profile's (saved or not),
 * or the base themes when the base item is selected (`id` not a profile).
 */
export function triedThemes(config, id, displays) {
  const profile = config.profiles.find((p) => p.id === id);
  const entries = profile
    ? Object.entries(profile.themes)
    : displays.map((d) => [d.key, d.theme]);
  return Object.fromEntries(entries.filter(([, path]) => path));
}

/** One rule in plain words: "when music is playing", "when game.exe is in front". */
export function ruleSummary(t, rule) {
  const app = rule.app?.trim();
  if (rule.source === 'sensor') return t('profileRules.sensor', { key: rule.key, comparison: t(`profileRules.${rule.compare}`), value: rule.value, seconds: rule.forSeconds ?? 0 });
  if (rule.source === 'time') return t('profileRules.time', { days: rule.days?.length ? rule.days.map(day => t(`profileRules.day${day}`)).join(', ') : t('profileRules.everyDay'), from: rule.from, to: rule.to });
  if (rule.source === 'idle') return t('profileRules.idle', { minutes: rule.minutes });
  if (rule.source === 'locked') return t('profileRules.locked');
  if (rule.source === 'mediaPlaying') {
    return app ? t('profiles.summary.mediaIn', { app }) : t('profiles.summary.media');
  }
  return t(rule.source === 'foreground' ? 'profiles.summary.foreground' : 'profiles.summary.process', { app: app || '…' });
}

/** What the list says under a profile: its first rule, and how many more. */
export function profileSummary(t, profile) {
  const [first, ...rest] = profile.rules;
  if (!first) return t('profiles.summary.none');
  const text = ruleSummary(t, first);
  return rest.length ? `${text} · ${t('profiles.summary.more', { count: rest.length })}` : text;
}

/** The icon of a profile: what its first rule watches. */
export function profileIcon(profile) {
  const source = profile.rules[0]?.source;
  if (source === 'mediaPlaying') return 'player';
  if (source === 'foreground' || source === 'process') return 'chip';
  if (source === 'sensor') return 'sliders';
  if (source === 'time' || source === 'idle') return 'clock';
  if (source === 'locked') return 'lock';
  return 'themes';
}

/** `landscape` or `portrait`, from a theme's canvas (portrait when unknown). */
export function orientationOf(theme) {
  return theme?.canvas && theme.canvas.width > theme.canvas.height ? 'landscape' : 'portrait';
}
