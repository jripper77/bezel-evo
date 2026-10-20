// Static 24-unit geometry and palettes mirrored from bezel-render/weather.rs.
// Kept outside the UI so the demo can test every condition family and style.
import { weatherFamily } from './weather-format.js';

const CLOUD = 'M5 16C-1 16 1 8 7 10C8 3 19 5 18 11C24 10 24 16 19 16L5 16';
const CROSS = 'M5 5L19 19M19 5L5 19';
const circle = (x, y, r) => `M${x + r} ${y}A${r} ${r} 0 1 0 ${x - r} ${y}A${r} ${r} 0 1 0 ${x + r} ${y}Z`;

/** Centre of the sun in the 24-unit box. */
export const sunCenter = (colored) => [12, colored ? 9 : 10];

/**
 * The parts of the icon of `code`, like the backend's: `{ role, index, path,
 * light, dark, fill }` with role `disc`, `rays`, `cloud`, `strip`, `drop`,
 * `bolt` or `cross` (strips and drops numbered by `index`).
 */
export function weatherIconParts(code, colored = false) {
  const family = weatherFamily(code), parts = [];
  const add = (role, path, light, dark, fill = false, index = 0) => parts.push({ role, index, path, light, dark, fill });
  if (family <= 1) {
    const [cx, cy] = sunCenter(colored);
    add('disc', circle(cx, cy, 4), '#ffd54f', '#ef7e16', true);
    const rays = Array.from({ length: 8 }, (_, i) => {
      const a = i * Math.PI / 4;
      return `M${cx + Math.cos(a) * 6} ${cy + Math.sin(a) * 6}L${cx + Math.cos(a) * 8} ${cy + Math.sin(a) * 8}`;
    }).join('');
    add('rays', rays, '#ffc107', '#ff9414');
  }
  if (family >= 1 && family <= 6) {
    add('cloud', CLOUD, '#e0f1ff', '#6792bc', true);
    const light = family === 6 ? '#ffe042' : family === 5 ? '#d7f4ff' : '#37c0f5';
    const dark = family === 6 ? '#ff8b1e' : family === 5 ? '#78ccf0' : '#1d70d6';
    if (family === 3) [19, 22].forEach((y, i) => add('strip', `M4 ${y}H20`, light, dark, false, i));
    if (family === 4 || family === 5) [6, 12, 18].forEach((x, i) => add('drop', `M${x} 19L${x - 1} 22${family === 5 ? `M${x - 1.5} 20H${x + 1.5}` : ''}`, light, dark, false, i));
    if (family === 6) add('bolt', 'M13 17L10 21H14L11 24', light, dark);
  }
  if (family === 7) add('cross', CROSS, '#a0aec0', '#506278');
  return parts;
}

/** The icon at rest: colour components (the cloud's extras joined), or one outline path. */
export function weatherIconPaths(code, colored = false) {
  const out = [];
  for (const part of weatherIconParts(code, colored)) {
    const extra = ['strip', 'drop', 'bolt'].includes(part.role);
    const last = out[out.length - 1];
    if (extra && last?.extra) last.path += part.path;
    else out.push({ path: part.path, light: part.light, dark: part.dark, fill: part.fill, extra });
  }
  const components = out.map(({ extra, ...component }) => component);
  return colored ? components : [{ path: components.map(p => p.path).join(''), fill: false }];
}
