// Canonical groups of the existing controls. Unknown controls stay visible.
const GROUPS = {
  layout: ['position', 'membership', 'actions'],
  data: ['source', 'content', 'scale', 'window', 'secondary-source', 'spectrum-source', 'apps', 'time'],
  shape: ['show', 'shape', 'visualization', 'style', 'spectrum-bars', 'spectrum-bands', 'arc', 'spacing', 'progress', 'segments', 'corners', 'outline', 'appearance', 'picture'],
  text: ['font'],
  color: ['color', 'spectrum-paint', 'fill', 'track', 'shadow', 'transparency', 'visibility', 'background'],
  behavior: ['transition', 'sensor-motion', 'stopped', 'preview', 'refresh'],
};
export function sectionFor(key, fallback = 'data') {
  return Object.entries(GROUPS).find(([, keys]) => keys.includes(key))?.[0] ?? fallback;
}

/** Real document values, formatted by the shared locale-aware formatter. */
export function summaryParts(id, e, locale = 'en') {
  if (!e) return [];
  const k = e?.kind ?? {};
  switch (id) {
    case 'layout': return e.frame ? [`${new Intl.NumberFormat(locale).format(e.frame.width)} × ${new Intl.NumberFormat(locale).format(e.frame.height)}`, { value: e.rotation ?? 0, unit: '°' }] : [];
    case 'data': return [k.binding?.key ?? k.source?.key ?? k.content?.key ?? k.content?.binding?.key, k.content?.text, k.content?.pattern ?? k.pattern, k.content?.city ?? k.city?.name];
    case 'shape': return [k.type === 'graph' && ['line', 'area', 'bars'].includes(k.style) ? { key: `graph.${k.style}` } : null,
      ['rect', 'ellipse'].includes(k.shape) ? { key: `shape.${k.shape}` } : null,
      k.startAngle == null ? null : { value: k.startAngle, unit: '°' }, k.sweep == null ? null : { value: k.sweep, unit: '°' }, k.thickness == null ? null : { value: k.thickness, unit: 'px' }];
    case 'text': return [k.style?.font?.family ?? k.font?.family, (k.style?.size ?? k.font?.size) == null ? null : { value: k.style?.size ?? k.font.size, unit: 'px' }];
    case 'color': return [typeof k.fill === 'string' ? k.fill : null, { value: (e?.opacity ?? 1) * 100, unit: '%' }];
    case 'behavior': return [e.valueTransition?.mode === 'smooth' ? { value: e.valueTransition.durationMs, unit: 'ms' } : null];
    case 'faces': return e?.card?.faces ?? [];
    default: return [];
  }
}

/** Resize with an optional session-only aspect ratio. */
export function resizedFrame(frame, patch, linked) {
  if (!linked || frame.width <= 0 || frame.height <= 0) return patch;
  if (patch.width != null) return { ...patch, height: Math.max(4, Math.round(frame.height * patch.width / frame.width)) };
  if (patch.height != null) return { ...patch, width: Math.max(4, Math.round(frame.width * patch.height / frame.height)) };
  return patch;
}
