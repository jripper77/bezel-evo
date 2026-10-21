// Text in pieces with styles of their own, laid out like the native renderer
// (crates/bezel-render/src/text/rich.rs) for the demo's drawing: parts of a
// value or a clock (D-2026-10-20-studio-ux-v2-06), digits of equal width
// (D-2026-10-20-studio-ux-v2-05), lines wider than their box (shrink,
// ellipsis, scroll), blinking separators and rolling digits. Pure: widths
// come from a `measure(text, face)` the caller gives (a canvas in the demo).

/** Line height as a multiple of the size, like the backend's. */
export const LINE_HEIGHT = 1.2;
/** Height of a capital over the baseline, in sizes: a raised part's top meets the number's. */
export const CAP_HEIGHT = 0.7;
/** How fast a line scrolls, px a second. */
export const SCROLL_SPEED = 40;
/** Space after a scrolling line before it comes round again, in sizes. */
export const SCROLL_GAP = 2;
/** Time between two frames of a scrolling line, ms. */
export const SCROLL_STEP_MS = 50;
/** Half a blink: separators show for the first half of every second, ms. */
export const BLINK_HALF_MS = 500;
/** What ends a cut line. */
export const ELLIPSIS = '…';
/** The overflow modes a text can have (absent: drawn past its box). */
export const OVERFLOWS = Object.freeze(['shrink', 'ellipsis', 'scroll']);
/** Value parts, then clock parts. */
export const VALUE_PARTS = Object.freeze(['prefix', 'number', 'unit', 'suffix']);
export const CLOCK_PARTS = Object.freeze(['hours', 'minutes', 'seconds', 'separators']);

const isDigit = (ch) => ch >= '0' && ch <= '9' && ch.length === 1;
const sizeOf = (s) => (Number(s?.size) > 0 ? Number(s.size) : 24);

/** Whether a text element is laid out in pieces: a value, a clock, the weather, or a fixed text with an overflow. */
export function laidInPieces(kind) {
  const type = kind?.content?.type;
  return ['sensor', 'clock', 'weather'].includes(type) || (type === 'static' && OVERFLOWS.includes(kind.style?.overflow));
}

/** The style of `part` if it changes something, else null. */
export function partStyle(style, part) {
  const p = style?.parts?.[part];
  if (!p) return null;
  const { blink, ...rest } = p;
  return Object.values(rest).some((v) => v !== undefined && v !== null && v !== '' && v !== 'base') ? rest : null;
}

/** Whether a clock's separators blink. */
export const blinks = (style) => style?.parts?.separators?.blink === true;

/** Whether separators are off at `ms` (the second half of each second), with motion allowed. */
export const separatorsOff = (style, ms, motion) => motion && blinks(style) && ((ms % 1000) + 1000) % 1000 >= BLINK_HALF_MS;

/** When blinking separators next change after `ms`, ms from it; null without. */
export const blinkDue = (style, ms, motion) => (motion && blinks(style) ? BLINK_HALF_MS - (((ms % BLINK_HALF_MS) + BLINK_HALF_MS) % BLINK_HALF_MS) : null);

/** The face, colour and raise of a piece with `part` (null: the text's own). */
export function pieceFace(style, part, scale = 1) {
  const base = sizeOf(style) * scale;
  const pct = Number(part?.sizePercent);
  const size = Number.isFinite(pct) && pct > 0 ? base * Math.min(400, Math.max(10, pct)) / 100 : base;
  return {
    family: part?.font || style?.font?.family || 'sans-serif',
    weight: part?.weight ?? style?.font?.weight ?? 400,
    italic: !!style?.font?.italic,
    size,
    color: part?.color ?? null,
    raise: part?.baseline === 'super' ? Math.max(0, (base - size) * CAP_HEIGHT) : 0,
  };
}

function metrics(size) {
  const height = Math.ceil(size * LINE_HEIGHT);
  return { height, ascent: (height - size) / 2 + size * 0.8 };
}

function measureLine(line, fallback) {
  line.width = line.frags.reduce((w, f) => w + f.width, 0);
  if (!line.frags.length) { line.ascent = fallback.ascent; line.height = fallback.height; return line; }
  line.ascent = Math.max(...line.frags.map((f) => f.ascent + f.raise));
  line.height = line.ascent + Math.max(...line.frags.map((f) => f.height - f.ascent - f.raise));
  return line;
}

/**
 * A chunk of one piece on one line: cells of text, each digit its own cell
 * of the widest digit's advance when `tabular` (or of its own width while it
 * rolls). `count` numbers the piece's digits.
 */
