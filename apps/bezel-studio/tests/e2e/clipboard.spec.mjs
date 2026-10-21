import { test, expect, watchErrors, expectAccessible } from './helpers.mjs';

/** Opens the object menu from the keyboard (Shift+F10) on a selected layer. */
async function objectMenu(page) {
  await page.locator('#layer-list .layer-row.selected .layer-name').first().focus();
  await page.keyboard.press('Shift+F10');
  return page.getByRole('menu');
}

// Copy and paste moved from the floating bar to the object menu (artboard
// Main): the cases stay, the menu takes the toolbar's.
test('copy and paste objects with keyboard, object menu and grouped Undo', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  // Nothing selected and nothing copied: nothing to copy or paste.
  await page.locator('#stage-scroll').click({ button: 'right', position: { x: 8, y: 8 } });
  await expect(page.getByRole('menu')).toHaveCount(0);
  await page.getByRole('button', { name: t('library.addWidget', { name: t('widget.clock') }), exact: true }).click();
  await page.getByRole('tab', { name: t('library.layers'), exact: true }).click();
  await page.locator('#grid-toggle').focus();
  await page.keyboard.press('Control+c');
  const menu = await objectMenu(page);
  await expect(menu.getByRole('menuitem', { name: t('menu.paste'), exact: true })).toBeEnabled();
  await page.keyboard.press('Escape');
  await expect(menu).toHaveCount(0);
  await page.locator('#grid-toggle').focus();
  await page.keyboard.press('Control+v');
  await page.getByRole('tab', { name: t('library.layers'), exact: true }).click();
  const layers = page.locator('#layer-list .layer-row');
  await expect(layers).toHaveCount(5);
  await page.locator('#undo').click();
  await expect(layers).toHaveCount(4);
  await page.locator('#redo').click();
  await expect(layers).toHaveCount(5);
  await page.locator('#grid-toggle').focus();
  await page.keyboard.press('Control+a');
  await (await objectMenu(page)).getByRole('menuitem', { name: t('menu.copy'), exact: true }).click();
  await (await objectMenu(page)).getByRole('menuitem', { name: t('menu.paste'), exact: true }).click();
  await expect(layers).toHaveCount(10);
  await expect(page.locator('.sel-box:not(.selection-union)')).toHaveCount(5);
  await page.locator('#undo').click();
  await expect(layers).toHaveCount(5);
  // Ordinary text editing must never paste editor objects.
  await page.locator('#theme-name').focus();
  await page.keyboard.press('Control+c');
  await page.keyboard.press('Control+v');
  await expect(layers).toHaveCount(5);
  await expectAccessible(page);
  expect(errors).toEqual([]);
});
