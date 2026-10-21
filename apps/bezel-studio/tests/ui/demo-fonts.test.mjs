import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { DEMO_FONT_CATALOG } from '../../src/demo-fonts.js';

const root = new URL('../../../../', import.meta.url);
const shipped = readFileSync(new URL('crates/bezel-render/src/fonts.rs', root), 'utf8');
const nativeFamilies = [...shipped.matchAll(/\("([^"]+)", \d+\.\.\d+\)/g)].map((m) => m[1]);

test('the demo exposes every native bundled family, including the template optical faces', () => {
  const bundled = DEMO_FONT_CATALOG.families.filter((f) => f.bundled);
  assert.deepEqual(bundled.map((f) => f.name).sort(), nativeFamilies.sort());
  for (const family of bundled) {
    assert.equal(new Set(family.faces.map((f) => `${f.weight}:${f.italic}`)).size, family.faces.length);
  }
});

test('all template typography resolves to an included face rather than a system fallback', () => {
  for (const slug of ['pixel-wide', 'nebula-wide', 'paper-wide', 'now-playing-wide']) {
    const theme = JSON.parse(readFileSync(new URL(`themes/${slug}/theme.json`, root), 'utf8'));
    for (const e of theme.elements) {
      if (e.kind.type !== 'text') continue;
      const font = e.kind.style.font;
      const family = DEMO_FONT_CATALOG.families.find((f) => f.name === font.family && f.bundled);
      assert.ok(family, `${slug}: ${font.family}`);
      assert.ok(family.faces.some((f) => f.weight === font.weight && f.italic === Boolean(font.italic)), `${slug}: ${font.family} ${font.weight}`);
    }
  }
  const files = readdirSync(new URL('themes/fonts/', root));
  for (const family of ['Fraunces', 'InstrumentSans', 'SpaceGrotesk', 'BricolageGrotesque']) {
    assert.ok(files.includes(`${family}-OFL.txt`));
  }
});

test('the canvas font names its family in quotes, so a name with digits or quotes is not misread', async () => {
  const { canvasFont } = await import('../../src/text-layout.js');
  assert.equal(canvasFont({ family: 'Fraunces 144pt', weight: 300, size: 96, italic: true }), 'italic 300 96px "Fraunces 144pt", sans-serif');
  assert.equal(canvasFont({ size: 10 }), '400 10px "sans-serif", sans-serif');
  assert.equal(canvasFont({ family: 'A "B"\\C\nD', weight: 600, size: 12 }), '600 12px "A \\"B\\"\\\\C D", sans-serif');
});
