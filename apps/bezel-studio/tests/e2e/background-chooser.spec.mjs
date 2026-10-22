import { test, expect, watchErrors, expectAccessible, setColor, themeInspector as inspector, backgroundCard as card, changeBackgroundButton as changeButton, backgroundChooser as chooser, backgroundSourceButton as source, openSection, themeSize, sizeText } from './helpers.mjs';
import { SCENARIOS } from '../../src/demo-data.js';

// The theme's one background (D-2026-10-16-studio-redesign-fidelity-25):
// a card for the background in use with only its kind's options, and
// "Change background…", a panel inside the inspector that sets nothing
// until confirmed (one undo step); Cancel, Back and Esc change nothing and
// give the focus back to "Change background…".
const SCREEN = SCENARIOS.turing88.screens[0].models[0].name;
// The theme's Background and Refresh sections, opened as a person would.
async function open(page, scenario) {
  await page.goto(`/index.html?demo=${scenario}`);
  await openSection(page, 'color');
  await openSection(page, 'behavior');
}
const confirm = (page, t, key) => chooser(page, t).getByRole('button', { name: t(key), exact: true });

test('a video stored on the screen: change it for another, confirmed in one undo step', async ({ page, t }) => {
  const errors = watchErrors(page);
  await open(page, 'vendorCard');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await expect(card(page, t, 'insp.bgSolid')).toContainText(t('insp.bgSolidHint'));

  // From a colour to a video of the screen: nothing changes until confirmed.
  await changeButton(page, t).click();
  await expect(chooser(page, t)).toBeVisible();
  await expect(chooser(page, t).getByRole('button', { name: t('insp.back'), exact: true })).toBeFocused();
  await expect(source(page, t, 'insp.bgSolid')).toHaveAttribute('aria-expanded', 'true');
  await expect(source(page, t, 'insp.bgSolid')).toContainText(t('insp.bgInUse'));
  await source(page, t, 'insp.bgScreenVideo').click();
  await expect(source(page, t, 'insp.bgScreenVideo')).toHaveAttribute('aria-expanded', 'true');
  await expect(source(page, t, 'insp.bgSolid')).toHaveAttribute('aria-expanded', 'false');
  await expect(confirm(page, t, 'insp.bgUseVideo')).toBeDisabled();
  await chooser(page, t).getByRole('radio', { name: /^AMD\.mp4/ }).check();
  await expect(chooser(page, t).locator('.bg-change-line')).toContainText('#0C0E16');
  await expect(chooser(page, t).locator('.bg-change-line strong')).toHaveText('AMD.mp4');
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));
  await expect(page.locator('#undo')).toBeDisabled();
  await expectAccessible(page);
  await confirm(page, t, 'insp.bgUseVideo').click();
  await expect(chooser(page, t)).toHaveCount(0);
  await expect(changeButton(page, t)).toBeFocused();
  const stored = card(page, t, 'insp.bgScreenVideo');
  await expect(stored.locator('.bg-name')).toHaveText('AMD.mp4');
  await expect(stored.locator('.bg-status')).toHaveText(t('insp.bgOnScreen', { screen: SCREEN }));
  await expect(stored.locator('.bg-status')).toHaveAttribute('data-tone', 'ok');
  await expect(inspector(page).getByRole('button', { name: t('deviceVideo.loop'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(page.locator('#status-main')).toHaveText(t('status.unsaved'));

  // The video in use is checked; another one is one more step.
  await changeButton(page, t).click();
  await expect(source(page, t, 'insp.bgScreenVideo')).toHaveAttribute('aria-expanded', 'true');
  await expect(source(page, t, 'insp.bgScreenVideo')).toContainText(t('insp.bgInUse'));
  await expect(chooser(page, t).getByRole('radio', { name: /^AMD\.mp4/ })).toBeChecked();
  await expect(confirm(page, t, 'insp.bgUseVideo')).toBeDisabled();
  await expect(chooser(page, t)).toContainText(t('insp.bgNoChange'));
  await chooser(page, t).getByRole('radio', { name: /^NVI\.mp4 / }).check();
  await expect(chooser(page, t).locator('.bg-change-line s')).toHaveText('AMD.mp4');
  await expect(chooser(page, t).locator('.bg-change-line strong')).toHaveText('NVI.mp4');
  await expect(chooser(page, t).getByRole('button', { name: t('insp.back'), exact: true })).toBeInViewport();
  await expect(confirm(page, t, 'insp.bgUseVideo')).toBeInViewport();
  await confirm(page, t, 'insp.bgUseVideo').click();
  await expect(stored.locator('.bg-name')).toHaveText('NVI.mp4');
  await expectAccessible(page);

  await page.locator('#undo').click();
  await expect(stored.locator('.bg-name')).toHaveText('AMD.mp4');
  await page.locator('#undo').click();
  await expect(card(page, t, 'insp.bgSolid')).toBeVisible();
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));
  expect(errors).toEqual([]);
});

