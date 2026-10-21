// Pure edits shared by the inspector and its tests. No document defaults are
// written merely by opening a panel.
import { boundKey } from './editor/widgets.js';
import { VALUE_PARTS, CLOCK_PARTS } from './text-layout.js';
import { appliedRotation, canRotate } from './editor/rotation.js';

export function partStylePatch(style, part, patch) {
  if (![...VALUE_PARTS, ...CLOCK_PARTS].includes(part)) return {};
  return { parts: { [part]: patch === null ? null : { ...style.parts?.[part], ...patch } } };
}

export const MIXED = Symbol('mixed');
export function commonValue(values) {
  if (!values.length) return MIXED;
  return values.every(value => JSON.stringify(value) === JSON.stringify(values[0])) ? values[0] : MIXED;
}

export function paintPath(element) {
  if (element.isGroup || element.card) return null;
  if (element.kind.type === 'text') return ['kind', 'style', 'paint'];
  if (['bar', 'ring', 'shape'].includes(element.kind.type)) return ['kind', 'fill'];
  if (['needle', 'graph'].includes(element.kind.type)) return ['kind', 'color'];
  return null;
}
export const atPath = (object, path) => path.reduce((value, key) => value?.[key], object);
export const pathPatch = (path, value) => path.reduceRight((out, key) => ({ [key]: out }), value);

export function commonSelection(elements) {
  const paths = elements.map(paintPath);
  return {
    frame: Object.fromEntries(['x', 'y', 'width', 'height'].map(key => [key, commonValue(elements.map(e => e.frame[key]))])),
    rotation: elements.length > 0 && elements.every(e => canRotate(null, e)) ? commonValue(elements.map(appliedRotation)) : null,
    opacity: commonValue(elements.map(e => e.opacity ?? 1)),
    sensor: elements.length > 0 && elements.every(e => boundKey(e)) ? commonValue(elements.map(boundKey)) : null,
    paint: elements.length > 0 && paths.every(Boolean) ? commonValue(elements.map((e, i) => atPath(e, paths[i]))) : null,
  };
}

/** One patch per object, later applied by one history command. */
export function selectionPatches(elements, field, value) {
  return elements.flatMap(element => {
    if (['x', 'y', 'width', 'height'].includes(field)) return [{ id: element.id, patch: { frame: { ...element.frame, [field]: ['width', 'height'].includes(field) ? Math.max(4, value) : value } } }];
    if (field === 'opacity') return [{ id: element.id, patch: { opacity: Math.max(0, Math.min(1, value)) } }];
    if (field === 'paint') {
      const path = paintPath(element);
      return path ? [{ id: element.id, patch: pathPatch(path, value) }] : [];
    }
    if (field === 'sensor' && boundKey(element)) return [{ id: element.id, patch: element.kind.type === 'text' ? { kind: { content: { key: value } } } : { kind: { binding: { key: value } } } }];
    return [];
  });
}
