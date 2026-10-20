import { icon } from '../dom.js';
import { ICONS } from '../icons.js';
import { TIME_PLACEMENTS } from '../../player-parts.js';
// Illustrations of the option tiles (24-unit paths drawn like every icon):
// the graph's styles, what the player shows and how its bar is cut.
export const ART = Object.freeze({
  line: ['M2 17l5-4 4 2 5-8 6 4'],
  area: ['M2 17l5-4 4 2 5-8 6 4v8H2z'],
  bars: ['M4 21v-5', 'M9 21V9', 'M14 21v-7', 'M19 21V5'],
  cover: ['M3 6h10v12H3z', 'M16 9h5', 'M16 14h3'],
  progress: ['M3 11h18', 'M3 11h8', 'M3 17h3', 'M18 17h3'],
  source: ['M3 8h18v8H3z', 'M7 12h8'],
  continuous: ['M3 12h18'],
  segments: ['M3 12h4', 'M10 12h4', 'M17 12h4'],
  dots: ['M4 12h.01', 'M9 12h.01', 'M14 12h.01', 'M19 12h.01'],
  // How a shape's picture covers it (artboard-style tiles over the frame).
  repeatCover: ['M3 4h18v16H3z', 'M3 17l6-7 4 4 3-3 5 5'],
  repeatContain: ['M3 4h18v16H3z', 'M7 8h10v8H7z'],
  repeatStretch: ['M3 4h18v16H3z', 'M7 8l10 8', 'M7 8h3', 'M7 8v3', 'M17 16h-3', 'M17 16v-3'],
  repeatCenter: ['M3 4h18v16H3z', 'M10 10h4v4h-4z'],
  repeatTile: ['M3 4h18v16H3z', 'M6 7h4v4H6z', 'M14 7h4v4h-4z', 'M6 13h4v4H6z', 'M14 13h4v4h-4z'],
  repeatTileX: ['M3 4h18v16H3z', 'M5 10h4v4H5z', 'M10 10h4v4h-4z', 'M15 10h4v4h-4z'],
  repeatTileY: ['M3 4h18v16H3z', 'M10 5h4v4h-4z', 'M10 10h4v4h-4z', 'M10 15h4v4h-4z'],
});
/** How a shape's picture covers it, in the order the tiles show. */
export const IMAGE_REPEATS = Object.freeze(['cover', 'contain', 'stretch', 'center', 'tile', 'tileX', 'tileY']);
/** The name of each repeat; stretch and center reuse the image fit's words. */
export const REPEAT_KEYS = Object.freeze({ cover: 'shape.repeat.cover', contain: 'shape.repeat.contain', stretch: 'fit.fill', center: 'framing.center', tile: 'shape.repeat.tile', tileX: 'shape.repeat.tileX', tileY: 'shape.repeat.tileY' });
/** Repeats that keep the picture's own size: scale and offset apply. */
export const NATURAL_REPEATS = Object.freeze(['center', 'tile', 'tileX', 'tileY']);
/** The player's parts: their name and drawing ("What to show"). */
export const PART_TILES = Object.freeze({
  cover: ['insp.cover', ART.cover],
  title: ['insp.partTitle', ICONS.text],
  artist: ['insp.partArtist', ICONS.person],
  time: ['insp.partTime', ICONS.clock],
  progress: ['insp.progress', ART.progress],
  source: ['insp.appName', ART.source],
});
/** Names of the player's time formats, of where its times sit and of its bar's patterns. */
export const TIME_FORMAT_KEYS = Object.freeze({
  elapsedTotal: 'insp.timeElapsedTotal', elapsedRemaining: 'insp.timeElapsedRemaining', elapsed: 'insp.timeElapsed', remaining: 'insp.timeRemaining', total: 'insp.timeTotal',
});
/** In the order of TIME_PLACEMENTS: in the text, at the sides, above. */
export const TIME_PLACEMENT_KEYS = Object.freeze(['insp.timesText', 'insp.timesSides', 'insp.timesAbove']);
export const PATTERN_KEYS = Object.freeze({ continuous: 'insp.patternContinuous', segments: 'insp.patternSegments', dots: 'insp.patternDots' });

