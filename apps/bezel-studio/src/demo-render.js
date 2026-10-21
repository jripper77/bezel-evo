import { gaugeBlocks, barSpan } from './gauge-layout.js';
import { drawTextShadow } from './text-shadow.js';
import { appliedRotation, centerOf } from './editor/rotation.js';
import { drawVisualization } from './demo-visualization.js';
import { applyTextCase } from './text-case.js';
import { boundKey } from './editor/widgets.js';
import { demoSensorValue, DEMO_SENSORS, DEMO_SYSTEM_TEXT } from './demo-sensors.js';
import { formatValue } from './value-format.js';
import { demoColorValue, withValueColor } from './value-color.js';
import { demoSpectrum, spectrumOf } from './spectrum.js';
import { translator } from './i18n/index.js';
import { colorAt } from './value-color.js';
import { cardPoses, drawCardProjection } from './editor/card-motion.js';
import { isShown, effectiveOpacity, visibleWithoutFace } from './editor/cards.js';
import { fadeLine } from './shape-fade.js';
import { weatherFamily, weatherText } from './weather-format.js';
import { HAND_LENGTHS, analogOf, clockParts, dialDate, dialDue, dialNumerals, formatClock, handAngles } from './clock-format.js';
import { blinkDue, fitText, laidInPieces, measureText, partStyle, separatorsOff } from './text-layout.js';
import { sunCenter, weatherIconParts, weatherIconPaths } from './weather-icon.js';
import { REST, amplitudeOf, iconPose, iconStep } from './weather-motion.js';
import { demoPeak, faint, fitGauge, gradientAt, lookOf, needleColor, numberShares, scaleNumber, share, tickShares } from './needle-look.js';
import { demoRoll, easedDemoValue, rollWheels } from './value-transition.js';
import { barGeometry, blockMetrics, blockRows, coverShare, fitPlayer, placeBlock, playerText, progressOf, PROGRESS_SPACING } from './player-parts.js';
// An approximate renderer for demo mode (browser only). The real preview is
// the Rust renderer's frame; this only has to look plausible and be fast.

function color(hex) {
  if (typeof hex !== 'string' || !hex.startsWith('#')) return 'transparent';
  const h = hex.slice(1);
  const n = (i) => parseInt(h.slice(i, i + 2), 16);
  const a = h.length === 8 ? n(6) / 255 : 1;
  return `rgba(${n(0)},${n(2)},${n(4)},${a})`;
}

function paint(p) {
  if (typeof p === 'string') return color(p);
  if (p && p.stops) return color(p.stops[0]?.[1]);
  return 'transparent';
}

function canvasPaint(ctx, value, frame) {
  if (!value?.stops) return paint(value);
  const gradient = ctx.createLinearGradient(...fadeLine(frame, value.angle ?? 0));
  for (const [at, hex] of [...value.stops].sort((a, b) => a[0] - b[0])) gradient.addColorStop(Math.max(0, Math.min(1, at)), color(hex));
  return gradient;
}

/** How long each frame of the demo's animated GIFs (every `*.gif` image) lasts, ms. */
export const DEMO_GIF_FRAME_MS = 100;
/** The colors the demo's GIFs cycle through, one per frame. */
const GIF_COLORS = ['#e4572e', '#f3a712', '#29bf12', '#4361ee'];
/** How long one play of a demo GIF lasts, ms. */
const GIF_TOTAL_MS = DEMO_GIF_FRAME_MS * GIF_COLORS.length;
const gifSpeed = (kind) => Math.min(4, Math.max(0.25, Number.isFinite(kind?.speed) ? kind.speed : 1));

/** The frame of an image object's demo GIF `ms` into the theme, with its speed and `once`, like the backend. */
export function demoGifFrame(kind, ms) {
  const local = ms * gifSpeed(kind);
  if (kind?.once && local >= GIF_TOTAL_MS) return GIF_COLORS.length - 1;
  return Math.floor(local / DEMO_GIF_FRAME_MS) % GIF_COLORS.length;
}

/** When that GIF next changes, ms from `ms`; null once a single play is over. */
export function demoGifDue(kind, ms) {
  const local = ms * gifSpeed(kind);
  const next = (Math.floor(local / DEMO_GIF_FRAME_MS) + 1) * DEMO_GIF_FRAME_MS;
  if (kind?.once && next >= GIF_TOTAL_MS) return null;
  return (next - local) / gifSpeed(kind);
}

function fraction(t, i) {
  return 0.5 + 0.4 * Math.sin(t / 3 + i);
}

/**
 * The value element `e` shows at `t` seconds, as a fraction: its reading
 * at the last sample (every `refresh` s), eased from the one before when
 * it has a smooth transition and motion is allowed, like the backend
 * (D-2026-10-15-studio-redesign-fidelity-18).
 */
function shownFraction(e, t, refresh, motion) {
  const key = boundKey(e);
  if (key) {
    const every = refresh > 0 ? refresh : 1;
    if (!Number.isFinite(demoSensorValue(key, Math.floor(t / every) * every))) return NaN;
    const value = easedDemoValue(e, s => demoSensorValue(key, s), t, refresh, motion);
    const b = e.kind.binding;
    return b ? Math.max(0, Math.min(1, (value - b.min) / (b.max - b.min))) : value / 100;
  }
  return easedDemoValue(e, (s) => fraction(s, e.id), t, refresh, motion);
}

/**
 * The demo's points of a graph at `t` seconds, oldest first, as fractions
 * of its height: like the backend (D-2026-10-15-studio-redesign-fidelity-16),
 * `history` points one `intervalSeconds` apart (one per `refreshSeconds`
 * without one), the newest taken at the last whole interval, so a graph
 * moves at its own pace.
 */
export function demoGraphPoints(kind, refreshSeconds, t, seed) {
  const every = kind.intervalSeconds > 0 ? kind.intervalSeconds : refreshSeconds > 0 ? refreshSeconds : 1;
  const count = Math.max(2, Math.min(3600, Math.round(kind.history) || 2));
  const newest = Math.floor(t / every) * every;
  return Array.from({ length: count }, (_, i) => fraction(newest - (count - 1 - i) * every, seed));
}

/** The session the demo's players show (always playing, 40% through). */
const DEMO_MEDIA = Object.freeze({ title: 'Demo track', artist: 'Demo artist', source: 'Demo media app', position: 84, duration: 210 });

// One decoration model for drawing and fitting in the browser demo.
function textFurniture({ content: c, style: s }) {
  const cover = c.type === 'player' && c.showCover !== false;
  const icon = c.type === 'weather' && c.showIcon !== false;
  const share = cover ? coverShare(c, playerText(c, DEMO_MEDIA), s.size) : null;
  const fraction = cover ? share.fraction : icon ? (c.iconSize == null ? 0.3 : 1) : 0;
  const desired = cover ? Infinity : icon ? c.iconSize ?? s.size * 2 : 0;
  const gap = Math.max(0, cover ? share.gap : icon ? c.iconGap ?? s.size * 0.3 : 0);
  return {
    layout: f => ({ gap: Math.min(gap, f.width), side: Math.min(desired, f.height, f.width * fraction, Math.max(0, f.width - gap)) }),
    fit: (width, height) => {
      const side = Number.isFinite(desired) ? desired : height;
      return { width: Math.ceil(fraction ? Math.max(width + side + gap, side / fraction) : width), height: Math.ceil(Math.max(height, side)) };
    },
  };
}

/** One time of a player measured with the canvas font. */
const timeLabel = (ctx, s) => (text) => ({ width: Math.ceil(ctx.measureText(text).width), height: Math.ceil(s.size * 1.2) });

