// The Studio UX v2 review page (phase studio-ux-v2, T-10): every board of the
// design beside the app's captures of it, dark and light, in Italian.
// Run from apps/bezel-studio:
//   node scripts/fidelity/ux2-gallery.mjs [--reuse] [--out absolute-directory]
// Without --reuse it captures the app first with shoot.mjs (the ux2 states of
// every area); the four templates come from shoot-templates.mjs's evidence.
import { spawn } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readdirSync, writeFileSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const APP = fileURLToPath(new URL('../../', import.meta.url));
const PHASE = fileURLToPath(new URL('../../../../.jdi/phases/studio-ux-v2/', import.meta.url));
const args = process.argv.slice(2);
const at = args.indexOf('--out');
const out = at < 0 ? join(PHASE, 'evidence/final') : resolve(args[at + 1]);
const shots = join(out, 'captures');

/** The app states that show each board, by area. */
const AREAS = {
  inspector: ['ux2-main', 'ux2-ring-closed', 'ux2-ring-open', 'ux2-linked-value', 'ux2-linked-text', 'ux2-linked-bar', 'ux2-linked-needle', 'ux2-linked-graph', 'ux2-linked-multi',
    'ux2-media-clock', 'ux2-media-analog', 'ux2-media-weather', 'ux2-media-player', 'ux2-media-spectrum', 'ux2-media-shape', 'ux2-media-image', 'ux2-media-gif',
    'ux2-card', 'ux2-theme-open', 'ux2-theme-closed'],
  shell: ['ux2-handles-ring', 'ux2-handles-selection', 'ux2-handles-needle', 'ux2-handles-bar', 'ux2-handles-graph', 'ux2-handles-image', 'ux2-handles-spectrum',
    'ux2-text-inline', 'ux2-value-part', 'ux2-card-entry', 'ux2-shape-image-entry', 'ux2-clock-length', 'ux2-alignment-distance', 'ux2-player-cover', 'ux2-object-bar',
    'ux2-trial-values', 'ux2-status-drag', 'ux2-status-sensors', 'ux2-status-bezel', 'ux2-status-screen'],
  library: ['widgets', 'sensors-libre', 'layers-selection', 'media-filled', 'ux2-profiles', 'ux2-themes'],
  screen: ['ux2-settings', 'ux2-storage'],
};

/** Each board and the captures that answer it (area/state). */
const PAIRS = {
  MainNew: ['inspector/ux2-main', 'library/widgets'],
  InspectorComponents: ['inspector/ux2-ring-closed', 'inspector/ux2-ring-open'],
  RingInspectorOpen: ['inspector/ux2-ring-open'],
  ValueInspectorOpen: ['inspector/ux2-linked-value', 'inspector/ux2-linked-text'],
  BarInspectorOpen: ['inspector/ux2-linked-bar'],
  NeedleInspectorOpen: ['inspector/ux2-linked-needle'],
  GraphInspectorOpen: ['inspector/ux2-linked-graph'],
  MultiInspectorOpen: ['inspector/ux2-linked-multi'],
  ClockInspectorOpen: ['inspector/ux2-media-clock', 'inspector/ux2-media-analog'],
  WeatherInspectorOpen: ['inspector/ux2-media-weather'],
  WeatherAnimations: ['inspector/ux2-media-weather'],
  PlayerInspectorOpen: ['inspector/ux2-media-player'],
  SpectrumInspectorOpen: ['inspector/ux2-media-spectrum'],
  ShapeInspectorOpen: ['inspector/ux2-media-shape'],
  ImageInspectorOpen: ['inspector/ux2-media-image', 'inspector/ux2-media-gif'],
  CardInspectorOpen: ['inspector/ux2-card'],
  ThemeInspectorOpen: ['inspector/ux2-theme-open', 'inspector/ux2-theme-closed'],
  CanvasHandles: ['shell/ux2-handles-ring', 'shell/ux2-handles-needle', 'shell/ux2-handles-bar', 'shell/ux2-handles-graph', 'shell/ux2-handles-image', 'shell/ux2-handles-spectrum',
    'shell/ux2-handles-selection', 'shell/ux2-text-inline', 'shell/ux2-value-part', 'shell/ux2-card-entry', 'shell/ux2-shape-image-entry', 'shell/ux2-clock-length',
    'shell/ux2-alignment-distance', 'shell/ux2-player-cover', 'shell/ux2-object-bar', 'shell/ux2-trial-values'],
  StatusBarNew: ['shell/ux2-status-drag', 'shell/ux2-status-sensors', 'shell/ux2-status-bezel', 'shell/ux2-status-screen'],
  LibraryPanelsNew: ['library/widgets', 'library/sensors-libre', 'library/layers-selection', 'library/media-filled'],
  ProfilesNew: ['library/ux2-profiles'],
  ThemesNew: ['library/ux2-themes'],
  ScreenPanelsOpen: ['screen/ux2-settings', 'screen/ux2-storage'],
};

