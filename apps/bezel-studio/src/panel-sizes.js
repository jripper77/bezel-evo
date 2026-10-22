// The widths of the two side panels (D-2026-10-09-studio-redesign-fidelity-7):
// the library panel (after the rail) and the inspector. Each is resized by
// its separator — dragged, or with the arrow keys (Shift for bigger steps),
// Home and End — kept between its minimum and maximum, reset by a double
// click and remembered across launches. Pure: storage is passed in.

/** Each panel's limits and starting width (CSS px); `side` is where it sits. */
export const PANELS = Object.freeze({
  library: Object.freeze({ min: 240, max: 480, initial: 272, side: 'start' }),
  inspector: Object.freeze({ min: 280, max: 520, initial: 360, side: 'end' }),
});

/** Where the widths are remembered. */
export const STORAGE_KEY = 'bezel.panelWidths';
/** An arrow key's step, and Shift+arrow's. */
export const STEP = 8;
export const BIG_STEP = 32;
/** The narrowest the stage gets before the panels give way. */
export const STAGE_MIN = 320;

/** The starting widths. */
export const initialWidths = () => ({ library: PANELS.library.initial, inspector: PANELS.inspector.initial });

/** `width` within `panel`'s limits, whole pixels; anything not a number is its starting width. */
export function clampWidth(panel, width) {
  const { min, max, initial } = PANELS[panel];
  if (typeof width !== 'number' || !Number.isFinite(width)) return initial;
  return Math.round(Math.min(max, Math.max(min, width)));
}

/**
 * The width a key on `panel`'s separator gives, or `null` for another key.
 * The arrow towards the stage widens the panel; Home is the minimum, End the maximum.
 */
export function widthAfterKey(panel, width, key, shift = false) {
  const { min, max, side } = PANELS[panel];
  const step = shift ? BIG_STEP : STEP;
  const grow = side === 'start' ? 'ArrowRight' : 'ArrowLeft';
  const shrink = side === 'start' ? 'ArrowLeft' : 'ArrowRight';
  if (key === grow) return clampWidth(panel, width + step);
  if (key === shrink) return clampWidth(panel, width - step);
  if (key === 'Home') return min;
  if (key === 'End') return max;
  return null;
}

/** The width after dragging `panel`'s separator `dx` pixels from where it was at `startWidth`. */
export function widthAfterDrag(panel, startWidth, dx) {
  return clampWidth(panel, startWidth + (PANELS[panel].side === 'start' ? dx : -dx));
}

/** The remembered widths (each within its limits), or the starting ones. */
export function readWidths(storage) {
  try {
    const saved = JSON.parse(storage?.getItem(STORAGE_KEY) ?? 'null');
    if (!saved || typeof saved !== 'object') return initialWidths();
    return { library: clampWidth('library', saved.library), inspector: clampWidth('inspector', saved.inspector) };
  } catch {
    return initialWidths();
  }
}

/** Remembers `widths`; a storage that refuses is ignored. */
export function writeWidths(storage, widths) {
  try {
    storage?.setItem(STORAGE_KEY, JSON.stringify({ library: clampWidth('library', widths.library), inspector: clampWidth('inspector', widths.inspector) }));
  } catch {
    // Private mode or a full quota: the widths last this session only.
  }
}

/**
 * The widths that fit `available` pixels (the window less the rail) while
 * the stage keeps STAGE_MIN: the panels give way, the inspector first,
 * never below their minimum.
 */
export function fitWidths(widths, available) {
  let { library, inspector } = widths;
  let excess = library + inspector + STAGE_MIN - available;
  if (excess <= 0) return { library, inspector };
  const fromInspector = Math.min(excess, inspector - PANELS.inspector.min);
  inspector -= fromInspector;
  excess -= fromInspector;
  library -= Math.min(excess, library - PANELS.library.min);
  return { library, inspector };
}
