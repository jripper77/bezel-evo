import { test, expect, watchErrors, expectAccessible, openPreferences } from './helpers.mjs';

// D-2026-11-09-update-check-1: Bezel Light's daily look at the latest
// release is a local preference, on by default, with a hint saying what it sends.
test('the update check is a switch in the Preferences, on by default and remembered', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await openPreferences(page, t);
  const section = page.getByRole('region', { name: t('prefs.updateTitle'), exact: true });
  const check = section.getByRole('switch', { name: t('prefs.updateCheck'), exact: true });
  await expect(check).toBeChecked();
  await expect(section.getByText(t('prefs.updateHint'), { exact: true })).toBeVisible();
  // The demo's Light has found no newer release.
  await expect(page.locator('#prefs-update-found')).toHaveCount(0);
  await expectAccessible(page);

  await check.uncheck();
  await expect(check).toBeEnabled();
  await page.getByRole('button', { name: t('common.done') }).click();
  await openPreferences(page, t);
  await expect(page.locator('#prefs-update-check')).not.toBeChecked();
  await page.locator('#prefs-update-check').check();
  await page.getByRole('button', { name: t('common.done') }).click();
  await openPreferences(page, t);
  await expect(page.locator('#prefs-update-check')).toBeChecked();
  expect(errors).toEqual([]);
});
