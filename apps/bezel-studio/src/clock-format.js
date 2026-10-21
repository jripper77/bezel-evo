// Mirrors the native clock formatter for inspector examples and browser previews.
export const CLOCK_PATTERNS = ['%H:%M', '%H:%M:%S', '%I:%M %p', '%d/%m/%Y', '%A %e %B', '%A', '%a %d %b', '%B %Y'];

/** Directives that are a part of the time (D-2026-10-20-studio-ux-v2-06). */
const TIME_PARTS = { H: 'hours', I: 'hours', M: 'minutes', S: 'seconds' };

/** The pieces of a clock's text: `{ kind: 'literal'|'time'|'other', part?, text }`. */
function pieces(pattern, date, language, system) {
  const chosen = (language || system).toLowerCase();
  const locale = chosen.startsWith('it') ? 'it' : chosen.startsWith('pt') ? 'pt-BR' : 'en';
  const day = new Intl.DateTimeFormat(locale, { weekday: 'long' }).format(date);
  const month = new Intl.DateTimeFormat(locale, { month: 'long' }).format(date);
  const two = (n) => String(n).padStart(2, '0');
  const short = (word) => [...word].slice(0, 3).join('');
  const values = { H: two(date.getHours()), I: two(date.getHours() % 12 || 12), p: date.getHours() < 12 ? 'AM' : 'PM', M: two(date.getMinutes()), S: two(date.getSeconds()), d: two(date.getDate()), e: String(date.getDate()), m: two(date.getMonth() + 1), Y: String(date.getFullYear()), y: two(((date.getFullYear() % 100) + 100) % 100), A: day, a: short(day), B: month, b: short(month), '%': '%' };
  const out = [];
  const literal = (text) => {
    const last = out[out.length - 1];
    if (last?.kind === 'literal') last.text += text; else out.push({ kind: 'literal', text });
  };
  for (let i = 0; i < pattern.length; i++) {
    if (pattern[i] !== '%') { literal(pattern[i]); continue; }
    if (i + 1 >= pattern.length) { literal('%'); break; }
    const code = pattern[++i];
    if (TIME_PARTS[code]) out.push({ kind: 'time', part: TIME_PARTS[code], text: values[code] });
    else out.push({ kind: 'other', text: values[code] ?? `%${code}` });
  }
  return out;
}

function applyCase(text, casing) {
  if (casing === 'upper') return text.toUpperCase();
  if (casing === 'title') return text.replace(/(^|[^\p{L}])(\p{L})/gu, (_, prefix, letter) => prefix + letter.toUpperCase());
  return text;
}

export function formatClock(pattern, date = new Date(), language = null, casing = 'normal', system = globalThis.navigator?.languages?.[0] ?? 'en') {
  return applyCase(pieces(pattern, date, language, system).map((p) => p.text).join(''), casing);
}

/**
 * The clock's text cut into its parts, like the native `clock_parts`:
 * `hours`, `minutes`, `seconds`, `separators` (fixed text between two of
 * those) and `null` for the rest, neighbours of the same part joined.
 * @returns {{part: string|null, text: string}[]}
 */
export function clockParts(pattern, date = new Date(), language = null, casing = 'normal', system = globalThis.navigator?.languages?.[0] ?? 'en') {
  const list = pieces(pattern, date, language, system);
  const out = [];
  list.forEach((piece, index) => {
    let part = piece.kind === 'time' ? piece.part : null;
    if (piece.kind === 'literal' && list.slice(0, index).some((p) => p.kind === 'time') && list[index + 1]?.kind === 'time') part = 'separators';
    const text = applyCase(piece.text, casing);
    const last = out[out.length - 1];
    if (last && last.part === part) last.text += text; else out.push({ part, text });
  });
  return out;
}

/** Original tip lengths in the native renderer's 64-unit dial. */
export const HAND_LENGTHS = Object.freeze({ hour: 11.7, minute: 20.9, second: 23 });

/** Relative hand size, bounded like AnalogDto; the original size is never saved. */
export function withHandLength(analog, hand, value) {
  const next = { ...analog }, key = `${hand}Length`;
  const length = Number.isFinite(value) ? Math.max(.1, Math.min(2, value)) : 1;
  if (length === 1) delete next[key]; else next[key] = length;
  return next;
}

/**
 * An analog clock's settings with every absent field at the backend's
 * default (D-2026-10-20-studio-ux-v2-07): hour marks, no numbers, three
 * rounded hands, a 2 px outline, the design's colours, stepping seconds,
 * no date.
 */
export function analogOf(analog) {
  const result = {
    ticks: 'hours', numbers: 'none', hourHand: true, minuteHand: true, secondHand: true, handShape: 'rounded', border: 2,
    face: '#0b0d0fff', marks: '#aeb5bdff', hourColor: '#e8eaedff', minuteColor: '#e8eaedff', secondColor: '#ff9248ff',
    sweep: false, date: 'none', ...(analog ?? {}),
  };
  for (const hand of Object.keys(HAND_LENGTHS)) {
    const key = `${hand}Length`, value = result[key];
    result[key] = Number.isFinite(value) ? Math.max(.1, Math.min(2, value)) : 1;
  }
  return result;
}

/** Where the hands point at `date`, degrees clockwise from 12, like the backend's; `sweep` (with motion) moves the seconds between seconds. */
export function handAngles(analog, date, sweep = true) {
  const a = analogOf(analog);
  const second = date.getSeconds() + (sweep && a.sweep ? date.getMilliseconds() / 1000 : 0);
  const minute = date.getMinutes() + second / 60;
  const hour = (date.getHours() % 12) + minute / 60;
  return { hour: hour * 30, minute: minute * 6, second: second * 6 };
}

/** When the dial next changes, ms from `date`: each second with a second hand, 33 ms while it sweeps; null without. */
export function dialDue(analog, date, sweep = true) {
  const a = analogOf(analog);
  if (!a.secondHand) return null;
  return sweep && a.sweep ? 33 : 1000 - date.getMilliseconds();
}

const ROMAN = ['I', 'II', 'III', 'IV', 'V', 'VI', 'VII', 'VIII', 'IX', 'X', 'XI', 'XII'];

/** The numbers on the dial: `[hour, text]`. */
export function dialNumerals(analog) {
  const { numbers } = analogOf(analog);
  const hours = Array.from({ length: 12 }, (_, i) => i + 1);
  if (numbers === 'quarters') return hours.filter((h) => h % 3 === 0).map((h) => [h, String(h)]);
  if (numbers === 'all') return hours.map((h) => [h, String(h)]);
  if (numbers === 'roman') return hours.map((h) => [h, ROMAN[h - 1]]);
  return [];
}

/** The date written on the dial, like the backend's: the day, or the day and the short month. */
export function dialDate(analog, date, language = null) {
  const { date: kind } = analogOf(analog);
  if (kind === 'day') return formatClock('%e', date, language);
  if (kind === 'dayMonth') return formatClock('%e %b', date, language);
  return '';
}
