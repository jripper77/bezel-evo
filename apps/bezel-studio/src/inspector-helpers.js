// The inspector's forms without the DOM: what the head says of an object, the
// theme refresh choices, the graph's timing and quick durations, the ring's
// arc drawing and the player's app chips.

/**
 * What the inspector's head says of `element`: its level among the theme's
 * objects (1 = the one at the back) out of how many, and the group or card
 * it belongs to, when it does.
 * @returns {{level: number, total: number, group: string|null}}
 */
export function layerFacts(theme, element) {
  const total = theme.elements.length;
  const level = theme.elements.findIndex((e) => e.id === element.id) + 1;
  const parentId = element.groupParent ?? element.cardMember?.parent ?? null;
  const parent = parentId === null ? null : theme.elements.find((e) => e.id === parentId);
  return { level, total, group: parent?.name ?? null };
}

/** The graph's quick durations, seconds (artboard GraphData). */
export const QUICK_DURATIONS = Object.freeze([30, 60, 300, 900, 3600]);

/** The theme's quick refresh values, seconds (D-2026-10-16-studio-redesign-fidelity-25). */
export const QUICK_REFRESHES = Object.freeze([0.5, 1, 2, 5]);

/**
 * The refresh values offered for a theme refreshing every `current`
 * seconds: the quick ones the backend allows (`min` and up), and the
 * theme's own value in its place when it is another, so none saved is lost.
 * @returns {{seconds: number, pressed: boolean}[]}
 */
export function refreshChoices(current, min = 0) {
  const values = QUICK_REFRESHES.filter((s) => s >= min);
  if (Number.isFinite(current) && current > 0 && !values.includes(current)) values.push(current);
  return values.sort((a, b) => a - b).map((seconds) => ({ seconds, pressed: seconds === current }));
}

/** The fewest and most points a graph draws (the backend's GRAPH_POINTS_MIN/MAX). */
export const HISTORY_MIN = 2;
export const HISTORY_MAX = 3600;

/** The longest time between two points of a graph, seconds (the backend's GRAPH_INTERVAL_MAX_SECONDS). */
export const GRAPH_INTERVAL_MAX = 3600;

/** Seconds in each unit a graph's duration (s, min, h) or interval (s, min) is written in. */
export const TIME_UNITS = Object.freeze({ s: 1, min: 60, h: 3600 });

/**
 * A graph's timing (D-2026-10-15-studio-redesign-fidelity-16): a point
 * every `interval` seconds, `points` of them, over `duration` seconds.
 * `own` says the graph has its own interval; without one (themes made
 * before it) it takes a point per refresh of the theme, as the backend does.
 * @param {{history: number, intervalSeconds?: number|null}} kind
 * @param {number} refreshSeconds
 * @returns {{interval: number, own: boolean, points: number, duration: number}}
 */
export function graphTiming(kind, refreshSeconds) {
  const own = Number.isInteger(kind.intervalSeconds) && kind.intervalSeconds > 0;
  const interval = own ? Math.min(kind.intervalSeconds, GRAPH_INTERVAL_MAX) : refreshSeconds > 0 ? refreshSeconds : 1;
  return { interval, own, points: kind.history, duration: kind.history * interval };
}

/**
 * The graph fields for a window of `duration` seconds with a point every
 * `interval` seconds: the interval in whole seconds (1 s to an hour), the
 * points it takes ({@link HISTORY_MIN} to {@link HISTORY_MAX}) and the
 * window they really cover, with `clamped` saying when the points were
 * bounded (`min`, `max`; null when not).
 * @returns {{history: number, intervalSeconds: number, duration: number, clamped: 'min'|'max'|null}}
 */
export function graphTimingPatch(duration, interval) {
  const every = Math.min(GRAPH_INTERVAL_MAX, Math.max(1, Math.round(Number(interval) || 1)));
  const wanted = Math.round((Number(duration) || 0) / every);
  const history = Math.min(HISTORY_MAX, Math.max(HISTORY_MIN, wanted));
  const clamped = wanted < HISTORY_MIN ? 'min' : wanted > HISTORY_MAX ? 'max' : null;
  return { history, intervalSeconds: every, duration: history * every, clamped };
}

/** The shortest and longest window a graph with a point every `interval` seconds can show, seconds. */
export function graphWindowRange(interval) {
  const every = Math.min(GRAPH_INTERVAL_MAX, Math.max(1, Math.round(Number(interval) || 1)));
  return { min: HISTORY_MIN * every, max: HISTORY_MAX * every };
}

/** An interval as its largest whole unit, seconds or minutes: `{value, unit}`. */
export function intervalParts(seconds) {
  if (seconds >= 60 && seconds % 60 === 0) return { value: seconds / 60, unit: 'min' };
  return { value: Math.round(seconds * 100) / 100, unit: 's' };
}

/** The units a graph's duration and the time between two of its points are written in. */
export const DURATION_UNITS = Object.freeze(['s', 'min', 'h']);
export const INTERVAL_UNITS = Object.freeze(['s', 'min']);

/** The grammatical number of a count for the words around it: `one` for exactly 1, else `other` (en, pt-BR, it). */
export const pluralOf = (value) => (value === 1 ? 'one' : 'other');

/**
 * The slots of a graph's window sentence (D-2026-10-15-studio-redesign-fidelity-17,
 * "Show the last [1] [hour], with one point every [30] [seconds]"): the
 * duration and the interval, each in its largest whole unit offered
 * (seconds when none fits) with its grammatical number, and the key of
 * the word before the duration, which agrees with its unit (hours are
 * feminine in Portuguese and Italian) and its number.
 * @param {{duration: number, interval: number}} timing see {@link graphTiming}
 * @returns {{last: string, duration: {value: number, unit: string, plural: string}, interval: {value: number, unit: string, plural: string}}}
 */
