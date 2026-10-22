import { test, expect, watchErrors, expectAccessible, clickAddWidget, openSection, setPanelWidths } from './helpers.mjs';

test('fade preview commits one drag as one Undo and supports radial keyboard editing', async ({ page, t }) => {
  const errors = watchErrors(page);
  await setPanelWidths(page, { inspector: 320 });
  await page.goto('/index.html?demo=turing88'); await clickAddWidget(page, t, 'shape');
  await openSection(page, 'color');
  const root = page.locator('#inspector'), fade = root.locator('[data-section="transparency"]');
  await fade.getByRole('button', { name: t('fade.linear'), exact: true }).click();
  const angle = () => fade.getByRole('spinbutton', { name: t('shape.fadeAngle'), exact: true });
  const preview = fade.getByRole('slider', { name: t('insp.fade.directionControl'), exact: true });
  await preview.scrollIntoViewIfNeeded();
  const box = await preview.boundingBox();
  await page.mouse.move(box.x + box.width * .8, box.y + box.height * .5);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width * .5, box.y + box.height * .8, { steps: 8 });
  await expect(angle()).toHaveValue('0'); // Preview only until release.
  await page.mouse.up(); await expect(angle()).toHaveValue('90');
  await page.locator('#undo').click(); await expect(angle()).toHaveValue('0');
  await expect(fade.getByRole('button', { name: t('fade.linear'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await preview.focus(); await preview.press('ArrowLeft'); await expect(angle()).toHaveValue('-5');
  await page.locator('#undo').click(); await expect(angle()).toHaveValue('0');
  await fade.getByRole('button', { name: t('fade.radial'), exact: true }).click();
  const radial = fade.getByRole('slider', { name: t('insp.fade.centerControl'), exact: true });
  await radial.focus(); await radial.press('ArrowRight');
  await expect(fade.getByRole('spinbutton', { name: t('fade.centerX'), exact: true })).toHaveValue('51');
  await expect(fade.getByRole('spinbutton', { name: t('fade.centerY'), exact: true })).toHaveValue('50');
  await expect(fade.getByRole('spinbutton', { name: t('fade.extent'), exact: true })).toHaveValue('50');
  await page.locator('#undo').click();
  await expect(fade.getByRole('spinbutton', { name: t('fade.centerX'), exact: true })).toHaveValue('50');
  await radial.scrollIntoViewIfNeeded(); const centre = await radial.boundingBox();
  await page.mouse.move(centre.x + 10, centre.y + 10); await page.mouse.down();
  await page.mouse.move(centre.x + 50, centre.y + 50);
  await radial.dispatchEvent('pointercancel'); await page.mouse.up();
  await expect(fade.getByRole('spinbutton', { name: t('fade.centerX'), exact: true })).toHaveValue('50');
  await page.locator('#undo').click();
  await expect(fade.getByRole('button', { name: t('fade.linear'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  expect(await root.evaluate(n => n.scrollWidth <= n.clientWidth + 1)).toBe(true);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('image asset card shows actual dimensions and reuses Media for browsing', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await page.locator('#tab-media').click();
  await page.getByRole('tab', { name: t('media.icons'), exact: true }).click();
  await page.locator('#icon-search').fill('cpu'); await page.locator('[data-icon="cpu"]').click();
  await openSection(page, 'data');
  const card = page.locator('#inspector .asset-field-card');
  await expect(card.locator('img')).toBeVisible();
  const size = await card.locator('img').evaluate(img => `${img.naturalWidth}×${img.naturalHeight}`);
  await expect(card.locator('.asset-field-meta')).toContainText(size);
  await expect(card.getByRole('combobox', { name: t('common.image'), exact: true })).toBeVisible();
  await card.getByRole('button', { name: t('insp.image.media'), exact: true }).click();
  await expect(page.locator('#tab-media')).toHaveAttribute('aria-selected', 'true');
  await expectAccessible(page); expect(errors).toEqual([]);
});
