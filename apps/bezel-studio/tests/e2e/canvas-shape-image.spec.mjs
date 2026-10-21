import { createWidget } from '../../src/editor/widgets.js';
import { test, expect, watchErrors, expectAccessible, openSection } from './helpers.mjs';
const canvas = { width: 480, height: 320 };
async function open(page, rotation) {
  const element = { ...createWidget('shape', canvas), id: 1, name: 'Pattern', frame: { x: 100, y: 80, width: 200, height: 120 }, rotation };
  element.kind.image = { asset: 'pattern.svg', repeat: 'tile', scale: 1, offsetX: 0, offsetY: 0, opacity: 1 };
  const theme = { schema: 1, name: 'Fill canvas', canvas, orientation: 'portrait', refreshSeconds: 1,
    background: { type: 'color', color: '#000000ff' }, elements: [element] };
  await page.route('**/demo-theme.js', route => route.fulfill({ contentType: 'text/javascript', body: `export const DEMO_THEME = ${JSON.stringify(theme)};` }));
  await page.goto('/index.html?demo=turing88'); await expect(page.locator('#theme-name')).toHaveValue('Fill canvas');
  await page.locator('#tab-layers').click(); await page.locator('.layer-name').click(); await openSection(page, 'shape');
}
const field = (page, t, key) => page.locator('#inspector').getByRole('spinbutton', { name: t(key), exact: true });
for (const rotation of [0, 90]) test(`shape image pans locally at ${rotation} degrees and scales without moving its frame`, async ({ page, t }) => {
  const errors = watchErrors(page); await open(page, rotation);
  const selected = page.locator('#overlay .sel-box[role="img"]'), frame = await selected.getAttribute('style');
  const box = await selected.boundingBox(), center = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
  await page.mouse.dblclick(center.x, center.y);
  await expect(page.locator('.canvas-edit-tools')).toBeVisible(); await expect(page.locator('#overlay .handle')).toHaveCount(0);
  const zoom = (await page.locator('#canvas-box').boundingBox()).width / canvas.width;
  await page.mouse.move(center.x, center.y); await page.mouse.down(); await page.mouse.move(center.x + 20, center.y, { steps: 5 });
  await expect(page.locator('.handle-readout')).toBeVisible(); await page.mouse.up();
  const x = field(page, t, 'shape.imageOffsetX'), y = field(page, t, 'shape.imageOffsetY');
  await expect(x).toHaveValue(String(rotation ? 0 : Math.round(20 / zoom)));
  await expect(y).toHaveValue(String(rotation ? Math.round(-20 / zoom) : 0));
  await expect(selected).toHaveAttribute('style', frame);
  await page.locator('#undo').click(); await expect(x).toHaveValue('0'); await expect(y).toHaveValue('0');
  await page.locator('#stage-scroll').focus(); await page.keyboard.press('Shift+ArrowRight'); await expect(x).toHaveValue('10');
  await page.locator('#undo').click(); await expect(x).toHaveValue('0');
  const size = field(page, t, 'shape.imageScale'); await expect(size).toHaveValue('100');
  await page.mouse.move(center.x, center.y); await page.mouse.wheel(0, -100);
  await expect(size).not.toHaveValue('100'); await expect(page.locator('#zoom-label')).toHaveText(`${Math.round(zoom * 100)}%`);
  await page.locator('#undo').click(); await expect(size).toHaveValue('100');
  await page.locator('#stage-scroll').focus(); await page.keyboard.press('+'); await expect(size).toHaveValue('110');
  await page.keyboard.press('Control+z'); await expect(size).toHaveValue('100');
  await page.keyboard.press('Escape');
  await expect(page.locator('.canvas-edit-tools')).toHaveCount(0); await expect(page.locator('#overlay .handle')).toHaveCount(8);
  await expect(size).toHaveValue('100');
  await expect(selected).toHaveAttribute('style', frame);
  await page.locator('#inspector').getByRole('button', { name: t('shape.repeat.cover'), exact: true }).click();
  await page.mouse.dblclick(center.x, center.y); await expect(page.locator('.canvas-edit-tools')).toHaveCount(0);
  await expectAccessible(page); expect(errors).toEqual([]);
});
