// Static guards of the window's look (D-2026-10-09-studio-redesign-4/-6):
// the CSP stays exactly as it is, no font is fetched from Google, the sheets
// of src/styles/ are linked once each in the contract's order, every colour
// is a token (`:root` and hex values live only in tokens.css: the light
// `:root` block and the dark one inside `@media (prefers-color-scheme:
// dark)`), no sheet imports another, every selector of an area sheet names
// a class or an id, the IBM Plex faces are the bundled files, in the
// weights that ship (400, 500, 600), every px font size is a `--fs-*` token,
// each pill tone is one base.css rule, and the sparkline width has one source.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const APP = fileURLToPath(new URL('../../', import.meta.url));
const SRC = join(APP, 'src');
const CONF = join(APP, 'src-tauri', 'tauri.conf.json');
const STYLES = join(SRC, 'styles');
const INDEX = join(SRC, 'index.html');
/** The sheets, in the order index.html links them (the cascade's order). */
const SHEETS = ['tokens', 'base', 'shell', 'library', 'inspector', 'card', 'screen', 'connect'];
/** The sheets of one area each: their rules only style that area's classes and ids. */
const AREA_SHEETS = ['shell', 'library', 'inspector', 'card', 'screen', 'connect'];

const CSP =
  "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data: blob:; " +
  "font-src 'self'; media-src 'self' data: blob:; connect-src ipc: http://ipc.localhost; " +
  "object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

const HEX = /#[0-9a-f]{3,8}\b/i;
const WEIGHTS = new Set(['400', '500', '600']);
const TEXT_FILES = /\.(?:css|html|js|mjs|json|svg)$/i;

/** Every file under `dir`, recursively. */
function walk(dir) {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    return statSync(path).isDirectory() ? walk(path) : [path];
  });
}

/**
 * The rules of a stylesheet as a tree: `{ prelude, body }` for a rule with
 * declarations, `{ prelude, rules }` for an at-rule that holds rules.
 * Comments are dropped first; strings in this stylesheet hold no braces.
 */
function parseRules(css) {
  const text = css.replace(/\/\*[\s\S]*?\*\//g, '');
  let at = 0;
  function block() {
    const rules = [];
    let start = at;
    while (at < text.length) {
      const char = text[at];
      if (char === '}') {
        at += 1;
        return rules;
      }
      if (char === ';' && text.slice(start, at).trim().startsWith('@')) {
        at += 1;
        start = at;
        continue;
      }
      if (char !== '{') {
        at += 1;
        continue;
      }
      const prelude = text.slice(start, at).trim();
      at += 1;
      rules.push(prelude.startsWith('@media') || prelude.startsWith('@supports') ? { prelude, rules: block() } : { prelude, body: body() });
      start = at;
    }
    return rules;
  }
  function body() {
    const from = at;
    let depth = 1;
    while (at < text.length && depth > 0) {
      if (text[at] === '{') depth += 1;
      if (text[at] === '}') depth -= 1;
      at += 1;
    }
    return text.slice(from, at - 1);
  }
  return block();
}

/** `[property, value]` of each declaration of a rule body. */
function declarations(body) {
  return body
    .split(';')
    .map((part) => part.trim())
    .filter((part) => part.includes(':'))
    .map((part) => {
      const colon = part.indexOf(':');
      return [part.slice(0, colon).trim().toLowerCase(), part.slice(colon + 1).trim()];
    });
}

/** Rules with declarations, each with the at-rule preludes around it. */
function flatten(rules, within = []) {
  return rules.flatMap((rule) => (rule.rules ? flatten(rule.rules, [...within, rule.prelude]) : [{ ...rule, within }]));
}

/** Whether a rule is one of the two token blocks. */
function isTokenBlock(rule) {
  if (rule.prelude !== ':root') return false;
  if (rule.within.length === 0) return true;
  return rule.within.length === 1 && /^@media\s*\(\s*prefers-color-scheme\s*:\s*dark\s*\)$/.test(rule.within[0]);
}

/** The rules of each sheet, by name. */
const sheets = Object.fromEntries(SHEETS.map((name) => [name, flatten(parseRules(readFileSync(join(STYLES, `${name}.css`), 'utf8')))]));
const rules = Object.values(sheets).flat();

test('shared multi-declaration CSS bodies have one owner across sheets (D-14)', () => {
  const owners = new Map();
  for (const [sheet, entries] of Object.entries(sheets)) {
    if (sheet === 'tokens') continue;
    for (const rule of entries.filter(r => !r.prelude.startsWith('@'))) {
      const body = declarations(rule.body).map(([k, v]) => `${k}:${v}`).sort();
      if (body.length < 3) continue; // A single common spacing value is not a primitive.
      const signature = JSON.stringify([rule.within, body]);
      if (!owners.has(signature)) owners.set(signature, new Set());
      owners.get(signature).add(sheet);
    }
  }
  assert.deepEqual([...owners].filter(([, files]) => files.size > 1).map(([body, files]) => [body, [...files]]), []);
});

test('an inline SVG path is defined in only one UI module (D-14)', () => {
  const owners = new Map();
  for (const file of readdirSync(join(SRC, 'ui')).filter(f => f.endsWith('.js'))) {
    for (const [, path] of readFileSync(join(SRC, 'ui', file), 'utf8').matchAll(/\bd="([^"]+)"/g)) {
      if (!owners.has(path)) owners.set(path, new Set());
      owners.get(path).add(file);
    }
  }
  assert.deepEqual([...owners].filter(([, files]) => files.size > 1).map(([path, files]) => [path, [...files]]), []);
});

