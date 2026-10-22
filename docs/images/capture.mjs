// README screenshots: real Studio UI and renderer, browser demo data only.
// From the repository root, after npm ci in apps/bezel-studio:
//   node docs/images/capture.mjs
import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { chromium } from '../../apps/bezel-studio/node_modules/@playwright/test/index.mjs';
import { createWidget } from '../../apps/bezel-studio/src/editor/widgets.js';
import { translator } from '../../apps/bezel-studio/src/i18n/index.js';

const output = fileURLToPath(new URL('./', import.meta.url));
const app = fileURLToPath(new URL('../../apps/bezel-studio/', import.meta.url));
const fonts = fileURLToPath(new URL('../../themes/fonts/', import.meta.url));
const t = translator('en');

// The example "Ember" document, assembled from Studio's ordinary widgets.
const size = { width: 480, height: 1920 };
const FONT = 'Inter';
const theme = {
  schema: 1, name: 'Ember', canvas: size, orientation: 'portrait', refreshSeconds: 1,
  background: { type: 'color', color: '#0B0D0FFF' },
  palette: [{ name: 'Ember', color: '#ff9248ff' }, { name: 'Panel', color: '#121518ff' }, { name: 'Text', color: '#e8eaedff' }, { name: 'Muted', color: '#aeb5bdff' }, { name: 'Hot', color: '#ff5a4fff' }],
  elements: [],
};
function widget(type, name, x, y, width, height, sensor = {}) {
  const { kind, valueTransition, card } = createWidget(type, size, sensor);
  const element = { id: theme.elements.length + 1, name, frame: { x, y, width, height }, opacity: 1, visible: true, locked: false, kind, ...(valueTransition ? { valueTransition } : {}), ...(card ? { card } : {}) };
  theme.elements.push(element);
  return element;
}
function text(name, content, x, y, width, height, fontSize, color = '#E8EAEDFF', align = 'left') {
  const element = widget('text', name, x, y, width, height);
  element.kind.content.text = content;
  Object.assign(element.kind.style, { size: fontSize, paint: color, align, font: { family: FONT, weight: 600, italic: false } });
  return element;
}
function panel(name, y, height) {
  const element = widget('shape', name, 24, y, 432, height);
  Object.assign(element.kind, { radius: 24, fill: '#121518FF' });
  return element;
}
const member = (element, card, face) => Object.assign(element, { cardMember: { parent: card.id, face } });

