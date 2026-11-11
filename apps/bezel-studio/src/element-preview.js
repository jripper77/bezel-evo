// The inspector's preview of one object, without the DOM
// (D-2026-10-09-studio-redesign-fidelity-8): the object is rendered alone by
// the backend (`renderElement`); this module decides when to ask (one render
// at a time, the last request wins, nothing in bursts), how big the answer
// is drawn, and what the object's sensor read lately (its peak).

import { createRenderScheduler } from './render-scheduler.js';

/** The largest upscale of the preview: a tiny object is not blown into blur. */
export const PREVIEW_MAX_SCALE = 4;

/** The longest side the backend is asked for, device pixels. */
export const PREVIEW_MAX_SIDE = 512;

/**
 * How big a rendered `frame` is drawn in a `box`, CSS pixels: it fits both
 * sides, never more than {@link PREVIEW_MAX_SCALE} times its size. `ratio`
 * is the device pixel ratio the frame was asked for (its pixels are device
 * pixels). Null when the frame or the box is empty.
 * @param {{width: number, height: number}|null} frame
 * @param {{width: number, height: number}} box
 * @param {number} [ratio]
 * @returns {{width: number, height: number}|null}
 */
export function fitPreview(frame, box, ratio = 1) {
  if (!frame || !(frame.width > 0 && frame.height > 0 && box.width > 0 && box.height > 0)) return null;
  const width = frame.width / ratio;
  const height = frame.height / ratio;
  const scale = Math.min(box.width / width, box.height / height, PREVIEW_MAX_SCALE);
  return { width: Math.max(1, Math.round(width * scale)), height: Math.max(1, Math.round(height * scale)) };
}

/**
 * Translation in frame pixels that centres the visible content, ignoring
 * transparent padding. Only the inspector's presentation uses this offset;
 * the rendered pixels, frame size and the widget's alignment stay unchanged.
 */
export function previewOffset(frame) {
  const pixels = frame?.pixels ?? frame?.rgba;
  if (!pixels || !(frame.width > 0 && frame.height > 0)) return { x: 0, y: 0 };
  let left = frame.width, right = -1, top = frame.height, bottom = -1;
  for (let y = 0; y < frame.height; y += 1) {
    for (let x = 0; x < frame.width; x += 1) {
      if (!(pixels[(y * frame.width + x) * 4 + 3] > 0)) continue;
      left = Math.min(left, x); right = Math.max(right, x);
      top = Math.min(top, y); bottom = Math.max(bottom, y);
    }
  }
  return right < 0 ? { x: 0, y: 0 } : {
    x: (frame.width - left - right - 1) / 2,
    y: (frame.height - top - bottom - 1) / 2,
  };
}

/**
 * The longest side to ask the backend for, device pixels: the box's longest
 * side at the device ratio, between 16 and {@link PREVIEW_MAX_SIDE}.
 */
export function previewSide(box, ratio = 1) {
  const side = Math.ceil(Math.max(box.width, box.height) * ratio);
  return Math.max(16, Math.min(PREVIEW_MAX_SIDE, Number.isFinite(side) ? side : PREVIEW_MAX_SIDE));
}

/**
 * Asks for renders one at a time. A request while one runs is remembered
 * (only the last one) and asked once the running one ends; an answer that
 * a later request overtook is dropped, never shown.
 * @param {{render: (key: any) => Promise<any>, show: (frame: any, key: any) => void, fail?: (error: any, key: any) => void}} deps
 */
export function createPreviewScheduler({ render, show, fail = () => {} }) {
  let running = false;
  let requested = false;
  let latest = null;
  let generation = 0;
  // The shared scheduler owns serialization and coalescing. This adapter
  // only rejects answers superseded by selection/property changes.
  const scheduler = createRenderScheduler({
    gesturing: () => false,
    render: async () => {
      if (!requested) return;
      requested = false;
      running = true;
      const key = latest;
      const asked = generation;
      try {
        const frame = await render(key);
        if (asked === generation) show(frame, key);
      } catch (error) {
        if (asked === generation) fail(error, key);
      } finally {
        running = false;
      }
    },
  });

  return {
    /** Asks for a render of `key` (coalesced with the one running). */
    request(key) {
      latest = key;
      requested = true;
      generation += 1;
      scheduler.request();
    },
    /** Drops what is pending and the answer of the running render. */
    cancel() {
      requested = false;
      generation += 1;
    },
    /** Whether a render runs now. */
    busy: () => running || requested,
  };
}

/** How many readings the peak keeps, at most: a graph's longest history. */
export const PEAK_LIMIT = 3600;

/** How many sensors the peak follows at once: the two series of a mirrored graph. */
export const PEAK_SENSORS = 2;

/**
 * The readings of the sensors shown, to tell each one's peak over a window
 * of time. It follows the last `sensors` keys observed: watching another
 * forgets the one observed least recently. A sample is recorded once (the
 * same `at` twice is one reading); unavailable readings are skipped.
 */
export function createPeakWatch(limit = PEAK_LIMIT, sensors = PEAK_SENSORS) {
  const watched = new Map();
  return {
    /** Records `reading` (`{value, display}`) of `sensor` read at `at` ms. */
    observe(sensor, reading, at) {
      const entries = watched.get(sensor) ?? [];
      // Most recently observed last, so the first key is the one to forget.
      watched.delete(sensor);
      watched.set(sensor, entries);
      if (watched.size > sensors) watched.delete(watched.keys().next().value);
      if (!reading || typeof reading.value !== 'number' || !Number.isFinite(reading.value)) return;
      if (entries.length && entries[entries.length - 1].at >= at) return;
      entries.push({ at, value: reading.value, display: reading.display ?? String(reading.value) });
      if (entries.length > limit) entries.splice(0, entries.length - limit);
    },
    /**
     * The lowest and highest readings of `sensors` together since
     * `now - windowMs`, as `{ min, max }` entries (null: none).
     */
    range(sensors, windowMs, now) {
      let min = null, max = null;
      for (const sensor of sensors) for (const entry of watched.get(sensor) ?? []) {
        if (entry.at < now - windowMs) continue;
        if (!min || entry.value < min.value) min = entry;
        if (!max || entry.value > max.value) max = entry;
      }
      return max && { min, max };
    },
    /** The highest reading of `sensor` since `now - windowMs` (null: none). */
    peak(sensor, windowMs, now) {
      let best = null;
      for (const entry of watched.get(sensor) ?? []) {
        if (entry.at < now - windowMs) continue;
        if (!best || entry.value > best.value) best = entry;
      }
      return best;
    },
  };
}