/** The player's progress block in `area`; returns the area left for the text. */
function drawProgress(ctx, c, s, area, shown) {
  const m = blockMetrics(c, DEMO_MEDIA, shown, timeLabel(ctx, s), s.size);
  if (!m) return area;
  const alone = shown.lines.length === 0;
  const block = placeBlock(area, m.height, alone, s.valign ?? 'top');
  const { row, labels } = blockRows(c, block, m, shown);
  const style = progressOf(c);
  const { bar, blocks, radius, filled, knob } = barGeometry(c, row, shown.progress);
  const fill = style.fill ? color(style.fill) : paint(s.paint);
  const path = new Path2D();
  for (const [a, b] of blocks) path.roundRect(bar.x + a, bar.y, b - a, bar.height, Math.min(radius, (b - a) / 2));
  ctx.save();
  ctx.fillStyle = color(style.track);
  ctx.fill(path);
  ctx.clip(path);
  ctx.fillStyle = fill;
  ctx.fillRect(bar.x, bar.y, filled, bar.height);
  ctx.restore();
  if (knob) { ctx.fillStyle = fill; ctx.beginPath(); ctx.arc(knob.cx, knob.cy, knob.d / 2, 0, Math.PI * 2); ctx.fill(); }
  ctx.fillStyle = paint(s.paint);
  for (const label of labels) {
    ctx.textAlign = label.align;
    ctx.textBaseline = label.valign === 'top' ? 'top' : 'middle';
    const x = label.align === 'right' ? label.box.x + label.box.width : label.box.x;
    const y = label.valign === 'top' ? label.box.y : label.box.y + label.box.height / 2;
    ctx.fillText(label.text, x, y);
  }
  ctx.textAlign = s.align === 'center' ? 'center' : s.align === 'right' ? 'right' : 'left';
  return { ...area, height: Math.max(0, area.height - m.height - (alone ? 0 : PROGRESS_SPACING)) };
}

/** One part of a weather icon (or, at rest, a whole component) in 24-unit space. */
function drawIconComponent(ctx, component, style, colour) {
  const colored = ['colored', 'dimensional'].includes(style);
  const path = new Path2D(component.path);
  ctx.fillStyle = ctx.strokeStyle = colored ? component.light : colour;
  if (colored && style === 'dimensional') {
    if (component.fill) { ctx.save(); ctx.translate(.5, .9); ctx.fillStyle = 'rgba(0,0,0,0.255)'; ctx.fill(path); ctx.restore(); }
    const g = ctx.createLinearGradient(...fadeLine({ x: 0, y: 0, width: 24, height: 24 }, 65));
    g.addColorStop(0, component.light); g.addColorStop(1, component.dark);
    ctx.fillStyle = ctx.strokeStyle = g;
  }
  if (colored ? component.fill : style === 'filled') ctx.fill(path);
  ctx.stroke(path);
}

/**
 * A weather icon like the backend's (crates/bezel-render/src/weather.rs): at
 * rest as one icon, moving with each part turned, shifted and faded by
 * `pose`, and the 3D shadow, halo and flash.
 */
export function drawWeatherIcon(ctx, style, colour, x, y, side, code, pose = REST) {
  const colored = ['colored', 'dimensional'].includes(style);
  ctx.save();
  ctx.translate(x, y);
  ctx.scale(side / 24, side / 24);
  ctx.lineWidth = Math.max(colored ? 1.5 : 1.6, 24 / side);
  ctx.lineCap = colored ? 'round' : 'butt';
  if (pose === REST) {
    for (const component of weatherIconPaths(code, colored)) drawIconComponent(ctx, component, style, colour);
    ctx.restore();
    return;
  }
  const k = 24 / 64, [cx, cy] = sunCenter(colored), family = weatherFamily(code);
  ctx.globalAlpha *= pose.whole;
  if (style === 'dimensional') {
    const follow = family >= 1 && family <= 6 ? pose.cloud : 0;
    ctx.beginPath(); ctx.ellipse((33 + follow) * k, 58 * k, 16 * k, 2.6 * k, 0, 0, Math.PI * 2);
    ctx.fillStyle = 'rgba(0,0,0,0.196)'; ctx.fill();
    if (pose.halo > 0 && family <= 1) {
      ctx.beginPath(); ctx.arc(cx, cy, 5.5, 0, Math.PI * 2);
      ctx.fillStyle = `rgba(255,213,79,${pose.halo})`; ctx.fill();
    }
  }
  for (const part of weatherIconParts(code, colored)) {
    ctx.save();
    if (part.role === 'disc') { ctx.translate(cx, cy); ctx.scale(1 + pose.disc, 1 + pose.disc); ctx.translate(-cx, -cy); }
    if (part.role === 'rays') { ctx.translate(cx, cy); ctx.rotate(pose.rays * Math.PI / 180); ctx.translate(-cx, -cy); }
    if (part.role === 'cloud') ctx.translate(pose.cloud * k, 0);
    if (part.role === 'strip') { const [dx, o] = pose.strips[part.index]; ctx.translate(dx * k, 0); ctx.globalAlpha *= o; }
    if (part.role === 'drop') { const [dx, dy, o] = pose.drops[part.index]; ctx.translate(dx * k, dy * k); ctx.globalAlpha *= o; }
    if (part.role === 'bolt') ctx.globalAlpha *= pose.bolt;
    drawIconComponent(ctx, part, style, colour);
    ctx.restore();
  }
  if (style === 'dimensional' && pose.flash > 0) { ctx.fillStyle = `rgba(255,255,255,${pose.flash})`; ctx.fillRect(0, 0, 24, 24); }
  ctx.restore();
}

/** When the texts drawn by this render next want a frame (scrolling lines, blinking separators), ms. */
let textDue = null;
const wantFrame = (ms) => { if (ms !== null) textDue = textDue === null ? ms : Math.min(textDue, ms); };

const fontOf = (face) => `${face.italic ? 'italic ' : ''}${face.weight} ${face.size}px ${face.family}, sans-serif`;
const canvasMeasure = (ctx) => (text, face) => { ctx.font = fontOf(face); return ctx.measureText(text).width; };

/**
 * The pieces of a value, a clock, the weather or a fixed text that
 * overflows, like the backend's (D-2026-10-20-studio-ux-v2-05/06): parts
 * with their style, separators off in the second half of a second, a
 * rolling number's wheels.
 */
function textPieces(e, text, t, value, refresh, motion, trial) {
  const k = e.kind, s = k.style;
  const piece = (body, part = null, extra = {}) => ({ text: applyTextCase(body, s.casing), part: part && partStyle(s, part), hidden: false, wheels: null, ...extra });
  if (k.content.type === 'sensor') {
    const key = boundKey(e);
    const roll = key && !trial ? demoRoll(e, (at) => demoSensorValue(key, at), t, refresh, motion) : null;
    const formatted = formatValue(DEMO_SYSTEM_TEXT[key] ?? (roll ? roll.to : value * 100), DEMO_SENSORS.find(row => row[0] === key)?.[3], k.content.format);
    const { number, unit } = formatted;
    const decimals = number.split('.')[1]?.length ?? 0;
    const wheels = roll ? rollWheels(roll, decimals, [...number].filter((ch) => ch >= '0' && ch <= '9').length) : null;
    return [piece(k.content.prefix ?? '', 'prefix'), piece(number, 'number', { wheels }), piece(unit, 'unit'), piece(k.content.suffix ?? '', 'suffix')];
  }
  if (k.content.type === 'clock') {
    const ms = t * 1000;
    wantFrame(blinkDue(s, ms, motion));
    const d = new Date((t + (trial?.clockOffsetSeconds ?? 0)) * 1000);
    return clockParts(k.content.pattern, d, k.content.language, k.content.casing)
      .map(({ part, text: body }) => piece(body, part, { hidden: part === 'separators' && separatorsOff(s, ms, motion) }));
  }
  return [piece(text)];
}