/** The selectors of a prelude, split at its top-level commas. */
function selectors(prelude) {
  const out = [];
  let depth = 0;
  let from = 0;
  for (let i = 0; i < prelude.length; i += 1) {
    if ('(['.includes(prelude[i])) depth += 1;
    if (')]'.includes(prelude[i])) depth -= 1;
    if (prelude[i] === ',' && depth === 0) {
      out.push(prelude.slice(from, i).trim());
      from = i + 1;
    }
  }
  return [...out, prelude.slice(from).trim()];
}

test('src/styles/ holds exactly the sheets of the contract, and index.html links each once, in order', () => {
  assert.deepEqual(readdirSync(STYLES).sort(), SHEETS.map((name) => `${name}.css`).sort());
  assert.equal(existsSync(join(SRC, 'styles.css')), false, 'the single sheet is gone');
  const links = [...readFileSync(INDEX, 'utf8').matchAll(/<link\s+rel="stylesheet"\s+href="([^"]+)"/g)].map((match) => match[1]);
  assert.deepEqual(links, SHEETS.map((name) => `styles/${name}.css`));
});

test('no sheet imports another', () => {
  const offenders = SHEETS.filter((name) => /@import\b/i.test(readFileSync(join(STYLES, `${name}.css`), 'utf8')));
  assert.deepEqual(offenders, []);
});

test('tokens.css holds only the two token blocks, and :root lives nowhere else', () => {
  assert.deepEqual(sheets.tokens.filter((rule) => !isTokenBlock(rule)).map((rule) => rule.prelude), []);
  const elsewhere = SHEETS.filter((name) => name !== 'tokens').flatMap((name) => sheets[name].filter((rule) => /:root\b/.test(rule.prelude)).map((rule) => `${name}: ${rule.prelude}`));
  assert.deepEqual(elsewhere, []);
});

