import { test, expect, clickAddWidget, openSection, expectAccessible, watchErrors } from './helpers.mjs';
const property = (page, id) => page.locator(`#overlay [data-property-handle="${id}"]`);
const field = (page, t, key) => page.locator('#inspector').getByRole('spinbutton', { name: t(key), exact: true });

test('ring measure drags highlight the actual inspector field and undo once', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'ring'); await openSection(page, 'shape');
  const before = Number(await field(page, t, 'insp.thickness').inputValue());
  const handle = property(page, 'thickness'); const box = await handle.boundingBox();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2 + 16, { steps: 5 });
  await expect(page.locator('.handle-readout')).toBeVisible();
  await expect(page.locator('#inspector .canvas-field-active')).toHaveCount(1);
  expect(Number(await field(page, t, 'insp.thickness').inputValue())).toBeGreaterThan(before);
  await page.mouse.up(); await expect(page.locator('.handle-readout')).toHaveCount(0);
  await page.locator('#undo').click(); await expect(field(page, t, 'insp.thickness')).toHaveValue(String(before));
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('property handles and the eight square handles are keyboard controls', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'shape'); await openSection(page, 'shape');
  const corners = field(page, t, 'inspector.radius'); const before = Number(await corners.inputValue());
  await property(page, 'corner-0').focus(); await property(page, 'corner-0').press('ArrowRight');
  expect(Number(await corners.inputValue())).toBeGreaterThan(before);
  await expect(property(page, 'corner-0')).toBeFocused();
  await page.locator('#undo').click(); await expect(corners).toHaveValue(String(before));
  await openSection(page, 'layout'); const width = field(page, t, 'inspector.width');
  const original = Number(await width.inputValue());
  await page.locator('#overlay .handle[data-h="e"]').focus(); await page.keyboard.press('ArrowRight');
  await expect(width).toHaveValue(String(original + 1));
  await expect(page.locator('#overlay .handle[data-h="e"]')).toBeFocused();
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('needle pivot supports coarse keyboard movement and one-step undo', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'needle'); await openSection(page, 'shape');
  await property(page, 'pivot').focus(); await property(page, 'pivot').press('Shift+ArrowRight');
  expect(Number(await field(page, t, 'inspector.pivotX').inputValue())).toBeGreaterThan(50);
  await expect(property(page, 'pivot')).toBeFocused();
  await page.locator('#undo').click(); await expect(field(page, t, 'inspector.pivotX')).toHaveValue('50');
  await expectAccessible(page); expect(errors).toEqual([]);
});
