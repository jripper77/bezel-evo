// Inspector section order, per-widget disclosure memory and localized readouts.
// This module has no DOM dependency; opening a section never edits the theme.

export const SECTION_ORDER = Object.freeze([
  'layout', 'data', 'shape', 'text', 'color', 'behavior', 'faces', 'motion', 'triggers',
]);

export const SECTION_STORAGE_KEY = 'bezel.inspectorSections';
const DEFAULT_OPEN = Object.freeze(['data', 'faces']);
const own = (object, key) => Object.hasOwn(object, key);

/**
 * Section state by widget kind. Missing sections start with Data (or Card
 * Faces) open. Storage is optional and untrusted: denied access, invalid JSON
 * and obsolete entries must never stop editing. Pass null for session memory.
 * Other panels pass their own section `ids`, those `open` at first and `key`.
 */
export function createSectionMemory({ storage, key = SECTION_STORAGE_KEY, ids = SECTION_ORDER, open = DEFAULT_OPEN } = {}) {
  const validId = (id) => ids.includes(id);
  if (storage === undefined) {
    try { storage = globalThis.localStorage; } catch { storage = null; }
  }
  const states = new Map();
  try {
    const saved = JSON.parse(storage?.getItem(key) ?? 'null');
    if (saved && typeof saved === 'object' && !Array.isArray(saved)) {
      for (const [kind, sections] of Object.entries(saved)) {
        if (!sections || typeof sections !== 'object' || Array.isArray(sections)) continue;
        states.set(kind, Object.fromEntries(Object.entries(sections)
          .filter(([id, value]) => validId(id) && typeof value === 'boolean')));
      }
    }
  } catch { /* Private browsing and corrupt preferences use the defaults. */ }

  function isOpen(kind, id) {
    if (!validId(id)) return false;
    const state = states.get(kind);
    return state && own(state, id) ? state[id] : open.includes(id);
  }

  return {
    isOpen,
    /** Alt+click always opens the chosen section and closes all its peers. */
    toggle(kind, id, { exclusive = false } = {}) {
      if (!validId(id)) return false;
      const next = exclusive
        ? Object.fromEntries(ids.map(section => [section, section === id]))
        : { ...states.get(kind), [id]: !isOpen(kind, id) };
      states.set(kind, next);
      try { storage?.setItem(key, JSON.stringify(Object.fromEntries(states))); } catch { /* Session memory still works. */ }
      return next[id];
    },
  };
}

/** Short summaries: localized numbers, optional unit, omitted empty values. */
export function summaryOf(parts, { locale = 'en', separator = ' · ' } = {}) {
  const format = new Intl.NumberFormat(locale, { maximumFractionDigits: 2 });
  return parts.flatMap(part => {
    if (part === null || part === undefined || part === '' || part === false) return [];
    if (typeof part === 'string') return [part];
    const value = typeof part === 'number' ? part : part.value;
    if (!Number.isFinite(value)) return [];
    const unit = typeof part === 'object' ? part.unit : null;
    return [`${format.format(value)}${unit ? `${unit === '°' || unit === '%' ? '' : ' '}${unit}` : ''}`];
  }).join(separator);
}