function frag(text, face, measure, tabular, rolling, count) {
  const digitAdvance = tabular ? Math.max(...'0123456789'.split('').map((d) => measure(d, face))) : 0;
  const cells = [];
  let x = 0, run = '';
  const flush = () => {
    if (!run) return;
    const w = measure(run, face);
    cells.push({ text: run, x, slot: w, w, digit: null });
    x += w; run = '';
  };
  for (const ch of text) {
    if (isDigit(ch) && (tabular || rolling)) {
      flush();
      const w = measure(ch, face);
      const slot = tabular ? Math.max(digitAdvance, w) : w;
      cells.push({ text: ch, x, slot, w, digit: count.n++ });
      x += slot;
    } else run += ch;
  }
  flush();
  const { height, ascent } = metrics(face.size);
  return { face, cells, width: x, height, ascent, raise: face.raise, hidden: false, wheels: null };
}

/**
 * Lays `pieces` (`{text, part, hidden, wheels}`) on lines at `scale` times
 * their size: `[{frags, width, ascent, height}]`; null when there is no text.
 */
export function layoutText(pieces, style, measure, { tabular = true, scale = 1 } = {}) {
  const lines = [{ frags: [] }];
  for (const piece of pieces) {
    const face = pieceFace(style, piece.part, scale);
    const count = { n: 0 };
    String(piece.text ?? '').split('\n').forEach((chunk, i) => {
      if (i > 0) lines.push({ frags: [] });
      if (!chunk) return;
      const f = frag(chunk, face, measure, tabular, !!piece.wheels, count);
      f.hidden = !!piece.hidden; f.wheels = piece.wheels ?? null; f.id = piece.id ?? null;
      lines[lines.length - 1].frags.push(f);
    });
  }
  const fallback = metrics(sizeOf(style) * scale);
  lines.forEach((line) => measureLine(line, fallback));
  return lines.some((l) => l.frags.length) ? lines : null;
}

/** Cuts `line` to `width`, ending with an ellipsis in the text's face. */
function ellipsize(line, width, style, measure) {
  const face = pieceFace(style, null);
  const end = frag(ELLIPSIS, face, measure, false, false, { n: 0 });
  const room = width - end.width;
  while (line.frags.length) {
    const last = line.frags[line.frags.length - 1];
    const before = line.frags.slice(0, -1).reduce((w, f) => w + f.width, 0);
    while (last.cells.length) {
      const cell = last.cells[last.cells.length - 1];
      if (before + cell.x + cell.slot <= room) break;
      if (cell.digit === null && cell.text.length > 1) {
        cell.text = [...cell.text].slice(0, -1).join('');
        cell.w = cell.slot = measure(cell.text, last.face);
      } else last.cells.pop();
    }
    const tail = last.cells[last.cells.length - 1];
    last.width = tail ? tail.x + tail.slot : 0;
    if (last.cells.length) break;
    line.frags.pop();
  }
  end.face = { ...end.face, color: line.frags[line.frags.length - 1]?.face.color ?? null };
  line.frags.push(end);
  line.width = line.frags.reduce((w, f) => w + f.width, 0);
}

/**
 * Everything the demo needs to draw `pieces` in `box`: the lines, where
 * each line starts (two starts while it scrolls), the top of the text and
 * when the next frame is due for a scrolling line (ms; null without).
 * `seconds` is how long the theme has shown; without `motion` a long line
 * holds still at its start.
 */
export function fitText(pieces, style, box, measure, { tabular = true, seconds = 0, motion = true } = {}) {
  let lines = layoutText(pieces, style, measure, { tabular });
  if (!lines) return null;
  const widest = () => Math.max(...lines.map((l) => l.width));
  const overflow = OVERFLOWS.includes(style?.overflow) && widest() > box.width && box.width > 0 ? style.overflow : null;
  if (overflow === 'shrink') {
    let scale = 1;
    for (let i = 0; i < 3 && widest() > box.width; i++) {
      scale *= box.width / widest();
      lines = layoutText(pieces, style, measure, { tabular, scale });
    }
  }
  if (overflow === 'ellipsis') lines.filter((l) => l.width > box.width).forEach((l) => ellipsize(l, box.width, style, measure));
  const gap = sizeOf(style) * SCROLL_GAP;
  const travelled = motion ? seconds * SCROLL_SPEED : 0;
  let dueMs = null;
  const starts = lines.map((line) => {
    if (overflow === 'scroll' && line.width > box.width) {
      const turn = line.width + gap;
      const left = box.x - (travelled % turn);
      if (motion) dueMs = SCROLL_STEP_MS;
      return [left, left + turn];
    }
    const free = box.width - line.width;
    return [box.x + (style?.align === 'center' ? free / 2 : style?.align === 'right' ? free : 0)];
  });
  const total = lines.reduce((h, l) => h + l.height, 0);
  const top = box.y + (style?.valign === 'middle' ? (box.height - total) / 2 : style?.valign === 'bottom' ? box.height - total : 0);
  return { lines, starts, top, dueMs };
}

/** Natural size of `pieces`: widest line and total height (ceil, at least 1). */
export function measureText(pieces, style, measure, options) {
  const lines = layoutText(pieces, style, measure, options);
  if (!lines) return null;
  return { width: Math.max(1, Math.ceil(Math.max(...lines.map((l) => l.width)))), height: Math.max(1, Math.ceil(lines.reduce((h, l) => h + l.height, 0))) };
}
