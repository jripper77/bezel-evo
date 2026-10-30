// Pixel-art weather icons, mirroring crates/bezel-render/src/weather_pixel.rs
// (a test keeps the sprites and palette equal): 16×16 sprites drawn cell by
// cell on whole pixels; a moving icon snaps the vector pose to the grid.
import { weatherFamily } from './weather-format.js';

/** Cells on each side of the grid. */
const GRID = 16;
/** Units of the pose (a 64-unit icon) in one cell. */
const UNITS_PER_CELL = 64 / GRID;
/** Degrees the vector rays turn while the pixel rays show one frame. */
const RAY_FRAME_DEGREES = 22.5;

/** One character per colour; `.` is transparent. */
export const PIXEL_PALETTE = Object.freeze({
  k: '#26304a', H: '#fff6c2', Y: '#ffd54f', O: '#f29f1d', R: '#ffc107', W: '#f5fbff', C: '#b5cde6',
  F: '#c3d3e4', B: '#37c0f5', N: '#1d70d6', S: '#e6f7ff', L: '#ffe042', M: '#ff8b1e', X: '#a0aec0',
});

/** The sprites, rows top to bottom. */
export const PIXEL_SPRITES = Object.freeze({
  disc: ['..kkkk..', '.kHYYYk.', 'kHYYYYOk', 'kYYYYYOk', 'kYYYYOOk', 'kYYYOOOk', '.kOOOOk.', '..kkkk..'],
  'rays-a': [
    '................', '.......RR.......', '.......RR.......', '...R........R...',
    '................', '................', '................', '.RR..........RR.',
    '.RR..........RR.', '................', '................', '................',
    '...R........R...', '.......RR.......', '.......RR.......', '................',
  ],
  'rays-b': [
    '................', '................', '..R....RR....R..', '...R........R...',
    '................', '................', '................', '..R..........R..',
    '..R..........R..', '................', '................', '................',
    '...R........R...', '..R....RR....R..', '................', '................',
  ],
  'small-disc': ['.kkkk.', 'kHYYYk', 'kYYYOk', 'kYYOOk', 'kOOOOk', '.kkkk.'],
  'small-rays-a': [
    '.....RR.....', '.....RR.....', '..R......R..', '............', '............', 'RR........RR',
    'RR........RR', '............', '............', '..R......R..', '.....RR.....', '.....RR.....',
  ],
  'small-rays-b': [
    '............', '.R...RR...R.', '..R......R..', '............', '............', '.R........R.',
    '.R........R.', '............', '............', '..R......R..', '.R...RR...R.', '............',
  ],
  cloud: ['....kkk.......', '...kWWWk.kk...', '..kWWWWWkWWk..', '.kWWWWWWWWWWk.', 'kWWWWWWWWWWWWk', 'kWWWWWWWWWWWCk', 'kCCWWWWWWCCCCk', '.kkkkkkkkkkkk.'],
  strip: ['FFFFFFFFFFFF'],
  drop: ['.B', '.B', 'N.'],
  flake: ['.S.', 'SSS', '.S.'],
  bolt: ['...LL', '..LL.', '.LLLL', '...LM', '..LM.', '..M..'],
  cross: ['XX....XX', 'XXX..XXX', '.XXXXXX.', '..XXXX..', '..XXXX..', '.XXXXXX.', 'XXX..XXX', 'XX....XX'],
});

/** A pose offset (units of a 64-unit icon) in whole cells. */
export const cells = (units) => (Number.isFinite(units) ? Math.floor(units / UNITS_PER_CELL + 0.5) : 0);
/** An opacity in three steps: 0, ½ or 1. */
export const steps = (opacity) => Math.floor(Math.min(1, Math.max(0, opacity)) * 2 + 0.5) / 2;
/** Which of the two ray frames shows while the vector rays have turned `degrees`. */
export const rayFrame = (degrees) => (Number.isFinite(degrees) ? ((Math.floor(degrees / RAY_FRAME_DEGREES) % 2) + 2) % 2 : 0);

/** The sprites of the icon of `code` at `pose`: `{ name, x, y, opacity }` in drawing order. */
export function pixelSprites(code, pose) {
  const family = weatherFamily(code), frame = rayFrame(pose.rays), cloud = cells(pose.cloud), out = [];
  const put = (name, x, y, opacity = 1) => out.push({ name, x, y, opacity });
  if (family === 0) { put('disc', 4, 4); put(['rays-a', 'rays-b'][frame], 0, 0); }
  else if (family === 1) { put('small-disc', 3, 3); put(['small-rays-a', 'small-rays-b'][frame], 0, 0); put('cloud', 2 + cloud, 7); }
  else if (family === 2) put('cloud', 1 + cloud, 4);
  else if (family === 3) {
    put('cloud', 1 + cloud, 2);
    [[2, 11], [3, 13]].forEach(([x, y], i) => { const [dx, o] = pose.strips[i]; put('strip', x + cells(dx), y, steps(o)); });
  } else if (family === 4 || family === 5) {
    put('cloud', 1 + cloud, 1);
    const [name, at] = family === 4 ? ['drop', [[3, 11], [7, 12], [11, 11]]] : ['flake', [[2, 11], [7, 12], [12, 11]]];
    at.forEach(([x, y], i) => { const [dx, dy, o] = pose.drops[i]; put(name, x + cells(dx), y + cells(dy), steps(o)); });
  } else if (family === 6) { put('cloud', 1 + cloud, 1); put('bolt', 6, 9, steps(pose.bolt)); }
  else put('cross', 4, 4);
  return out;
}

/** Draws the pixel icon of `code` in the `side`-pixel square at (x, y), like the backend. */
export function drawPixelWeather(ctx, x, y, side, code, pose) {
  if (!(side > 0)) return;
  const cell = side >= GRID ? Math.floor(side / GRID) : side / GRID;
  const snap = (v) => (cell >= 1 ? Math.floor(v + 0.5) : v);
  const ox = snap(x + (side - cell * GRID) / 2), oy = snap(y + (side - cell * GRID) / 2);
  const whole = steps(pose.whole);
  ctx.save();
  for (const sprite of pixelSprites(code, pose)) {
    const alpha = whole * sprite.opacity;
    if (alpha <= 0) continue;
    ctx.globalAlpha = alpha;
    PIXEL_SPRITES[sprite.name].forEach((row, gy) => [...row].forEach((c, gx) => {
      const cx = sprite.x + gx, cy = sprite.y + gy;
      if (c === '.' || cx < 0 || cy < 0 || cx >= GRID || cy >= GRID) return;
      ctx.fillStyle = PIXEL_PALETTE[c];
      ctx.fillRect(ox + cx * cell, oy + cy * cell, cell, cell);
    }));
  }
  ctx.restore();
}
