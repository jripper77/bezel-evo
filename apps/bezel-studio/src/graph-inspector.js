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
