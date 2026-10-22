import { test, expect, watchErrors, expectAccessible, themeInspector as inspector, backgroundCard as card, changeBackgroundButton as changeButton, backgroundChooser as chooser, backgroundSourceButton as source, openSection, storageFileAction } from './helpers.mjs';

// The theme's background is one card with only the options of its kind
// (D-2026-10-16-studio-redesign-fidelity-25, replacing the three tabs of
// D-23): a solid colour shows its colour, a video from the computer its
// status and framing, a video stored on the screen its loop, its video
// windows and the link to Screen › Storage. Each change is one undo step.
const section = (page) => inspector(page).locator('[data-section="background"]');

test('each kind shows only its own options, and each change is one undo step', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await openSection(page, 'color');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  // No tabs of kinds any more: one card, a solid colour's options only.
  await expect(section(page).getByRole('group', { name: t('insp.bgSolid'), exact: true })).toHaveCount(1);
  await expect(card(page, t, 'insp.bgSolid').locator('.bg-type')).toHaveText(t('insp.bgSolid'));
  await expect(card(page, t, 'insp.bgSolid')).toContainText(t('insp.bgSolidHint'));
  await expect(card(page, t, 'insp.bgSolid').locator('.bg-status')).toHaveCount(0);
  await expect(section(page).locator('.color-button')).toHaveCount(1);
  await expect(section(page).getByRole('button', { name: t('deviceVideo.loop'), exact: true })).toHaveCount(0);
  await expect(page.locator('#framing-canvas')).toHaveCount(0);
  await expectAccessible(page);

  // A video from the computer: its status and framing, no colour.
  await changeButton(page, t).click();
  await source(page, t, 'insp.bgComputer').click();
  await chooser(page, t).getByRole('button', { name: t('insp.bgPickFile'), exact: true }).click();
  await chooser(page, t).getByRole('button', { name: t('insp.bgUseVideo'), exact: true }).click();
  const video = card(page, t, 'bg.video');
  await expect(video.locator('.bg-type')).toHaveText(t('insp.bgComputer'));
  await expect(video).toContainText('ferias.mp4');
  await expect(video).toContainText(t('insp.bgPcHint'));
  await expect(inspector(page)).toContainText(t('inspector.video.checkWhenLive'));
  await expect(page.locator('#framing-canvas')).toBeVisible();
  await expect(section(page).locator('.color-button')).toHaveCount(0);

  // Back to a solid colour by keyboard: the last colour comes back, one step.
  await changeButton(page, t).focus();
  await page.keyboard.press('Enter');
  await source(page, t, 'insp.bgSolid').focus();
  await page.keyboard.press('Enter');
  await expect(source(page, t, 'insp.bgSolid')).toHaveAttribute('aria-expanded', 'true');
  await expect(chooser(page, t).locator('.bg-change-line strong')).toHaveText('#0C0E16');
  await chooser(page, t).getByRole('button', { name: t('insp.bgUseColor'), exact: true }).focus();
  await page.keyboard.press('Enter');
  await expect(card(page, t, 'insp.bgSolid')).toBeVisible();
  await expect(changeButton(page, t)).toBeFocused();
  await expect(video).toHaveCount(0);

  await page.locator('#undo').click();
  await expect(card(page, t, 'bg.video')).toContainText('ferias.mp4');
  await page.locator('#undo').click();
  await expect(card(page, t, 'insp.bgSolid')).toBeVisible();
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('a video stored on the screen: its loop, its video windows and the link to Screen › Storage', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=vendorCard');
  await openSection(page, 'color');
  await page.getByRole('tab', { name: t('library.screen'), exact: true }).click();
  await page.getByRole('tab', { name: t('screen.storageTab'), exact: true }).click();
  const sd = page.getByRole('region', { name: t('storage.medium.sd'), exact: true });
  await (await storageFileAction(page, t, sd, 'AMD.mp4', 'background')).click();
  await page.locator('#tab-widgets').click();
  const stored = card(page, t, 'insp.bgScreenVideo');
  await expect(stored.locator('.bg-type')).toHaveText(t('insp.bgScreenVideo'));
  await expect(stored.locator('.bg-name')).toHaveText('AMD.mp4');
  await expect(stored).toContainText(t('insp.bgScreenHint'));
  await expect(inspector(page).getByRole('button', { name: t('deviceVideo.loop'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(inspector(page)).toContainText(t('insp.bgWindowsHint'));
  await expect(inspector(page)).toContainText(t('insp.bgWindowsNone'));
  await expect(inspector(page).getByRole('button', { name: t('insp.bgSelect'), exact: true })).toHaveCount(0);
  await expect(page.locator('#framing-canvas')).toHaveCount(0);
  await expectAccessible(page);
  await inspector(page).getByRole('button', { name: t('insp.bgOpenStorage'), exact: true }).click();
  await expect(page.locator('#tab-screen')).toHaveAttribute('aria-selected', 'true');
  await expect(page.locator('#subtab-storage')).toHaveAttribute('aria-selected', 'true');
  await page.locator('#tab-widgets').click();
  await page.locator('#undo').click();
  await expect(card(page, t, 'insp.bgSolid')).toBeVisible();
  expect(errors).toEqual([]);
});
