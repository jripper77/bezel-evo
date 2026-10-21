import { visualization } from './visualization.js';

export const graphSeriesOf = element => element.visualization?.style === 'mirrored' ? 'mirrored' : 'single';
export const graphStyleOf = element => element.kind.style === 'bars' && ['rounded', 'mirrored'].includes(element.visualization?.style) && element.visualization.roundedBars !== false ? 'rounded' : element.kind.style;

export function graphSeriesPatch(element, series) {
  return { visualization: { ...visualization(series === 'mirrored' ? 'mirrored' : 'rounded', element.visualization ?? {}), roundedBars: graphStyleOf(element) === 'rounded' } };
}

export function graphStylePatch(element, style) {
  return { kind: { style: style === 'rounded' ? 'bars' : style }, visualization: {
    ...visualization(graphSeriesOf(element) === 'mirrored' ? 'mirrored' : 'rounded', element.visualization ?? {}), roundedBars: style === 'rounded',
  } };
}

/** Legacy width depends on sample slots and gaps, never on lineWidth. */
export function graphBarWidth(element) {
  const pitch = element.frame.width / Math.max(1, element.kind.history);
  if (element.visualization?.barWidth != null) return Math.min(pitch, element.visualization.barWidth);
  const gap = Math.max(0, Math.min(element.visualization?.gap ?? (pitch >= 4 ? pitch * .2 : 0), pitch * .8));
  return pitch - gap;
}

export function graphThicknessPatch(element, width) {
  if (element.kind.style !== 'bars') return { kind: { lineWidth: Math.max(1, width) } };
  const visual = element.visualization ?? graphStylePatch(element, 'bars').visualization;
  return { visualization: { ...visual, barWidth: width == null ? null : Math.max(1, width) } };
}
