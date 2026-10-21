// Visual review for the four data-only templates. Native frames come from
// bundled_themes.rs. The browser uses the existing demo renderer unchanged;
// its image-decoder limitation is explicit in the generated review page.
// node scripts/fidelity/shoot-templates.mjs [native preview directory] [output]
import { chromium } from '@playwright/test';
import { createServer } from 'node:http';
import { readFileSync, readdirSync, mkdirSync, copyFileSync, writeFileSync } from 'node:fs';
import { resolve, extname, join, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(fileURLToPath(new URL('../../../../', import.meta.url)));
const native = resolve(process.argv[2] ?? join(root, 'target/scratch/t9d/previews'));
const out = resolve(process.argv[3] ?? join(root, '.jdi/phases/studio-ux-v2/evidence/t9d'));
const boards = [['pixel-wide', 'ThemePixel'], ['nebula-wide', 'ThemeNebula'], ['paper-wide', 'ThemePaper'], ['now-playing-wide', 'ThemeNowPlaying']];
const types = { '.js': 'text/javascript', '.html': 'text/html', '.json': 'application/json', '.ttf': 'font/ttf', '.otf': 'font/otf', '.svg': 'image/svg+xml' };
const server = createServer((req, res) => {
  const path = resolve(root, `.${decodeURIComponent(new URL(req.url, 'http://local').pathname)}`);
  if (!path.startsWith(root + sep) || !['themes/', 'apps/bezel-studio/src/', '.jdi/phases/studio-ux-v2/design/boards/'].some(prefix => path.startsWith(resolve(root, prefix) + sep))) {
    res.writeHead(404); res.end(); return;
  }
  try { res.setHeader('Content-Type', types[extname(path)] ?? 'application/octet-stream'); res.end(readFileSync(path)); }
  catch { res.writeHead(404); res.end(); }
});
await new Promise(done => server.listen(0, '127.0.0.1', done));
const origin = `http://127.0.0.1:${server.address().port}`;
const weights = { Thin: 100, ExtraLight: 200, Light: 300, Regular: 400, Medium: 500, SemiBold: 600, Bold: 700, ExtraBold: 800, Black: 900, Italic: 400 };
const names = { Doto: ['Doto'], Doto_Rounded: ['Doto Rounded'], IBMPlexMono: ['IBM Plex Mono'], Inter: ['Inter'], JetBrainsMono: ['JetBrains Mono'], Fraunces: ['Fraunces', 'Fraunces 144pt'], InstrumentSans: ['Instrument Sans'], SpaceGrotesk: ['Space Grotesk'], BricolageGrotesque: ['Bricolage Grotesque', 'Bricolage Grotesque 96pt'] };
const fonts = readdirSync(join(root, 'themes/fonts')).flatMap(file => {
  const match = file.match(/^(.+)-([A-Za-z]+)\.(?:ttf|otf)$/);
  if (!match || !names[match[1]]) return [];
  const italic = match[2].endsWith('Italic');
  const weight = weights[match[2].replace(/Italic$/, '') || 'Regular'];
  return names[match[1]].map(family => ({ family, weight, style: italic ? 'italic' : 'normal', url: `${origin}/themes/fonts/${file}` }));
});
const css = fonts.map(f => `@font-face{font-family:'${f.family}';font-weight:${f.weight};font-style:${f.style};src:url('${f.url}')} `).join('\n');
const browser = await chromium.launch();
const manifest = [];
try {
  mkdirSync(join(out, 'native'), { recursive: true });
  for (const [slug] of boards) for (const suffix of ['it', 'unavailable.it']) {
    copyFileSync(join(native, `${slug}.${suffix}.png`), join(out, 'native', `${slug}.${suffix}.png`));
  }
  for (const scheme of ['dark', 'light']) {
    const context = await browser.newContext({ viewport: { width: 1920, height: 480 }, locale: 'it-IT', timezoneId: 'Europe/Rome', colorScheme: scheme });
    await context.addInitScript(() => {
      const OriginalDate = Date;
      globalThis.Date = class extends OriginalDate {
        constructor(...args) { super(...(args.length ? args : ['2026-09-30T19:05:42Z'])); }
        static now() { return new OriginalDate('2026-09-30T19:05:42Z').valueOf(); }
      };
    });
    await context.route('https://fonts.googleapis.com/**', route => route.fulfill({ contentType: 'text/css', body: css }));
    await context.route('https://fonts.gstatic.com/**', route => route.abort());
    for (const [slug, board] of boards) {
      const page = await context.newPage();
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      await page.goto(`${origin}/.jdi/phases/studio-ux-v2/design/boards/${board}.html`);
      await page.evaluate(() => document.fonts.ready);
      const directory = join(out, scheme);
      mkdirSync(directory, { recursive: true });
      await page.screenshot({ path: join(directory, `${slug}.board.png`) });
      const theme = JSON.parse(readFileSync(join(root, 'themes', slug, 'theme.json'), 'utf8'));
      const checks = await page.evaluate(async ({ theme, origin, fonts, slug }) => {
        document.body.replaceChildren(); document.body.style.margin = '0';
        for (const font of fonts) {
          const loaded = new FontFace(font.family, `url(${font.url})`, { weight: String(font.weight), style: font.style });
          document.fonts.add(await loaded.load());
        }
        const { renderApprox } = await import(`${origin}/apps/bezel-studio/src/demo-render.js`);
        const canvas = document.createElement('canvas'); canvas.width = 1920; canvas.height = 480;
        document.body.append(canvas);
        const ctx = canvas.getContext('2d'); ctx.fillStyle = theme.background.color; ctx.fillRect(0, 0, 1920, 480);
        // Compose the real vector asset once: renderApprox deliberately has no
        // image decoder. This is review plumbing, not another widget renderer.
        for (const e of theme.elements.filter(e => e.kind.type === 'image')) {
          const image = new Image(); image.src = `${origin}/themes/${slug}/${e.kind.asset}`;
          await image.decode(); ctx.drawImage(image, e.frame.x, e.frame.y, e.frame.width, e.frame.height);
        }
        const overlay = { ...theme, background: { type: 'color', color: '#00000000' }, elements: theme.elements.filter(e => e.kind.type !== 'image') };
        const frame = renderApprox(overlay, Date.now() / 1000, null, new Map(), 1, false);
        const layer = new OffscreenCanvas(1920, 480);
        layer.getContext('2d').putImageData(new ImageData(frame.rgba, 1920, 480), 0, 0);
        ctx.drawImage(layer, 0, 0);
        return theme.elements.filter(e => e.kind.type === 'text').map(e => ({ family: e.kind.style.font.family, available: document.fonts.check(`${e.kind.style.font.weight} 20px "${e.kind.style.font.family}"`) }));
      }, { theme, origin, fonts, slug });
      if (errors.length || checks.some(check => !check.available)) throw new Error(`${slug}: ${JSON.stringify({ errors, checks })}`);
      await page.screenshot({ path: join(directory, `${slug}.demo.png`) });
      manifest.push({ slug, board, scheme, locale: 'it', fontsAvailable: true, svgComposedByRunner: true });
      await page.close();
    }
    await context.close();
  }
  writeFileSync(join(out, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
  writeFileSync(join(out, 'index.html'), `<!doctype html><html lang="it"><meta charset="utf-8"><title>T-9d — temi nativi</title><style>body{font:16px system-ui;background:#17191c;color:#eee;margin:24px}img{display:block;width:100%;height:auto;border:1px solid #555;margin:8px 0 24px}h2{margin-top:40px}select{font:inherit}p{max-width:1100px}a{color:#ffb47a}</style><h1>T-9d — confronto dei quattro temi</h1><p>Frame nativi Rust e demo browser in italiano. Dati simulati esclusivamente nei test: non sono salvati nei temi. I valori cambiano secondo il PC. Il frame «indisponibile» verifica sensori e meteo assenti, nessuna sessione e audio non disponibile.</p><p>Il demo mantiene i suoi dati di esempio e i segnaposto delle copertine; questo runner compone gli SVG locali poiché il demo esistente non decodifica file immagine. Non è una prova di uguaglianza dei pixel con Rust. Chiaro/scuro riguarda l’ambiente Studio: i colori propri del tema restano invariati.</p><label>Ambiente <select id="scheme"><option>dark</option><option>light</option></select></label>${boards.map(([slug, board]) => `<section><h2>${slug}</h2><p>Tavola PNG originale</p><img src="../../design/png/${board}.png" alt="Tavola ${board}"><p>Renderer Rust · dati presenti</p><img src="native/${slug}.it.png" alt="${slug} nativo"><p>Renderer Rust · dati indisponibili</p><img src="native/${slug}.unavailable.it.png" alt="${slug} senza dati"><details><summary>Tavola HTML con font locali e demo browser</summary><img data-image="${slug}.board.png" src="dark/${slug}.board.png" alt="Tavola con font inclusi"><img data-image="${slug}.demo.png" src="dark/${slug}.demo.png" alt="Demo approssimato"></details></section>`).join('')}<script>document.querySelector('#scheme').onchange=e=>document.querySelectorAll('[data-image]').forEach(i=>i.src=e.target.value+'/'+i.dataset.image)</script></html>`);
  console.log(`Wrote ${manifest.length} Italian comparisons to ${out}`);
} finally {
  await browser.close();
  await new Promise(done => server.close(done));
}

