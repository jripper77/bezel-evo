// The editor's shell in demo mode, in pt-BR and en, light and dark: it loads
// without console errors or serious accessibility violations, every panel
// reads in one language, keyboard edits undo (arrows on tabs only switch
// tabs), unsaved edits are asked about before they could be lost (another
// theme, closing the window, quitting from the tray), the screen turns
// between vertical and horizontal, and the preferences switch the language,
// set the sensors and keep the Light switch.
import { test, expect, watchErrors, expectAccessible, prefixOf, layers, libraryTabKey, openSection, themeSize, sizeText } from './helpers.mjs';
import { translator } from '../../src/i18n/index.js';

test('the editor opens the demo theme', async ({ page, t, lang }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await expect(page.locator('html')).toHaveAttribute('lang', lang);
  await expect(page.locator('#screen-select')).toContainText('Turing Smart Screen 8.8"');
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));
  // The Bezel chip says the last render time (T-9b).
  await expect(page.locator('#status-render')).toHaveText(/^\d+ ms$/);
  // With nothing selected the inspector shows the theme, with its Background section.
  await expect(page.locator('#inspector')).toHaveClass(/inspector-theme/);
  await expect(page.locator('#inspector').getByRole('heading', { name: t('common.background'), exact: true })).toBeVisible();
  const box = await page.locator('#canvas-box').boundingBox();
  expect(Math.round(box.height / box.width)).toBe(4);
  await expectAccessible(page);
  for (const tab of ['sensors', 'layers', 'themes', 'media', 'screen']) {
    await page.getByRole('tab', { name: t(libraryTabKey(tab)) }).click();
    await expectAccessible(page);
  }
  expect(errors).toEqual([]);
});

test('every panel reads in one language, with no key or placeholder showing', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=desktop');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  const leftovers = /\b(?:common|storage|error|importWarning|inspector|library|toast|top|status|screen|prefs|desktop|udev|sensor|widget|category|themes|media|layers|unsaved|restart)\.[a-zA-Z]|\{[a-z]+\}|undefined|NaN/;
  const tabs = ['widgets', 'sensors', 'layers', 'themes', 'media', 'screen'];
  for (const tab of tabs) {
    await page.getByRole('tab', { name: t(libraryTabKey(tab)) }).click();
    expect(await page.locator('body').innerText()).not.toMatch(leftovers);
  }
  await page.getByRole('tab', { name: t('screen.storageTab') }).click();
  await expect(page.getByRole('region', { name: t('storage.medium.internal') })).toBeVisible();
  expect(await page.locator('body').innerText()).not.toMatch(leftovers);
  await page.getByRole('button', { name: t('prefs.title') }).click();
  await expect(page.getByRole('dialog', { name: t('prefs.title') })).toBeVisible();
  expect(await page.locator('body').innerText()).not.toMatch(leftovers);
  expect(errors).toEqual([]);
});

test('keyboard move and undo', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await page.getByRole('tab', { name: t('library.layers') }).click();
  await page.getByRole('button', { name: /^CPU/ }).click();
  await openSection(page, 'layout');
  const x = page.getByRole('spinbutton', { name: t('inspector.x'), exact: true });
  await expect(x).toHaveValue('90');

  await page.keyboard.press('Shift+ArrowRight');
  await page.keyboard.press('ArrowRight');
  await expect(x).toHaveValue('101');
  await page.keyboard.press('Control+z');
  await expect(x).toHaveValue('100');
  await page.keyboard.press('Control+z');
  await expect(x).toHaveValue('90');
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));
  await page.keyboard.press('Control+Shift+z');
  await expect(x).toHaveValue('100');

  // Typing in a field does not nudge; the field edit is one undo step.
  await x.fill('120');
  await x.press('Enter');
  await x.press('ArrowLeft');
  await expect(x).toHaveValue('120');
  await page.getByRole('button', { name: t('top.undo') }).click();
  await expect(x).toHaveValue('100');

  // Duplicate (named in the UI's language), then delete, then save.
  await page.getByRole('button', { name: /^CPU/ }).click();
  await page.keyboard.press('Control+d');
  await expect(layers(page)).toHaveCount(4);
  await expect(page.locator('#layer-list')).toContainText(t('layers.copyOf', { name: 'CPU' }));
  await page.keyboard.press('Delete');
  await expect(layers(page)).toHaveCount(3);
  await page.keyboard.press('Control+s');
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));
  await expect(page.locator('#toast')).toHaveText(t('toast.saved'));
  expect(errors).toEqual([]);
});

