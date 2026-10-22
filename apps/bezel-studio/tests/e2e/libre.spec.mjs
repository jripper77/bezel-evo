import { test, expect, expectAccessible, watchErrors } from './helpers.mjs';

const openStatus = async (page, t) => {
  await page.locator('#status-libre-toggle').click();
  // Titled with how many sensors read (T-9b, board StatusBarNew): found by its id.
  const popup = page.locator('#libre-popover');
  await expect(popup).toBeVisible();
  return popup;
};

test('compact status shows probes and counts without hardware details', async ({ page, t }, info) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88&libre=partial');
  await expect(page.locator('#status-libre-label')).toHaveText(t('sensorChip.active', { available: 28, total: 29 }));
  await expect(page.locator('#status-libre-toggle')).toHaveAttribute('data-tone', 'warn');
  const popup = await openStatus(page, t);
  await expect(popup.locator('#libre-popover-title')).toHaveText(t('sensorChip.activeTitle', { available: 28, total: 29 }));
  await expect(popup.getByRole('listitem')).toHaveCount(2);
  const libre = popup.locator('.probe-row[data-probe="LibreHardwareMonitor"]');
  await expect(libre).toHaveAttribute('data-state', 'partial');
  await expect(libre).toContainText(t('sensorChip.sensorsMany', { count: 3 }));
  await expect(libre.locator('button[data-status="available"]')).toHaveText(t('sensorChip.count.available', { count: 2 }));
  await expect(libre.locator('button[data-status="unavailable"]')).toHaveText(t('sensorChip.count.unavailable', { count: 1 }));
  await expect(libre.getByRole('button', { name: t('sensorChip.restartProbe', { name: 'LibreHardwareMonitor' }), exact: true })).toBeVisible();
  await expect(popup.locator('.probe-restart')).toHaveCount(1);
  await expect(popup.locator('.libre-popover-actions .probe-restart')).toHaveCount(0);
  await expect(popup).not.toContainText('Corsair');
  await expect(popup).not.toContainText('lhm.');
  await expectAccessible(page);
  await popup.screenshot({ path: info.outputPath('sensor-probes.png') });
  await page.keyboard.press('Escape');
  await expect(popup).toBeHidden();
  await expect(page.locator('#status-libre-toggle')).toBeFocused();
  expect(errors).toEqual([]);
});

test('failed count opens the filtered library and still detects recovery', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88&libre=partial');
  const popup = await openStatus(page, t);
  await popup.locator('.probe-row[data-probe="LibreHardwareMonitor"] button[data-status="unavailable"]').click();
  await expect(popup).toBeHidden();
  const panel = page.locator('#panel-sensors');
  await expect(panel).toBeVisible();
  // A filtered opening shows its filters beside the search.
  await expect(panel.locator('.filters-toggle')).toHaveAttribute('aria-expanded', 'true');
  await expect(panel.getByRole('combobox', { name: t('sensorsPanel.probe'), exact: true })).toHaveValue('LibreHardwareMonitor');
  await expect(panel.getByRole('combobox', { name: t('sensorsPanel.status'), exact: true })).toHaveValue('unavailable');
  await expect(panel.locator('.sensor-row')).toHaveCount(1);
  await expect(panel.locator('.sensor-row')).toHaveAttribute('data-key', 'lhm.psu.corsair.0.voltage.3');
  await panel.getByRole('button', { name: t('sensorsPanel.startLibre'), exact: true }).click();
  await expect(panel.locator('.sensor-row')).toHaveCount(0);
  await expect(panel.locator('#sensor-result-count')).toHaveText(t('sensorsPanel.results', { shown: 0, total: 29 }));
  await panel.getByRole('combobox', { name: t('sensorsPanel.status'), exact: true }).selectOption('available');
  await expect(panel.locator('.sensor-row')).toHaveCount(3);
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('probe, state, category and search combine and generic open resets filters', async ({ page, t }, info) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88&libre=partial');
  let popup = await openStatus(page, t);
  await popup.locator('.probe-row[data-probe="LibreHardwareMonitor"] .probe-main').click();
  const panel = page.locator('#panel-sensors');
  const rows = panel.locator('.sensor-row');
  await expect(rows).toHaveCount(3);
  await panel.getByRole('combobox', { name: t('sensorsPanel.status'), exact: true }).selectOption('available');
  await expect(rows).toHaveCount(2);
  // Categories fold now instead of filtering: a search narrows to the processor.
  await panel.getByRole('searchbox').fill('cpu');
  await expect(rows).toHaveCount(1);
  await panel.getByRole('searchbox').fill('not-a-sensor');
  await expect(rows).toHaveCount(0);
  popup = await openStatus(page, t);
  await popup.getByRole('button', { name: t('sensorStatus.openSensors'), exact: true }).click();
  await expect(panel.getByRole('searchbox')).toHaveValue('');
  // An unfiltered opening folds the filters away, both reset.
  await expect(panel.locator('.filters-toggle')).toHaveAttribute('aria-expanded', 'false');
  await expect(panel.locator('select[data-sensor-filter="probe"]')).toHaveValue('all');
  await expect(panel.locator('select[data-sensor-filter="status"]')).toHaveValue('all');
  await expect(rows).toHaveCount(29);
  await openStatus(page, t);
  await page.screenshot({ path: info.outputPath('sensor-filters-overview.png') });
  expect(errors).toEqual([]);
});

test('restart, dismissal and other probes work without Libre', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88&libre=error');
  const popup = await openStatus(page, t);
  const restart = popup.locator('.probe-row[data-probe="LibreHardwareMonitor"] .probe-restart');
  await restart.click();
  await expect(page.locator('#status-libre-label')).toHaveText(t('sensorChip.restarting'));
  await expect(restart).toBeDisabled();
  await expect(page.locator('#status-libre-label')).toHaveText(/\d+\/\d+/);
  await expect(restart).toBeEnabled();
  await expectAccessible(page);
  await page.locator('#status-main').click();
  await expect(popup).toBeHidden();
  await page.goto('/index.html?demo=turing88');
  const other = await openStatus(page, t);
  await expect(other.getByRole('listitem')).toHaveCount(1);
  await expect(other.locator('.probe-restart')).toHaveCount(0);
  await expectAccessible(page);
  expect(errors).toEqual([]);
});
