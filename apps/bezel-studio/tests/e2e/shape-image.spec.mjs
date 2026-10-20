import { test, expect, watchErrors, expectAccessible } from './helpers.mjs';

// A shape's picture: chosen from the theme's images (an SVG icon here), how
// it covers the shape, and scale/offset only for the natural-size repeats.
test('a shape takes an SVG picture, tiles it, scales it and drops it again', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');

  // An SVG in the theme: insert an offline icon.
  await page.getByRole('tab', { name: t('library.media'), exact: true }).click();
  await page.getByRole('tab', { name: t('media.icons'), exact: true }).click();
  await page.locator('#icon-search').fill('cpu');
  await page.locator('[data-icon="cpu"]').click();
  const inspector = page.locator('#inspector');
  await expect(inspector.getByRole('combobox', { name: t('common.image'), exact: true })).toHaveValue(/tabler-cpu.*\.svg$/);
  const svg = await inspector.getByRole('combobox', { name: t('common.image'), exact: true }).inputValue();

  // A shape: its inspector offers the picture after the fill.
  await page.locator('#tab-widgets').click();
  await page.getByRole('button', { name: t('library.addWidget', { name: t('widget.shape') }), exact: true }).click();
  const picture = inspector.getByRole('combobox', { name: t('common.image'), exact: true });
  await expect(picture).toHaveValue('');
  const repeats = inspector.getByRole('group', { name: t('shape.repeat'), exact: true });
  await expect(repeats).toHaveCount(0);
  await picture.selectOption(svg);
  await expect(picture).toHaveValue(svg);

  // Fill (crop) by default; size and offsets only for the natural-size modes.
  await expect(repeats.getByRole('button', { name: t('shape.repeat.cover'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(repeats.getByRole('button')).toHaveCount(7);
  const size = inspector.getByRole('spinbutton', { name: t('shape.imageScale'), exact: true });
  await expect(size).toHaveCount(0);
  await repeats.getByRole('button', { name: t('shape.repeat.tile'), exact: true }).click();
  await expect(repeats.getByRole('button', { name: t('shape.repeat.tile'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(size).toHaveValue('100');
  await size.fill('50'); await size.press('Tab');
  await expect(size).toHaveValue('50');
  const right = inspector.getByRole('spinbutton', { name: t('shape.imageOffsetX'), exact: true });
  await right.fill('4'); await right.press('Tab');
  await expect(right).toHaveValue('4');
  await expect(inspector.getByRole('slider', { name: t('shape.imageOpacity'), exact: true })).toHaveValue('100');
  await expectAccessible(page);

  // Undo steps back one change at a time; back to Stretch hides the size.
  await page.locator('#undo').click();
  await expect(inspector.getByRole('spinbutton', { name: t('shape.imageOffsetX'), exact: true })).toHaveValue('0');
  await expect(inspector.getByRole('spinbutton', { name: t('shape.imageScale'), exact: true })).toHaveValue('50');
  await repeats.getByRole('button', { name: t('fit.fill'), exact: true }).click();
  await expect(inspector.getByRole('spinbutton', { name: t('shape.imageScale'), exact: true })).toHaveCount(0);

  // No picture: the repeat choice goes away with it.
  await picture.selectOption('');
  await expect(picture).toHaveValue('');
  await expect(repeats).toHaveCount(0);
  expect(errors).toEqual([]);
});