test('Cancel, Back and Esc change nothing and give the focus back', async ({ page, t }) => {
  const errors = watchErrors(page);
  await open(page, 'turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');

  // The picked file is a draft; Cancel leaves both the background and theme media unchanged.
  await changeButton(page, t).click();
  await source(page, t, 'insp.bgComputer').click();
  await expect(chooser(page, t)).toContainText(t('media.empty'));
  await chooser(page, t).getByRole('button', { name: t('insp.bgPickFile'), exact: true }).click();
  await expect(chooser(page, t).getByRole('radio', { name: /^ferias\.mp4/ })).toBeChecked();
  await expect(chooser(page, t).locator('.bg-change-line strong')).toHaveText('ferias.mp4');
  await expect(confirm(page, t, 'insp.bgUseVideo')).toBeEnabled();
  await chooser(page, t).getByRole('button', { name: t('dialog.cancel'), exact: true }).click();
  await expect(chooser(page, t)).toHaveCount(0);
  await expect(changeButton(page, t)).toBeFocused();
  await expect(card(page, t, 'insp.bgSolid')).toBeVisible();
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));
  await expect(page.locator('#undo')).toBeDisabled();

  await expect(page.locator('.media-item')).toHaveCount(0);

  // A colour picked, then Esc.
  await changeButton(page, t).click();
  await setColor(page, t, chooser(page, t), t('inspector.color'), '#ff0000');
  await expect(chooser(page, t).locator('.bg-change-line strong')).toHaveText('#FF0000');
  await expect(confirm(page, t, 'insp.bgUseColor')).toBeEnabled();
  await page.keyboard.press('Escape');
  await expect(chooser(page, t)).toHaveCount(0);
  await expect(changeButton(page, t)).toBeFocused();
  await expect(card(page, t, 'insp.bgSolid').getByRole('button', { name: t('inspector.color'), exact: true })).toContainText('#0c0e16');

  // The video from the computer picked, then Back, by keyboard.
  await page.keyboard.press('Enter');
  await source(page, t, 'insp.bgComputer').click();
  await chooser(page, t).getByRole('button', { name: t('insp.bgPickFile'), exact: true }).click();
  await chooser(page, t).getByRole('radio', { name: /^ferias\.mp4/ }).check();
  await chooser(page, t).getByRole('button', { name: t('insp.back'), exact: true }).focus();
  await page.keyboard.press('Enter');
  await expect(chooser(page, t)).toHaveCount(0);
  await expect(changeButton(page, t)).toBeFocused();
  await expect(page.locator('#undo')).toBeDisabled();
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));

  // Confirmed, it is the background in one step, with what the screen does with it.
  await changeButton(page, t).click();
  await source(page, t, 'insp.bgComputer').click();
  await chooser(page, t).getByRole('button', { name: t('insp.bgPickFile'), exact: true }).click();
  await chooser(page, t).getByRole('radio', { name: /^ferias\.mp4/ }).check();
  await confirm(page, t, 'insp.bgUseVideo').click();
  await expect(card(page, t, 'bg.video')).toContainText('ferias.mp4');
  await expect(card(page, t, 'bg.video').locator('.bg-status')).toHaveText(t('insp.bgToCopy'));
  await expect(inspector(page)).toContainText(t('inspector.video.checkWhenLive'));
  await page.locator('#undo').click();
  await expect(card(page, t, 'insp.bgSolid')).toBeVisible();
  await expect(page.locator('#undo')).toBeDisabled();
  expect(errors).toEqual([]);
});

test('a screen video that is missing: send it back only with a local copy', async ({ page, t }) => {
  const errors = watchErrors(page);
  await open(page, 'screenVideoMissing');
  const stored = card(page, t, 'insp.bgScreenVideo');
  await expect(stored.locator('.bg-name')).toHaveText('relogio.mp4');
  await expect(stored.locator('.bg-status')).toHaveText(t('insp.bgNotOnScreen', { screen: SCREEN }));
  await expect(stored.locator('.bg-status')).toHaveAttribute('data-tone', 'error');
  // Its video windows: how many, and Select picks them on the canvas.
  await expect(inspector(page)).toContainText(t('insp.bgWindowsHint'));
  await expect(inspector(page)).toContainText(t('insp.bgWindowsMany', { count: 2 }));
  await expectAccessible(page);
  await stored.getByRole('button', { name: t('insp.bgSendFile', { name: 'relogio.mp4' }), exact: true }).click();
  await expect(page.locator('#subtab-storage')).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByRole('dialog', { name: t('storage.restore.title', { to: t('storage.to.sd') }) })).toBeVisible();
  await expect(page.getByRole('dialog').getByRole('checkbox', { name: /relogio\.mp4/ })).toBeChecked();
  await expect(page.getByRole('dialog').getByRole('checkbox', { name: /foto\.png/ })).not.toBeChecked();
  await page.getByRole('dialog').getByRole('button', { name: t('dialog.cancel'), exact: true }).click();
  await page.locator('#tab-widgets').click();
  await inspector(page).getByRole('button', { name: t('insp.bgSelect'), exact: true }).click();
  await expect(inspector(page)).toContainText(t('inspector.multi', { count: 2 }));
  expect(errors).toEqual([]);
});