/** Draws `pieces` in `f` with the backend's layout; a rolling digit turns inside its line. */
function drawPieces(ctx, s, f, pieces, t, motion, tabular) {
  const measure = canvasMeasure(ctx);
  const fit = fitText(pieces, s, f, measure, { tabular, seconds: t, motion });
  if (!fit) return;
  wantFrame(fit.dueMs);
  ctx.save();
  ctx.beginPath(); ctx.rect(f.x, f.y, f.width, f.height); ctx.clip();
  ctx.textAlign = 'left'; ctx.textBaseline = 'alphabetic';
  let top = fit.top;
  fit.lines.forEach((line, i) => {
    const baseline = top + line.ascent;
    for (const start of fit.starts[i]) {
      let x = start;
      for (const frag of line.frags) {
        if (!frag.hidden) {
          ctx.font = fontOf(frag.face);
          ctx.fillStyle = frag.face.color ? color(frag.face.color) : paint(s.paint);
          const y = baseline - frag.raise;
          for (const cell of frag.cells) {
            const left = x + cell.x;
            const wheel = cell.digit !== null ? frag.wheels?.[cell.digit] : null;
            if (!wheel) { ctx.fillText(cell.text, left + (cell.slot - cell.w) / 2, y); continue; }
            const [digit, turn] = wheel;
            ctx.save();
            ctx.beginPath(); ctx.rect(left - 1, top, cell.slot + 2, line.height); ctx.clip();
            for (const [n, dy] of [[digit, -turn * line.height], [(digit + 1) % 10, (1 - turn) * line.height]]) {
              if (n !== digit && turn <= 0) continue;
              const ch = String(n);
              ctx.fillText(ch, left + (cell.slot - ctx.measureText(ch).width) / 2, y + dy);
            }
            ctx.restore();
          }
        }
        x += frag.width;
      }
    }
    top += line.height;
  });
  ctx.restore();
}

/**
 * An analog clock like the backend's (crates/bezel-render/src/analog_clock.rs):
 * the design's 64-unit dial scaled to the box, marks, numbers, date, hands
 * and hub.
 */
function drawDial(ctx, e, t, motion, trial) {
  const k = e.kind, a = analogOf(k.content.analog), f = e.frame;
  const unit = Math.max(0, Math.min(f.width, f.height)) / 64;
  if (!(unit > 0)) return;
  const date = new Date((t + (trial?.clockOffsetSeconds ?? 0)) * 1000);
  const cx = f.x + f.width / 2, cy = f.y + f.height / 2;
  const at = (radius, angle) => [cx + radius * unit * Math.sin(angle * Math.PI / 180), cy - radius * unit * Math.cos(angle * Math.PI / 180)];
  const line = (from, to, width, colour, cap = 'butt') => {
    ctx.beginPath(); ctx.moveTo(...from); ctx.lineTo(...to);
    ctx.lineWidth = width; ctx.lineCap = cap; ctx.strokeStyle = colour; ctx.stroke();
  };
  ctx.beginPath(); ctx.arc(cx, cy, 28 * unit, 0, Math.PI * 2);
  ctx.fillStyle = color(a.face); ctx.fill();
  if (a.border > 0) { ctx.save(); ctx.globalAlpha *= 0.3; ctx.lineWidth = a.border; ctx.strokeStyle = color(a.marks); ctx.stroke(); ctx.restore(); }
  const count = a.ticks === 'minutes' ? 60 : a.ticks === 'hours' ? 12 : 0;
  for (let i = 0; i < count; i++) {
    const hourly = count === 12 || i % 5 === 0, angle = i * 360 / count;
    line(at(26, angle), at(hourly ? 21 : 24, angle), (hourly ? 2 : 1) * unit, color(a.marks));
  }
  const font = (size) => `${k.style?.font?.italic ? 'italic ' : ''}${k.style?.font?.weight ?? 400} ${size}px ${k.style?.font?.family ?? 'sans-serif'}, sans-serif`;
  ctx.fillStyle = color(a.marks); ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
  const roman = a.numbers === 'roman';
  ctx.font = font((roman ? 5 : 6.5) * unit);
  for (const [hour, text] of dialNumerals(a)) ctx.fillText(text, ...at((count ? 15.5 : 21) - (roman ? 1 : 0), hour * 30));
  const day = dialDate(a, date, k.content.language);
  if (day) { ctx.font = font(5 * unit); ctx.fillText(day, cx, cy + 9 * unit); }
  const angles = handAngles(a, date, motion);
  const hands = [[a.hourHand, HAND_LENGTHS.hour * a.hourLength, 3.5, 0, angles.hour, a.hourColor], [a.minuteHand, HAND_LENGTHS.minute * a.minuteLength, 2.5, 0, angles.minute, a.minuteColor], [a.secondHand, HAND_LENGTHS.second * a.secondLength, 1.2, 6, angles.second, a.secondColor]];
  for (const [shown, length, width, tail, angle, colour] of hands) {
    if (!shown) continue;
    const tip = at(length, angle), w = width * unit * (a.handShape === 'thin' ? 0.6 : a.handShape === 'arrow' ? 0.8 : 1);
    line(at(tail, angle + 180), tip, w, color(colour), a.handShape === 'rounded' ? 'round' : 'butt');
    if (a.handShape === 'arrow') {
      const r = angle * Math.PI / 180, head = Math.min(w * 3, length * unit * 0.5), side = w * 1.25;
      const base = [tip[0] - head * Math.sin(r), tip[1] + head * Math.cos(r)];
      ctx.beginPath(); ctx.moveTo(...tip);
      ctx.lineTo(base[0] + Math.cos(r) * side, base[1] + Math.sin(r) * side);
      ctx.lineTo(base[0] - Math.cos(r) * side, base[1] - Math.sin(r) * side);
      ctx.closePath(); ctx.fillStyle = color(colour); ctx.fill();
    }
  }
  const hub = [[a.secondHand, a.secondColor], [a.minuteHand, a.minuteColor], [a.hourHand, a.hourColor]].find(([shown]) => shown);
  if (hub) { ctx.beginPath(); ctx.arc(cx, cy, 2.5 * unit, 0, Math.PI * 2); ctx.fillStyle = color(hub[1]); ctx.fill(); }
  wantFrame(dialDue(a, date, motion));
}

/**
 * The extended needle like the backend's (crates/bezel-render/src/needle_gauge.rs):
 * arc, zones or gradient, ticks, numbers, peak mark, hand, cap and the
 * value under the pivot.
 */
