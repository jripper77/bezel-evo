import profileRules, { PREFIXES as PROFILE_RULES } from './areas/profile-rules.js';
// Tiny i18n: pick a locale from the browser, translate keys with {param}
// substitution, and apply `data-i18n*` attributes to the DOM.
import ptBR from './pt-BR.js';
import enBase from './en.js';
import itBase from './it.js';
import shell, { PREFIXES as SHELL } from './areas/shell.js';
import library, { PREFIXES as LIBRARY } from './areas/library.js';
import inspector, { PREFIXES as INSPECTOR } from './areas/inspector.js';
import card, { PREFIXES as CARD } from './areas/card.js';
import screen, { PREFIXES as SCREEN } from './areas/screen.js';
import connect, { PREFIXES as CONNECT } from './areas/connect.js';
import pickers, { PREFIXES as PICKERS } from './areas/pickers.js';

// Labels derived from enum values (widget kinds, storage stages, etc.)
// resolve to the same canonical text as static callers. Only the canonical
// keys below have translations; this map contains no copies of prose.
export const SHARED_LABELS = Object.freeze({
  'insp.image.anchor.topLeft': 'prefs.debugCorner.top-left',
  'insp.image.anchor.topRight': 'prefs.debugCorner.top-right',
  'insp.image.anchor.bottomLeft': 'prefs.debugCorner.bottom-left',
  'insp.image.anchor.bottomRight': 'prefs.debugCorner.bottom-right',

  'insp.clock.ticks.none': 'fade.none',
  'insp.clock.ticks': 'insp.needle.ticks',
  'insp.clock.ticks.hours': 'insp.clockPart.hours',
  'insp.clock.handShape.arrow': 'insp.needle.shape.arrow',
  'insp.clock.marks': 'insp.needle.marks',

  'insp.needle.numbers.none': 'card.effect.none',
  'insp.needle.ticks.none': 'fade.none',
  'insp.needle.hand': 'widget.needle',
  'insp.needle.cap.ring': 'widget.ring',
  'insp.needle.track.gradient': 'insp.gradient',
  "framing.rotation": "common.rotation",
  "insp.rotation": "common.rotation",
  "storage.kindOne.image": "common.image",
  "storage.kindOne.video": "common.video",
  "inspector.direction": "common.direction",
  "inspector.alignLeft": "common.left",
  "inspector.alignRight": "common.right",
  "card.direction.left": "common.left",
  "card.direction.right": "common.right",
  "orientation.landscape": "axis.horizontal",
  "inspector.valign": "axis.vertical",
  "orientation.portrait": "axis.vertical",
  "gifs.kind.gif": "collection.filter.gif",
  "gifs.kind.sticker": "collection.filter.sticker",
  "storage.plan.action.rename": "common.rename",
  "widget.image": "common.image",
  "storage.filter.sort.name": "common.name",
  "widget.shape": "common.shape",
  "widget.text": "common.text",
  "standby.video.list": "common.video",
  "standby.choice.album": "standby.album.title",
  "standby.title": "standby.summary.atShutdown",
  "storage.protected.boot": "storage.bootTitle",
  "storage.filter.sort.kind": "storage.filter.kind",
  "storage.kind.image": "storage.filter.kind.image",
  "storage.kind.video": "storage.filter.kind.video",
  "storage.reason.live": "storage.liveReason",
  "storage.refused.invalidChar": "storage.plan.refused.invalidChar",
  "storage.reason.noCard": "storage.plan.refused.noCard",
  "storage.refused.noCard": "storage.plan.refused.noCard",
  "storage.report.stage.upload": "storage.report.stage.preflight",
  "storage.report.stage.verify": "storage.report.stage.preflight"
});
export const canonicalLabelKey = key => SHARED_LABELS[key] ?? key;

