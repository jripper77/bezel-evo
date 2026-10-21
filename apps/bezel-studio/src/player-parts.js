// The media player's parts and progress bar
// (D-2026-10-16-studio-redesign-fidelity-26): the rules of
// bezel_core::domain::playback and bezel-render's player block, for the
// inspector and the demo renderer. Every field is optional in a theme;
// absent fields draw the player as it was before the parts existed.

/** The progress bar before it had options (`progressStyle` absent). */
export const PROGRESS_DEFAULTS = Object.freeze({
  fill: null, track: '#64646464', thickness: 4, ends: 'round', knob: false, pattern: 'continuous', segments: 12, times: 'text',
});
/** Thinnest and thickest bar, pixels. */
export const PROGRESS_THICKNESS = Object.freeze({ min: 1, max: 64 });
/** Fewest and most blocks of a segmented bar. */
export const PROGRESS_SEGMENTS = Object.freeze({ min: 2, max: 100 });
/** Space between the text and the bar, pixels. */
export const PROGRESS_SPACING = 4;
/** Space between a time and the bar beside it, in text sizes. */
const TIME_GAP_EM = 0.4;
/** Space between the times and the bar under them, pixels. */
const ABOVE_GAP = 2;
/** Shortest bar between two times, in text sizes. */
const MIN_BAR_EM = 3;
/** Share of the width a cover takes beside other parts. */
const COVER_SHARE = 0.4;

export const TIME_FORMATS = Object.freeze(['elapsedTotal', 'elapsedRemaining', 'elapsed', 'remaining', 'total']);
export const TIME_PLACEMENTS = Object.freeze(['text', 'sides', 'above']);
export const PROGRESS_PATTERNS = Object.freeze(['continuous', 'segments', 'dots']);
/** The parts of "What to show", in the order of their tiles. */
export const PLAYER_PARTS = Object.freeze(['cover', 'title', 'artist', 'time', 'progress', 'source']);

/** The bar's style with every field filled in. */
export function progressOf(content) {
  return { ...PROGRESS_DEFAULTS, ...(content?.progressStyle ?? {}) };
}

/** Whether the times are printed: absent `showTime` follows `showProgress`. */
export function showsTime(content) {
  return content.showTime ?? content.showProgress !== false;
}

/** Which parts are on. */
export function partsOf(content) {
  return {
    cover: content.showCover !== false,
    title: content.showTitle !== false,
    artist: content.showArtist !== false,
    time: showsTime(content),
    progress: content.showProgress !== false,
    source: Boolean(content.showSource),
  };
}

const PART_FIELD = Object.freeze({ cover: 'showCover', title: 'showTitle', artist: 'showArtist', time: 'showTime', progress: 'showProgress', source: 'showSource' });

/**
 * The content patch that turns `part` on or off. Turning the bar on or off
 * writes the times' own state first, so they no longer follow it.
 */
export function partPatch(content, part) {
  const on = partsOf(content)[part];
  const patch = { [PART_FIELD[part]]: !on };
  if (part === 'progress' && content.showTime == null) patch.showTime = showsTime(content);
  return patch;
}

/** `m:ss`, minutes unbounded. */
export function clockTime(seconds) {
  const s = Number.isFinite(seconds) ? Math.max(0, Math.floor(seconds)) : 0;
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
}

/** The times of `media`, `[left, right]`, either may be null. */
export function playerTimes(content, media) {
  const elapsed = clockTime(media.position);
  const remaining = `-${clockTime(media.duration - media.position)}`;
  const total = clockTime(media.duration);
  switch (content.timeFormat ?? 'elapsedTotal') {
    case 'elapsedRemaining': return [elapsed, remaining];
    case 'elapsed': return [elapsed, null];
    case 'remaining': return [null, remaining];
    case 'total': return [null, total];
    default: return [elapsed, total];
  }
}

/** The widest the times get while `media` plays, so the bar keeps its length. */
export function widestTimes(content, media) {
  const [left, right] = playerTimes(content, { ...media, position: 0 });
  return [left === null ? null : clockTime(media.duration), right];
}

