// Screenshots of the design canvas and of the studio (demo mode, locale it)
// for the fidelity work: one recipe per area (scripts/fidelity/recipes),
// each naming its artboards and the app states that match them (a state
// may pick another demo scenario with `demo` and add parameters with
// `query`, e.g. `&libre=partial`).
//
//   node scripts/fidelity/shoot.mjs [--area all|shell|library|inspector|card|screen]
//        [--artboard Name] [--scheme both|dark|light] [--only mock|app] [--out DIR]
//   node scripts/fidelity/shoot.mjs --diff DIR_A DIR_B
//
// It starts its own servers (the e2e static server for the app, the DC shim
// for the mockups), serves the Google Fonts the mockups ask for from the
// bundled woff2 files, and writes `{out}/{scheme}/mock/{Artboard}.png`,
// `{out}/{scheme}/app/{area}/{state}.png` and `{out}/manifest.json`. An
// artboard the shim could not expand is shot as it is and marked STATIC.
// `--diff` compares two such trees pixel by pixel and exits 1 on a difference.
import { spawn } from 'node:child_process';
import { mkdirSync, readFileSync, readdirSync, statSync, writeFileSync, existsSync } from 'node:fs';
import { createServer } from 'node:net';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { DESIGN, startMockServer } from './serve-mock.mjs';

const APP = fileURLToPath(new URL('../../', import.meta.url));
const FONTS = join(APP, 'src', 'assets', 'fonts', 'ibm-plex');
const DEFAULT_OUT = fileURLToPath(new URL('../../../../.jdi/phases/studio-redesign-fidelity/gap/', import.meta.url));
const AREAS = ['shell', 'library', 'inspector', 'card', 'screen'];
const VIEWPORT = { width: 1440, height: 900 };
/** Every app shot sees the same clock, so dates and demo sensors match. */
const FIXED_TIME = new Date('2026-10-09T10:00:00Z');
const SETTLE_MS = 2500;
const STEP_MS = 600;

/** The bundled file for each face a mockup asks Google Fonts for. */
const FACES = [
  ['IBM Plex Sans', 400, 'IBMPlexSans-Regular'],
  ['IBM Plex Sans', 500, 'IBMPlexSans-Medium'],
  ['IBM Plex Sans', 600, 'IBMPlexSans-SemiBold'],
  ['IBM Plex Sans', 700, 'IBMPlexSans-SemiBold'],
  ['IBM Plex Mono', 400, 'IBMPlexMono-Regular'],
  ['IBM Plex Mono', 500, 'IBMPlexMono-Medium'],
  ['IBM Plex Mono', 600, 'IBMPlexMono-Medium'],
];
const FONT_CSS = FACES.map(([family, weight, file]) => `@font-face { font-family: '${family}'; font-style: normal; font-weight: ${weight}; font-display: block; src: url(https://fonts.gstatic.com/bezel/${file}.woff2) format('woff2'); }`).join('\n');

function parseArgs(argv) {
  const args = { area: 'all', artboard: null, scheme: 'both', only: null, out: DEFAULT_OUT, diff: null };
  for (let i = 0; i < argv.length; i += 1) {
    const flag = argv[i];
    if (flag === '--merge') {
      args.merge = true;
    } else if (flag === '--diff') {
      args.diff = [argv[i + 1], argv[i + 2]];
      i += 2;
    } else if (['--area', '--artboard', '--scheme', '--only', '--out', '--state'].includes(flag)) {
      args[flag.slice(2)] = argv[i + 1];
      i += 1;
    } else {
      throw new Error(`unknown argument ${flag}`);
    }
  }
  if (args.area !== 'all' && !AREAS.includes(args.area)) throw new Error(`--area is one of all, ${AREAS.join(', ')}`);
  if (!['both', 'dark', 'light'].includes(args.scheme)) throw new Error('--scheme is both, dark or light');
  if (args.only && !['mock', 'app'].includes(args.only)) throw new Error('--only is mock or app');
  return args;
}

function freePort() {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      server.close(() => resolve(port));
    });
  });
}

async function waitFor(url, tries = 100) {
  for (let i = 0; i < tries; i += 1) {
    try {
      if ((await fetch(url)).ok) return;
    } catch {
      // not up yet
    }
    await new Promise((done) => setTimeout(done, 100));
  }
  throw new Error(`${url} did not answer`);
}

/** The e2e static server (scripts/serve-e2e.py) on a free port. */
async function startAppServer() {
  const port = await freePort();
  const python = process.platform === 'win32' ? 'python' : 'python3';
  const child = spawn(python, ['scripts/serve-e2e.py', String(port), 'src'], { cwd: APP, stdio: 'ignore' });
  const url = `http://localhost:${port}`;
  await waitFor(`${url}/index.html`);
  return { url, close: () => child.kill() };
}

