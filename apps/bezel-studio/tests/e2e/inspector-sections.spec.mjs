import { openSection, test, expect, watchErrors, expectAccessible } from './helpers.mjs';

const addWidget = async (page, t, widget) => {
  await page.locator('#tab-widgets').click();
  await page.getByRole('button', { name: t('library.addWidget', { name: t(`widget.${widget}`) }), exact: true }).click();
};

test('canonical independent sections remember each kind across edits, Undo and reload', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await addWidget(page, t, 'ring');
  const inspector = page.locator('#inspector');
  const toggle = id => inspector.locator(`[data-inspector-section="${id}"] > .accordion-heading > button`);
  expect(await inspector.locator(':scope > [data-inspector-section]').evaluateAll(nodes => nodes.map(n => n.dataset.inspectorSection)))
    .toEqual(['layout', 'data', 'shape', 'color', 'behavior']);
  await expect(toggle('data')).toHaveAttribute('aria-expanded', 'true');
  await expect(toggle('shape')).toHaveAttribute('aria-expanded', 'false');
  await expect(inspector.locator('[data-inspector-section="shape"] .accordion-summary')).toContainText('24');
  await page.locator('#save').click();
  await toggle('shape').click();
  await expect(toggle('data')).toHaveAttribute('aria-expanded', 'true');
  await expect(toggle('shape')).toHaveAttribute('aria-expanded', 'true');
  await toggle('color').click({ modifiers: ['Alt'] });
  for (const id of ['layout', 'data', 'shape', 'behavior']) await expect(toggle(id)).toHaveAttribute('aria-expanded', 'false');
  await expect(toggle('color')).toHaveAttribute('aria-expanded', 'true');
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));
  await openSection(page, 'layout');
  await openSection(page, 'layout');
  const x = inspector.getByRole('spinbutton', { name: t('inspector.x'), exact: true });
  const before = await x.inputValue();
  await x.fill('60'); await x.press('Tab');
  await expect(x).toHaveValue('60');
  await page.locator('#undo').click();
  await expect(x).toHaveValue(before);
  await expect(toggle('color')).toHaveAttribute('aria-expanded', 'true');
  const ring = await inspector.getByLabel(t('common.name'), { exact: true }).inputValue();
  const types = inspector.getByRole('group', { name: t('insp.type'), exact: true });
  await types.getByRole('button', { name: t('widget.bar'), exact: true }).click();
  await expect(types.getByRole('button', { name: t('widget.bar'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#undo').click();
  await expect(types.getByRole('button', { name: t('widget.ring'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await addWidget(page, t, 'bar');
  await expect(toggle('data')).toHaveAttribute('aria-expanded', 'true');
  await expect(toggle('color')).toHaveAttribute('aria-expanded', 'false');
  await page.locator('#tab-layers').click();
  await page.locator('.layer-name').filter({ hasText: ring }).first().click();
  await expect(toggle('color')).toHaveAttribute('aria-expanded', 'true');
  await toggle('color').focus(); await page.keyboard.press('Home');
  await expect(toggle('layout')).toBeFocused();
  await page.keyboard.press('End'); await expect(toggle('behavior')).toBeFocused();
  await page.reload(); await addWidget(page, t, 'ring');
  await expect(toggle('color')).toHaveAttribute('aria-expanded', 'true');
  await expect(toggle('data')).toHaveAttribute('aria-expanded', 'false');
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('the preview shows the selected object rendered alone, on or off the canvas', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await addWidget(page, t, 'ring');
  const inspector = page.locator('#inspector');
  const name = await inspector.getByLabel(t('common.name'), { exact: true }).inputValue();
  const preview = inspector.getByRole('img', { name: t('inspector.previewOf', { name }), exact: true });
  await expect(preview).toBeVisible();
  await expect(inspector.locator('.object-header')).toContainText(t('insp.live'));
  await expect.poll(() => preview.evaluate(canvas => {
    if (canvas.width < 8 || canvas.height < 8) return false;
    const pixels = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data;
    return pixels.some((v, i) => i % 4 === 3 && v > 0);
  })).toBe(true);
  // Rendered alone, it shows even moved off the canvas.
  await openSection(page, 'layout');
  const x = inspector.getByRole('spinbutton', { name: t('inspector.x'), exact: true });
  await x.fill('5000'); await x.press('Tab');
  await expect(x).toHaveValue('5000');
  await expect(preview).toBeVisible();
  await expect(inspector.locator('.inspector-preview-off')).toBeHidden();
  await expect.poll(() => preview.evaluate(canvas => canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data.some((v, i) => i % 4 === 3 && v > 0))).toBe(true);
  await page.locator('#undo').click();
  await expect(preview).toBeVisible();
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('a card has Layout, Faces, Shape, Color and Behavior sections; its faces are a list, the one in view marked', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await addWidget(page, t, 'card');
  const inspector = page.locator('#inspector');
  expect(await inspector.locator(':scope > [data-inspector-section]').evaluateAll(nodes => nodes.map(n => n.dataset.inspectorSection)))
    .toEqual(['layout', 'faces', 'shape', 'color', 'behavior']);
  const faces = inspector.getByRole('list', { name: t('card.tab.faces'), exact: true });
  await inspector.getByRole('button', { name: t('card.addFace'), exact: true }).click();
  const current = faces.locator('[aria-current="true"]');
  await expect(current).toHaveAttribute('data-face', '1');
  await expect(current).toContainText(t('layers.inView'));
  await faces.getByRole('button', { name: t('cardInsp.editFace', { name: t('card.faceName', { number: 1 }) }), exact: true }).click();
  await expect(current).toHaveAttribute('data-face', '0');
  await expect(faces.getByText(t('layers.inView'), { exact: true })).toHaveCount(1);
  // ▶ in the header tries the next face; triggers are in Behavior, with the transition.
  await expect(inspector.getByRole('button', { name: t('insp.tryObject'), exact: true })).toBeEnabled();
  await openSection(page, 'behavior');
  const behavior = inspector.locator('[data-inspector-section="behavior"]');
  await expect(behavior.getByRole('group', { name: t('card.effect'), exact: true })).toBeVisible();
  await expect(behavior.getByRole('button', { name: t('trigger.add'), exact: true })).toBeEnabled();
  await expectAccessible(page); expect(errors).toEqual([]);
});
