// The statusbar's left side (D-2026-10-20-studio-ux-v2-11, board
// StatusBarNew): the selected objects with their box, and while a gesture
// runs what it does, where the box snapped and a key that helps then.

const union = (frames) => {
  const x = Math.min(...frames.map(f => f.x)), y = Math.min(...frames.map(f => f.y));
  return { x, y, width: Math.max(...frames.map(f => f.x + f.width)) - x, height: Math.max(...frames.map(f => f.y + f.height)) - y };
};

// Literal keys, so the translations' check can see every one.
const GESTURES = Object.freeze({ move: 'shell.gesture.move', resize: 'shell.gesture.resize', rotate: 'shell.gesture.rotate' });
const HINTS = Object.freeze({ move: 'shell.gestureHint.move', resize: 'shell.gestureHint.resize', rotate: 'shell.gestureHint.rotate' });
const SNAPS = Object.freeze({ centre: 'shell.snapped.centre', edge: 'shell.snapped.edge' });

/** A guide through the box's middle on its axis says "centred", any other "aligned to an edge". */
function snapOf(box, guides) {
  if (!guides?.length) return null;
  const centre = guides.some(g => Math.abs(g.position - (g.axis === 'x' ? box.x + box.width / 2 : box.y + box.height / 2)) < 0.5);
  return centre ? 'centre' : 'edge';
}

/**
 * @param {{elements: object[], selection: (number|string)[], gesture?: {mode: 'move'|'resize'|'rotate', guides?: {axis: 'x'|'y', position: number}[]}|null}} state
 * @param {(key: string, params?: object) => string} t
 * @param {(n: number) => string} [number] the UI's number format
 * @returns {{empty: boolean, element: object|null, title: string, figures: string[], snap: string|null, hint: string|null}}
 */
export function selectionStatus({ elements, selection, gesture = null }, t, number = String) {
  const chosen = elements.filter(e => selection.includes(e.id));
  if (!chosen.length) return { empty: true, element: null, title: t('shell.noSelection'), figures: [], snap: null, hint: null };
  const single = chosen.length === 1 ? chosen[0] : null;
  const box = single ? single.frame : union(chosen.map(e => e.frame));
  const n = v => number(Math.round(v));
  const figures = [t('shell.position', { x: n(box.x), y: n(box.y) }), t('shell.size', { width: n(box.width), height: n(box.height) })];
  if (single && Math.round(single.rotation ?? 0) !== 0) figures.push(t('framing.degrees', { degrees: n(single.rotation) }));
  const name = single ? single.name : t('layers.count', { count: chosen.length });
  if (!gesture) return { empty: false, element: single, title: name, figures, snap: null, hint: null };
  const snap = gesture.mode === 'rotate' ? null : snapOf(box, gesture.guides);
  return {
    empty: false,
    element: single,
    title: t(GESTURES[gesture.mode], { name }),
    figures,
    snap: snap && t(SNAPS[snap]),
    hint: t(HINTS[gesture.mode]),
  };
}
