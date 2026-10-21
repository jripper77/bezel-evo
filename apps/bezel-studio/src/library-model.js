// What the library panels compute without the DOM (artboards Main, Sensors,
// Layers, Media): the widget groups and their search, the sensors' recent
// history and how often it arrives, the sparkline of a series, where a
// sensor dropped on the canvas binds, what a layer row says about its
// object, the Layers filter and the Media totals.
import { boundKey, widgetOf } from './editor/widgets.js';
import { contains } from './editor/geometry.js';
import { isShown, owners } from './editor/cards.js';

/**
 * The Widgets panel's groups, in order (D-2026-10-20-studio-ux-v2-14, board
 * MainNew): values, time and media, text and shapes, containers. `group` is
 * the Group tile: it groups the selection instead of adding an object.
 */
export const WIDGET_GROUPS = Object.freeze([
  Object.freeze({ id: 'values', widgets: Object.freeze(['value', 'ring', 'bar', 'needle', 'graph']) }),
  Object.freeze({ id: 'timeMedia', widgets: Object.freeze(['clock', 'weather', 'player', 'spectrum']) }),
  Object.freeze({ id: 'textShapes', widgets: Object.freeze(['text', 'image', 'shape']) }),
  Object.freeze({ id: 'containers', widgets: Object.freeze(['card', 'group']) }),
]);

/** How many tiles the Widgets panel has in all. */
export const WIDGET_COUNT = WIDGET_GROUPS.reduce((n, g) => n + g.widgets.length, 0);

const fold = (text) => String(text ?? '').normalize('NFD').replace(/\p{M}/gu, '').toLowerCase();

/** Whether every word of `query` is in one of `texts` (accents and case ignored). */
export function matchesQuery(query, ...texts) {
  const words = fold(query).split(/\s+/).filter(Boolean);
  if (!words.length) return true;
  const hay = texts.map(fold).join(' ');
  return words.every((w) => hay.includes(w));
}

/**
 * The groups the Widgets search keeps: each widget whose name (`labelOf`)
 * or id matches, groups left empty dropped.
 * @param {(widget: string) => string} labelOf
 */
export function widgetGroups(labelOf, query = '') {
  return WIDGET_GROUPS
    .map((g) => ({ id: g.id, widgets: g.widgets.filter((w) => matchesQuery(query, labelOf(w), w)) }))
    .filter((g) => g.widgets.length);
}

/**
 * The recent readings of every sensor (the Sensors panel's sparklines) and
 * when each sample arrived. Only numbers count; a sensor that stops reading
 * keeps what it had. `limit` samples per sensor.
 */
export function createHistory(limit = 30) {
  const series = new Map();
  const times = [];
  return {
    /** A sample: `readings` by key (`{value}`), taken at `at` ms. */
    push(readings, at) {
      if (Number.isFinite(at)) {
        times.push(at);
        if (times.length > limit) times.shift();
      }
      for (const [key, r] of Object.entries(readings ?? {})) {
        if (r?.unavailable || !Number.isFinite(r?.value)) continue;
        if (!series.has(key)) series.set(key, []);
        const values = series.get(key);
        values.push(r.value);
        if (values.length > limit) values.shift();
      }
    },
    /** The values of `key`, oldest first. */
    values: (key) => [...(series.get(key) ?? [])],
    /** When the samples arrived, oldest first. */
    times: () => [...times],
  };
}

/**
 * How often samples arrive, in whole seconds (at least 1): the median gap
 * between the times given; `null` with fewer than two.
 */
export function updateSeconds(times) {
  const gaps = times.slice(1).map((t, i) => t - times[i]).filter((g) => g > 0).sort((a, b) => a - b);
  if (!gaps.length) return null;
  const mid = gaps.length >> 1;
  const median = gaps.length % 2 ? gaps[mid] : (gaps[mid - 1] + gaps[mid]) / 2;
  return Math.max(1, Math.round(median / 1000));
}

/**
 * The path of a sparkline `width`×`height` (a flat series draws in the
 * middle); `null` with fewer than two values. The width is the caller's: the
 * Library reads it from the `--spark-width` token.
 */