async function routeFonts(context) {
  await context.route(/^https:\/\/fonts\.googleapis\.com\//, (route) => route.fulfill({ contentType: 'text/css', body: FONT_CSS }));
  await context.route(/^https:\/\/fonts\.gstatic\.com\//, (route) => {
    const file = new URL(route.request().url()).pathname.split('/').pop();
    route.fulfill({ contentType: 'font/woff2', body: readFileSync(join(FONTS, file)) });
  });
}

async function loadRecipes(area) {
  const names = area === 'all' ? AREAS : [area];
  return Promise.all(names.map(async (name) => (await import(`./recipes/${name}.mjs`)).default));
}

async function shootMocks(browser, mockUrl, artboards, scheme, out, design = DESIGN) {
  const canvasPath = join(design, 'canvas.json');
  const canvas = existsSync(canvasPath) ? JSON.parse(readFileSync(canvasPath, 'utf8')) : null;
  const indexPath = join(design, 'index.json');
  const index = existsSync(indexPath) ? JSON.parse(readFileSync(indexPath, 'utf8')) : {};
  const entries = [];
  for (const name of artboards) {
    const extension = existsSync(join(design, `${name}.html`)) ? 'html' : 'dc.html';
    const preview = canvas ? null : extension === 'html' ? (index[name] ?? { width: 1280, height: 1000 }) : JSON.parse(readFileSync(join(design, `${name}.dc.html`), 'utf8').match(/data-props='([^']+)'/)[1]).$preview;
    const board = canvas?.boards[`${name}.dc.html`] ?? { w: preview.width, h: preview.height };
    const context = await browser.newContext({ viewport: { width: board.w, height: board.h }, colorScheme: scheme });
    await routeFonts(context);
    const page = await context.newPage();
    const response = await page.goto(`${mockUrl}/${name}.${extension}`);
    await page.evaluate(() => document.fonts.ready);
    const mode = response.headers()['x-dc-mode'] ?? 'static';
    const reason = response.headers()['x-dc-reason'];
    const file = join(out, scheme, 'mock', `${name}.png`);
    mkdirSync(dirname(file), { recursive: true });
    await page.screenshot({ path: file, fullPage: extension === 'html' });
    await context.close();
    entries.push({ kind: 'mock', scheme, artboard: name, file, mode, reason: reason && decodeURIComponent(reason) });
    console.log(`mock ${scheme} ${name}${mode === 'static' ? ` STATIC (${decodeURIComponent(reason ?? '')})` : ''}`);
  }
  return entries;
}

/** The steps a recipe can take on the app page. */
function appSteps(page) {
  const settle = () => page.waitForTimeout(STEP_MS);
  return {
    page,
    click: async (selector) => {
      await page.locator(selector).click();
      await settle();
    },
    tab: async (id) => {
      await page.locator(`#tab-${id}`).click();
      await settle();
    },
    add: async (widget) => {
      await page.locator('#tab-widgets').click();
      await page.locator(`.widget-tile[data-widget="${widget}"]`).click();
      await settle();
    },
    inspectorTab: async (index) => {
      await page.locator('#inspector [role="tab"]').nth(index).click();
      await settle();
    },
  };
}

async function shootApp(browser, appUrl, recipe, scheme, out) {
  const entries = [];
  for (const state of recipe.states) {
    const context = await browser.newContext({ viewport: state.viewport ?? VIEWPORT, colorScheme: scheme, locale: 'it-IT' });
    const page = await context.newPage();
    await page.clock.setFixedTime(FIXED_TIME);
    await page.goto(`${appUrl}/index.html?demo=${state.demo ?? 'turing88'}${state.query ?? ''}`);
    await page.evaluate(() => document.fonts.ready);
    await page.waitForTimeout(SETTLE_MS);
    await state.run(appSteps(page));
    const file = join(out, scheme, 'app', recipe.area, `${state.name}.png`);
    mkdirSync(dirname(file), { recursive: true });
    // The render time is the one value the clock cannot fix.
    await page.screenshot({ path: file, animations: 'disabled', caret: 'hide', mask: [page.locator('#status-render')] });
    let detail;
    if (state.capture) {
      detail = join(out, scheme, 'app', recipe.area, `${state.name}-detail.png`);
      await page.locator(state.capture).screenshot({ path: detail, animations: 'disabled', caret: 'hide' });
    }
    await context.close();
    entries.push({ kind: 'app', scheme, area: recipe.area, state: state.name, file, ...(detail ? { detail } : {}) });
    console.log(`app ${scheme} ${recipe.area}/${state.name}`);
  }
  return entries;
}

