// Native visualization presets; all settings remain editable theme data.
export const VISUAL_STYLES = Object.freeze({ bar: ['standard', 'matrix'], ring: ['standard', 'pixel', 'dial'], graph: ['standard', 'rounded', 'mirrored'] });
export function visualization(style, previous = {}) {
  if (style === 'standard') return null;
  return { columns: 16, rows: 6, gap: 3, radius: 2, ticks: 31, marker: true, secondary: null, secondaryColor: '#e879a4ff', ...previous, style };
}
export function visualStyles(kind) { return VISUAL_STYLES[kind] ?? []; }

const ART = Object.freeze({
  bar: ['M2 9h20v6H2z', 'M2 9h12v6H2'],
  ring: ['M5 19a9 9 0 1 1 14 0', 'M5 19a9 9 0 0 1-2-9'],
  graph: ['M2 19l4-6 4 2 4-9 4 4 4-5'],
  matrix: Array.from({ length: 15 }, (_, i) => { const x=2+(i%5)*4.3, y=6+Math.floor(i/5)*4.3; return `M${x} ${y}h2.5v2.5h-2.5z`; }),
  pixel: ['M3 16h2v2H3z','M2 11h2v2H2z','M4 6h2v2H4z','M8 3h2v2H8z','M13 2h2v2h-2z','M18 5h2v2h-2z','M20 10h2v2h-2z','M19 15h2v2h-2z'],
  dial: ['M6 18a8 8 0 1 1 12 0','M2 12h2','M5 4l1 2','M12 1v2','M19 4l-1 2','M20 12h2','M4 18l2-1','M18 17l2 1','M5 9a1.5 1.5 0 1 0 3 0a1.5 1.5 0 1 0-3 0'],
  rounded: ['M4 20V12','M9 20V6','M14 20V10','M19 20V3'],
  mirrored: ['M2 12h20','M3 9l4-4 4 3 5-5 5 4','M3 15l4 3 4-1 5 3 5-3'],
});

/** A schematic preview of a preset, including the existing classic geometry. */
export function visualArt(kind, style) { return ART[style === 'standard' ? kind : style] ?? []; }

/** Explicitly fit near-square matrix cells into the available screen height. */
export function matrixFrame(frame, visual, canvas) {
  const columns=Math.max(1,Math.min(64,visual.columns ?? 16));
  const rows=Math.max(1,Math.min(32,visual.rows ?? 6));
  const available=Math.max(4,canvas.height-Math.max(0,frame.y));
  return { ...frame, height:Math.max(4,Math.min(available,Math.round(frame.width*rows/columns))) };
}