test('arrow keys on tabs switch tabs and leave the element alone', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await page.getByRole('tab', { name: t('library.layers') }).click();
  await page.getByRole('button', { name: /^CPU/ }).click();
  await openSection(page, 'layout');
  const x = page.getByRole('spinbutton', { name: t('inspector.x'), exact: true });
  await expect(x).toHaveValue('90');

  await page.getByRole('tab', { name: t('library.layers') }).focus();
  await page.keyboard.press('ArrowRight');
  const themes = page.getByRole('tab', { name: t('library.themes') });
  await expect(themes).toBeFocused();
  await expect(themes).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('tab', { name: t('library.screen') })).toHaveAttribute('aria-selected', 'true');

  // The Screen panel's subtabs as well.
  const settings = page.getByRole('tab', { name: t('screen.settingsTab') });
  await settings.focus();
  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('tab', { name: t('screen.storageTab') })).toBeFocused();
  await page.keyboard.press('ArrowLeft');
  await expect(settings).toBeFocused();

  await expect(x).toHaveValue('90');
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));
  await expect(page.getByRole('button', { name: t('top.undo') })).toBeDisabled();
  expect(errors).toEqual([]);
});

test('unsaved edits are not lost to another theme', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await page.getByRole('tab', { name: t('library.layers') }).click();
  await page.getByRole('button', { name: /^CPU/ }).click();
  await page.keyboard.press('ArrowRight');
  await expect(page.locator('#status-main')).toHaveText(t('status.unsaved'));

  await page.getByRole('tab', { name: t('library.themes') }).click();
  const newHorizontal = page.getByRole('button', { name: t('themes.newHorizontal') });
  const dialog = page.getByRole('dialog', { name: t('unsaved.title') });
  await newHorizontal.click();
  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText(t('unsaved.body', { name: 'Demo' }));
  await expect(dialog.getByRole('button', { name: t('common.save') })).toBeFocused();
  await expectAccessible(page);

  // Cancel and Esc keep the edits.
  await dialog.getByRole('button', { name: t('dialog.cancel') }).click();
  await expect(dialog).toHaveCount(0);
  await expect(newHorizontal).toBeFocused();
  await newHorizontal.click();
  await expect(dialog).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(dialog).toHaveCount(0);
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await expect(page.locator('#status-main')).toHaveText(t('status.unsaved'));

  // Discard lets the new theme in.
  await newHorizontal.click();
  await dialog.getByRole('button', { name: t('unsaved.discard') }).click();
  await expect(page.locator('#theme-name')).toHaveValue(t('themes.untitled'));
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));

  // Save keeps them, then the chosen theme opens.
  await page.locator('#theme-name').fill('Rascunho');
  await page.locator('#theme-name').press('Tab');
  await expect(page.locator('#status-main')).toHaveText(t('status.unsaved'));
  await page.locator('#theme-grid .theme-card').first().click();
  await dialog.getByRole('button', { name: t('common.save') }).click();
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));
  await expect(page.locator('#theme-grid .theme-card')).toHaveCount(2);
  await expect(page.locator('#theme-grid')).toContainText('Rascunho');
  expect(errors).toEqual([]);
});

test('closing the window over unsaved edits asks first', async ({ page, t }) => {
  const errors = watchErrors(page);
  const closeButton = () => page.evaluate(() => window.dispatchEvent(new Event('bezel-demo-close')));
  const html = page.locator('html');
  const dialog = page.getByRole('dialog', { name: t('unsaved.title') });

  // Nothing unsaved: the window closes at once.
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await closeButton();
  await expect(html).toHaveAttribute('data-demo-window', 'closed');
  await expect(dialog).toHaveCount(0);

  await page.goto('/index.html?demo=turing88');
  await page.getByRole('tab', { name: t('library.layers') }).click();
  await page.getByRole('button', { name: /^CPU/ }).click();
  await page.keyboard.press('Delete');
  await expect(page.locator('#status-main')).toHaveText(t('status.unsaved'));
  await closeButton();
  await expect(dialog).toBeVisible();
  await dialog.getByRole('button', { name: t('dialog.cancel') }).click();
  await expect(dialog).toHaveCount(0);
  expect(await html.getAttribute('data-demo-window')).toBeNull();
  await closeButton();
  await dialog.getByRole('button', { name: t('unsaved.discard') }).click();
  await expect(html).toHaveAttribute('data-demo-window', 'closed');
  expect(errors).toEqual([]);
});

