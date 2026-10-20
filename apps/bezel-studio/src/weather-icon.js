// Static 24-unit geometry and palettes mirrored from bezel-render/weather.rs.
// Kept outside the UI so the demo can test every condition family and style.
import { weatherFamily } from './weather-format.js';

const CLOUD = 'M5 16C-1 16 1 8 7 10C8 3 19 5 18 11C24 10 24 16 19 16L5 16';
const CROSS = 'M5 5L19 19M19 5L5 19';
const circle = (x, y, r) => `M${x + r} ${y}A${r} ${r} 0 1 0 ${x - r} ${y}A${r} ${r} 0 1 0 ${x + r} ${y}Z`;

export function weatherIconPaths(code, colored = false) {
  const family = weatherFamily(code), parts = [];
  const add = (path, light, dark, fill = false) => parts.push({ path, light, dark, fill });
  if (family <= 1) {
    const cy = colored ? 9 : 10;
    add(circle(12, cy, 4), '#ffd54f', '#ef7e16', true);
    const rays = Array.from({ length: 8 }, (_, i) => {
      const a = i * Math.PI / 4;
      return `M${12 + Math.cos(a) * 6} ${cy + Math.sin(a) * 6}L${12 + Math.cos(a) * 8} ${cy + Math.sin(a) * 8}`;
    }).join('');
    add(rays, '#ffc107', '#ff9414');
  }
  if (family >= 1 && family <= 6) {
    add(CLOUD, '#e0f1ff', '#6792bc', true);
    let extra = '';
    if (family === 3) extra = 'M4 19H20M4 22H20';
    if (family === 4 || family === 5) extra = [6, 12, 18].map(x => `M${x} 19L${x - 1} 22${family === 5 ? `M${x - 1.5} 20H${x + 1.5}` : ''}`).join('');
    if (family === 6) extra = 'M13 17L10 21H14L11 24';
    if (extra) add(extra, family === 6 ? '#ffe042' : family === 5 ? '#d7f4ff' : '#37c0f5', family === 6 ? '#ff8b1e' : family === 5 ? '#78ccf0' : '#1d70d6');
  }
  if (family === 7) add(CROSS, '#a0aec0', '#506278');
  return colored ? parts : [{ path: parts.map(p => p.path).join(''), fill: false }];
}