function drawNeedleLook(ctx, e, frac, t, refresh, motion, trial = null) {
  const k = e.kind, look = lookOf(k.look), b = k.binding ?? { min: 0, max: 100 }, f = e.frame;
  const g = fitGauge(f, k.pivot ?? [0.5, 0.5], k.startAngle ?? 0, k.sweep ?? 0, look.arcWidth / 2 + 1);
  if (!(g.radius > 0)) return;
  const [px, py] = g.pivot;
  const angle = (s) => g.start + g.sweep * s;
  const at = (r, a) => [px + r * Math.sin(a * Math.PI / 180), py - r * Math.cos(a * Math.PI / 180)];
  const rad = (a) => (a - 90) * Math.PI / 180;
  const arc = (from, to, colour, cap) => {
    ctx.beginPath();
    const a = angle(from), z = angle(to);
    ctx.arc(px, py, g.radius, rad(Math.min(a, z)), rad(Math.max(a, z)));
    ctx.lineWidth = look.arcWidth; ctx.lineCap = cap === 'round' ? 'round' : 'butt'; ctx.strokeStyle = color(colour); ctx.stroke();
  };
  if (look.arcWidth > 0) {
    if (look.track === 'gradient') {
      const steps = Math.max(1, Math.ceil(Math.abs(g.sweep) / 2));
      for (let i = 0; i < steps; i++) {
        const a = i / steps, z = Math.min(1, (i + 1) / steps + 0.002);
        arc(a, z, gradientAt(look, b, (a + z) / 2), i === 0 || i + 1 === steps ? look.arcCap : 'butt');
      }
    } else {
      arc(0, 1, faint(look), look.arcCap);
      if (look.track === 'zones') for (const zone of look.zones.slice(0, 8)) {
        const a = share(b, zone.from), z = share(b, zone.to);
        if (Math.abs(a - z) > 1e-9) arc(Math.min(a, z), Math.max(a, z), zone.color, 'butt');
      }
    }
  }
  const inner = g.radius - look.arcWidth / 2 - 2, long = Math.max(4, g.radius * 0.1);
  const { major, minor } = tickShares(look);
  ctx.strokeStyle = color(look.marks); ctx.lineCap = 'butt';
  for (const [shares, length, width] of [[major, long, Math.max(1.5, g.radius / 60)], [minor, long / 2, Math.max(1, g.radius / 120)]]) {
    for (const s of shares) { ctx.beginPath(); ctx.moveTo(...at(inner, angle(s))); ctx.lineTo(...at(inner - length, angle(s))); ctx.lineWidth = width; ctx.stroke(); }
  }
  const size = Math.min(64, Math.max(8, g.radius * 0.12));
  const style = look.center?.style ?? {};
  ctx.font = `${style.font?.weight ?? 400} ${size}px ${style.font?.family ?? 'Inter'}, sans-serif`;
  ctx.fillStyle = color(look.marks); ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
  const language = globalThis.document?.documentElement?.lang ?? 'en';
  for (const s of numberShares(look)) ctx.fillText(scaleNumber(b.min + (b.max - b.min) * s, language), ...at(inner - (major.length ? long : 0) - look.numberGap - size * 0.6, angle(s)));
  const value = Number.isFinite(frac) ? b.min + frac * (b.max - b.min) : null;
  const reach = g.radius * look.length / 100;
  const key = boundKey(e);
  const peak = value === null ? null : demoPeak(look, (s) => (key ? demoSensorValue(key, s) : b.min + fraction(s, e.id) * (b.max - b.min)), t, refresh);
  if (peak !== null && peak > value) {
    ctx.save(); ctx.globalAlpha *= 0.35; ctx.setLineDash([Math.max(2, k.width * 0.8), Math.max(2, k.width * 0.8)]);
    ctx.beginPath(); ctx.moveTo(px, py); ctx.lineTo(...at(reach, angle(share(b, peak)))); ctx.lineWidth = Math.max(1, k.width * 0.5); ctx.strokeStyle = color(look.marks); ctx.stroke();
    ctx.restore();
  }
  const colour = color(needleColor(look, value, k.color));
  const a = angle(value === null ? 0 : share(b, value)), r = a * Math.PI / 180;
  const tip = at(reach, a), tail = at(g.radius * look.tail / 100, a + 180), [nx, ny] = [Math.cos(r), Math.sin(r)], w = k.width;
  ctx.fillStyle = ctx.strokeStyle = colour;
  const line = (width) => { ctx.beginPath(); ctx.moveTo(...tail); ctx.lineTo(...tip); ctx.lineWidth = width; ctx.lineCap = 'butt'; ctx.stroke(); };
  if (!k.asset) {
    if (look.shape === 'tapered') {
      const h = Math.max(1, w);
      ctx.beginPath(); ctx.moveTo(...tip); ctx.lineTo(px + nx * h, py + ny * h); ctx.lineTo(tail[0] + nx * h / 2, tail[1] + ny * h / 2);
      ctx.lineTo(tail[0] - nx * h / 2, tail[1] - ny * h / 2); ctx.lineTo(px - nx * h, py - ny * h); ctx.closePath(); ctx.fill();
    } else if (look.shape === 'arrow') {
      line(w);
      const dx = Math.sin(r), dy = -Math.cos(r), head = w * 3.5, base = [tip[0] - dx * head, tip[1] - dy * head];
      ctx.beginPath(); ctx.moveTo(tip[0] + dx * w, tip[1] + dy * w); ctx.lineTo(base[0] + nx * w * 1.5, base[1] + ny * w * 1.5);
      ctx.lineTo(base[0] - nx * w * 1.5, base[1] - ny * w * 1.5); ctx.closePath(); ctx.fill();
    } else if (look.shape === 'dot') {
      line(Math.max(1, w * 0.6)); ctx.beginPath(); ctx.arc(...tip, Math.max(2, w * 1.5), 0, Math.PI * 2); ctx.fill();
    } else line(w);
  }
  const capR = look.capDiameter / 2;
  if (look.cap !== 'none' && capR > 0) {
    ctx.beginPath(); ctx.arc(px, py, capR, 0, Math.PI * 2); ctx.fillStyle = color(look.capColor); ctx.fill();
    if (look.cap === 'ring') { ctx.lineWidth = Math.min(4, Math.max(1, capR * 0.45)); ctx.strokeStyle = colour; ctx.stroke(); }
  }
  if (look.center) {
    const s = look.center.style ?? {}, sz = Math.max(1, s.size ?? 24), half = Math.max(g.radius, sz * 2);
    const box = { x: px - half, y: py + capR + sz * 0.2, width: half * 2, height: sz * 1.3 };
    const text = { ...e, frame: box, kind: { type: 'text', content: { type: 'sensor', key: key ?? b.key, format: look.center.format, prefix: look.center.prefix ?? '', suffix: look.center.suffix ?? '' }, style: s } };
    const raw = trial ? value : easedDemoValue(e, at => demoSensorValue(key, at), t, refresh, motion);
    drawText(ctx, text, t, Number.isFinite(raw) ? raw / 100 : NaN, false, trial, refresh, motion);
  }
}