/** The four templates: their native frames and demo pictures from T-9d. */
const TEMPLATES = { ThemePixel: 'pixel-wide', ThemeNebula: 'nebula-wide', ThemePaper: 'paper-wide', ThemeNowPlaying: 'now-playing-wide' };

if (!args.includes('--reuse')) {
  for (const [area, states] of Object.entries(AREAS)) {
    const code = await new Promise((done, reject) => {
      const child = spawn(process.execPath, ['scripts/fidelity/shoot.mjs', '--area', area, '--only', 'app', '--scheme', 'both', '--state', states.join(','), '--out', shots, '--merge'],
        { cwd: APP, stdio: 'inherit' });
      child.once('error', reject);
      child.once('exit', done);
    });
    if (code !== 0) throw new Error(`shoot.mjs --area ${area} exited ${code}`);
  }
}

mkdirSync(join(out, 'boards'), { recursive: true });
const rel = (path) => relative(out, path).replaceAll('\\', '/');
const esc = (text) => String(text).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);
const board = (name) => {
  const from = join(PHASE, 'design/png', `${name}.png`);
  const to = join(out, 'boards', `${name}.png`);
  if (existsSync(from)) copyFileSync(from, to);
  return existsSync(to) ? rel(to) : null;
};
const capture = (scheme, key) => {
  const [area, state] = key.split('/');
  for (const name of [`${state}-detail.png`, `${state}.png`]) {
    const file = join(shots, scheme, 'app', area, name);
    if (existsSync(file)) return rel(file);
  }
  return null;
};
const missing = [];
const figure = (src, caption) => src ? `<figure><a href="${src}"><img loading="lazy" src="${src}" alt="${esc(caption)}"></a><figcaption>${esc(caption)}</figcaption></figure>` : '';
const sections = Object.entries(PAIRS).map(([name, keys]) => {
  const shotsOf = keys.flatMap((key) => ['dark', 'light'].map((scheme) => {
    const src = capture(scheme, key);
    if (!src) missing.push(`${scheme}/${key}`);
    return figure(src, `${key.split('/')[1]} · ${scheme === 'dark' ? 'scuro' : 'chiaro'}`);
  }));
  return `<section id="${name}"><h2>${name}</h2><div class="pair"><div class="board">${figure(board(name), 'tavola')}</div><div class="app">${shotsOf.join('')}</div></div></section>`;
});
const t9d = join(PHASE, 'evidence/t9d');
for (const [name, slug] of Object.entries(TEMPLATES)) {
  const files = existsSync(t9d) ? walk(t9d).filter((f) => f.includes(slug) && f.endsWith('.png')) : [];
  if (!files.length) missing.push(`t9d/${slug}`);
  sections.push(`<section id="${name}"><h2>${name}</h2><div class="pair"><div class="board">${figure(board(name), 'tavola')}</div><div class="app">${files.map((f) => figure(rel(f), relative(t9d, f).replaceAll('\\', '/'))).join('')}</div></div></section>`);
}

function walk(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((d) => (d.isDirectory() ? walk(join(dir, d.name)) : [join(dir, d.name)]));
}

const nav = [...Object.keys(PAIRS), ...Object.keys(TEMPLATES)].map((name) => `<a href="#${name}">${name}</a>`).join(' ');
writeFileSync(join(out, 'index.html'), `<!doctype html>
<html lang="it"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Studio UX v2 · confronto finale</title>
<style>
:root { color-scheme: light dark; --bg: #f4f5f7; --fg: #15171a; --muted: #5d636b; --card: #ffffff; --line: #d9dde3; }
@media (prefers-color-scheme: dark) { :root { --bg: #0e1013; --fg: #e8eaed; --muted: #9aa1aa; --card: #171a1e; --line: #2a2f36; } }
body { margin: 0; padding: 16px; background: var(--bg); color: var(--fg); font: 14px/1.5 system-ui, sans-serif; }
nav { display: flex; flex-wrap: wrap; gap: 4px 12px; margin-bottom: 16px; }
nav a, figcaption { color: var(--muted); }
section { margin: 0 0 32px; padding: 16px; border: 1px solid var(--line); border-radius: 10px; background: var(--card); }
.pair { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1.4fr); gap: 16px; }
.app { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 12px; align-content: start; }
figure { margin: 0; } img { max-width: 100%; height: auto; border: 1px solid var(--line); border-radius: 6px; }
@media (max-width: 800px) { .pair { grid-template-columns: 1fr; } }
</style></head><body>
<h1>Studio UX v2 · confronto finale con le tavole</h1>
<p>Catture dell'app in italiano dopo l'integrazione (T-10), scuro e chiaro, accanto alla tavola. Clic su un'immagine per vederla intera.${missing.length ? ` Mancano: ${esc(missing.join(', '))}.` : ''}</p>
<nav>${nav}</nav>
${sections.join('\n')}
</body></html>
`);
console.log(`gallery: ${join(out, 'index.html')}${missing.length ? ` (missing ${missing.length}: ${missing.join(', ')})` : ''}`);