test('quitting from the tray over unsaved edits shows the window and asks', async ({ page, t }) => {
  const errors = watchErrors(page);
  const html = page.locator('html');
  const quit = () => page.evaluate(() => window.dispatchEvent(new Event('bezel-demo-quit')));
  const dialog = page.getByRole('dialog', { name: t('unsaved.title') });

  // Nothing unsaved: the app ends at once.
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await quit();
  await expect(html).toHaveAttribute('data-demo-window', 'quit');
  await expect(dialog).toHaveCount(0);

  // Live with unsaved edits: Quit shows the window and asks; Cancel keeps
  // the app, Discard ends it.
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await page.getByRole('tab', { name: t('library.layers') }).click();
  await page.getByRole('button', { name: /^CPU/ }).click();
  await page.keyboard.press('Delete');
  await expect(page.locator('#status-main')).toHaveText(t('status.unsaved'));
  await page.getByRole('switch').click({ force: true });
  await expect(page.getByRole('switch')).toBeChecked();
  await quit();
  await expect(html).toHaveAttribute('data-demo-window', 'open');
  await expect(dialog).toBeVisible();
  await expectAccessible(page);
  await dialog.getByRole('button', { name: t('dialog.cancel') }).click();
  await expect(html).toHaveAttribute('data-demo-window', 'open');
  await quit();
  await dialog.getByRole('button', { name: t('unsaved.discard') }).click();
  await expect(html).toHaveAttribute('data-demo-window', 'quit');
  expect(errors).toEqual([]);
});

async function expectInside(page, inner, outer) {
  const o = await page.locator(outer).boundingBox();
  for (const b of await page.locator(inner).all()) {
    const i = await b.boundingBox();
    expect(i.x).toBeGreaterThanOrEqual(o.x - 1);
    expect(i.y).toBeGreaterThanOrEqual(o.y - 1);
    expect(i.x + i.width).toBeLessThanOrEqual(o.x + o.width + 1);
    expect(i.y + i.height).toBeLessThanOrEqual(o.y + o.height + 1);
  }
}