async function shoot(args) {
  const recipes = (await loadRecipes(args.area)).map(recipe => ({ ...recipe,
    states: recipe.states.filter(state => !args.state || args.state.split(',').includes(state.name)),
  }));
  if (args.state && !recipes.some(recipe => recipe.states.length)) throw new Error(`Unknown state: ${args.state}`);
  const schemes = args.scheme === 'both' ? ['dark', 'light'] : [args.scheme];
  const browser = await chromium.launch();
  const mock = args.only === 'app' ? null : await startMockServer();
  const app = args.only === 'mock' ? null : await startAppServer();
  const extraMocks = new Map();
  const entries = [];
  try {
    if (mock) for (const recipe of recipes) for (const design of recipe.mockDesigns ?? []) {
      if (!extraMocks.has(design.root)) extraMocks.set(design.root, await startMockServer({ root: design.root }));
    }
    for (const scheme of schemes) {
      for (const recipe of recipes) {
        const artboards = recipe.artboards.filter((name) => !args.artboard || name === args.artboard);
        if (mock) entries.push(...(await shootMocks(browser, mock.url, artboards, scheme, args.out)));
        for (const design of recipe.mockDesigns ?? []) {
          const boards = design.artboards.filter((name) => !args.artboard || name === args.artboard);
          if (mock) entries.push(...(await shootMocks(browser, extraMocks.get(design.root).url, boards, scheme, args.out, design.root)));
        }
        if (app && (!args.artboard || artboards.length || recipe.mockDesigns?.some(design => design.artboards.includes(args.artboard)))) entries.push(...(await shootApp(browser, app.url, recipe, scheme, args.out)));
      }
    }
  } finally {
    await browser.close();
    await mock?.close();
    for (const server of extraMocks.values()) await server.close();
    app?.close();
  }
  mkdirSync(args.out, { recursive: true });
  const captured = entries.map(entry => ({ ...entry, file: relative(args.out, entry.file).replaceAll('\\', '/'), ...(entry.detail ? { detail: relative(args.out, entry.detail).replaceAll('\\', '/') } : {}) }));
  const identity = entry => JSON.stringify([entry.kind, entry.scheme, entry.artboard ?? [entry.area, entry.state]]);
  const updated = new Set(captured.map(identity));
  const kept = args.merge ? JSON.parse(readFileSync(join(args.out, 'manifest.json'), 'utf8')).filter(entry => !updated.has(identity(entry))) : [];
  writeFileSync(join(args.out, 'manifest.json'), `${JSON.stringify([...kept, ...captured], null, 2)}\n`);
  console.log(`${entries.length} screenshots in ${args.out}`);
}

function pngs(root) {
  return readdirSync(root).flatMap((name) => {
    const path = join(root, name);
    if (statSync(path).isDirectory()) return pngs(path);
    return name.endsWith('.png') ? [path] : [];
  });
}

/** How many pixels of two PNGs differ (decoded in the browser), or null when the sizes differ. */
async function pixelDiff(page, a, b) {
  return page.evaluate(async ([left, right]) => {
    const decode = async (base64) => createImageBitmap(await (await fetch(`data:image/png;base64,${base64}`)).blob());
    const [one, two] = await Promise.all([decode(left), decode(right)]);
    if (one.width !== two.width || one.height !== two.height) return null;
    const pixels = (bitmap) => {
      const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
      const context = canvas.getContext('2d');
      context.drawImage(bitmap, 0, 0);
      return context.getImageData(0, 0, bitmap.width, bitmap.height).data;
    };
    const [p, q] = [pixels(one), pixels(two)];
    let differ = 0;
    for (let i = 0; i < p.length; i += 4) {
      if (p[i] !== q[i] || p[i + 1] !== q[i + 1] || p[i + 2] !== q[i + 2] || p[i + 3] !== q[i + 3]) differ += 1;
    }
    return differ;
  }, [readFileSync(a).toString('base64'), readFileSync(b).toString('base64')]);
}

async function diff([left, right]) {
  const files = pngs(left).map((path) => relative(left, path));
  const browser = await chromium.launch();
  const page = await browser.newPage();
  let failed = 0;
  for (const file of files) {
    let result;
    try {
      result = await pixelDiff(page, join(left, file), join(right, file));
    } catch {
      result = 'missing';
    }
    const same = result === 0;
    if (!same) failed += 1;
    console.log(`${same ? 'same' : 'DIFF'} ${file.replaceAll('\\', '/')}${same ? '' : ` (${result === null ? 'size' : result} px)`}`);
  }
  await browser.close();
  console.log(`${files.length - failed}/${files.length} identical`);
  if (failed) process.exitCode = 1;
}

const args = parseArgs(process.argv.slice(2));
await (args.diff ? diff(args.diff) : shoot(args));
