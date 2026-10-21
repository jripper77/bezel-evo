import { pickSensor, test, expect, watchErrors, expectAccessible, clickAddWidget, colorButton, setColor, openInspectorTab, toggleButton } from './helpers.mjs';

test('colour rules edit values and colours, reorder, remove and Undo', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await clickAddWidget(page, t, 'bar');
  await openInspectorTab(page, t, 'inspector.tab.look');
  const section = page.locator('[data-section="value-color"]');
  const toggle = toggleButton(section, t('insp.colorByValue'));
  await expect(toggle).toHaveAttribute('aria-pressed', 'false');
  await toggle.click();
  const add = section.getByRole('button', { name: t('insp.colorAdd'), exact: true });
  await add.click();
  const rows = section.locator('[data-color-row]');
  const firstValue = rows.first().getByRole('spinbutton', { name: t('insp.colorThresholdValue'), exact: true });
  await firstValue.fill('60'); await firstValue.press('Tab');
  await setColor(page, t, rows.first(), t('inspector.color'), '#ff8800');
  await add.click();
  await expect(rows).toHaveCount(2);
  const secondValue = rows.last().getByRole('spinbutton', { name: t('insp.colorThresholdValue'), exact: true });
  await secondValue.fill('80'); await secondValue.press('Tab');
  await setColor(page, t, rows.last(), t('inspector.color'), '#ff0000');
  await rows.last().getByRole('button', { name: t('insp.colorMoveUp') }).focus();
  await page.keyboard.press('Enter');
  await expect(firstValue).toHaveValue('80');
  await expect(rows.first().getByRole('button', { name: t('insp.colorMoveDown') })).toBeFocused();
  await expect(colorButton(rows.first(), t('inspector.color'))).toHaveAttribute('data-color', /^#ff0000/);
  await page.locator('#undo').click(); await expect(firstValue).toHaveValue('60');
  await rows.last().getByRole('button', { name: t('insp.colorRemove') }).click();
  await expect(rows).toHaveCount(1);
  await page.locator('#undo').click(); await expect(rows).toHaveCount(2);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('demo pixels cross the threshold with the eased drawn sensor value', async ({ page }) => {
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  const pixels = await page.evaluate(async () => {
    const { renderApprox } = await import('/demo-render.js');
    const { demoSensorValue } = await import('/demo-sensors.js');
    const { createWidget } = await import('/editor/widgets.js');
    const made = createWidget('bar', { width: 100, height: 20 });
    const element = { ...made, id: 1, visible: true, opacity: 1, frame: { x: 0, y: 0, width: 100, height: 20 },
      kind: { ...made.kind, radius: 0, binding: { key: 'cpu.usage', min: 0, max: 100 } },
      colorByValue: { base: '#00c800ff', thresholds: [{ value: demoSensorValue('cpu.usage', 0), color: '#ff0000ff' }] } };
    const theme = { canvas: { width: 100, height: 20 }, background: { type: 'color', color: '#000000ff' }, refreshSeconds: 1, elements: [element] };
    const at = (time, motion = true) => {
      const frame = renderApprox(theme, time, null, new Map(), 1, motion);
      return Array.from(frame.rgba.slice((10 * 100 + 1) * 4, (10 * 100 + 1) * 4 + 4));
    };
    return { easing: at(0), settled: at(.4), reduced: at(0, false) };
  });
  expect(pixels.easing).toEqual([0, 200, 0, 255]);
  expect(pixels.settled).toEqual([255, 0, 0, 255]);
  expect(pixels.reduced).toEqual([255, 0, 0, 255]);
  await expectAccessible(page);
});

test('shapes choose a rule sensor; turning the rule off preserves fixed paint', async ({ page, t }) => {
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await clickAddWidget(page, t, 'shape');
  const section = page.locator('[data-section="value-color"]');
  const toggle = toggleButton(section, t('insp.colorByValue'));
  await toggle.click();
  await pickSensor(page, t, section, 'cpu.temperature');
  await expect(section.getByRole('combobox', { name: t('inspector.sensor'), exact: true })).toHaveAttribute('data-sensor-key', 'cpu.temperature');
  await toggle.click();
  await expect(section.locator('select')).toHaveCount(0);
  await page.locator('#undo').click();
  await expect(section.getByRole('combobox', { name: t('inspector.sensor'), exact: true })).toHaveAttribute('data-sensor-key', 'cpu.temperature');
  await expectAccessible(page);
});