test('switch between vertical and horizontal', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  const group = page.getByRole('group', { name: t('top.orientation') });
  const vertical = group.getByRole('button', { name: t('axis.vertical'), exact: true });
  const horizontal = group.getByRole('button', { name: t('axis.horizontal'), exact: true });
  const turn = group.getByRole('button', { name: t('top.turn') });
  // The inspector's Vertical | Horizontal (D-2026-10-16-studio-redesign-fidelity-25) keep the turn, like the stage's.
  const orientation = page.locator('#inspector').getByRole('group', { name: t('inspector.orientation'), exact: true });
  const shows = (axis) => expect(orientation.getByRole('button', { name: t(`axis.${axis}`), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(vertical).toHaveAttribute('aria-pressed', 'true');
  await expect(horizontal).toHaveAttribute('aria-pressed', 'false');
  await expect(turn).toHaveAttribute('aria-pressed', 'true');
  await shows('vertical');

  await horizontal.click();
  await expect(horizontal).toHaveAttribute('aria-pressed', 'true');
  await expect(vertical).toHaveAttribute('aria-pressed', 'false');
  await expect(turn).toHaveAttribute('aria-pressed', 'true');
  await shows('horizontal');
  await expect(themeSize(page)).toHaveText(new RegExp(`${sizeText({ width: 1920, height: 480 })}$`));
  const wide = await page.locator('#canvas-box').boundingBox();
  expect(wide.width).toBeGreaterThan(wide.height);
  await page.keyboard.press('Control+a');
  await expect(page.locator('.sel-box:not(.selection-union)')).toHaveCount(3);
  await expectInside(page, '.sel-box', '#canvas-box');
  await page.keyboard.press('Escape');
  await expect(page.locator('#status-main')).toHaveText(t('status.unsaved'));
  await expectAccessible(page);

  // One undo step brings the vertical layout back.
  await page.keyboard.press('Control+z');
  await expect(vertical).toHaveAttribute('aria-pressed', 'true');
  await shows('vertical');
  const tall = await page.locator('#canvas-box').boundingBox();
  expect(tall.height).toBeGreaterThan(tall.width);
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));
  await expect(page.getByRole('button', { name: t('top.undo') })).toBeDisabled();

  // The 180° turn toggles, by mouse and by keyboard, and keeps the layout.
  await turn.click();
  await expect(turn).toHaveAttribute('aria-pressed', 'false');
  await shows('vertical');
  await expect(themeSize(page)).toHaveText(new RegExp(`${sizeText({ width: 480, height: 1920 })}$`));
  await turn.focus();
  await page.keyboard.press('Space');
  await expect(turn).toHaveAttribute('aria-pressed', 'true');
  await shows('vertical');

  // The inspector's buttons are the same command, the turn kept.
  await orientation.getByRole('button', { name: t('axis.horizontal'), exact: true }).click();
  await expect(horizontal).toHaveAttribute('aria-pressed', 'true');
  await shows('horizontal');
  await expect(turn).toHaveAttribute('aria-pressed', 'true');
  await expect(themeSize(page)).toHaveText(new RegExp(`${sizeText({ width: 1920, height: 480 })}$`));
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('the language follows the system until one is chosen', async ({ page, t, lang }) => {
  const errors = watchErrors(page);
  const html = page.locator('html');
  const other = lang === 'pt-BR' ? 'en' : 'pt-BR';
  const u = translator(other);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await expect(html).toHaveAttribute('lang', lang);
  await page.getByRole('button', { name: t('prefs.title') }).click();
  const dialog = page.getByRole('dialog', { name: t('prefs.title') });
  const language = dialog.getByRole('combobox', { name: t('prefs.language') });
  await expect(language).toBeFocused();
  await expect(language).toHaveValue('');
  await expect(language.locator('option:checked')).toHaveText(t('prefs.languageSystem', { language: t(`language.${lang}`) }));
  await expectAccessible(page);

  // Another language applies at once, the dialog included.
  await language.selectOption(other);
  await expect(html).toHaveAttribute('lang', other);
  await expect(page.getByRole('dialog', { name: u('prefs.title') })).toBeVisible();
  await expect(page.getByRole('tab', { name: u('common.sensors') })).toBeVisible();
  await expect(page.locator('#status-main')).toHaveText(u('status.saved'));
  await expect(page.locator('#inspector').getByRole('heading', { name: u('common.background'), exact: true })).toBeVisible();
  await expectAccessible(page);
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).toHaveCount(0);
  await expect(page.getByRole('button', { name: u('prefs.title') })).toBeFocused();

  // Back to the system's.
  await page.getByRole('button', { name: u('prefs.title') }).click();
  await page.getByRole('dialog').getByRole('combobox', { name: u('prefs.language') }).selectOption('');
  await expect(html).toHaveAttribute('lang', lang);
  await page.getByRole('dialog').getByRole('button', { name: t('common.done') }).click();
  await expect(page.getByRole('tab', { name: t('common.sensors') })).toBeVisible();
  expect(errors).toEqual([]);
});

test('the ping target is set in the preferences', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await page.getByRole('button', { name: t('prefs.title') }).click();
  const dialog = page.getByRole('dialog', { name: t('prefs.title') });
  const host = dialog.getByRole('textbox', { name: t('prefs.pingHost') });
  await expect(host).toHaveValue('8.8.8.8');

  // A value that is no host is refused next to the field.
  await host.fill('-c 5 host');
  await host.press('Enter');
  await expect(dialog.getByRole('alert')).toHaveText(t('error.invalidHost', { host: '-c 5 host' }));
  await expect(host).toHaveAttribute('aria-invalid', 'true');
  await expect(host).toHaveValue('-c 5 host');
  await expect(host).toBeFocused();
  await expectAccessible(page);
  await host.fill('1.1.1.1');
  await host.press('Enter');
  await expect(page.locator('#toast')).toHaveText(t('prefs.sensorsSaved'));
  await expect(dialog.getByRole('alert')).toHaveCount(0);

  await expect(host).toHaveValue('1.1.1.1');
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('closing always hands over to Light and protects unsaved live edits', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  // No close preference: Light always keeps the screen once Studio closes.
  await page.getByRole('button', { name: t('prefs.title') }).click();
  const prefs = page.getByRole('dialog', { name: t('prefs.title') });
  await expect(prefs.getByRole('switch', { name: t('prefs.debugEnable') })).toBeVisible();
  await expect(prefs.locator('#prefs-light-on-close')).toHaveCount(0);
  await expectAccessible(page);
  await page.getByRole('button', { name: t('common.done') }).click();
  await page.getByRole('switch').click({ force: true });
  await expect(page.getByRole('switch')).toBeChecked();
  await page.getByRole('tab', { name: t('library.layers') }).click();
  await page.getByRole('button', { name: /^CPU/ }).click();
  await page.keyboard.press('Delete');
  await expect(page.locator('#status-main')).toHaveText(t('status.unsaved'));
  const close = () => page.evaluate(() => window.dispatchEvent(new Event('bezel-demo-close')));
  const dialog = page.getByRole('dialog', { name: t('unsaved.title') });
  await close();
  await expect(dialog).toBeVisible();
  await dialog.getByRole('button', { name: t('dialog.cancel') }).click();
  expect(await page.locator('html').getAttribute('data-demo-window')).toBeNull();
  await close();
  await dialog.getByRole('button', { name: t('unsaved.discard') }).click();
  await expect(page.locator('html')).toHaveAttribute('data-demo-window', 'closed');
  expect(errors).toEqual([]);
});