/** What is printed for `media`: `{lines, times: [left, right]|null, progress: fraction|null}`. */
export function playerText(content, media) {
  const parts = partsOf(content);
  const timed = media.duration > 0;
  const bar = parts.progress && timed;
  const times = parts.time && timed ? playerTimes(content, media) : null;
  const lines = [];
  if (parts.title) lines.push(media.title);
  if (parts.artist) lines.push(media.artist);
  if (parts.source) lines.push(media.source);
  const beside = times && bar && progressOf(content).times !== 'text';
  if (times && !beside) lines.push(times.filter((v) => v !== null).join(' / '));
  return {
    lines,
    times: beside ? times : null,
    progress: bar ? Math.min(1, Math.max(0, media.position / media.duration)) : null,
  };
}

/** Corner radius of the bar and its blocks. */
export const progressRadius = (style) => (style.ends === 'square' ? 0 : style.thickness / 2);
/** Diameter of the knob, 0 without one. */
export const knobDiameter = (style) => (style.knob ? Math.max(style.thickness * 2, style.thickness + 6) : 0);
/** Height of the bar with its knob. */
export const barHeight = (style) => Math.max(style.thickness, knobDiameter(style));

/** Blocks along a bar `length` long as `[start, end]` offsets. */
export function progressBlocks(style, length) {
  const t = style.thickness;
  if (!(length > 0 && t > 0)) return [];
  if (style.pattern !== 'segments' && style.pattern !== 'dots') return [[0, length]];
  let count;
  let gap;
  if (style.pattern === 'segments') {
    count = Math.max(1, style.segments);
    gap = Math.max(t * 0.75, 2);
  } else {
    count = Math.max(1, Math.floor((length + t * 0.6) / (t * 1.6)));
    gap = count > 1 ? (length - count * t) / (count - 1) : 0;
  }
  const size = (length - gap * (count - 1)) / count;
  if (size <= 0) return [[0, length]];
  return Array.from({ length: count }, (_, i) => [i * (size + gap), i * (size + gap) + size]);
}

/**
 * Sizes of the progress block, or null without a bar. `label(text)`
 * measures one time as `{width, height}`; `size` is the text size.
 */
export function blockMetrics(content, media, shown, label, size) {
  if (shown.progress === null) return null;
  const style = progressOf(content);
  const bar = barHeight(style);
  const plain = { height: bar, left: 0, right: 0, line: 0, gap: 0, minWidth: 0 };
  if (!shown.times || style.times === 'text') return plain;
  const [l, r] = widestTimes(content, media);
  const left = l === null ? { width: 0, height: 0 } : label(l);
  const right = r === null ? { width: 0, height: 0 } : label(r);
  const line = Math.max(left.height, right.height);
  const gap = Math.max(0, size * TIME_GAP_EM);
  const labels = (left.width > 0) + (right.width > 0);
  return style.times === 'sides'
    ? { height: Math.max(bar, line), left: left.width, right: right.width, line, gap, minWidth: left.width + right.width + gap * labels + size * MIN_BAR_EM }
    : { height: line + ABOVE_GAP + bar, left: left.width, right: right.width, line, gap, minWidth: left.width + right.width + (labels > 1 ? gap : 0) };
}

/** Where the block sits in `area`: under the text, or alone by `valign`. */
export function placeBlock(area, height, alone, valign) {
  let y = area.y + area.height - height;
  if (alone && valign === 'top') y = area.y;
  else if (alone && valign === 'middle') y = area.y + (area.height - height) / 2;
  return { x: area.x, y, width: area.width, height };
}

/**
 * The rows of a placed block: the bar's row and the times with their box
 * and alignment.
 */
