import { test, expect, watchErrors, expectAccessible, libraryTabKey } from './helpers.mjs';

async function openProfiles(page, t) {
  await page.getByRole('tab', { name: t(libraryTabKey('themes')), exact: true }).click();
  await page.locator('#theme-profiles').click();
  return page.getByRole('dialog', { name: t('profiles.title'), exact: true });
}
const item = (dialog, name) => dialog.locator('.pf-list .pf-item').filter({ hasText: name });

test('profiles are managed from Themes and saved independently of the editor', async ({ page, t }, info) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  const original = await page.locator('#theme-name').inputValue();
  let dialog = await openProfiles(page, t);
  // Who decides: Automatic with the base themes as its fallback.
  await expect(dialog.getByRole('radio', { name: t('profiles.automatic'), exact: true })).toBeChecked();
  await expect(dialog.getByRole('combobox', { name: t('profiles.fallback'), exact: true }).locator('option').first()).toHaveText(t('profiles.currentThemes'));
  await expect(dialog.getByRole('combobox', { name: t('profiles.manualShows'), exact: true })).toBeDisabled();
  // The base themes come first and are shown by default.
  await expect(item(dialog, t('profiles.currentThemes'))).toHaveAttribute('aria-current', 'true');
  await expect(dialog.locator('.pf-theme-current')).toHaveText(['Demo']);
  await expect(dialog.getByRole('button', { name: t('profiles.remove'), exact: true })).toHaveCount(0);

  // The transition, tried without saving.
  await dialog.getByRole('combobox', { name: t('card.effect'), exact: true }).selectOption('wipe');
  await dialog.getByRole('spinbutton', { name: t('profiles.durationMs'), exact: true }).fill('850');
  const up = dialog.getByRole('button', { name: t('card.direction.up'), exact: true });
  await up.click();
  await expect(up).toHaveAttribute('aria-pressed', 'true');
  await dialog.getByRole('button', { name: t('profiles.preview'), exact: true }).click();
  // With the base themes selected, it tries them: one theme for the one display.
  const tried = () => page.evaluate(() => window.demoTransitionPreviews);
  await expect.poll(tried).toHaveLength(1);
  const [first] = await tried();
  expect(first.transition).toEqual({ effect: 'wipe', direction: 'up', durationMs: 850 });
  expect(Object.values(first.themes)).toHaveLength(1);
  const help = dialog.getByRole('button', { name: t('profiles.transitionHelp'), exact: true });
  await help.click();
  await expect(help).toHaveAttribute('aria-expanded', 'true');
  await expect(dialog.getByText(t('profiles.transitionHint'))).toBeVisible();

  // A new profile, its rule a sentence; the list sums it up.
  await dialog.getByRole('button', { name: t('profiles.new'), exact: true }).click();
  await dialog.getByRole('textbox', { name: t('profiles.nameLabel'), exact: true }).fill('Music');
  await dialog.getByRole('button', { name: t('profiles.addRule'), exact: true }).click();
  await dialog.getByRole('textbox', { name: t('profiles.app'), exact: true }).fill('Spotify');
  await dialog.getByRole('spinbutton', { name: t('trigger.priority'), exact: true }).fill('30');
  await dialog.getByRole('spinbutton', { name: t('profiles.returnDelay'), exact: true }).fill('5');
  await expect(item(dialog, 'Music').locator('.pf-item-summary')).toHaveText(t('profiles.summary.mediaIn', { app: 'Spotify' }));
  await expect(item(dialog, 'Music')).toHaveAttribute('aria-current', 'true');
  // Tried from the profile, unsaved: towards its own themes.
  await dialog.getByRole('button', { name: t('profiles.preview'), exact: true }).click();
  await expect.poll(tried).toHaveLength(2);
  expect((await tried())[1].themes).toEqual(first.themes);

  // From Automatic to Manual with that profile.
  await dialog.getByRole('radio', { name: t('profiles.manual'), exact: true }).check();
  const manual = dialog.getByRole('combobox', { name: t('profiles.manualShows'), exact: true });
  await expect(manual).toBeEnabled();
  await expect(dialog.getByRole('combobox', { name: t('profiles.fallback'), exact: true })).toBeDisabled();
  await manual.selectOption({ label: 'Music' });
  await expectAccessible(page);
  if (info.project.name === 'dark-en') await dialog.screenshot({ path: '../../dist/profiles-editor.png' });
  await dialog.getByRole('button', { name: t('common.save'), exact: true }).click();
  await expect(dialog).not.toBeVisible();
  await expect(page.locator('#theme-name')).toHaveValue(original);

  // Reopened: the choice, the transition and the rule are back.
  dialog = await openProfiles(page, t);
  await expect(dialog.getByRole('radio', { name: t('profiles.manual'), exact: true })).toBeChecked();
  await expect(dialog.getByRole('combobox', { name: t('profiles.manualShows'), exact: true }).locator('option:checked')).toHaveText('Music');
  await expect(dialog.getByRole('combobox', { name: t('card.effect'), exact: true })).toHaveValue('wipe');
  await expect(dialog.getByRole('spinbutton', { name: t('profiles.durationMs'), exact: true })).toHaveValue('850');
  await expect(dialog.getByRole('button', { name: t('card.direction.up'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await item(dialog, 'Music').click();
  await expect(dialog.getByRole('textbox', { name: t('profiles.nameLabel'), exact: true })).toHaveValue('Music');
  await expect(dialog.getByRole('textbox', { name: t('profiles.app'), exact: true })).toHaveValue('Spotify');
  await expect(dialog.getByRole('spinbutton', { name: t('trigger.priority'), exact: true })).toHaveValue('30');
  await expect(dialog.getByRole('spinbutton', { name: t('profiles.returnDelay'), exact: true })).toHaveValue('5');

  // Deleting asks first, in the app; the manual choice goes back to the base themes.
  await dialog.getByRole('button', { name: t('profiles.remove'), exact: true }).click();
  const confirm = page.getByRole('dialog', { name: t('profiles.deleteTitle', { name: 'Music' }), exact: true });
  await expect(confirm).toBeVisible();
  await confirm.getByRole('button', { name: t('profiles.remove'), exact: true }).click();
  await expect(item(dialog, 'Music')).toHaveCount(0);
  await expect(dialog.getByRole('combobox', { name: t('profiles.manualShows'), exact: true })).toHaveValue('');
  await dialog.getByRole('button', { name: t('common.save'), exact: true }).click();
  dialog = await openProfiles(page, t);
  await expect(dialog.locator('.pf-list .pf-item')).toHaveCount(1);
  await page.keyboard.press('Escape');
  await expect(dialog).not.toBeVisible();
  await expect(page.locator('#theme-profiles')).toBeFocused();
  expect(errors).toEqual([]);
});

test('a profile not on show that is incomplete is shown before saving', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  const dialog = await openProfiles(page, t);
  await dialog.getByRole('button', { name: t('profiles.new'), exact: true }).click();
  await dialog.getByRole('button', { name: t('profiles.addRule'), exact: true }).click();
  await dialog.getByRole('combobox', { name: t('profiles.event'), exact: true }).selectOption('foreground');
  await item(dialog, t('profiles.currentThemes')).click();
  await dialog.getByRole('button', { name: t('common.save'), exact: true }).click();
  await expect(dialog).toBeVisible();
  await expect(item(dialog, t('profiles.new'))).toHaveAttribute('aria-current', 'true');
  await expect(dialog.getByRole('textbox', { name: t('profiles.app'), exact: true })).toHaveJSProperty('validity.valueMissing', true);
  expect(errors).toEqual([]);
});

test('one profile keeps a separate theme assignment for each of two displays', async ({ page, t }, info) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=two');
  await page.getByRole('tab', { name: t(libraryTabKey('themes')), exact: true }).click();
  await page.locator('#theme-scope [data-scope="all"]').click();
  const roundName = 'Midnight 2.1" round';
  for (const name of ['Demo', roundName]) {
    await page.getByRole('button', { name: t('themesPanel.duplicateLabel', { name }), exact: true }).click();
  }
  let dialog = await openProfiles(page, t);
  await expect(dialog.locator('.pf-theme-current')).toHaveText(['Demo', roundName]);
  await dialog.getByRole('button', { name: t('profiles.new'), exact: true }).click();
  await dialog.getByRole('textbox', { name: t('profiles.nameLabel'), exact: true }).fill('Gaming');
  const gamingId = await dialog.locator('.pf-item[aria-current="true"]').getAttribute('data-item');
  const wide = dialog.locator('[data-screen="/dev/ttyACM1"] select');
  const round = dialog.locator('[data-screen="COM3"] select');
  await expect(wide.locator('option:checked')).toHaveText('Demo');
  await expect(round.locator('option:checked')).toHaveText(roundName);
  await expect(wide.locator('option')).not.toContainText([roundName]);
  await wide.selectOption({ label: t('layers.copyOf', { name: 'Demo' }) });
  await round.selectOption({ label: t('layers.copyOf', { name: roundName }) });
  await dialog.getByRole('button', { name: t('profiles.addRule'), exact: true }).click();
  await dialog.getByRole('combobox', { name: t('profiles.event'), exact: true }).selectOption('foreground');
  await dialog.getByRole('textbox', { name: t('profiles.app'), exact: true }).fill('game.exe');
  await expect(item(dialog, 'Gaming').locator('.pf-item-summary')).toHaveText(t('profiles.summary.foreground', { app: 'game.exe' }));
  if (info.project.name === 'dark-en') await dialog.locator('.pf-detail').screenshot({ path: '../../dist/profiles-two-displays.png' });
  await dialog.getByRole('button', { name: t('profiles.new'), exact: true }).click();
  await dialog.getByRole('textbox', { name: t('profiles.nameLabel'), exact: true }).fill('Music');
  const musicId = await dialog.locator('.pf-item[aria-current="true"]').getAttribute('data-item');
  await expect(dialog.locator('[data-screen="/dev/ttyACM1"] option:checked')).toHaveText('Demo');
  await expect(dialog.locator('[data-screen="COM3"] option:checked')).toHaveText(roundName);
  await dialog.getByRole('combobox', { name: t('profiles.fallback'), exact: true }).selectOption(musicId);
  await expectAccessible(page);
  await dialog.getByRole('button', { name: t('common.save'), exact: true }).click();
  dialog = await openProfiles(page, t);
  await expect(dialog.getByRole('radio', { name: t('profiles.automatic'), exact: true })).toBeChecked();
  await expect(dialog.getByRole('combobox', { name: t('profiles.fallback'), exact: true })).toHaveValue(musicId);
  await dialog.locator(`.pf-item[data-item="${gamingId}"]`).click();
  await expect(dialog.locator('[data-screen="/dev/ttyACM1"] option:checked')).toHaveText(t('layers.copyOf', { name: 'Demo' }));
  await expect(dialog.locator('[data-screen="COM3"] option:checked')).toHaveText(t('layers.copyOf', { name: roundName }));
  await dialog.locator(`.pf-item[data-item="${musicId}"]`).click();
  await expect(dialog.locator('[data-screen="COM3"] option:checked')).toHaveText(roundName);
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  expect(errors).toEqual([]);
});

test('System separates audio and power, shows text in full and omits uptime sparklines', async ({ page, t }, info) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await page.getByRole('tab', { name: t(libraryTabKey('sensors')), exact: true }).click();
  const panel = page.locator('#panel-sensors');
  // The PC's information is a folded category (T-9a): open it.
  await panel.getByRole('button', { name: t('category.system'), exact: true }).click();
  await expect(panel.locator('.sensor-subgroup')).toHaveText([t('sensorsPanel.group.system'), t('sensorsPanel.group.audio'), t('sensorsPanel.group.power')]);
  const host = panel.locator('[data-key="system.hostname"] .value');
  await expect(host).toHaveText('bezel-demo-workstation');
  await expect(host).toHaveAttribute('title', 'bezel-demo-workstation');
  await expect(panel.locator('[data-key="system.audio.muted"] .value')).toHaveText(t('sensorsPanel.state.off'));
  await expect(panel.locator('[data-key="system.power.ac"] .value')).toHaveText(t('sensorsPanel.state.on'));
  await expect(panel.locator('[data-key="system.uptime"] svg')).toHaveCount(0);
  await expect(panel.locator('.sensor-row-text svg')).toHaveCount(0);
  await expectAccessible(page);
  if (info.project.name === 'dark-en') await panel.screenshot({ path: '../../dist/system-sensors.png' });
  expect(errors).toEqual([]);
});
