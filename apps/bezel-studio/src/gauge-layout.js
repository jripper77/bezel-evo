// Native gauge geometry, mirrored from bezel-render/gauges.rs and path.rs.
export function gaugeBlocks(length, segments, wrap = false) {
  if (!segments) return [[0, length]];
  const count = Math.max(1, segments.count), gap = Number.isFinite(segments.gap) ? Math.max(0, segments.gap) : 0;
  const size = (length - gap * (wrap ? count : count - 1)) / count;
  return size > 0 ? Array.from({ length: count }, (_, i) => [i * (size + gap), i * (size + gap) + size]) : [[0, length]];
}

export function barSpan(frame, direction, start, end) {
  const length = Math.max(0, end - start);
  if (direction === 'rightToLeft') return { x: frame.x + frame.width - end, y: frame.y, width: length, height: frame.height };
  if (direction === 'bottomToTop') return { x: frame.x, y: frame.y + frame.height - end, width: frame.width, height: length };
  if (direction === 'topToBottom') return { x: frame.x, y: frame.y + start, width: frame.width, height: length };
  return { x: frame.x + start, y: frame.y, width: length, height: frame.height };
}