test('every selector of an area sheet names a class or an id', () => {
  const offenders = AREA_SHEETS.flatMap((name) => sheets[name]
    .filter((rule) => !rule.prelude.startsWith('@'))
    .flatMap((rule) => selectors(rule.prelude).filter((selector) => !/[.#][A-Za-z_-]/.test(selector)).map((selector) => `${name}: ${selector}`)));
  assert.deepEqual(offenders, []);
});

test('the selector check splits at top-level commas only', () => {
  assert.deepEqual(selectors('.a, :is(.b, c) d, [x="1,2"]'), ['.a', ':is(.b, c) d', '[x="1,2"]']);
});

test('the CSP of tauri.conf.json is the fixed one', () => {
  const conf = JSON.parse(readFileSync(CONF, 'utf8'));
  assert.equal(conf.app.security.csp, CSP);
});

test('no file under src/ nor tauri.conf.json mentions a Google Fonts host', () => {
  const files = [...walk(SRC).filter((path) => TEXT_FILES.test(path)), CONF];
  const offenders = files.filter((path) => /fonts\.(?:googleapis|gstatic)/i.test(readFileSync(path, 'utf8')));
  assert.deepEqual(offenders.map((path) => relative(APP, path)), []);
});

test('the sheets have both token blocks, and the accent is defined once', () => {
  const tokens = rules.filter(isTokenBlock);
  assert.equal(tokens.length, 2, 'a light :root and a dark :root');
  const accents = tokens.flatMap((rule) => declarations(rule.body)).filter(([property]) => property === '--accent');
  assert.deepEqual(accents, [['--accent', '#FF9248']]);
});

test('no hex colour in a declaration of any sheet outside the token blocks', () => {
  const offenders = rules
    .filter((rule) => !isTokenBlock(rule))
    .flatMap((rule) => declarations(rule.body).filter(([, value]) => HEX.test(value)).map(([property, value]) => `${rule.prelude} { ${property}: ${value} }`));
  assert.deepEqual(offenders, []);
});

test('every @font-face url() is a bundled file and every font-weight is 400, 500 or 600', () => {
  const faces = rules.filter((rule) => rule.prelude === '@font-face');
  assert.ok(faces.length >= 5, 'IBM Plex Sans 400/500/600 and Mono 400/500');
  const urls = faces.flatMap((rule) => [...rule.body.matchAll(/url\(\s*(['"]?)([^'")]+)\1\s*\)/g)].map((match) => match[2]));
  assert.ok(urls.length >= faces.length);
  for (const url of urls) {
    assert.match(url, /^\.\.\/assets\/fonts\/ibm-plex\/[\w-]+\.woff2$/);
    assert.ok(existsSync(join(STYLES, url)), `${url} exists`);
  }
  for (const rule of faces) assert.match(rule.body, /font-display\s*:\s*swap/);
  const weights = rules.flatMap((rule) => declarations(rule.body).filter(([property]) => property === 'font-weight').map(([, value]) => [rule.prelude, value]));
  const wrong = weights.filter(([, value]) => !WEIGHTS.has(value) && !['inherit', 'normal'].includes(value));
  assert.deepEqual(wrong, []);
});

test('the parser keeps at-rules apart and sees through comments', () => {
  const parsed = flatten(parseRules(':root { --a: #fff; }\n/* #000 { } */\n@media (prefers-color-scheme: dark) { :root { --a: #000; } }\n.x { color: var(--a); }'));
  assert.deepEqual(parsed.map((rule) => [rule.prelude, rule.within.length, isTokenBlock(rule)]), [[':root', 0, true], [':root', 1, true], ['.x', 0, false]]);
});

test('each pill tone is a base.css primitive, defined by one rule (D-14)', () => {
  const tones = rules.filter((rule) => !rule.prelude.startsWith('@'))
    .flatMap((rule) => selectors(rule.prelude).map((selector) => [selector, rule]))
    .filter(([selector]) => /^\.pill\[data-tone="[a-z]+"\]$/.test(selector));
  const owners = tones.map(([selector, rule]) => [selector, Object.keys(sheets).find((name) => sheets[name].includes(rule))]);
  assert.deepEqual(owners.filter(([, sheet]) => sheet !== 'base'), [], 'no area sheet adds a tone');
  const seen = owners.map(([selector]) => selector);
  assert.deepEqual(seen, [...new Set(seen)], 'one rule per tone');
  assert.deepEqual(seen.map((selector) => selector.match(/"([a-z]+)"/)[1]).sort(), ['accent', 'danger', 'info', 'ok', 'warn']);
});

test('every font size in px is a --fs-* token: no sheet but tokens.css writes one', () => {
  const offenders = rules.filter((rule) => !isTokenBlock(rule))
    .flatMap((rule) => declarations(rule.body).filter(([property, value]) => property === 'font-size' && /\d(?:\.\d+)?px\b/.test(value)).map(([, value]) => `${rule.prelude} { font-size: ${value} }`));
  assert.deepEqual(offenders, []);
  const scale = sheets.tokens.filter(isTokenBlock).flatMap((rule) => declarations(rule.body)).filter(([property]) => property.startsWith('--fs-'));
  assert.deepEqual(scale.filter(([, value]) => !/^\d+(?:\.\d+)?px$/.test(value)), [], 'the scale itself is in px');
});

test('the sparkline width has one source, --spark-width, read by its sheet and by ui/library.js', () => {
  const token = sheets.tokens.filter(isTokenBlock).flatMap((rule) => declarations(rule.body)).filter(([property]) => property === '--spark-width');
  assert.deepEqual(token, [['--spark-width', '60px']]);
  const valueOf = (prelude, property) => sheets.library.filter((rule) => rule.prelude === prelude).flatMap((rule) => declarations(rule.body)).find(([p]) => p === property)?.[1];
  assert.equal(valueOf('.sparkline', 'width'), 'var(--spark-width)');
  // D-2026-10-20-studio-ux-v2-14: two-line rows; the sparkline keeps the token's width in its own cell.
  assert.equal(valueOf('.sensor-row', 'grid-template-columns'), 'minmax(0, 1fr) auto', 'name and value, then key and sparkline');
  const library = readFileSync(join(SRC, 'ui', 'library.js'), 'utf8');
  assert.match(library, /getPropertyValue\('--spark-width'\)/);
  assert.doesNotMatch(library, /viewBox', '0 0 \d/, 'the drawing takes its width from the token');
});