export function sparkPath(values, width, height = 20, pad = 2) {
  const v = values.filter(Number.isFinite);
  if (v.length < 2) return null;
  const min = Math.min(...v);
  const span = Math.max(...v) - min;
  const step = width / (v.length - 1);
  const y = (n) => (span ? height - pad - ((n - min) / span) * (height - 2 * pad) : height / 2);
  const r = (n) => Math.round(n * 10) / 10;
  return v.map((n, i) => `${i ? 'L' : 'M'}${r(i * step)} ${r(y(n))}`).join(' ');
}

/** The source the sensors of a group share (shown beside its title), or `null`. */
export function sharedSource(items) {
  const sources = new Set(items.map((s) => s.source).filter(Boolean));
  return sources.size === 1 ? [...sources][0] : null;
}

/**
 * Where a sensor dropped at `point` (canvas pixels) binds: the topmost
 * visible, unlocked object there that reads a sensor, and the patch that
 * makes it read `sensor` (a range for the quantity, `range`); `null` over
 * nothing such (the drop then adds a value).
 * @param {object[]} elements the theme's, in paint order
 * @param {{key: string, quantity?: string}} sensor
 * @param {(quantity: string|undefined) => {min: number, max: number}} range
 */
export function sensorDropTarget(elements, point, sensor, range) {
  const theme = { elements };
  for (const e of [...elements].reverse()) {
    if (e.card || e.isGroup || boundKey(e) === null || !isShown(theme, e) || owners(theme, e).some((parent) => parent.locked)) continue;
    if (!contains(e.frame, point.x, point.y)) continue;
    const patch = e.kind.type === 'text'
      ? { kind: { content: { key: sensor.key } } }
      : { kind: { binding: { key: sensor.key, ...range(sensor.quantity) } } };
    return { id: e.id, patch };
  }
  return null;
}

/**
 * What a layer row says beside the name: the sensor it reads (mono), the
 * number of faces of a card, or nothing (the row then names its kind).
 */
export function layerMeta(e) {
  if (e.card) return { faces: e.card.faces.length };
  if (e.isGroup) return {};
  const key = boundKey(e);
  return key ? { sensor: key } : {};
}

/**
 * The Layers filter over `layerGroups()`'s tree: an object stays when its
 * name, kind (`kindOf`) or sensor matches, a container when it matches (all
 * of it) or holds one that does (only those). Card faces keep their place.
 * An optional widget type combines with the query on the same object.
 * @param {object[]} nodes
 * @param {(e: object) => string} kindOf translated kind
 */
export function filterLayers(nodes, query, kindOf, type = 'all') {
  const searching = Boolean(fold(query).trim());
  if (!searching && type === 'all') return nodes;
  const hit = (e) => (type === 'all' || widgetOf(e) === type)
    && (!searching || matchesQuery(query, e.name, kindOf(e), boundKey(e) ?? ''));
  const visit = (list) => list.flatMap((node) => {
    if (hit(node.element)) return [node];
    const children = visit(node.children);
    const groups = node.groups.map((g) => ({ ...g, nodes: visit(g.nodes) }));
    const kept = children.length || groups.some((g) => g.nodes.length);
    if (!kept) return [];
    return [{ ...node, children, groups: groups.map((g) => ({ ...g, elements: g.nodes.map((n) => n.element) })) }];
  });
  return visit(nodes);
}

/** How many objects a tree shows (containers and members). */
export function countLayers(nodes) {
  return nodes.reduce((n, node) => n + 1 + countLayers(node.children) + node.groups.reduce((m, g) => m + countLayers(g.nodes), 0), 0);
}

/** The Media list's totals: how many files and their bytes (`null` when no size is known). */
export function mediaTotals(items) {
  const sizes = items.map((a) => a.bytes).filter((b) => typeof b === 'number' && Number.isFinite(b));
  return { count: items.length, bytes: sizes.length ? sizes.reduce((a, b) => a + b, 0) : null };
}
