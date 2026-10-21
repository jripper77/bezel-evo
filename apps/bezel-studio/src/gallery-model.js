// Library facts are read-only: opening the gallery never opens a theme document.
import { boundKey } from './editor/widgets.js';
import { isShown, visibleWithoutFace } from './editor/cards.js';
import { supportsValueColor } from './value-color.js';

/** Sensor references, including managed feeds, like Theme::sensor_keys. */
export function themeSensorKeys(theme) {
  const keys = new Set();
  const add = key => { if (key) keys.add(key); };
  for (const e of theme.elements) {
    const parent = theme.elements.find(p => p.id === e.cardMember?.parent), card = parent?.card;
    const sampled = isShown(theme, e) || (visibleWithoutFace(theme, e) && parent?.visible !== false && card &&
      (card.triggers?.length || card.rotationSeconds != null || (card.transition && card.transition.effect !== 'none')));
    if (!sampled) continue;
    add(boundKey(e)); add(e.sensorMotion?.sensor);
    if (e.colorByValue && supportsValueColor(e) && e.kind.content?.type !== 'spectrum') add(boundKey(e) ?? e.colorByValue.sensor);
    if (e.kind.type === 'graph' && e.visualization?.style === 'mirrored') add(e.visualization.secondary?.key);
    if (e.kind.content?.type === 'weather') {
      const base = `weather.${e.kind.content.latitude.toFixed(6)}:${e.kind.content.longitude.toFixed(6)}`;
      add(`${base}.temperature`); add(`${base}.code`);
    }
    if (['player', 'spectrum'].includes(e.kind.content?.type)) add('media.sessions');
    for (const rule of e.card?.triggers ?? []) if (rule.source !== 'timer') add(rule.source === 'mediaPlaying' ? 'media.sessions' : 'activity.processes');
  }
  return [...keys].sort();
}
/** Provider-managed feeds are not hardware absent from this PC's catalog. */
export const catalogKeys = entry => (entry.sensors ?? []).filter(key => !key.startsWith('weather.') && !['media.sessions', 'activity.processes'].includes(key));
export const missingSensors = (entry, catalog) => catalog === null ? [] : catalogKeys(entry).filter(key => !catalog.some(sensor => sensor.key === key));
export function themeUses(entry, overview) {
  if (!overview) return [];
  const uses = [], active = overview.config.profiles.find(p => p.id === overview.status?.active?.profile);
  for (const display of overview.displays) {
    const current = overview.status?.preview?.themes?.[display.key] ?? active?.themes[display.key] ?? display.theme;
    if (overview.status && current === entry.location && display.connected) uses.push({ kind: 'screen', name: display.name, key: display.key });
    if (display.theme === entry.location) uses.push({ kind: 'base', name: display.name, key: display.key });
  }
  for (const profile of overview.config.profiles) if (Object.values(profile.themes).includes(entry.location)) uses.push({ kind: 'profile', name: profile.name, key: profile.id });
  return uses;
}
export const recentThemes = entries => [...entries].sort((a, b) => (b.modifiedMs ?? 0) - (a.modifiedMs ?? 0) || a.name.localeCompare(b.name));

// Rust omits empty/default fields; property order and omission cannot change rule identity.
const ruleVersion = rule => !rule ? null : [rule.source, rule.app ?? '', rule.priority, rule.returnSeconds,
  rule.key ?? '', rule.compare ?? null, rule.value ?? null, rule.forSeconds ?? 0,
  rule.days ?? [], rule.from ?? '', rule.to ?? '', rule.minutes ?? 0];
/** The UI's selected row never determines the active profile or rule state. */
export function ruleNow(overview, id, index, draft) {
  const saved = overview?.config.profiles.find(p => p.id === id)?.rules[index];
  if (draft && JSON.stringify(ruleVersion(saved)) !== JSON.stringify(ruleVersion(draft))) return null;
  return overview?.status?.rules?.[id]?.[index] ?? null;
}