export function blockRows(content, block, m, shown) {
  const style = progressOf(content);
  const bar = barHeight(style);
  const [left, right] = shown.times ?? [null, null];
  const labels = [];
  if (shown.times && style.times === 'sides') {
    const right_ = block.x + block.width;
    if (left !== null) labels.push({ text: left, align: 'left', valign: 'middle', box: { x: block.x, y: block.y, width: m.left, height: block.height } });
    if (right !== null) labels.push({ text: right, align: 'right', valign: 'middle', box: { x: right_ - m.right, y: block.y, width: m.right, height: block.height } });
    const start = block.x + (left !== null ? m.left + m.gap : 0);
    const end = right_ - (right !== null ? m.right + m.gap : 0);
    return { row: { x: start, y: block.y + (block.height - bar) / 2, width: Math.max(0, end - start), height: bar }, labels };
  }
  if (shown.times && style.times === 'above') {
    const line = { x: block.x, y: block.y, width: block.width, height: m.line };
    if (left !== null) labels.push({ text: left, align: 'left', valign: 'top', box: line });
    if (right !== null) labels.push({ text: right, align: 'right', valign: 'top', box: line });
    return { row: { x: block.x, y: block.y + m.line + ABOVE_GAP, width: block.width, height: bar }, labels };
  }
  return { row: block, labels };
}

/** The bar in `row` at `fraction`: its box, blocks, radius, filled length and knob. */
export function barGeometry(content, row, fraction) {
  const style = progressOf(content);
  const bar = { x: row.x, y: row.y + (row.height - style.thickness) / 2, width: row.width, height: style.thickness };
  const d = knobDiameter(style);
  let knob = null;
  if (d > 0 && bar.width > 0) {
    const half = d / 2;
    const cx = Math.min(Math.max(bar.x + bar.width * fraction, bar.x + half), Math.max(bar.x + bar.width - half, bar.x + half));
    knob = { cx, cy: row.y + row.height / 2, d };
  }
  return { bar, blocks: progressBlocks(style, bar.width), radius: progressRadius(style), filled: bar.width * fraction, knob };
}

/** How the cover shares the box: all of it alone, else 40% and a gap. */
export function coverShare(content, shown, size) {
  const alone = shown.lines.length === 0 && shown.progress === null;
  return alone ? { fraction: 1, gap: 0 } : { fraction: COVER_SHARE, gap: Math.max(0, content.coverGap ?? size * 0.3) };
}

/** A timed session: the parts that show while something plays, without its words. */
const TIMED_MEDIA = Object.freeze({ title: '', artist: '', source: '', position: 0, duration: 1 });

/**
 * The cover's square in `frame`, at its top left, and the gap after it
 * (bezel-render TextFurniture::layout). The editor has no live session, so
 * `media` defaults to a timed one. Null without a cover.
 */
export function coverFrame(frame, content, size, media = TIMED_MEDIA) {
  if (content.showCover === false) return null;
  const { fraction, gap } = coverShare(content, playerText(content, media), size);
  const room = Math.min(gap, Math.max(0, frame.width));
  const side = Math.max(0, Math.min(frame.height, frame.width * fraction, frame.width - room));
  return { x: frame.x, y: frame.y, width: side, height: side, gap: room, alone: fraction === 1 };
}

/**
 * Natural size of a playing player: its visible parts. `text` is the
 * measured text block (`{width, height}`, null when empty), `metrics` the
 * block's, `old` the current frame. A side with nothing to measure keeps
 * the frame. Null when nothing shows.
 */
export function fitPlayer(content, shown, text, metrics, old, size) {
  const empty = shown.lines.length === 0;
  const t = empty ? { width: 0, height: 0 } : text;
  if (!t) return null;
  const reserve = metrics ? metrics.height + (empty ? 0 : PROGRESS_SPACING) : 0;
  const width = Math.max(t.width, metrics?.minWidth ?? 0);
  const height = t.height + reserve;
  if (content.showCover === false) {
    if (empty && !metrics) return null;
    return { width: width > 0 ? Math.ceil(width) : old.width, height: Math.ceil(height) };
  }
  if (empty && !metrics) {
    const side = Math.min(old.width, old.height);
    return { width: side, height: side };
  }
  if (empty) return { width: old.width, height: Math.ceil(Math.max(old.height, height)) };
  const { fraction, gap } = coverShare(content, shown, size);
  return { width: Math.ceil(Math.max(width + height + gap, height / fraction)), height: Math.ceil(height) };
}