text('Overview label', 'SYSTEM / OVERVIEW', 32, 24, 416, 42, 24, '#FF9248FF');
const clock = widget('clock', 'Clock', 32, 74, 416, 128);
Object.assign(clock.kind.style, { size: 104, align: 'left', font: { family: FONT, weight: 700, italic: false } });
const date = widget('clock', 'Date', 32, 202, 416, 42);
date.kind.content.pattern = '%A, %d %B';
Object.assign(date.kind.style, { size: 27, align: 'left', paint: '#AEB5BDFF', font: { family: FONT, weight: 500, italic: false } });
panel('Processor panel', 280, 660);
text('Processor label', 'PROCESSOR', 52, 306, 376, 48, 28, '#FF9248FF');
const cpu = widget('ring', 'CPU load', 94, 386, 292, 292, { key: 'cpu.usage', quantity: 'percent' });
Object.assign(cpu.kind, { fill: '#FF9248FF', track: '#2C3238FF', thickness: 22 });
cpu.colorByValue = { base: '#FF9248FF', thresholds: [{ value: 70, color: '#FFC24BFF' }, { value: 90, color: '#FF5A4FFF' }] };
const value = widget('value', 'CPU percentage', 118, 468, 244, 104, { key: 'cpu.usage', quantity: 'percent' });
Object.assign(value.kind.style, { size: 74, align: 'center', font: { family: FONT, weight: 700, italic: false } });
text('CPU caption', 'CPU LOAD', 130, 568, 220, 42, 24, '#AEB5BDFF', 'center');
for (const [name, key, x, caption] of [['Memory usage', 'memory.percent', 52, 'RAM USED'], ['GPU usage', 'gpu.usage', 270, 'GPU LOAD']]) {
  const metric = widget('value', name, x, 750, 172, 86, { key, quantity: 'percent' });
  Object.assign(metric.kind.style, { size: 58, align: 'left', font: { family: FONT, weight: 700, italic: false } });
  text(`${name} caption`, caption, x, 844, 176, 42, 23, '#AEB5BDFF');
}
// A Card with two faces, turning every 30 s with one slide transition.
const card = widget('card', 'History card', 24, 972, 432, 366);
Object.assign(card.kind, { radius: 24, fill: '#121518FF' });
card.card = {
  faces: ['CPU history', 'GPU temperature'], activeFace: 0,
  transition: { effect: 'slide', direction: 'left', durationMs: 650, includeBase: false },
  triggers: [{ source: 'timer', intervalSeconds: 30, nextFace: true, app: '', face: 0, priority: 0, returnSeconds: 0, returnFace: null }],
};
member(text('History label', 'PROCESSOR HISTORY', 52, 998, 376, 48, 26, '#FF9248FF'), card, 0);
const graph = member(widget('graph', 'CPU history', 52, 1082, 376, 190, { key: 'cpu.usage', quantity: 'percent' }), card, 0);
Object.assign(graph.kind, { color: '#FF9248FF', fill: '#FF924830', lineWidth: 4 });
member(text('GPU label', 'GPU TEMPERATURE', 52, 998, 376, 48, 26, '#FF9248FF'), card, 1);
const gpu = member(widget('graph', 'GPU history', 52, 1082, 376, 190, { key: 'gpu.temperature', quantity: 'celsius' }), card, 1);
Object.assign(gpu.kind, { color: '#FF9248FF', fill: '#FF924830', lineWidth: 4 });
panel('Memory panel', 1370, 266);
text('Memory label', 'MEMORY', 52, 1396, 376, 48, 28, '#FF9248FF');
const ram = widget('value', 'Memory used', 52, 1460, 376, 82, { key: 'memory.percent', quantity: 'percent' });
Object.assign(ram.kind.style, { size: 64, align: 'left', font: { family: FONT, weight: 700, italic: false } });
const bar = widget('bar', 'Memory bar', 52, 1570, 376, 16, { key: 'memory.percent', quantity: 'percent' });
Object.assign(bar.kind, { fill: '#FF9248FF', track: '#2C3238FF', radius: 8 });
panel('Player panel', 1660, 190);
const player = widget('player', 'Now playing', 44, 1680, 392, 150);
Object.assign(player.kind.content, {
  showCover: true, showTitle: true, showArtist: true, showTime: true, showProgress: true, timeFormat: 'elapsedTotal',
  progressStyle: { fill: '#FF9248FF', track: '#2C3238FF', thickness: 6, ends: 'round', knob: true, pattern: 'segments', segments: 16, times: 'sides' },
});
Object.assign(player.kind.style, { size: 24, paint: '#E8EAEDFF', font: { family: FONT, weight: 600, italic: false } });
text('Demo label', 'EMBER / DEMO DATA', 32, 1866, 416, 40, 22, '#AEB5BDFF');

// Studio's own steps, by the labels a user reads.
const inspector = page => page.locator('#inspector');
async function selectLayer(page, name) {
  await page.locator('#tab-layers').click();
  await page.locator('.layer-name').filter({ has: page.locator('.layer-label').getByText(name, { exact: true }) }).first().click();
}
// The inspector's independent sections (Layout, Data, Shape, Text, Color, Behavior).
async function openSections(page, ...ids) {
  for (const id of ids) {
    const button = page.locator(`#inspector [data-inspector-section="${id}"] > .accordion-heading > button`);
    if (await button.count() && await button.getAttribute('aria-expanded') !== 'true') await button.click();
  }
  await page.waitForTimeout(500);
}
async function widgetsAtTop(page) {
  await page.locator('#tab-widgets').click();
  await page.waitForTimeout(300);
  await page.locator('#panel-widgets').evaluate(panel => { for (let node = panel; node; node = node.parentElement) node.scrollTop = 0; panel.querySelectorAll('*').forEach(node => { node.scrollTop = 0; }); });
}

