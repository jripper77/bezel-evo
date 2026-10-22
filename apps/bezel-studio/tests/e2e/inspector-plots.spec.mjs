import { test, expect, clickAddWidget, openSection, openGraphPrecision, setPanelWidths, expectAccessible } from './helpers.mjs';

test('graph precision is secondary, stays open while edited, and timing changes undo once', async ({ page, t }) => {
  await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'graph');
  const window = page.locator('#inspector [data-section="window"]');
  const details = window.locator('details');
  await expect(details).not.toHaveAttribute('open');
  const duration = window.getByRole('spinbutton', { name: t('insp.durationLabel'), exact: true });
  await expect(duration).toBeHidden();
  await expect(window.getByRole('group', { name: t('insp.quickDurations'), exact: true })).toBeVisible();
  await openGraphPrecision(page);
  await duration.fill('5'); await duration.press('Tab');
  await expect(details).toHaveAttribute('open', '');
  await expect(duration).toHaveValue('5');
  await page.locator('#undo').click(); await expect(duration).toHaveValue('1');
  await details.locator('summary').click(); await expect(duration).toBeHidden();
  await openSection(page, 'shape');
  await page.locator('#inspector').getByRole('button', { name: t('graph.area'), exact: true }).click();
  await openSection(page, 'color');
  const fill = page.locator('#inspector').getByRole('spinbutton', { name: t('insp.areaFill'), exact: true });
  await expect(fill).toHaveValue('25');
  await fill.fill('60'); await fill.press('Tab');
  await expect(fill).toHaveValue('60');
  await page.locator('#undo').click(); await expect(fill).toHaveValue('25');
  await expectAccessible(page);
});

test('spectrum frequencies fit together in the narrow panel, with movement icons and Undo', async ({ page, t }) => {
  await page.goto('/index.html?demo=turing88');
  await setPanelWidths(page, { inspector: 320 }); await page.reload();
  await clickAddWidget(page, t, 'spectrum');
  await openSection(page, 'data');
  const panel = page.locator('#inspector');
  const bands = panel.locator('[data-section="spectrum-bands"] [data-columns="3"]');
  await expect(bands.getByRole('spinbutton')).toHaveCount(3);
  const boxes = await bands.getByRole('spinbutton').evaluateAll(inputs => inputs.map(input => ({ top: input.getBoundingClientRect().top, width: input.clientWidth })));
  expect(boxes.every(box => Math.abs(box.top - boxes[0].top) < 2)).toBe(true);
  const high = bands.getByRole('spinbutton', { name: t('spectrum.high'), exact: true });
  await high.fill('24000'); await high.press('Tab');
  await expect(high).toHaveValue('24000');
  expect(await high.evaluate(input => {
    const ctx = document.createElement('canvas').getContext('2d'); ctx.font = getComputedStyle(input).font;
    return ctx.measureText(input.value).width <= input.clientWidth;
  })).toBe(true);
  await openSection(page, 'behavior');
  const motion = panel.getByRole('group', { name: t('insp.spectrum.motion'), exact: true });
  await expect(motion.locator('button svg')).toHaveCount(3);
  const before = await motion.locator('[aria-pressed="true"]').getAttribute('data-value');
  await motion.getByRole('button', { name: t('spectrum.snappy'), exact: true }).click();
  await expect(motion.getByRole('button', { name: t('spectrum.snappy'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#undo').click();
  await expect(motion.locator(`[data-value="${before}"]`)).toHaveAttribute('aria-pressed', 'true');
  await expectAccessible(page);
});