/** The texts of each area (`{en, 'pt-BR', it}`) and the key prefixes it owns. */
export const AREAS = Object.freeze({
  shell: { prefixes: SHELL, tables: shell },
  library: { prefixes: LIBRARY, tables: library },
  inspector: { prefixes: INSPECTOR, tables: inspector },
  card: { prefixes: CARD, tables: card },
  screen: { prefixes: SCREEN, tables: screen },
  connect: { prefixes: CONNECT, tables: connect },
  pickers: { prefixes: PICKERS, tables: pickers },
  profileRules: { prefixes: PROFILE_RULES, tables: profileRules },
});

/**
 * One table per locale: the base table with every area's keys added. A key
 * defined twice (in the base and an area, or in two areas) is an error.
 * @param {Record<string, Record<string, string>>} bases by locale
 * @param {Record<string, Record<string, string>>[]} areas each by locale
 */
export function mergeTables(bases, areas) {
  return Object.fromEntries(Object.entries(bases).map(([locale, base]) => {
    const table = { ...base };
    for (const area of areas) {
      for (const [key, text] of Object.entries(area[locale] ?? {})) {
        if (Object.hasOwn(table, key)) throw new Error(`i18n: ${locale} key "${key}" is defined twice`);
        table[key] = text;
      }
    }
    return [locale, Object.freeze(table)];
  }));
}

const merged = mergeTables({ 'pt-BR': ptBR, en: enBase, it: itBase }, Object.values(AREAS).map((area) => area.tables));
const en = merged.en;

export const LOCALES = Object.freeze(merged);

/**
 * The best supported locale for a list of browser languages.
 * @param {readonly string[]} languages e.g. navigator.languages
 * @returns {'pt-BR' | 'en' | 'it'}
 */
export function pickLocale(languages) {
  for (const lang of languages) {
    const lower = String(lang).toLowerCase();
    if (lower.startsWith('pt')) return 'pt-BR';
    if (lower.startsWith('en')) return 'en';
    if (lower.startsWith('it')) return 'it';
  }
  return 'en';
}

/**
 * A translator bound to a locale. Unknown keys come back as the key itself;
 * `t.has(key)` tells whether a key is known.
 * @param {'pt-BR' | 'en' | 'it'} locale
 */
export function translator(locale) {
  const table = LOCALES[locale] ?? en;
  const t = (key, params = {}) => {
    const canonical = canonicalLabelKey(key);
    const text = table[canonical] ?? en[canonical] ?? key;
    return text.replace(/\{(\w+)\}/g, (_, name) => String(params[name] ?? `{${name}}`));
  };
  t.has = (key) => canonicalLabelKey(key) in table || canonicalLabelKey(key) in en;
  return t;
}

/**
 * The language of the guide pages for a UI locale: the guide is written in
 * English and Portuguese only, so any other locale reads the English one.
 * @param {string} locale
 * @returns {'pt-BR' | 'en'}
 */
export function guideLocale(locale) {
  return locale === 'pt-BR' ? 'pt-BR' : 'en';
}

/** The `{name}` placeholders of a text, sorted and once each. */
export function placeholders(text) {
  return [...new Set([...String(text).matchAll(/\{(\w+)\}/g)].map((m) => m[1]))].sort();
}

/**
 * Applies `data-i18n` (text), `data-i18n-title`, `data-i18n-aria-label` and `data-i18n-placeholder`.
 * @param {ParentNode} root
 * @param {(key: string) => string} t
 */
export function applyTranslations(root, t) {
  for (const el of root.querySelectorAll('[data-i18n]')) el.textContent = t(el.dataset.i18n);
  for (const el of root.querySelectorAll('[data-i18n-title]')) el.title = t(el.dataset.i18nTitle);
  for (const el of root.querySelectorAll('[data-i18n-aria-label]')) {
    el.setAttribute('aria-label', t(el.dataset.i18nAriaLabel));
  }
  for (const el of root.querySelectorAll('[data-i18n-placeholder]')) el.placeholder = t(el.dataset.i18nPlaceholder);
}
