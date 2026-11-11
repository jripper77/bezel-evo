import { test, expect, watchErrors, expectAccessible, openPreferences } from './helpers.mjs';

// D-2026-11-09-update-check-1: Bezel Light's daily look at the latest
// release is a local preference, on by default, with a hint saying what it sends.
test('the update check is a switch in the Preferences, on by default and remembered', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await openPreferences(page, t, 'prefs.updateTitle');
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

// D-2026-11-11-preferences-layout-2: Check now asks at once and says what it found.
test('Check now says the installed version is the latest, or names the newer one', async ({ page, t }, info) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await openPreferences(page, t, 'prefs.updateTitle');
  const dialog = page.getByRole('dialog', { name: t('prefs.title') });
  const status = page.locator('#prefs-update-status');
  await expect(dialog.getByText(t('app.title', { version: '0.2.28' }), { exact: true })).toBeVisible();
  await expect(status).toHaveText(t('prefs.updateNever'));
  const checkNow = dialog.getByRole('button', { name: t('prefs.updateCheckNow'), exact: true });
  await checkNow.click();
  await expect(status).toContainText(t('prefs.updateStatusLatest', { version: '0.2.28', when: '' }).split(':')[0]);
  await expect(page.locator('#prefs-update-found')).toHaveCount(0);
  await expectAccessible(page);

  await page.evaluate(() => { globalThis.demoLatestRelease = '0.3.0'; });
  await dialog.getByRole('button', { name: t('prefs.updateCheckNow'), exact: true }).click();
  await expect(page.locator('#prefs-update-found')).toContainText(t('prefs.updateFound', { version: '0.3.0' }));
  // The column marks the page with the news, and the dialog reopens there.
  await expect(dialog.locator('#prefs-tab-updates .prefs-nav-dot')).toHaveCount(1);
  await dialog.screenshot({ path: info.outputPath('preferences-updates.png') });
  await page.getByRole('button', { name: t('common.done') }).click();
  await openPreferences(page, t);
  await expect(page.getByRole('tab', { name: t('prefs.updateTitle'), exact: true })).toHaveAttribute('aria-selected', 'true');
  expect(errors).toEqual([]);
});

test('the Preferences pages are a column of tabs the arrow keys move through', async ({ page, t }, info) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await openPreferences(page, t);
  const dialog = page.getByRole('dialog', { name: t('prefs.title') });
  const tab = (key) => dialog.getByRole('tab', { name: t(key), exact: true });
  await expect(tab('prefs.page.general')).toBeFocused();
  await expect(dialog.getByRole('combobox', { name: t('prefs.language') })).toBeVisible();
  await page.keyboard.press('ArrowDown');
  await expect(tab('prefs.page.screens')).toBeFocused();
  await expect(dialog.getByRole('switch', { name: t('prefs.idleSaver'), exact: true })).toBeVisible();
  await expect(dialog.getByRole('combobox', { name: t('prefs.language') })).toHaveCount(0);
  await page.keyboard.press('End');
  await expect(tab('prefs.page.diagnostics')).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('ArrowDown');
  await expect(tab('prefs.page.general')).toHaveAttribute('aria-selected', 'true');
  await dialog.screenshot({ path: info.outputPath('preferences.png') });
  await expectAccessible(page);
  expect(errors).toEqual([]);
});