const SHOTS = [
  // Hero: the shell, the library and the CPU ring with its live preview.
  { name: 'studio-hero', run: async page => {
    await selectLayer(page, 'CPU load');
    await widgetsAtTop(page);
  } },
  { name: 'studio-color-picker', run: async page => {
    await selectLayer(page, 'CPU load');
    await widgetsAtTop(page);
    await openSections(page, 'color');
    await inspector(page).getByRole('button', { name: t('inspector.fill'), exact: true }).first().click();
    await page.locator('.picker-popover').waitFor();
  } },
  { name: 'studio-font-picker', run: async page => {
    await selectLayer(page, 'Clock');
    await widgetsAtTop(page);
    await openSections(page, 'text');
    await inspector(page).getByRole('button', { name: t('inspector.fontFamily'), exact: true }).click();
    await page.locator('.picker-popover').waitFor();
  } },
  { name: 'studio-card-faces', run: async page => {
    await selectLayer(page, 'History card');
  } },
  { name: 'studio-player', run: async page => {
    await selectLayer(page, 'Now playing');
    await widgetsAtTop(page);
    await openSections(page, 'data', 'shape', 'text', 'color');
    await inspector(page).locator('[data-section="progress"]').scrollIntoViewIfNeeded();
  } },
  { name: 'studio-background', run: async page => {
    await page.locator('#tab-media').click();
    await inspector(page).getByRole('button', { name: t('insp.bgChange'), exact: true }).click();
    const head = inspector(page).locator('.bg-source-head').filter({ hasText: t('insp.bgComputer') });
    if ((await head.getAttribute('aria-expanded')) !== 'true') await head.click();
  } },
  { name: 'studio-sensors', query: '&libre=partial', run: async page => {
    await page.locator('#tab-sensors').click();
    await page.waitForTimeout(2500);
    await page.locator('#status-libre-toggle').click();
    await page.locator('#libre-popover').waitFor();
  } },
  { name: 'studio-light', scheme: 'light', run: async page => {
    await selectLayer(page, 'CPU load');
    await openSections(page, 'color');
    await inspector(page).locator('[data-section="value-color"]').scrollIntoViewIfNeeded();
  } },
  { name: 'studio-themes', run: async page => {
    await page.locator('#tab-themes').click();
    await page.getByRole('button', { name: t('themesPanel.gallery'), exact: true }).click();
    const dialog = page.getByRole('dialog', { name: t('library.themes'), exact: true });
    await dialog.waitFor();
    await dialog.getByRole('button', { name: t('themesPanel.filters'), exact: true }).click();
    await dialog.locator('#theme-scope [data-scope=all]').click();
    await dialog.getByRole('button', { name: t('themesPanel.filters'), exact: true }).click();
  } },
  { name: 'studio-profiles', run: async page => {
    await page.locator('#tab-themes').click();
    await page.locator('#theme-profiles').click();
    const dialog = page.getByRole('dialog').first();
    await dialog.waitFor();
    // Two example profiles, in this page only: nothing is saved.
    for (const [name, rule] of [['Night', t('widget.clock')], ['Music', t('profiles.mediaPlaying')]]) {
      await dialog.locator('#pf-add').click();
      await dialog.locator('#pf-name').fill(name);
      await dialog.locator('.pf-add-types').getByRole('button', { name: rule, exact: true }).click();
    }
    await dialog.getByRole('group', { name: t('profiles.transition'), exact: true }).getByRole('button', { name: t('profiles.effect.curtain'), exact: true }).click();
  } },
];

const port = await new Promise(resolve => {
  const server = createServer();
  server.listen(0, '127.0.0.1', () => { const port = server.address().port; server.close(() => resolve(port)); });
});
const server = spawn(process.platform === 'win32' ? 'python' : 'python3', ['scripts/serve-e2e.py', String(port), 'src'], { cwd: app, stdio: 'ignore', windowsHide: true });
const base = `http://localhost:${port}`;
const only = process.argv.slice(2);
let browser;