export function graphSentence(timing) {
  const shown = (seconds, units, parts) => {
    const whole = parts(seconds);
    const { value, unit } = units.includes(whole.unit) ? whole : { value: seconds, unit: 's' };
    return { value, unit, plural: pluralOf(value) };
  };
  const duration = shown(timing.duration, DURATION_UNITS, durationParts);
  const interval = shown(timing.interval, INTERVAL_UNITS, intervalParts);
  const hours = duration.unit === 'h';
  const last = duration.plural === 'one' ? (hours ? 'insp.lastHour' : 'insp.lastOne') : (hours ? 'insp.lastHours' : 'insp.lastMany');
  return { last, duration, interval };
}

/** The most tick marks a graph's time ruler draws before it thins them out. */
export const RULER_TICKS_MAX = 60;

/**
 * The tick marks of a graph's time ruler, from −duration (0) to now (1):
 * where its `points` fall, the newest at now and one per interval before
 * it; past `max` marks only one point in `every` is drawn, counted from now.
 * @returns {{ticks: number[], every: number}} positions ascending, 0..1
 */
export function graphRuler(points, max = RULER_TICKS_MAX) {
  const count = Math.max(1, Math.round(Number(points) || 1));
  const every = Math.ceil(count / Math.max(1, max));
  const ticks = [];
  for (let back = 0; back < count; back += every) ticks.unshift(Math.round((1 - back / count) * 10000) / 10000);
  return { ticks, every };
}

/**
 * A duration as its largest whole unit: `{value, unit}` with `unit` one of
 * `s`, `min`, `h` (90 s stays `90 s`).
 */
export function durationParts(seconds) {
  if (seconds >= 3600 && seconds % 3600 === 0) return { value: seconds / 3600, unit: 'h' };
  if (seconds >= 60 && seconds % 60 === 0) return { value: seconds / 60, unit: 'min' };
  return { value: Math.round(seconds * 100) / 100, unit: 's' };
}

/**
 * The quick durations of a graph taking a point every `intervalSeconds`
 * and drawing `history` of them: how many points each takes, whether that
 * is the graph's window now, and whether the graph can draw that many.
 * @returns {{seconds: number, samples: number, pressed: boolean, disabled: boolean}[]}
 */
export function quickDurations(intervalSeconds, history) {
  const interval = intervalSeconds > 0 ? intervalSeconds : 1;
  return QUICK_DURATIONS.map((seconds) => {
    const samples = Math.round(seconds / interval);
    return { seconds, samples, pressed: samples === history && samples * interval === seconds, disabled: samples < HISTORY_MIN || samples > HISTORY_MAX };
  });
}

/**
 * The ring's arc drawn small (artboard Main, "Arco"): an SVG path on a
 * `size` square, angles as the renderer reads them (0° at twelve o'clock,
 * clockwise), and where it starts (the handle). A sweep of a whole turn or
 * more is drawn as a full circle.
 * @returns {{d: string, start: {x: number, y: number}, full: boolean}}
 */
export function arcDrawing(startAngle, sweep, clockwise = true, { size = 64, radius = 24 } = {}) {
  const c = size / 2;
  const point = (degrees) => {
    const a = ((degrees - 90) * Math.PI) / 180;
    return { x: round2(c + radius * Math.cos(a)), y: round2(c + radius * Math.sin(a)) };
  };
  const span = Math.min(Math.abs(Number(sweep) || 0), 360);
  const start = point(startAngle);
  if (span >= 360) {
    const opposite = point(startAngle + 180);
    return { d: `M${start.x} ${start.y}A${radius} ${radius} 0 1 1 ${opposite.x} ${opposite.y}A${radius} ${radius} 0 1 1 ${start.x} ${start.y}`, start, full: true };
  }
  const end = point(startAngle + (clockwise ? span : -span));
  const large = span > 180 ? 1 : 0;
  return { d: `M${start.x} ${start.y}A${radius} ${radius} 0 ${large} ${clockwise ? 1 : 0} ${end.x} ${end.y}`, start, full: false };
}

const round2 = (n) => Math.round(n * 100) / 100;

/** An application's name as people read it: no `.exe`, no folder. */
export const appName = (source) => String(source ?? '').split(/[\\/]/).pop().replace(/\.exe$/i, '').trim();

const sameApp = (a, b) => appName(a).toLowerCase() === appName(b).toLowerCase();

/**
 * The player's app chips (artboard PlayerData) from the last sample's
 * `activity`: one per application with a media session (playing ones
 * first, then by name), the one the player follows pressed. `custom` is
 * the source the player follows when no chip names it (typed by hand).
 * @param {{media?: {source: string, playing: boolean}[]}|null} activity
 * @param {string} source the player's `source` (empty: automatic)
 * @returns {{chips: {name: string, playing: boolean, pressed: boolean}[], custom: string|null}}
 */
export function appChips(activity, source = '') {
  const chips = [];
  for (const session of activity?.media ?? []) {
    const name = appName(session.source);
    if (!name) continue;
    const known = chips.find((c) => sameApp(c.name, name));
    if (known) known.playing ||= Boolean(session.playing);
    else chips.push({ name, playing: Boolean(session.playing), pressed: false });
  }
  chips.sort((a, b) => Number(b.playing) - Number(a.playing) || a.name.localeCompare(b.name));
  const wanted = String(source ?? '').trim();
  for (const chip of chips) chip.pressed = wanted !== '' && sameApp(chip.name, wanted);
  return { chips, custom: wanted !== '' && !chips.some((c) => c.pressed) ? wanted : null };
}
