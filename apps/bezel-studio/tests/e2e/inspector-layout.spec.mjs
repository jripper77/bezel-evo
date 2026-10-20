import { test, expect, clickAddWidget, expectAccessible, watchErrors } from './helpers.mjs';

test('short numbers stay compact and conditional colour stays with the paint', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  const inspector = page.locator('#inspector');
  for (const kind of ['bar', 'ring', 'graph', 'value', 'needle', 'spectrum']) {
    await page.locator('#tab-widgets').click();
    await clickAddWidget(page, t, kind);
    const rotation = inspector.getByRole('spinbutton', { name: t('insp.rotation'), exact: true });
    const box = await rotation.boundingBox();
    expect(box.width).toBeLessThan(115);
    expect(box.width).toBeGreaterThan(60);
    await expect(rotation.locator('..').locator('.fld-prefix')).toHaveText(t('insp.rotation'));
    expect(await rotation.evaluate(input => {
      const row = input.closest('.fld');
      return input.getBoundingClientRect().left - row.querySelector('.fld-prefix').getBoundingClientRect().right;
    })).toBeLessThanOrEqual(12);
    await inspector.getByRole('tab', { name: t('inspector.tab.look'), exact: true }).click();
    const rules = inspector.locator('[data-section="value-color"]');
    await expect(rules).toBeVisible();
    expect(await rules.evaluate(node => {
      const paint = node.previousElementSibling;
      return paint?.matches('.color-field') || node.parentElement.querySelector('.color-field') !== null;
    })).toBe(true);
    await expect(rules.getByRole('slider')).toHaveCount(0);
    await expect(inspector.locator('[data-section="corners"] [type="range"]')).toHaveCount(0);
  }
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('shape groups separate fill, transparency, outline and corners in both themes', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await clickAddWidget(page, t, 'shape');
  const inspector = page.locator('#inspector');
  // Shapes have only Look properties, so the inspector omits the tab strip.
  const panel = inspector;
  expect(await panel.locator(':scope > [data-section]').evaluateAll(nodes => nodes.map(node => node.dataset.section).filter(key => !['position', 'actions', 'membership'].includes(key))))
    .toEqual(['shape', 'color', 'picture', 'outline', 'corners', 'transparency', 'sensor-motion']);
  await expect(panel.locator('[data-section="transparency"]').getByRole('slider', { name: t('inspector.opacity'), exact: true })).toBeVisible();
  await inspector.getByRole('button', { name: t('insp.cornersEach'), exact: true }).click();
  const corners = inspector.locator('.corner-radii');
  await expect(corners.getByRole('spinbutton')).toHaveCount(4);
  // Each control and its drawing must fit the inspector without horizontal clipping.
  const fits = await panel.locator('input, .corner-shape, .number-control').evaluateAll(nodes => nodes.every(node => {
    const box = node.getBoundingClientRect(), panel = node.closest('#inspector').getBoundingClientRect();
    return box.left >= panel.left && box.right <= panel.right + 1;
  }));
  expect(fits).toBe(true);
  await inspector.evaluate(node => { node.style.height = `${node.scrollHeight}px`; node.style.maxHeight = 'none'; node.style.flex = 'none'; node.scrollTop = 0; });
  // Keep the fixed status footer outside the full-height inspector capture.
  await page.locator('.statusbar').evaluate(node => { node.style.visibility = 'hidden'; });
  await inspector.screenshot({ path: test.info().outputPath('inspector-shape.png') });
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('membership stays visible across tabs and changing a card keeps Undo', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await clickAddWidget(page, t, 'card');
  await clickAddWidget(page, t, 'bar');
  const inspector = page.locator('#inspector');
  const membership = inspector.locator(':scope > [data-section="membership"]');
  const card = membership.getByRole('combobox', { name: t('widget.card'), exact: true });
  const parent = await card.inputValue();
  expect(parent).not.toBe('');
  for (const tab of ['look', 'data']) {
    await inspector.getByRole('tab', { name: t(`inspector.tab.${tab}`), exact: true }).click();
    await expect(card).toBeVisible();
    await expect(inspector.locator('.inspector-type select')).toBeVisible();
  }
  await card.selectOption('');
  await expect(membership.getByRole('combobox', { name: t('card.face'), exact: true })).toHaveCount(0);
  await page.locator('#undo').click();
  await expect(card).toHaveValue(parent);
  await expect(membership.getByRole('combobox', { name: t('card.face'), exact: true })).toBeVisible();
  await inspector.evaluate(node => { node.style.height = `${node.scrollHeight}px`; node.style.maxHeight = 'none'; node.style.flex = 'none'; node.scrollTop = 0; });
  await page.locator('.statusbar').evaluate(node => { node.style.visibility = 'hidden'; });
  await inspector.screenshot({ path: test.info().outputPath('inspector-context.png') });
  await expectAccessible(page);
  expect(errors).toEqual([]);
});
