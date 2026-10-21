import { test, expect, clickAddWidget, expectAccessible, watchErrors, layers } from './helpers.mjs';
const bar = page => page.locator('#overlay .canvas-object-bar');
const action = (page, name) => bar(page).locator(`[data-object-bar="${name}"]`);

test('the bar over the selection opens its sensor, tries it, duplicates and shares the object menu', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'ring');
  await expect(bar(page)).toHaveAttribute('role', 'toolbar');
  // Above the selection and its rotation handle.
  const box = await bar(page).boundingBox(), rotate = await page.locator('#overlay [data-rotate]').boundingBox();
  expect(box.y + box.height).toBeLessThanOrEqual(rotate.y);
  const key = await page.locator('#inspector [data-sensor-key]').first().getAttribute('data-sensor-key');
  await expect(action(page, 'sensor')).toHaveText(key);
  await action(page, 'sensor').click();
  await expect(page.locator('.sensor-picker-popover')).toBeVisible();
  await page.keyboard.press('Escape'); await expect(page.locator('.sensor-picker-popover')).toHaveCount(0);
  await action(page, 'try').click();
  await expect(page.locator('#inspector .inspector-try')).toHaveAttribute('aria-pressed', 'true');
  await action(page, 'menu').click();
  const menu = page.getByRole('menu', { name: t('menu.title') });
  await expect(menu).toBeVisible();
  await expect(menu.getByRole('menuitem', { name: t('common.duplicate'), exact: true })).toBeVisible();
  await page.keyboard.press('Escape'); await expect(menu).toHaveCount(0);
  await expect(action(page, 'menu')).toBeFocused();
  const count = await layers(page).count();
  await action(page, 'duplicate').click();
  await expect(layers(page)).toHaveCount(count + 1);
  await expect(action(page, 'duplicate')).toBeFocused();
  await page.locator('#undo').click(); await expect(layers(page)).toHaveCount(count);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('the bar hides while dragging and offers only common actions for several objects', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'shape');
  await expect(action(page, 'sensor')).toHaveCount(0);
  const frame = await page.locator('#overlay .sel-box').boundingBox();
  await page.mouse.move(frame.x + frame.width / 2, frame.y + frame.height / 2); await page.mouse.down();
  await page.mouse.move(frame.x + frame.width / 2 + 30, frame.y + frame.height / 2 + 10, { steps: 5 });
  await expect(bar(page)).toHaveCount(0);
  await page.mouse.up(); await expect(bar(page)).toHaveCount(1);
  await page.keyboard.press('Control+a');
  await expect(action(page, 'try')).toHaveCount(0);
  await expect(action(page, 'duplicate')).toBeVisible(); await expect(action(page, 'menu')).toBeVisible();
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('the bar moves below a selection whose top would hide under the stage toolbar', async ({ page }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await page.locator('#tab-layers').click();
  // The demo theme's clock sits at the top of the canvas, under the toolbar.
  await page.locator('#layer-list .layer-name').getByText('Clock', { exact: true }).first().click();
  const toolbar = await page.locator('.stage-tools').boundingBox(), shown = await bar(page).boundingBox();
  const frame = await page.locator('#overlay .sel-box').boundingBox();
  const overlaps = shown.y < toolbar.y + toolbar.height && shown.y + shown.height > toolbar.y && shown.x < toolbar.x + toolbar.width && shown.x + shown.width > toolbar.x;
  expect(overlaps).toBe(false);
  expect(shown.y).toBeGreaterThan(frame.y + frame.height);
  await expectAccessible(page); expect(errors).toEqual([]);
});
