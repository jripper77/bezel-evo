// T-9b (D-2026-10-20-studio-ux-v2-11, boards MainNew and StatusBarNew): the
// save state beside the theme's name, ⋯ with the version, the statusbar's
// left side and the active profile in the screen chip.
import { test, expect, watchErrors, expectAccessible, clickAddWidget } from './helpers.mjs';

test('the save state sits under the theme name and ⋯ holds the preferences and the version', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  const state = page.locator('.topbar #save-state');
  await expect(state.locator('#status-main')).toHaveText(t('status.saved'));
  await expect(state).toHaveAttribute('data-dirty', 'false');
  await clickAddWidget(page, t, 'ring');
  await expect(state.locator('#status-main')).toHaveText(t('status.unsaved'));
  await expect(state).toHaveAttribute('data-dirty', 'true');
  await page.getByRole('button', { name: t('shell.more'), exact: true }).click();
  const menu = page.getByRole('menu', { name: t('shell.more') });
  await expect(menu.getByRole('menuitem', { name: t('app.version', { version: 'demo' }) })).toBeDisabled();
  await menu.getByRole('menuitem', { name: t('prefs.title'), exact: true }).click();
  await expect(page.getByRole('dialog', { name: t('prefs.title') })).toBeVisible();
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('the statusbar says what is selected and, while dragging, what the gesture does', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  const left = page.locator('#status-selection');
  await expect(left).toHaveText(t('shell.noSelection'));
  await clickAddWidget(page, t, 'ring');
  await expect(left.locator('strong')).toHaveText(t('widget.ring'));
  await expect(left).toContainText('X ');
  const frame = await page.locator('#overlay .sel-box').boundingBox();
  await page.mouse.move(frame.x + frame.width / 2, frame.y + frame.height / 2); await page.mouse.down();
  await page.mouse.move(frame.x + frame.width / 2 + 30, frame.y + frame.height / 2 + 40, { steps: 6 });
  await expect(left.locator('strong')).toHaveText(t('shell.gesture.move', { name: t('widget.ring') }));
  await expect(left.locator('.status-hint')).toHaveText(t('shell.gestureHint.move'));
  await page.mouse.up();
  await expect(left.locator('strong')).toHaveText(t('widget.ring'));
  await expect(left.locator('.status-hint')).toHaveCount(0);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('the screen chip shows the profile the engine has active, and nothing without one', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#screen-chip-profile')).toBeHidden();
  await page.route('**/demo-backend.js', async route => {
    const response = await route.fetch();
    const source = await response.text();
    const body = source.replace("let profileConfig = { profiles: [], automatic: true, manual: null, defaultProfile: null };",
      "let profileConfig = { profiles: [{ id: 'g', name: 'Gioco', rules: [], themes: {} }], automatic: false, manual: 'g', defaultProfile: null };");
    expect(body).not.toBe(source);
    await route.fulfill({ response, body });
  });
  await page.reload();
  const chip = page.locator('#screen-chip-profile');
  await expect(chip).toBeVisible({ timeout: 10_000 });
  await expect(chip).toContainText('Gioco');
  await expect(chip.locator('small')).toHaveText(t('shell.profileManual'));
  await page.locator('#status-screen-toggle').click();
  const popover = page.locator('#screen-popover');
  await expect(popover).toBeVisible();
  // The profile already visible in the chip must be present on the first
  // opening, without waiting for another device or profile refresh.
  expect(await popover.textContent()).toContain(t('shell.profileActive', { name: 'Gioco', mode: t('shell.profileManual') }));
  await expectAccessible(page); expect(errors).toEqual([]);
});
