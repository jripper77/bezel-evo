// One read-only thumbnail request per backend, location and file revision.
import { thumbnailKey } from './theme-filter.js';
const backends = new WeakMap();
export function themePreview(bridge, entry) {
  let cache = backends.get(bridge);
  if (!cache) { cache = new Map(); backends.set(bridge, cache); }
  const key = thumbnailKey(entry);
  if (!cache.has(key)) cache.set(key, Promise.resolve().then(() => bridge.themeThumbnail(entry.location)).catch(() => null));
  return cache.get(key);
}