function drawText(ctx, e, t, value, measure = false, trial = null, refresh = 1, motion = true) {
  if (e.kind.content?.type === 'clock' && e.kind.content.analog) {
    if (measure) return null;
    drawDial(ctx, e, t, motion, trial);
    return;
  }
  const k = e.kind;
  const s = k.style;
  let text = k.content.text ?? '';
  if (k.content.type === 'clock') {
    const d = new Date((t + (trial?.clockOffsetSeconds ?? 0)) * 1000);
    text = formatClock(k.content.pattern, d, k.content.language, k.content.casing);
  } else if (k.content.type === 'weather') {
    text = weatherText(k.content, 20, trial?.weatherCode ?? 3);
  } else if (k.content.type === 'player') {
    text = playerText(k.content, trial ? { ...DEMO_MEDIA, position: trial.elapsedMs / 1000 } : DEMO_MEDIA).lines.join('\n');
  } else if (k.content.type === 'sensor') {
    text = `${k.content.prefix ?? ''}${Number.isFinite(value) ? Math.round(value * 100) : '\u2014'}${k.content.format?.showUnit === false ? '' : '%'}${k.content.suffix ?? ''}`;
  }
  text = applyTextCase(text, s.casing);
  ctx.fillStyle = paint(s.paint);
  ctx.font = `${s.font?.italic ? 'italic ' : ''}${s.font?.weight ?? 400} ${s.size}px ${s.font?.family ?? 'sans-serif'}, sans-serif`;
  if (measure && k.content.type === 'player') {
    const shown = playerText(k.content, DEMO_MEDIA);
    const lines = shown.lines.length ? applyTextCase(shown.lines.join('\n'), s.casing).split('\n') : [];
    const block = lines.length ? { width: Math.max(...lines.map(line => ctx.measureText(line).width)), height: lines.length * Math.ceil(s.size * 1.2) } : null;
    const m = blockMetrics(k.content, DEMO_MEDIA, shown, timeLabel(ctx, s), s.size);
    return fitPlayer(k.content, shown, block, m, e.frame, s.size);
  }
  const pieces = laidInPieces(k) || k.content.type === 'static';
  const tabular = s.tabularDigits !== false && k.content.type !== 'static';
  if (measure && pieces) {
    const size = measureText(textPieces(e, text, t, value, refresh, false, trial), s, canvasMeasure(ctx), { tabular });
    return size && textFurniture(k).fit(size.width, size.height);
  }
  if (measure) {
    const lines = text.split('\n');
    const width = Math.max(...lines.map(line => ctx.measureText(line).width));
    const height = lines.length * Math.ceil(s.size * 1.2);
    return textFurniture(k).fit(width, height);
  }
  ctx.textAlign = s.align === 'center' ? 'center' : s.align === 'right' ? 'right' : 'left';
  ctx.textBaseline = s.valign === 'top' ? 'top' : s.valign === 'bottom' ? 'bottom' : 'middle';
  let f = e.frame;
  if (k.content.type === 'weather' && k.content.showIcon !== false) {
    const { side, gap } = textFurniture(k).layout(f);
    const code = trial?.weatherCode ?? 3;
    const moving = motion && !trial ? iconStep(k.content.iconMotion) : null;
    const pose = moving ? iconPose(weatherFamily(code), t, amplitudeOf(k.content.iconMotion), k.content.iconSpeed ?? 1) : REST;
    if (moving) wantFrame(moving);
    drawWeatherIcon(ctx, k.content.iconStyle, paint(['colored', 'dimensional'].includes(k.content.iconStyle) ? s.paint : k.content.iconColor ?? s.paint), f.x, f.y + (f.height - side) / 2, side, code, pose);
    f = { ...f, x: f.x + side + gap, width: Math.max(0, f.width - side - gap) };
  }
  if (k.content.type==='player') {
    ctx.textBaseline='top';
    if (k.content.showCover !== false) {
      const { side, gap } = textFurniture(k).layout(f);
      drawPlayerCover(ctx, { x: f.x, y: f.y, width: side, height: side }, k.content, target => {
        target.fillStyle = '#334155'; target.fillRect(f.x, f.y, side, side);
        target.fillStyle = paint(s.paint); target.font = ctx.font; target.textBaseline = 'top';
        target.fillText('\u266b', f.x + side * .3, f.y + side * .3);
      });
      f = { ...f, x: f.x + side + gap, width: f.width - side - gap };
    }
    f = drawProgress(ctx, k.content, s, f, playerText(k.content, trial ? { ...DEMO_MEDIA, position: trial.elapsedMs / 1000 } : DEMO_MEDIA));
  }
  const drawGlyphs = (target, style) => {
    target.font = `${style.font?.italic ? 'italic ' : ''}${style.font?.weight ?? 400} ${style.size}px ${style.font?.family ?? 'sans-serif'}, sans-serif`;
    target.textAlign = style.align === 'center' ? 'center' : style.align === 'right' ? 'right' : 'left';
    target.textBaseline = style.valign === 'top' ? 'top' : style.valign === 'bottom' ? 'bottom' : 'middle';
    if (pieces) { drawPieces(target, style, f, textPieces({ ...e, kind: { ...k, style } }, text, t, value, refresh, motion, trial), t, motion, tabular); return; }
    target.fillStyle = paint(style.paint);
    const x = style.align === 'center' ? f.x + f.width / 2 : style.align === 'right' ? f.x + f.width : f.x;
    const y = style.valign === 'top' ? f.y : style.valign === 'bottom' ? f.y + f.height : f.y + f.height / 2;
    if (['weather', 'player'].includes(k.content.type)) {
      const lines = text.split('\n'), lineHeight = Math.ceil(style.size * 1.2);
      const top = style.valign === 'top' ? f.y : style.valign === 'bottom' ? f.y + f.height - lines.length * lineHeight : f.y + (f.height - lines.length * lineHeight) / 2;
      target.textBaseline = 'top';
      lines.forEach((line, i) => target.fillText(line, x, top + i * lineHeight, f.width));
    } else target.fillText(text, x, y, f.width);
  };
  drawTextShadow(ctx, s, f, drawGlyphs);
  drawGlyphs(ctx, s);
}

function drawSpectrum(ctx, e, seconds) {
  const c = spectrumOf(e.kind.content), f = e.frame, mid = f.y + f.height / 2, bottom = f.y + f.height;
  const values = (c.demo ? demoSpectrum(c, seconds) : Array(c.bands).fill(0)).map(v => Math.min(1, v * c.gain));
  const tr = translator(globalThis.document?.documentElement?.lang ?? 'en');
  ctx.save(); ctx.globalAlpha = e.opacity ?? 1; ctx.beginPath(); ctx.rect(f.x,f.y,f.width,f.height); ctx.clip();
  const fill = e.kind.style.paint;
  let base = paint(fill);
  if (fill?.stops) { base = ctx.createLinearGradient(f.x, f.y, f.x, f.y + f.height); for (const [stop,hex] of fill.stops) base.addColorStop(stop,color(hex)); }
  // Same geometry as the screen renderer (bezel-render spectrum.rs, D-AS-11).
  const level = value => e.colorByValue ? color(colorAt(e.colorByValue, value * 100)) : base;
  const cell = f.width / c.bands, width = Math.max(0,Math.min(c.thickness,cell-c.gap));
  const peakOf = value => Math.min(1, value + .08);
  const bar = (x, y, w, h) => { ctx.beginPath(); ctx.roundRect(x, y, w, h, c.corners.map(v => Math.min(v, w / 2, h / 2))); ctx.fill(); };
  ctx.fillStyle = base;
  if (c.style === 'mirrored') {
    if (c.centre) ctx.fillRect(f.x,mid-.5,f.width,1);
    values.forEach((value,i) => {
      const x=f.x+i*cell+(cell-width)/2, h=value*Math.max(0,f.height/2-2);
      ctx.fillStyle = level(value);
      if(h>0) { ctx.beginPath(); ctx.roundRect(x,mid-h,width,h*2,c.corners.map(v=>Math.min(v,width/2,h))); ctx.fill(); }
      if(c.peaks && value>0) { const peak=peakOf(value)*Math.max(0,f.height/2-2);ctx.fillRect(x,mid-peak-1,width,1);ctx.fillRect(x,mid+peak,width,1); }
    });
  } else if (c.style === 'bars') {
    if (c.centre) ctx.fillRect(f.x, bottom - 1, f.width, 1);
    const full = Math.max(0, f.height - 3);
    values.forEach((value, i) => {
      const x = f.x + i * cell + (cell - width) / 2;
      ctx.fillStyle = level(value);
      if (value > 0) bar(x, bottom - value * full, width, value * full);
      if (c.peaks && value > 0) ctx.fillRect(x, bottom - peakOf(value) * full - 2, width, 1);
    });
  } else if (c.style === 'blocks') {
    const space = Math.min(Math.max(c.gap, 1), c.block), pitch = c.block + space;
    const count = Math.max(0, Math.floor((f.height + space) / pitch));
    const block = (x, k) => { ctx.fillStyle = level((k + 1) / count); bar(x, bottom - k * pitch - c.block, width, c.block); };
    values.forEach((value, i) => {
      const x = f.x + i * cell + (cell - width) / 2, lit = Math.min(count, Math.round(value * count));
      for (let k = 0; k < lit; k++) block(x, k);
      const peak = Math.min(count, Math.round(peakOf(value) * count));
      if (c.peaks && value > 0 && peak > lit) block(x, peak - 1);
    });
  } else if (c.style === 'line') {
    const room = Math.max(0, f.height - c.stroke - 1);
    const points = values.map((value, i) => [f.x + (i + .5) * cell, bottom - c.stroke / 2 - value * room]);
    const stroke = level(Math.max(0, ...values));
    const curve = () => {
      ctx.beginPath(); ctx.moveTo(f.x, points[0][1]); ctx.lineTo(...points[0]);
      for (let i = 1; i < points.length; i++) { const [a, b] = [points[i - 1], points[i]]; ctx.quadraticCurveTo(a[0], a[1], (a[0] + b[0]) / 2, (a[1] + b[1]) / 2); }
      ctx.lineTo(...points.at(-1)); ctx.lineTo(f.x + f.width, points.at(-1)[1]);
    };
    if (c.fill) { curve(); ctx.lineTo(f.x + f.width, bottom); ctx.lineTo(f.x, bottom); ctx.closePath(); ctx.fillStyle = stroke; ctx.globalAlpha = (e.opacity ?? 1) * .35; ctx.fill(); ctx.globalAlpha = e.opacity ?? 1; }
    ctx.fillStyle = base; if (c.centre) ctx.fillRect(f.x, bottom - 1, f.width, 1);
    curve(); ctx.strokeStyle = stroke; ctx.lineWidth = c.stroke; ctx.lineCap = 'round'; ctx.lineJoin = 'round'; ctx.stroke();
  } else if (c.style === 'radial') {
    const cx = f.x + f.width / 2, cy = mid, outer = Math.max(0, Math.min(f.width, f.height) / 2 - 3);
    const inner = outer * c.inner / 100, room = outer - inner, radius = Math.max(inner, room / 2);
    const w = Math.max(.5, Math.min(c.thickness, 2 * Math.PI * radius / c.bands - c.gap));
    if (c.centre && inner >= 1) { ctx.beginPath(); ctx.arc(cx, cy, inner, 0, 2 * Math.PI); ctx.strokeStyle = base; ctx.lineWidth = 1; ctx.stroke(); }
    if (room > 0) values.forEach((value, i) => {
      ctx.save(); ctx.translate(cx, cy); ctx.rotate(i * 2 * Math.PI / c.bands); ctx.fillStyle = level(value);
      if (value > 0) bar(-w / 2, -inner - value * room, w, value * room);
      if (c.peaks && value > 0) ctx.fillRect(-w / 2, -inner - peakOf(value) * room - 2, w, 1);
      ctx.restore();
    });
  }
  ctx.font = `${e.kind.style.size}px sans-serif`; ctx.textBaseline='top'; ctx.fillStyle=base;
  ctx.fillText(tr(c.demo ? 'spectrum.demo' : 'spectrum.disabled'), f.x, f.y);
  ctx.restore();
}

