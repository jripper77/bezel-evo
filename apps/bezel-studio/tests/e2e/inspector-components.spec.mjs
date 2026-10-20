import { test, expect, watchErrors, expectAccessible, clickAddWidget, openSection } from './helpers.mjs';

test.describe('Italian numeric fields', () => {
  test.use({ locale: 'it-IT' });
  test('localized size prefixes leave four digits readable in the narrow inspector', async ({ page, t }) => {
    const errors = watchErrors(page);
    await page.goto('/index.html?demo=turing88');
    await clickAddWidget(page, t, 'ring');
    const inspector = page.locator('#inspector');
    const width = inspector.getByRole('spinbutton', { name: t('inspector.width'), exact: true });
    await width.fill('4321'); await width.press('Tab');
    for (const label of ['inspector.width', 'inspector.height']) {
      const field = inspector.getByRole('spinbutton', { name: t(label), exact: true });
      await expect(field).toHaveValue('4321');
      expect(await field.evaluate(input => {
        const context = document.createElement('canvas').getContext('2d');
        context.font = getComputedStyle(input).font;
        return context.measureText(input.value).width <= input.clientWidth;
      })).toBe(true);
    }
    await expectAccessible(page); expect(errors).toEqual([]);
  });
});

test('linked size keeps proportions with one Undo and linking never changes the document', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'ring');
  const inspector = page.locator('#inspector');
  const width = inspector.getByRole('spinbutton', { name: t('inspector.width'), exact: true });
  const height = inspector.getByRole('spinbutton', { name: t('inspector.height'), exact: true });
  const link = inspector.getByRole('button', { name: t('insp.linkSize'), exact: true });
  const before = [await width.inputValue(), await height.inputValue()];
  await expect(link).toHaveAttribute('aria-pressed', 'true');
  await width.fill('400'); await width.press('Tab');
  await expect(width).toHaveValue('400'); await expect(height).toHaveValue('400');
  await page.locator('#undo').click();
  await expect(width).toHaveValue(before[0]); await expect(height).toHaveValue(before[1]);
  await page.locator('#save').click(); await link.click();
  await expect(link).toHaveAttribute('aria-pressed', 'false');
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));
  await width.fill('500'); await width.press('Tab');
  await expect(height).toHaveValue(before[1]);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('theme and multi selection keep their complete controls in canonical sections', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  const inspector = page.locator('#inspector');
  await openSection(page, 'color'); await openSection(page, 'behavior');
  await expect(inspector.getByRole('button', { name: t('insp.bgChange'), exact: true })).toBeVisible();
  await expect(inspector.getByRole('group', { name: t('inspector.refresh'), exact: true })).toBeVisible();
  await inspector.getByRole('button', { name: t('insp.bgChange'), exact: true }).click();
  await expect(inspector.getByRole('region', { name: t('insp.bgChooserTitle'), exact: true })).toBeVisible();
  await clickAddWidget(page, t, 'ring'); await clickAddWidget(page, t, 'bar');
  await page.locator('#tab-layers').click();
  const rows = page.locator('#layer-list .layer-row');
  await rows.nth(0).click(); await rows.nth(1).click({ modifiers: ['Control'] });
  await openSection(page, 'layout');
  await expect(inspector.getByRole('heading', { name: t('inspector.multi', { count: 2 }), exact: true })).toBeVisible();
  await expect(inspector.getByRole('button', { name: t('group.create'), exact: true })).toBeVisible();
  await expect(inspector.getByRole('button', { name: t('common.duplicate'), exact: true })).toBeVisible();
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('optional field bindings and parts support keyboard actions without nested controls', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await page.evaluate(async labels => {
    const { numberField, partsRow, iconChoice, fadeField } = await import('/ui/fields.js');
    const output = document.createElement('output');
    const commit = value => { output.textContent = JSON.stringify(value); };
    document.querySelector('#inspector').replaceChildren(
      numberField(labels.size, 20, commit, { prefix: 'R', unit: 'px', binding: { label: labels.sensor, connected: false, onClick: () => commit('binding') } }),
      partsRow(labels.text, [['number', labels.text]], 'number', commit, () => numberField(labels.width, 100, commit)),
      iconChoice(labels.shape, 'ring', [['ring', labels.ring, 'type-anello'], ['bar', labels.bar, 'type-barra']], commit),
      fadeField({ mode: labels.fade, none: labels.none, linear: labels.linear, radial: labels.radial }, null, commit), output,
    );
  }, { size: t('inspector.size'), sensor: t('inspector.sensor'), text: t('common.text'), width: t('inspector.width'), shape: t('common.shape'), ring: t('widget.ring'), bar: t('widget.bar'), fade: t('fade.mode'), none: t('fade.none'), linear: t('fade.linear'), radial: t('fade.radial') });
  const inspector = page.locator('#inspector');
  await inspector.getByRole('button', { name: t('inspector.sensor'), exact: true }).focus();
  await page.keyboard.press('Space');
  await expect(inspector.locator('output')).toHaveText('"binding"');
  await inspector.getByRole('button', { name: t('widget.bar'), exact: true }).click();
  await expect(inspector.locator('output')).toHaveText('"bar"');
  await inspector.getByRole('button', { name: t('fade.radial'), exact: true }).click();
  await expect(inspector.locator('output')).toContainText('"radial":[0.5,0.5,0.5]');
  await expect(inspector.locator('label button, button button')).toHaveCount(0);
  await expectAccessible(page); expect(errors).toEqual([]);
});
