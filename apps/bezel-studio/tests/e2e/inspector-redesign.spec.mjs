import { openSection, openInspectorTab, pickSensor, test, expect, clickAddWidget, expectAccessible, watchErrors } from './helpers.mjs';

test('all widget inspectors separate navigation from geometry and keep integrated labels readable', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  const inspector = page.locator('#inspector');
  for (const kind of ['value', 'text', 'clock', 'weather', 'bar', 'ring', 'needle', 'graph', 'spectrum', 'player', 'image', 'shape', 'card']) {
    await page.locator('#tab-widgets').click();
    await clickAddWidget(page, t, kind);
    const position = inspector.locator('[data-section="position"]');
    const geometry = position.locator('.geometry-fields, .size-field');
    await expect(geometry.getByRole('spinbutton')).toHaveCount(4);
    await expect(geometry.locator('.fld-prefix')).toHaveText([t('insp.xPrefix'), t('insp.yPrefix'), t('insp.widthLabel'), t('insp.heightLabel')]);
    expect(await geometry.locator('.geometry-fields input, .size-field input').evaluateAll(nodes => nodes.every(node => node.clientWidth >= 38))).toBe(true);
    expect(await inspector.evaluate(node => node.scrollWidth <= node.clientWidth + 1)).toBe(true);
    for (const section of await inspector.locator(':scope > [data-inspector-section]').all()) {
      await openSection(page, await section.getAttribute('data-inspector-section'));
      await expect(position).toBeVisible();
      await expect(inspector.locator('.object-header .inspector-preview')).toBeVisible();
      await expect(inspector.locator('[data-section="membership"]')).toBeVisible();
      expect(await inspector.evaluate(node => node.scrollWidth <= node.clientWidth + 1)).toBe(true);
    }
    for (const section of await inspector.locator(':scope > [data-inspector-section]').all()) {
      await section.locator(':scope > .accordion-heading > button').click();
      await expect(section.locator('.accordion-summary')).toBeVisible();
      await expect(section.locator('.accordion-summary')).not.toHaveText('');
    }
  }
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('sensor formatting offers temperature controls only for temperatures and transition readouts stay compact', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await clickAddWidget(page, t, 'value');
  const inspector = page.locator('#inspector');
  const source = inspector.getByRole('combobox', { name: t('inspector.sensor'), exact: true });
  await pickSensor(page, t, inspector, 'gpu.usage');
  await expect(inspector.getByRole('button', { name: t('inspector.fahrenheit'), exact: true })).toHaveCount(0);
  await pickSensor(page, t, inspector, 'cpu.temperature');
  await expect(inspector.getByRole('button', { name: t('inspector.fahrenheit'), exact: true })).toBeVisible();
  const toggle = inspector.getByRole('button', { name: t('inspector.showUnit'), exact: true });
  expect(await toggle.evaluate(input => ['none', 'normal'].includes(getComputedStyle(input.parentElement.querySelector('span'), '::before').content))).toBe(true);
  await openSection(page, 'behavior');
  const transition = inspector.locator('[data-section="transition"]');
  await transition.getByRole('button', { name: t('insp.transitionSmooth'), exact: true }).click();
  await expect(transition.locator('.slider-field-inline output')).toHaveText('400 ms');
  await transition.getByRole('button', { name: t('fade.none'), exact: true }).click();
  await expect(transition.getByRole('slider')).toHaveCount(0);
  await transition.getByRole('button', { name: t('insp.transitionSmooth'), exact: true }).click();
  await pickSensor(page, t, inspector, 'gpu.usage');
  for (const tab of ['data', 'look']) {
    await openInspectorTab(page, t, `inspector.tab.${tab}`);
    await expectAccessible(page);
    await inspector.evaluate(node => { node.style.height = `${node.scrollHeight}px`; node.style.maxHeight = 'none'; node.style.flex = 'none'; node.scrollTop = 0; });
    await page.locator('.statusbar').evaluate(node => { node.style.visibility = 'hidden'; });
    await inspector.screenshot({ path: test.info().outputPath(`inspector-${tab}.png`) });
  }
  expect(errors).toEqual([]);
});
