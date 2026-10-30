import { WEATHER_CONDITIONS } from './i18n/weather.js';
// Localised weather text shared by the demo renderer and its examples.
export function weatherFamily(code) {
  if (code === 0) return 0;
  if (code === 1 || code === 2) return 1;
  if (code === 3) return 2;
  if (code === 45 || code === 48) return 3;
  if ((code >= 51 && code <= 67) || (code >= 80 && code <= 82)) return 4;
  if ((code >= 71 && code <= 77) || code === 85 || code === 86) return 5;
  if (code >= 95 && code <= 99) return 6;
  return 7;
}
/** City, temperature and condition one per line: the layout of a weather without one. */
export const WEATHER_LAYOUT = Object.freeze(['city', 'temperature', 'condition'].map(field => Object.freeze({ field })));
/** The layout of `content`, each field once in its order. */
export const weatherLayout = (content) => (content.layout?.length === 3 ? content.layout : WEATHER_LAYOUT);
/** The layout to save: none for city, temperature and condition one per line. */
export function layoutPatch(lines) {
  const plain = lines.every((line, i) => line.field === WEATHER_LAYOUT[i].field && !line.hidden && !line.inline);
  return plain ? null : lines.map(({ field, hidden, inline }) => ({ field, ...(hidden ? { hidden: true } : {}), ...(inline ? { inline: true } : {}) }));
}

/** `lines` with the one at `from` moved to `to`. */
export function moveLine(lines, from, to) {
  const next = [...lines];
  next.splice(to, 0, ...next.splice(from, 1));
  return next;
}
/**
 * The weather text in segments like the backend's `Weather::segments`: each
 * shown field `{ field, text }`, and between two of them `{ field: null }`
 * with a new line or, inline, a space.
 */
export function weatherSegments(content, temperature, code, system = globalThis.navigator?.languages?.[0] ?? 'en') {
  const language = (content.language || system).toLowerCase();
  const labels = WEATHER_CONDITIONS[language.startsWith('it') ? 'it' : language.startsWith('pt') ? 'pt-BR' : 'en'];
  const value = content.fahrenheit ? temperature * 1.8 + 32 : temperature;
  const words = {
    city: content.label?.trim() || content.city,
    temperature: `${Math.round(value)}\u00b0${content.fahrenheit ? 'F' : 'C'}`,
    condition: labels[weatherFamily(code)],
  };
  const out = [];
  for (const line of weatherLayout(content).filter(line => !line.hidden)) {
    if (out.length) out.push({ field: null, text: line.inline ? ' ' : '\n' });
    out.push({ field: line.field, text: words[line.field] });
  }
  return out;
}
export const weatherText = (...args) => weatherSegments(...args).map(segment => segment.text).join('');
/** The sensor keys of `content`'s temperature and condition code, like the backend's `Weather::keys`. */
export function weatherKeys(content) {
  const base = `weather.${Number(content.latitude).toFixed(6)}:${Number(content.longitude).toFixed(6)}`;
  return [`${base}.temperature`, `${base}.code`];
}
