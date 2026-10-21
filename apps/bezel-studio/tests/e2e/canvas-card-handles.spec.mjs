import { createWidget } from '../../src/editor/widgets.js';
import { test, expect, expectAccessible, watchErrors } from './helpers.mjs';
const canvas = { width: 480, height: 320 };
const card = { ...createWidget('card', canvas), id: 1, name: 'Card', frame: { x: 40, y: 40, width: 300, height: 200 } };
card.card.faces = ['First', 'Second'];
const member = face => ({ ...createWidget('text', canvas), id: face + 2, name: `Member ${face}`, frame: { x: 80, y: 80, width: 100, height: 50 }, cardMember: { parent: 1, face } });
async function open(page, locked = false) {
  const theme = { schema: 1, name: 'Card canvas', canvas, orientation: 'portrait', refreshSeconds: 1,
    background: { type: 'color', color: '#000000ff' }, elements: [{ ...card, locked }, member(0), member(1)] };
  await page.route('**/demo-theme.js', route => route.fulfill({ contentType: 'text/javascript', body: `export const DEMO_THEME = ${JSON.stringify(theme)};` }));
  await page.goto('/index.html?demo=turing88'); await expect(page.locator('#theme-name')).toHaveValue('Card canvas');
  await page.locator('#tab-layers').click();
  await page.locator('.layer-row[data-element-id="1"] .layer-name').click();
}
const face = (page, id) => page.locator(`#overlay [data-canvas-face="${id}"]`);
async function memberPoint(page) {
  const bounds = await page.locator('#canvas-box').boundingBox();
  return { x: bounds.x + 130 / canvas.width * bounds.width, y: bounds.y + 105 / canvas.height * bounds.height };
}

test('Card canvas face choices switch and add through shared commands with Undo', async ({ page, t }) => {
  const errors = watchErrors(page); await open(page);
  const choices = page.locator('.canvas-card-faces');
  await expect(face(page, 0)).toHaveAttribute('aria-pressed', 'true');
  await expect(page.locator('#overlay [data-rotate]')).toHaveCount(0);
  await face(page, 1).focus(); await page.keyboard.press('Enter');
  await expect(face(page, 1)).toHaveAttribute('aria-pressed', 'true'); await expect(face(page, 1)).toBeFocused();
  await page.locator('#undo').click(); await expect(face(page, 0)).toHaveAttribute('aria-pressed', 'true');
  await choices.getByRole('button', { name: t('card.addFace'), exact: true }).click();
  await expect(face(page, 2)).toHaveText(t('card.faceName', { number: 3 })); await expect(face(page, 2)).toBeFocused();
  await expect(face(page, 2)).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#undo').click(); await expect(face(page, 2)).toHaveCount(0);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('double click enters Card members and Escape returns to the Card', async ({ page, t }) => {
  const errors = watchErrors(page); await open(page);
  const point = await memberPoint(page);
  await page.mouse.click(point.x, point.y);
  await expect(page.locator('.layer-row.selected')).toHaveAttribute('data-element-id', '1');
  await page.mouse.dblclick(point.x, point.y);
  await expect(page.locator('.layer-row.selected')).toHaveAttribute('data-element-id', '2');
  await expect(page.locator('.card-edit-boundary')).toHaveCount(1);
  await expect(page.locator('#overlay [data-rotate]')).toHaveCount(0);
  await expect(page.locator('.canvas-text-editor')).toHaveCount(0);
  await page.mouse.dblclick(point.x, point.y); await expect(page.locator('.canvas-text-editor')).toBeFocused();
  await page.keyboard.press('Escape'); await expect(page.locator('.card-edit-boundary')).toHaveCount(1);
  await page.keyboard.press('Escape'); await expect(page.locator('.layer-row.selected')).toHaveAttribute('data-element-id', '1');
  await expect(page.locator('.card-edit-boundary')).toHaveCount(0);
  await page.mouse.dblclick(point.x, point.y);
  await page.getByRole('button', { name: t('shell.exitCard'), exact: true }).click();
  await expect(page.locator('.layer-row.selected')).toHaveAttribute('data-element-id', '1');
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('locked Cards show disabled face choices and cannot enter an inline editor', async ({ page }) => {
  const errors = watchErrors(page); await open(page, true);
  for (const button of await page.locator('.canvas-card-faces button').all()) await expect(button).toBeDisabled();
  const point = await memberPoint(page); await page.mouse.dblclick(point.x, point.y);
  await expect(page.locator('.card-edit-boundary, .canvas-text-editor, #overlay .handle')).toHaveCount(0);
  await expectAccessible(page); expect(errors).toEqual([]);
});
