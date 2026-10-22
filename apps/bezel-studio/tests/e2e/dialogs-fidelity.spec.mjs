import { test, expect, expectAccessible, watchErrors } from './helpers.mjs';

test('profile rule actions use six accessible tiles and compact priority fields', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=two');
  await page.locator('#tab-themes').click(); await page.locator('#theme-profiles').click();
  const dialog = page.locator('.profiles-dialog');
  await dialog.locator('#pf-add').click();
  const tiles = dialog.locator('.pf-add-types .option-tile');
  await expect(tiles).toHaveCount(6);
  for (const tile of await tiles.all()) { await expect(tile.locator('svg')).toHaveCount(1); expect(await tile.getAttribute('aria-pressed')).toBeNull(); }
  const boxes = await Promise.all((await tiles.all()).map(tile => tile.boundingBox()));
  expect(boxes[0].y).toBe(boxes[2].y); expect(boxes[3].y).toBe(boxes[5].y); expect(boxes[3].y).toBeGreaterThan(boxes[0].y);
  await dialog.locator('.pf-add-types').getByRole('button', { name: t('inspector.sensor'), exact: true }).click();
  const priority = dialog.getByRole('spinbutton', { name: t('trigger.priority'), exact: true });
  await expect(priority).toHaveValue('10');
  expect((await priority.locator('..').boundingBox()).width).toBeLessThan(240);
  await priority.fill('42'); await priority.press('Tab');
  await expect(dialog.locator('.pf-save-status')).toHaveText(t('profileRules.ui.save.saved'));
  await dialog.getByRole('group', { name: t('profiles.transition'), exact: true }).getByRole('button', { name: t('profiles.effect.curtain'), exact: true }).click();
  await expect(dialog.getByRole('spinbutton', { name: t('profiles.durationMs'), exact: true })).toBeVisible();
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('theme gallery contains real previews in low strips and groups the three secondary actions', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await page.locator('#tab-themes').click(); await page.locator('#theme-gallery').click();
  const dialog = page.locator('.themes-dialog');
  const strips = dialog.locator('.theme-card .thumb');
  await expect(strips.first()).toBeVisible();
  for (const strip of await strips.all()) expect((await strip.boundingBox()).height).toBe(60);
  const actions = dialog.locator('.gallery-actions .segmented > button');
  await expect(actions).toHaveCount(3);
  const boxes = await Promise.all((await actions.all()).map(button => button.boundingBox()));
  expect(boxes[0].y).toBe(boxes[1].y); expect(boxes[1].y).toBe(boxes[2].y);
  await expect(dialog.getByRole('button', { name: t('storage.deleteAction'), exact: true })).toBeDisabled();
  await expect(dialog.locator('.gallery-object-head .gallery-preview')).toBeVisible();
  expect(await dialog.evaluate(node => node.scrollWidth <= node.clientWidth)).toBe(true);
  await expectAccessible(page); expect(errors).toEqual([]);
});