async function open(colorScheme, query = '') {
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, colorScheme, locale: 'en-US', timezoneId: 'UTC' });
  await context.addInitScript(() => localStorage.setItem('bezel.panelWidths', JSON.stringify({ library: 280, inspector: 360 })));
  // Windows ports in the pictures, whatever addresses the demo fixtures use.
  await context.route('**/demo-data.js', async route => {
    const body = (await (await route.fetch()).text()).replaceAll('/dev/ttyACM1', 'COM5').replaceAll('/dev/ttyACM0', 'COM4');
    await route.fulfill({ contentType: 'text/javascript', body });
  });
  // The example document, in memory: not a UI mockup and not a bundled theme.
  await context.route('**/demo-theme.js', route => route.fulfill({ contentType: 'text/javascript', body: `export const DEMO_THEME = ${JSON.stringify(theme)};` }));
  // Inter, the font Bezel includes, so the demo canvas draws what the screen would.
  await context.route('**/__capture-fonts/*', route => route.fulfill({ contentType: 'font/ttf', body: readFileSync(fonts + route.request().url().split('/').pop()) }));
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.clock.setFixedTime(new Date('2026-10-15T12:00:00Z'));
  await page.goto(`${base}/index.html?demo=turing88${query}`);
  await page.evaluate(async faces => {
    for (const [file, weight] of faces) {
      const face = new FontFace('Inter', await (await fetch(`/__capture-fonts/${file}`)).arrayBuffer(), { weight: String(weight) });
      document.fonts.add(await face.load());
    }
  }, [['Inter-Regular.ttf', 400], ['Inter-Medium.ttf', 500], ['Inter-SemiBold.ttf', 600], ['Inter-Bold.ttf', 700]]);
  await page.evaluate(() => document.fonts.ready);
  await page.locator('#theme-name').filter({ visible: true }).waitFor();
  await page.waitForFunction(() => document.getElementById('theme-name').value === 'Ember');
  await page.waitForTimeout(1500);
  return { context, page, errors };
}

try {
  for (let attempt = 0; ; attempt++) {
    try { if ((await fetch(`${base}/index.html`)).ok) break; } catch {}
    if (attempt > 60) throw new Error('Demo server did not start');
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  browser = await chromium.launch();
  for (const shot of SHOTS.filter(shot => !only.length || only.includes(shot.name))) {
    const { context, page, errors } = await open(shot.scheme ?? 'dark', shot.query);
    await shot.run(page);
    await page.waitForTimeout(1200);
    await page.screenshot({ path: `${output}${shot.name}.png`, animations: 'disabled', caret: 'hide' });
    console.log(`${shot.name}.png`);
    if (errors.length) throw new Error(`${shot.name}: ${errors.join('\n')}`);
    await context.close();
  }
  if (!only.length || only.includes('screen-render')) await screenRender();
} finally {
  await browser?.close();
  server.kill();
}

// A deliberately generic device illustration using the exact demo frame.
// No claim about a manufacturer's case, product photograph or hardware test.
async function screenRender() {
  const { context, page } = await open('dark');
  const canvas = await page.locator('#preview').evaluate(canvas => canvas.toDataURL('image/png'));
  const render = await context.newPage();
  const font = readFileSync(`${app}src/assets/fonts/ibm-plex/IBMPlexSans-Regular.woff2`).toString('base64');
  await render.setViewportSize({ width: 1200, height: 900 });
  await render.setContent(`<style>
    @font-face{font-family:Plex;src:url(data:font/woff2;base64,${font})}
    *{box-sizing:border-box}body{margin:0;background:#0B0D0F;color:#E8EAED;font-family:Plex,sans-serif}
    main{width:1200px;height:900px;display:flex;align-items:center;justify-content:center;gap:120px;background:radial-gradient(ellipse at 70% 70%,#241B14 0,transparent 55%)}
    .copy{width:420px}.eyebrow{font-size:16px;letter-spacing:4px;color:#FF9248}h1{font-size:64px;font-weight:400;line-height:1.1;letter-spacing:-2px;margin:30px 0}p{font-size:21px;color:#AEB5BD;line-height:1.6}.note{font-size:14px;margin-top:56px}
    .device{padding:12px;background:linear-gradient(110deg,#343A40,#121518 24%,#1A1D21);border:1px solid #43494E;border-radius:22px;box-shadow:0 40px 80px #000A,0 0 50px #FF92480A;transform:rotate(-4deg)}
    .device img{display:block;width:182px;height:728px;border-radius:9px}.ledge{height:8px;width:270px;background:#262C30;border:1px solid #353B40;border-radius:50%;margin:22px -30px -28px;box-shadow:0 18px 20px #0008}
    </style><main><div class="copy"><div class="eyebrow">BEZEL EVO</div><h1>Your screen.<br>Your layout.</h1><p>Clock, live PC stats and a little<br>room for your own style.</p><p class="note">Illustrative device render<br>Ember example theme / demo data</p></div><div class="device"><img alt="Ember theme with clock and CPU, GPU and memory widgets" src="${canvas}"><div class="ledge"></div></div></main>`);
  await render.evaluate(() => document.fonts.ready);
  await render.screenshot({ path: `${output}screen-render.png` });
  console.log('screen-render.png');
  await context.close();
}
