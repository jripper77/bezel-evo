import { test, expect, watchErrors, expectAccessible, openPreferences } from './helpers.mjs';

test('Debug owns logging and its screen-readings suboption independently', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await openPreferences(page, t);
  const logs = page.getByRole('region', { name: t('prefs.logsTitle'), exact: true });
  const openLogs = logs.getByRole('button', { name: t('prefs.openLogs'), exact: true });
  await expect(openLogs).toBeVisible();
  await expect(page.getByRole('button', { name: t('prefs.openLogs'), exact: true })).toHaveCount(1);
  await openLogs.click();
  await expect(page.getByRole('dialog')).toBeVisible();
  const debug = page.locator('#prefs-debug');
  const overlay = page.locator('#prefs-debug-overlay');
  const corner = page.locator('#prefs-debug-corner');
  await expect(debug).not.toBeChecked();
  await expect(overlay).toBeDisabled();
  await expect(corner).toBeDisabled();
  await debug.check();
  await expect(overlay).toBeEnabled();
  await expect(overlay).not.toBeChecked();
  await overlay.check();
  for (const value of ['top-left', 'top-right', 'bottom-left', 'bottom-right']) {
    await corner.selectOption(value);
    await expect(corner).toHaveValue(value);
  }
  await page.getByRole('button', { name: t('common.done') }).click();
  await openPreferences(page, t);
  await expect(debug).toBeChecked();
  await expect(overlay).toBeChecked();
  await expect(corner).toHaveValue('bottom-right');
  await debug.uncheck();
  await expect(overlay).toBeDisabled();
  await expect(corner).toBeDisabled();
  await expect(openLogs).toBeVisible();
  await expect(openLogs).toBeEnabled();
  await debug.check();
  await overlay.uncheck();
  await expect(corner).toBeDisabled();
  await expect(debug).toBeChecked();
  await expectAccessible(page);
  expect(errors).toEqual([]);
  await page.screenshot({ path: test.info().outputPath('debug-preferences.png') });
});
