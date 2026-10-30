import { createWidget } from '../../src/editor/widgets.js';
import { test, expect, watchErrors, expectAccessible, openSection } from './helpers.mjs';
const canvas = { width: 480, height: 320 };
async function open(page, widget) {
  const element = { ...createWidget(widget, canvas), id: 1, name: 'Target', frame: { x: 100, y: 80, width: 200, height: 120 } };
  if (widget === 'shape') element.kind = { ...element.kind, strokeWidth: 2, stroke: '#ffffffff' };
  const theme = { schema: 1, name: 'From computer', canvas, orientation: 'portrait', refreshSeconds: 1,
    background: { type: 'color', color: '#000000ff' }, elements: [element] };
  await page.route('**/demo-theme.js', route => route.fulfill({ contentType: 'text/javascript', body: `export const DEMO_THEME = ${JSON.stringify(theme)};` }));
  await page.goto('/index.html?demo=turing88'); await expect(page.locator('#theme-name')).toHaveValue('From computer');
  await page.locator('#tab-layers').click(); await page.locator('.layer-name').click();
}

for (const widget of ['shape', 'image']) test(`a ${widget} takes a picture from the computer without visiting Media`, async ({ page, t }) => {
  const errors = watchErrors(page); await open(page, widget);
  for (const id of ['data', 'shape', 'color']) await openSection(page, id);
  const picker = page.locator('#inspector .image-field select').first();
  await expect(picker).toHaveCount(1);
  await picker.selectOption({ label: t('insp.image.fromComputer') });
  await expect(picker).toHaveValue(/^assets\/image-\d+\.png$/);
  await page.locator('#tab-media').click();
  await expect(page.locator('.media-item.in-use .media-uses')).toContainText('Target');
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('an outline colour lines up with its width', async ({ page }) => {
  await open(page, 'shape'); for (const id of ['shape', 'color']) await openSection(page, id);
  const row = page.locator('#inspector .fld-grid:has(> .number-field):has(> .color-field)').first();
  const width = await row.locator('.number-control').boundingBox(), colour = await row.locator('.color-button').boundingBox();
  expect(Math.abs((width.y + width.height) - (colour.y + colour.height))).toBeLessThanOrEqual(1);
});