test('without a local copy only Change background is offered', async ({ page, t }) => {
  const errors = watchErrors(page);
  await open(page, 'screenVideoGone');
  const stored = card(page, t, 'insp.bgScreenVideo');
  await expect(stored.locator('.bg-status')).toHaveText(t('insp.bgNotOnScreen', { screen: SCREEN }));
  await expect(stored.getByRole('button')).toHaveText([t('insp.bgChange')]);
  expect(errors).toEqual([]);
});

test('a screen that does not play stored videos disables that source, with the reason', async ({ page, t }) => {
  const errors = watchErrors(page);
  await open(page, 'noPlayback');
  await changeButton(page, t).click();
  const screen = source(page, t, 'insp.bgScreenVideo');
  await expect(screen).toBeDisabled();
  await expect(chooser(page, t).locator('#bg-source-screen-reason')).toHaveText(t('deviceVideo.unsupported'));
  await expect(screen).toHaveAttribute('aria-describedby', 'bg-source-screen-reason');
  await expect(source(page, t, 'insp.bgComputer')).toBeEnabled();
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('orientation is two buttons and the refresh has quick values, each one undo step', async ({ page, t }) => {
  const errors = watchErrors(page);
  await open(page, 'turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  const axes = inspector(page).getByRole('group', { name: t('inspector.orientation'), exact: true });
  // The two axes are icon buttons, named for assistive technology.
  await expect(axes.getByRole('button')).toHaveCount(2);
  await expect(axes.getByRole('button').nth(0)).toHaveAccessibleName(t('axis.vertical'));
  await expect(axes.getByRole('button').nth(1)).toHaveAccessibleName(t('axis.horizontal'));
  await expect(axes.getByRole('button', { name: t('axis.vertical'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await axes.getByRole('button', { name: t('axis.horizontal'), exact: true }).click();
  await expect(axes.getByRole('button', { name: t('axis.horizontal'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(themeSize(page)).toHaveText(new RegExp(`${sizeText({ width: 1920, height: 480 })}$`));

  const refresh = inspector(page).getByRole('region', { name: t('inspector.refresh'), exact: true });
  await expect(refresh.getByRole('button', { name: t('unit.seconds', { value: 1 }), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await refresh.getByRole('button', { name: t('unit.seconds', { value: 2 }), exact: true }).click();
  await expect(refresh.getByRole('button', { name: t('unit.seconds', { value: 2 }), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(refresh.getByRole('button', { name: t('unit.seconds', { value: 1 }), exact: true })).toHaveAttribute('aria-pressed', 'false');
  await expectAccessible(page);
  await page.locator('#undo').click();
  await expect(refresh.getByRole('button', { name: t('unit.seconds', { value: 1 }), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#undo').click();
  await expect(axes.getByRole('button', { name: t('axis.vertical'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  expect(errors).toEqual([]);
});

// Expose only the demo's read/picker seams in these tests, while using the
// actual app, store and backend for every edit, rendering and undo assertion.
async function controllableDemo(page) {
  await page.route('**/bridge.js', async (route) => {
    const response = await route.fetch();
    const body = (await response.text()).replace("return { mode: 'demo', ...demo };", `
      win.backgroundDemo = demo;
      return { mode: 'demo', ...demo,
        listDevices: () => win.backgroundDevices ?? demo.listDevices(),
        pickMedia: async () => {
          const picked = await (win.backgroundPicker ? win.backgroundPicker() : demo.pickMedia());
          win.backgroundPickReturned = true;
          return picked;
        },
      };
    `);
    await route.fulfill({ response, body });
  });
}

test('disconnecting with a stored video picked disables confirmation; reconnecting clears the old pick', async ({ page, t }) => {
  const errors = watchErrors(page);
  await controllableDemo(page);
  await open(page, 'vendorCard');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await changeButton(page, t).click();
  await source(page, t, 'insp.bgScreenVideo').click();
  await chooser(page, t).getByRole('radio', { name: /^AMD\.mp4/ }).check();
  await expect(confirm(page, t, 'insp.bgUseVideo')).toBeEnabled();
  await page.evaluate(() => { window.backgroundDevices = { screens: [], desktopMode: [] }; });
  // The normal device rescan updates the open chooser, even over a solid background.
  await expect(source(page, t, 'insp.bgScreenVideo')).toBeDisabled({ timeout: 7000 });
  await expect(chooser(page, t).locator('#bg-source-screen-reason')).toHaveText(t('screen.none'));
  await expect(confirm(page, t, 'insp.bgUseVideo')).toBeDisabled();
  await expect(page.locator('#undo')).toBeDisabled();
  await page.evaluate(() => { window.backgroundDevices = null; });
  await expect(source(page, t, 'insp.bgScreenVideo')).toBeEnabled({ timeout: 7000 });
  await expect(chooser(page, t).getByRole('radio', { name: /^AMD\.mp4/ })).not.toBeChecked();
  await expect(confirm(page, t, 'insp.bgUseVideo')).toBeDisabled();
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('a late file picker does not change a reopened chooser', async ({ page, t }) => {
  const errors = watchErrors(page);
  await controllableDemo(page);
  await open(page, 'turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await page.evaluate(() => {
    window.backgroundPicker = () => new Promise((resolve) => { window.finishBackgroundPick = resolve; });
  });
  await changeButton(page, t).click();
  await source(page, t, 'insp.bgComputer').click();
  await page.locator('#bg-pick-file').click();
  await expect(page.locator('#bg-pick-file')).toBeDisabled();
  await page.locator('#bg-cancel').click();
  await changeButton(page, t).click();
  await page.evaluate(async () => {
    window.finishBackgroundPick(await window.backgroundDemo.pickMedia());
  });
  // The native pick completed, but importing still waits for confirmation.
  await expect.poll(() => page.evaluate(() => window.backgroundPickReturned)).toBe(true);
  expect(await page.evaluate(() => window.backgroundDemo.assets())).toEqual([]);
  await expect(source(page, t, 'insp.bgSolid')).toHaveAttribute('aria-expanded', 'true');
  await expect(confirm(page, t, 'insp.bgUseColor')).toBeDisabled();
  await expect(page.locator('#undo')).toBeDisabled();
  await page.locator('#bg-cancel').click();
  await expect(card(page, t, 'insp.bgSolid').getByRole('button', { name: t('inspector.color'), exact: true })).toContainText('#0c0e16');
  expect(errors).toEqual([]);
});

test('a saved refresh outside the quick choices stays selectable and survives undo', async ({ page, t, lang }) => {
  const errors = watchErrors(page);
  await page.route('**/demo-theme.js', async (route) => {
    const response = await route.fetch();
    await route.fulfill({ response, body: (await response.text()).replace('refreshSeconds: 1,', 'refreshSeconds: 1.3,') });
  });
  await open(page, 'turing88');
  const refresh = inspector(page).getByRole('region', { name: t('inspector.refresh'), exact: true });
  await expect(refresh.getByRole('button')).toHaveCount(5);
  const saved = refresh.getByRole('button', { name: t('unit.seconds', { value: (1.3).toLocaleString(lang) }), exact: true });
  await expect(saved).toHaveAttribute('aria-pressed', 'true');
  await refresh.getByRole('button', { name: t('unit.seconds', { value: (0.5).toLocaleString(lang) }), exact: true }).click();
  await page.locator('#undo').click();
  await expect(saved).toHaveAttribute('aria-pressed', 'true');
  await expect(page.locator('#undo')).toBeDisabled();
  expect(errors).toEqual([]);
});

test('restoring the missing background refreshes its card from the storage bridge answer', async ({ page, t }) => {
  const errors = watchErrors(page);
  await open(page, 'screenVideoMissing');
  const current = card(page, t, 'insp.bgScreenVideo');
  await current.getByRole('button', { name: t('insp.bgSendFile', { name: 'relogio.mp4' }), exact: true }).click();
  const restore = page.getByRole('dialog', { name: t('storage.restore.title', { to: t('storage.to.sd') }) });
  await expect(restore.getByRole('checkbox', { checked: true })).toHaveCount(1);
  await restore.getByRole('button', { name: t('storage.rename.next'), exact: true }).click();
  const plan = page.getByRole('dialog', { name: t('storage.plan.title.restore', { to: t('storage.to.sd') }) });
  await expect(plan.getByRole('listitem')).toHaveCount(1);
  await expect(plan).toContainText('relogio.mp4');
  await plan.getByRole('button', { name: t('storage.plan.action.restore'), exact: true }).click();
  await expect(page.getByRole('region', { name: t('storage.report.finished') })).toContainText(t('storage.report.done.restore', { count: 1 }));
  await page.locator('#tab-widgets').click();
  await expect(current.locator('.bg-status')).toHaveText(t('insp.bgOnScreen', { screen: SCREEN }));
  await expect(current.locator('#bg-send-stored')).toHaveCount(0);
  await expect(page.locator('#undo')).toBeDisabled();
  await expectAccessible(page);
  expect(errors).toEqual([]);
});
