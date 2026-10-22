import { test, expect, clickAddWidget, openSection, expectAccessible } from './helpers.mjs';

test('ring geometry follows Style, groups compact controls and keeps each change undoable', async ({ page, t }) => {
  await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'ring');
  await openSection(page, 'shape');
  const shape = page.locator('#inspector [data-inspector-section="shape"]');
  await expect(shape.locator('.accordion-body > .property-section').first()).toHaveAttribute('data-section', 'visualization');
  const clockwise = shape.getByRole('button', { name: t('inspector.clockwise'), exact: true });
  const reverse = shape.getByRole('button', { name: t('insp.counterclockwise'), exact: true });
  await expect(clockwise.locator('svg')).toHaveCount(1);
  expect(await clockwise.textContent()).toBe('');
  const thickness = shape.getByRole('spinbutton', { name: t('insp.thickness'), exact: true });
  const boxes = await Promise.all([thickness.boundingBox(), clockwise.boundingBox()]);
  expect(Math.abs(boxes[0].y - boxes[1].y)).toBeLessThan(8);
  await reverse.click(); await expect(reverse).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#undo').click(); await expect(clockwise).toHaveAttribute('aria-pressed', 'true');
  const count = shape.getByRole('spinbutton', { name: t('inspector.count'), exact: true });
  await expect(count).toBeDisabled();
  await shape.getByRole('button', { name: t('inspector.segments'), exact: true }).click();
  await expect(count).toBeEnabled();
  await count.fill('24'); await count.press('Tab');
  await expect(count).toHaveValue('24');
  await page.locator('#undo').click(); await expect(count).toHaveValue('10');
  expect(await count.evaluate(input => getComputedStyle(input).borderTopWidth)).toBe('0px');
  expect(await shape.evaluate(node => node.scrollWidth <= node.clientWidth)).toBe(true);
  await expectAccessible(page);
});