function drawElement(ctx, e, t, refresh = 1, motion = true, trial = null) {
  const angle = appliedRotation(e);
  if (!angle) { drawUprightElement(ctx, e, t, refresh, motion, trial); return; }
  const center = centerOf(e.frame);
  ctx.save();
  ctx.translate(center.x, center.y); ctx.rotate(angle * Math.PI / 180); ctx.translate(-center.x, -center.y);
  ctx.beginPath(); ctx.rect(e.frame.x, e.frame.y, e.frame.width, e.frame.height); ctx.clip();
  drawUprightElement(ctx, { ...e, rotation: 0 }, t, refresh, motion, trial);
  ctx.restore();
}

function drawUprightElement(ctx, e, t, refresh = 1, motion = true, trial = null) {
  if (e.kind.content?.type === 'spectrum') { drawSpectrum(ctx, trial ? { ...e, kind: { ...e.kind, content: { ...e.kind.content, demo: true } } } : e, trial ? trial.elapsedMs / 1000 : t); return; }
  let value = shownFraction(e, t, refresh, motion);
  if (trial && boundKey(e)) value = (Number.isFinite(value) ? value : 0) + (Math.max(1, Number.isFinite(value) ? value : 0) - (Number.isFinite(value) ? value : 0)) * trial.pulse;
  e = withValueColor(e, demoColorValue(e, value, key => demoSensorValue(key, Math.floor(t / refresh) * refresh)));
  const k = e.kind;
  const f = e.frame;
  if (['shape', 'image'].includes(k.type) && k.fade && f.width > 0 && f.height > 0) {
    const layer = new OffscreenCanvas(Math.ceil(f.width), Math.ceil(f.height));
    const lc = layer.getContext('2d');
    lc.translate(-f.x, -f.y);
    drawElement(lc, { ...e, opacity: 1, kind: { ...k, fade: null } }, t, refresh, motion, trial);
    applyCanvasFade(lc, k.fade, f);
    ctx.save(); ctx.globalAlpha = e.opacity ?? 1;
    ctx.drawImage(layer, f.x, f.y); ctx.restore();
    return;
  }
  const frac = value;
  ctx.save();
  ctx.globalAlpha = e.opacity ?? 1;
  const graphPoints = trial && k.type === 'graph' ? Array.from({ length: 60 }, (_, i) => .5 + .4 * Math.sin(i * .25 + trial.elapsedMs / 1000)) : null;
  if (drawVisualization(ctx,e,frac,k.type==='graph'?(graphPoints ?? demoGraphPoints(k,refresh,t,e.id)):null,k.type==='graph'&&e.visualization?.secondary?(graphPoints ?? demoGraphPoints({...k,binding:e.visualization.secondary},refresh,t,e.id+197)):null)) { ctx.restore(); return; }
  switch (k.type) {
    case 'text':
      drawText(ctx, e, t, frac, false, trial, refresh, motion);
      break;
    case 'shape':
      ctx.fillStyle = k.videoWindow ? '#1f2330' : paint(k.fill);
      ctx.beginPath();
      if (k.shape === 'ellipse') ctx.ellipse(f.x + f.width / 2, f.y + f.height / 2, f.width / 2, f.height / 2, 0, 0, Math.PI * 2);
      else ctx.roundRect(f.x, f.y, f.width, f.height, k.corners ?? k.radius ?? 0);
      ctx.fill();
      if (k.image?.asset && !k.videoWindow) { ctx.save(); ctx.clip(); drawPicturePlaceholder(ctx, f, k.image); ctx.restore(); }
      if (k.stroke && k.strokeWidth > 0) { ctx.strokeStyle = color(k.stroke); ctx.lineWidth = k.strokeWidth; ctx.stroke(); }
      break;
    case 'bar': {
      const length = ['bottomToTop', 'topToBottom'].includes(k.direction) ? f.height : f.width;
      ctx.beginPath();
      for (const [a, b] of gaugeBlocks(length, k.segments)) {
        const box = barSpan(f, k.direction, a, b);
        ctx.roundRect(box.x, box.y, box.width, box.height, Math.min(Math.max(0, k.radius ?? 0), box.width / 2, box.height / 2));
      }
      ctx.fillStyle = canvasPaint(ctx, k.track, f); ctx.fill();
      ctx.save(); ctx.clip();
      ctx.fillStyle = canvasPaint(ctx, k.fill, f);
      const filled = barSpan(f, k.direction, 0, Number.isFinite(frac) ? Math.max(0, Math.min(1, frac)) * length : 0);
      ctx.fillRect(filled.x, filled.y, filled.width, filled.height);
      ctx.restore();
      break;
    }
    case 'ring': {
      const thickness = Math.min(k.thickness, Math.min(f.width, f.height) / 2);
      const r = Math.min(f.width, f.height) / 2 - thickness / 2;
      if (r <= 0) break;
      const cx = f.x + f.width / 2;
      const cy = f.y + f.height / 2;
      const start = ((k.startAngle - 90) * Math.PI) / 180;
      const direction = k.clockwise === false ? -1 : 1;
      const sweep = (Math.max(0, Math.min(360, k.sweep)) * Math.PI) / 180;
      const pieces = gaugeBlocks(sweep, k.segments ? { ...k.segments, gap: k.segments.gap * Math.PI / 180 } : null, k.sweep >= 360);
      const arcs = limit => { for (const [a, b] of pieces) { if (a >= limit) break; ctx.moveTo(cx + Math.cos(start + direction * a) * r, cy + Math.sin(start + direction * a) * r); ctx.arc(cx, cy, r, start + direction * a, start + direction * Math.min(b, limit), direction < 0); } };
      const value = k.testFull ? 1 : frac;
      ctx.lineWidth = thickness;
      ctx.lineCap = k.roundCaps ? 'round' : 'butt';
      ctx.strokeStyle = paint(k.track);
      ctx.beginPath();
      arcs(sweep);
      ctx.stroke();
      if (k.fill?.scale === 'arc') {
        const p = k.fill;
        const gradient = ctx.createConicGradient(start, cx, cy);
        const span = Math.min(1, k.sweep / 360);
        const mid = Math.min(100, Math.max(0, p.transition)) / 100;
        const half = (a, b) => {
          const channel = (i) => Math.round((parseInt(a.slice(i, i + 2), 16) + parseInt(b.slice(i, i + 2), 16)) / 2).toString(16).padStart(2, '0');
          return `#${[1, 3, 5, 7].map(channel).join('')}`;
        };
        const stops = [[0, p.start], [mid * span, half(p.start, p.end)], [span, p.end], [1, p.end]];
        if (direction < 0) {
          stops.forEach((stop) => { stop[0] = 1 - stop[0]; });
          stops.reverse();
        }
        for (const [position, hex] of stops) gradient.addColorStop(position, color(hex));
        ctx.strokeStyle = gradient;
      } else ctx.strokeStyle = canvasPaint(ctx, k.fill, f);
      if (value > 0) {
        ctx.beginPath();
        arcs(sweep * value);
        ctx.stroke();
      }
      break;
    }
    case 'needle': {
      if (k.look) { drawNeedleLook(ctx, e, frac, t, refresh, motion, trial); break; }
      const px = f.x + f.width * (k.pivot?.[0] ?? 0.5);
      const py = f.y + f.height * (k.pivot?.[1] ?? 0.5);
      const angle = (((k.startAngle + k.sweep * frac) - 90) * Math.PI) / 180;
      const len = Math.min(f.width, f.height) * 0.45;
      ctx.strokeStyle = color(k.color);
      ctx.lineWidth = k.width;
      ctx.lineCap = 'round';
      ctx.beginPath();
      ctx.moveTo(px, py);
      ctx.lineTo(px + Math.cos(angle) * len, py + Math.sin(angle) * len);
      ctx.stroke();
      break;
    }
    case 'graph': {
      ctx.strokeStyle = color(k.color);
      ctx.lineWidth = k.lineWidth;
      ctx.beginPath();
      const points = graphPoints ?? demoGraphPoints(k, refresh, t, e.id);
      const n = points.length;
      for (let i = 0; i < n; i += 1) {
        const x = f.x + (f.width * i) / (n - 1);
        const y = f.y + f.height * (1 - points[i]);
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.stroke();
      break;
    }
    case 'image':
      drawClippedMedia(ctx, f, k, () => {
      if (String(k.asset).toLowerCase().endsWith('.gif')) {
        ctx.fillStyle = GIF_COLORS[demoGifFrame(k, trial?.elapsedMs ?? t * 1000)];
        ctx.fillRect(f.x, f.y, f.width, f.height);
      }
      ctx.strokeStyle = k.color ? color(k.color) : 'rgba(255,255,255,0.4)';
      ctx.setLineDash([6, 6]);
      ctx.strokeRect(f.x, f.y, f.width, f.height);
      });
      break;
    default:
      break;
  }
  ctx.restore();
}

/** Side of a demo picture at 100 %, CSS pixels (the demo decodes no images). */
const DEMO_PICTURE = 32;

/**
 * Where a shape's picture lands, as the screen renderer places it: a
 * checkered copy for each tile, the fitted area for the box modes. The
 * demo has no image decoder, so the copies stand in for the picture.
 */
function drawPicturePlaceholder(ctx, f, image) {
  const repeat = image.repeat ?? 'cover';
  ctx.globalAlpha *= Math.max(0, Math.min(1, image.opacity ?? 1));
  const copy = (x, y, w, h) => {
    ctx.fillStyle = 'rgba(255,255,255,0.55)'; ctx.fillRect(x, y, w, h);
    ctx.fillStyle = 'rgba(0,0,0,0.35)'; ctx.fillRect(x, y, w / 2, h / 2); ctx.fillRect(x + w / 2, y + h / 2, w / 2, h / 2);
  };
  if (repeat === 'cover' || repeat === 'stretch') { copy(f.x, f.y, f.width, f.height); return; }
  if (repeat === 'contain') { const s = Math.min(f.width, f.height); copy(f.x + (f.width - s) / 2, f.y + (f.height - s) / 2, s, s); return; }
  const side = Math.max(1, DEMO_PICTURE * (image.scale ?? 1));
  const [ox, oy] = [image.offsetX ?? 0, image.offsetY ?? 0];
  if (repeat === 'center') { copy(f.x + (f.width - side) / 2 + ox, f.y + (f.height - side) / 2 + oy, side, side); return; }
  const start = (from, offset) => from + offset - Math.ceil(offset / side) * side;
  const xs = repeat === 'tileY' ? [f.x + ox] : [];
  const ys = repeat === 'tileX' ? [f.y + oy] : [];
  if (!xs.length) for (let x = start(f.x, ox); x < f.x + f.width; x += side) xs.push(x);
  if (!ys.length) for (let y = start(f.y, oy); y < f.y + f.height; y += side) ys.push(y);
  for (const y of ys) for (const x of xs) copy(x, y, side, side);
}

/** A small fixed pseudo-random sequence, so the stars stay where they are. */
const STARS = Array.from({ length: 28 }, (_, i) => [((i * 7919) % 997) / 997, ((i * 104729) % 991) / 991 * 0.4, (i * 0.37) % 1]);

/**
 * One picture of the demo's video at `ms`, drawn upright in `w`×`h`: a dusk
 * sky with twinkling stars, a sun, hills that drift over a strip of grass,
 * and an orange orb crossing the sky. Every picture differs, so a playing
 * preview visibly moves; the grass is always at the bottom.
 */
function drawScene(ctx, w, h, ms) {
  const sky = ctx.createLinearGradient(0, 0, 0, h);
  sky.addColorStop(0, '#1e3a8a');
  sky.addColorStop(0.55, '#7c3aed');
  sky.addColorStop(0.85, '#f59e0b');
  ctx.fillStyle = sky;
  ctx.fillRect(0, 0, w, h);
  for (const [x, y, phase] of STARS) {
    ctx.fillStyle = `rgba(255,255,255,${0.35 + 0.65 * Math.abs(Math.sin(ms / 400 + phase * 6))})`;
    ctx.fillRect(x * w, y * h, Math.max(2, h * 0.008), Math.max(2, h * 0.008));
  }
  ctx.fillStyle = '#fde68a';
  ctx.beginPath();
  ctx.arc(w * 0.78, h * 0.62, h * 0.14, 0, Math.PI * 2);
  ctx.fill();
  const drift = (ms * 0.04) % w;
  ctx.fillStyle = '#312e81';
  ctx.beginPath();
  ctx.moveTo(0, h);
  for (let x = 0; x <= w; x += w / 48) ctx.lineTo(x, h * (0.7 + 0.06 * Math.sin(((x + drift) / w) * Math.PI * 6)));
  ctx.lineTo(w, h);
  ctx.fill();
  ctx.fillStyle = '#16a34a';
  ctx.fillRect(0, h * 0.86, w, h * 0.14);
  const loop = (ms % DEMO_VIDEO_LOOP_MS) / DEMO_VIDEO_LOOP_MS;
  const r = h * 0.11;
  ctx.fillStyle = '#f97316';
  ctx.beginPath();
  ctx.arc(-r + loop * (w + 2 * r), h * (0.32 + 0.08 * Math.sin(loop * Math.PI * 2)), r, 0, Math.PI * 2);
  ctx.fill();
}

/** How long the demo's video scene takes to loop, ms. */
export const DEMO_VIDEO_LOOP_MS = 10_200;

/**
 * The video's own picture at `ms`, as the file holds it: `source` pixels,
 * the scene upright, or turned 90° clockwise for the panel (`preTurned`,
 * the vendor's way: the scene's top on the right).
 */
function sourcePicture(video, ms) {
  const { width, height } = video.source;
  const picture = new OffscreenCanvas(width, height);
  const ctx = picture.getContext('2d');
  if (video.preTurned) {
    ctx.translate(width, 0);
    ctx.rotate(Math.PI / 2);
    drawScene(ctx, height, width, ms);
  } else {
    drawScene(ctx, width, height, ms);
  }
  return picture;
}

/**
 * What the canvas shows under the elements: the color; a video's picture
 * (`video`: playing, or its poster) turned and framed like the backend does,
 * the pad color around it with Fit; black otherwise (a picture, or a video
 * with neither poster nor decoder).
 * @param {{source: {width:number, height:number}, preTurned?: boolean, rotation: number, framing: object, box: {x:number, y:number, width:number, height:number}, ms: number}|null} video
 */
function drawBackdrop(ctx, background, canvas, video) {
  if (background?.type === 'color' || !video) {
    ctx.fillStyle = background?.type === 'color' ? color(background.color) : '#000';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    return;
  }
  ctx.fillStyle = video.framing.fit === 'contain' ? color(video.framing.padColor) : '#000';
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  const { box } = video;
  const [width, height] = video.rotation % 180 === 90 ? [box.height, box.width] : [box.width, box.height];
  ctx.save();
  ctx.translate(box.x + box.width / 2, box.y + box.height / 2);
  ctx.rotate((video.rotation * Math.PI) / 180);
  ctx.drawImage(sourcePicture(video, video.ms), -width / 2, -height / 2, width, height);
  ctx.restore();
}

function drawCardTransition(ctx, theme, parent, transition, t) {
  const { width, height } = theme.canvas, include = transition.settings.includeBase;
  const members = theme.elements.filter(e => e.id === parent.id || e.cardMember?.parent === parent.id);
  const draw = (target, e) => drawElement(target, { ...e, opacity: effectiveOpacity(theme, e) }, t, theme.refreshSeconds);
  if (!include) for (const e of members) if (visibleWithoutFace(theme, e) && (e.id === parent.id || e.cardMember.face == null)) draw(ctx, e);
  const combined = new OffscreenCanvas(width, height), cc = combined.getContext('2d');
  const group = new OffscreenCanvas(width, height), gc = group.getContext('2d');
  for (const pose of cardPoses(transition, parent.frame)) {
    gc.clearRect(0, 0, width, height);
    for (const e of members) if (visibleWithoutFace(theme, e) && (e.cardMember?.face === pose.face || include && (e.id === parent.id || e.cardMember.face == null))) draw(gc, e);
    cc.save(); if (!pose.projection) cc.setTransform(...pose.matrix); cc.globalAlpha = pose.alpha;
    cc.globalCompositeOperation = transition.settings.effect === 'fade' ? 'lighter' : 'source-over';
    if (pose.projection) {
      // Shade once, before projecting the image's two-pixel strips. A
      // filter on cc would re-filter the whole source for every strip.
      const shaded = new OffscreenCanvas(width, height), sc = shaded.getContext('2d');
      sc.filter = `brightness(${pose.shade})`;
      sc.drawImage(group, 0, 0);
      drawCardProjection(cc, shaded, pose.projection);
    } else {
      cc.filter = `brightness(${pose.shade})`;
      cc.drawImage(group, 0, 0);
    }
    cc.restore();
  }
  ctx.save();
  if (transition.settings.effect === 'slide') { const b = parent.frame; ctx.beginPath(); ctx.rect(b.x, b.y, b.width, b.height); ctx.clip(); }
  ctx.drawImage(combined, 0, 0); ctx.restore();
}

/**
 * Renders a theme to RGBA pixels; `video` is the video background's picture
 * the demo backend chose (see `drawBackdrop`), if any. `scale` (at most 1)
 * reduces the picture: the canvas shrinks, the theme is drawn the same.
 * Without `motion` values show their last sample (no easing). `textMs`
 * says when a scrolling line or blinking separators want the next frame.
 * @returns {{width:number, height:number, rgba: Uint8ClampedArray, textMs: number|null}}
 */
export function renderApprox(theme, t, video = null, transitions = new Map(), scale = 1, motion = true, trial = null) {
  const width = Math.max(1, Math.round(theme.canvas.width * scale));
  const height = Math.max(1, Math.round(theme.canvas.height * scale));
  const canvas = new OffscreenCanvas(width, height);
  const ctx = canvas.getContext('2d');
  ctx.scale(scale, scale);
  textDue = null;
  const device = theme.background.type === 'deviceVideo';
  const windows = theme.elements.some((e) => isShown(theme, e) && e.kind.type === 'shape' && e.kind.videoWindow);
  if (device) {
    ctx.fillStyle = windows ? color(theme.background.color) : '#1f2330';
    ctx.fillRect(0, 0, theme.canvas.width, theme.canvas.height);
  } else drawBackdrop(ctx, theme.background, theme.canvas, video);
  for (const e of theme.elements) {
    if (e.isGroup) continue;
    if (transitions.has(e.id)) { drawCardTransition(ctx, theme, e, transitions.get(e.id), t); continue; }
    if (transitions.has(e.cardMember?.parent) || !isShown(theme, e)) continue;
    drawElement(ctx, { ...e, opacity: effectiveOpacity(theme, e) }, trial && e.kind.type === 'image' ? trial.elapsedMs / 1000 : t, theme.refreshSeconds, motion, trial?.elementId === e.id ? trial : null);
  }
  return { width, height, rgba: ctx.getImageData(0, 0, width, height).data, textMs: textDue };
}

/** Natural dimensions from the same formatter and canvas font used to draw. */
export function measureApprox(theme, id, t) {
  const e = theme.elements.find(e => e.id === id);
  if (!e || e.kind.type !== 'text') return null;
  const ctx = new OffscreenCanvas(1, 1).getContext('2d');
  const size = drawText(ctx, e, t, shownFraction(e, t, theme.refreshSeconds, true), true, null, theme.refreshSeconds);
  if (!size || size.width <= 0 || size.height <= 0) return null;
  const style = e.kind.style, f = e.frame;
  const ax = style.align === 'right' ? 1 : style.align === 'center' ? 0.5 : 0;
  const ay = style.valign === 'bottom' ? 1 : style.valign === 'middle' ? 0.5 : 0;
  return { x: f.x + (f.width - size.width) * ax, y: f.y + (f.height - size.height) * ay, ...size };
}

/** The common opacity ramp, on an isolated image/shape/cover canvas. */
export function applyCanvasFade(ctx, fade, frame) {
  if (!fade) return;
  ctx.save(); ctx.globalCompositeOperation = 'destination-in';
  if (fade.radial) { ctx.translate(frame.x, frame.y); ctx.scale(frame.width, frame.height); }
  const gradient = fade.radial ? ctx.createRadialGradient(fade.radial[0], fade.radial[1], 0, ...fade.radial) : ctx.createLinearGradient(...fadeLine(frame, fade.angle));
  gradient.addColorStop(0, `rgba(0,0,0,${fade.start})`);
  gradient.addColorStop(fade.midpoint ?? .5, `rgba(0,0,0,${(fade.start + fade.end) / 2})`);
  gradient.addColorStop(1, `rgba(0,0,0,${fade.end})`);
  ctx.fillStyle = gradient;
  if (fade.radial) ctx.fillRect(0, 0, 1, 1); else ctx.fillRect(frame.x, frame.y, frame.width, frame.height);
  ctx.restore();
}

/** Same cover corner clipping and isolated fade as the native renderer. */
export function drawPlayerCover(ctx, frame, content, draw) {
  const target = content.coverFade ? new OffscreenCanvas(Math.ceil(frame.width), Math.ceil(frame.height)).getContext('2d') : ctx;
  target.save();
  if (content.coverFade) target.translate(-frame.x, -frame.y);
  drawClippedMedia(target, frame, { corners: content.coverCorners, radius: content.coverRadius }, draw, true);
  if (content.coverFade) applyCanvasFade(target, content.coverFade, frame);
  target.restore();
  if (content.coverFade) ctx.drawImage(target.canvas, frame.x, frame.y);
}

/** Shared fitted-media corner clip, matching native path::rounded_corners. */
export function drawClippedMedia(ctx, frame, geometry, draw, force = false) {
  const radii = geometry.corners ?? geometry.radius ?? 0;
  if (!force && !(Array.isArray(radii) ? radii.some(radius => radius > 0) : radii > 0)) { draw(ctx); return; }
  ctx.save(); ctx.beginPath(); ctx.roundRect(frame.x, frame.y, frame.width, frame.height, radii); ctx.clip(); draw(ctx); ctx.restore();
}
