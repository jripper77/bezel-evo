// The shared font picker (D-2026-10-16-studio-redesign-fidelity-28): the
// renderer's families (the demo's catalogue), search, filters, sections,
// names in their own letters, the object's text as the sample, only the
// faces a family has, a warning for a theme font the machine lacks,
// keyboard, focus back on the button, a long list drawn as it scrolls.
import { test, expect, watchErrors, expectAccessible, clickAddWidget, openInspectorTab, layers } from './helpers.mjs';
import { DEMO_FONT_CATALOG } from '../../src/demo-fonts.js';

const fontButton = (page, t) => page.locator('#inspector').getByRole('button', { name: t('inspector.fontFamily'), exact: true });
const picker = (page, t) => page.getByRole('dialog', { name: t('fontPicker.title'), exact: true });
// The Style control: a row of choices up to six faces, a list beyond (T-7b).
// The object's font: its family, its face and the missing-font note (a clock's parts have their own pickers).
const fontBlock = (page) => page.locator('#inspector [data-section="font"]').first();
const faceSelect = (page, t) => fontBlock(page).getByRole('group', { name: t('inspector.style'), exact: true })
  .or(fontBlock(page).getByRole('combobox', { name: t('inspector.style'), exact: true }));
const faceValues = (page, t) => faceSelect(page, t).evaluate((n) => (n.tagName === 'SELECT' ? [...n.options] : [...n.querySelectorAll('button')]).map((o) => o.value || o.dataset.value));
const faceShown = (page, t) => faceSelect(page, t).evaluate((n) => (n.tagName === 'SELECT' ? n.value : n.querySelector('button[aria-pressed="true"]')?.dataset.value ?? ''));
async function pickFace(page, t, value) {
  const control = faceSelect(page, t);
  if (await control.evaluate((n) => n.tagName === 'SELECT')) await control.selectOption(value);
  else await control.locator(`button[data-value="${value}"]`).click();
}
const familyOf = (locator) => locator.evaluate((n) => getComputedStyle(n).fontFamily);

async function addText(page, t) {
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await clickAddWidget(page, t, 'text');
  await openInspectorTab(page, t, 'inspector.tab.look');
  await expect(fontButton(page, t)).toBeVisible();
}

test('search, keyboard pick, real faces, focus back and one undo step', async ({ page, t }) => {
  const errors = watchErrors(page);
  await addText(page, t);
  const button = fontButton(page, t);
  await expect(button).toContainText('Inter');
  expect(await familyOf(button.locator('.font-button-name'))).toMatch(/^"?Inter"?,/);
  // Inter's faces in the demo catalogue: 400, 500, 600, 700, nothing else.
  const inter = DEMO_FONT_CATALOG.families.find((f) => f.name === 'Inter');
  await expect.poll(() => faceValues(page, t)).toHaveLength(inter.faces.length);

  await button.click();
  const dialog = picker(page, t);
  await expect(dialog).toBeVisible();
  const search = dialog.getByRole('combobox', { name: t('fontPicker.search'), exact: true });
  await expect(search).toBeFocused();
  await expectAccessible(page);
  await search.fill('geor');
  const options = dialog.getByRole('option');
  await expect(options).toHaveCount(1);
  await expect(options.first()).toHaveText(/Georgia/);
  expect(await familyOf(options.first().locator('.font-option-name'))).toMatch(/Georgia/);
  // The sample is the object's own text, in the family under the cursor.
  const sample = dialog.locator('.font-sample');
  await page.keyboard.press('Escape');
  await openInspectorTab(page, t, 'inspector.tab.data');
  const text = await page.locator('#inspector').getByRole('textbox', { name: t('common.text'), exact: true }).inputValue();
  expect(text.length).toBeGreaterThan(0);
  await openInspectorTab(page, t, 'inspector.tab.look');
  await fontButton(page, t).click();
  await search.fill('geor');
  await expect(sample).toHaveText(text);
  await expect.poll(() => familyOf(sample)).toMatch(/Georgia/);

  // Arrows move in the list, Enter picks; focus goes back to the button.
  await search.fill('cons');
  await expect(options.first()).toHaveText(/Consolas/);
  const first = await search.getAttribute('aria-activedescendant');
  await page.keyboard.press('ArrowDown');
  await expect(search).not.toHaveAttribute('aria-activedescendant', first);
  await page.keyboard.press('ArrowUp');
  await expect(search).toHaveAttribute('aria-activedescendant', first);
  await page.keyboard.press('Enter');
  await expect(dialog).toHaveCount(0);
  await expect(fontButton(page, t)).toBeFocused();
  await expect(fontButton(page, t)).toContainText('Consolas');
  // Consolas has 400, 400 italic, 700 and 700 italic: exactly those.
  const values = await faceValues(page, t);
  expect(values).toEqual(['400', '400i', '700', '700i']);
  // A new text is Semibold (600): Consolas has no 600, its nearest is Bold.
  await expect.poll(() => faceShown(page, t)).toBe('700');
  await pickFace(page, t, '400i');
  await expect.poll(() => faceShown(page, t)).toBe('400i');

  // Each change is one undo step: the face, then the family.
  await page.locator('#undo').click();
  await expect.poll(() => faceShown(page, t)).toBe('700');
  await page.locator('#undo').click();
  await expect(fontButton(page, t)).toContainText('Inter');

  // Escape closes without a change.
  await fontButton(page, t).click();
  await expect(picker(page, t)).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(picker(page, t)).toHaveCount(0);
  await expect(fontButton(page, t)).toBeFocused();
  await expect(fontButton(page, t)).toContainText('Inter');
  expect(errors).toEqual([]);
});

test('filters, sections with counts, recent picks and a long list drawn as it scrolls', async ({ page, t }) => {
  const errors = watchErrors(page);
  await addText(page, t);
  await fontButton(page, t).click();
  const dialog = picker(page, t);
  const system = dialog.getByRole('group', { name: new RegExp(`^${t('fontPicker.system')}`) });
  const bundled = dialog.getByRole('group', { name: new RegExp(`^${t('fontPicker.bundled')}`) });
  const inTheme = dialog.getByRole('group', { name: new RegExp(`^${t('fontPicker.inTheme')}`) });
  const systemCount = DEMO_FONT_CATALOG.families.filter((f) => !f.bundled).length;
  await expect(system.locator('.count')).toHaveText(String(systemCount));
  await expect(bundled.getByRole('option')).toHaveCount(DEMO_FONT_CATALOG.families.filter((f) => f.bundled).length);
  await expect(inTheme.getByRole('option', { name: /Inter/ })).toHaveCount(1);
  // Not every system family is drawn at once; scrolling draws the rest.
  const drawn = await system.getByRole('option').count();
  expect(drawn).toBeLessThan(systemCount);
  for (let i = 0; i < 6 && (await system.getByRole('option').count()) < systemCount; i += 1) {
    await dialog.locator('.font-list').evaluate((n) => { n.scrollTop = n.scrollHeight; });
    await page.waitForTimeout(50);
  }
  await expect(system.getByRole('option')).toHaveCount(systemCount);

  // Mono: only monospaced families, in every section.
  await dialog.getByRole('button', { name: t('fontPicker.mono'), exact: true }).click();
  await expect(dialog.getByRole('button', { name: t('fontPicker.mono'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  const monoNames = DEMO_FONT_CATALOG.families.filter((f) => f.category === 'mono').map((f) => f.name);
  const shown = await dialog.locator('.font-option-name').allTextContents();
  expect(shown.length).toBeGreaterThan(0);
  expect(shown.every((name) => monoNames.includes(name))).toBe(true);
  await dialog.getByRole('button', { name: t('fontPicker.serif'), exact: true }).click();
  await expect(dialog.getByRole('option', { name: /Georgia/ })).toHaveCount(1);
  await dialog.getByRole('option', { name: /Georgia/ }).click();
  await expect(fontButton(page, t)).toContainText('Georgia');

  // A recent pick is offered again on another text; Georgia is now in the theme.
  await clickAddWidget(page, t, 'text');
  await openInspectorTab(page, t, 'inspector.tab.look');
  await fontButton(page, t).click();
  await expect(inTheme.getByRole('option', { name: /Georgia/ })).toHaveCount(1);
  await dialog.getByRole('button', { name: t('fontPicker.mono'), exact: true }).click();
  await dialog.getByRole('option', { name: /Courier New/ }).click();
  await clickAddWidget(page, t, 'text');
  await openInspectorTab(page, t, 'inspector.tab.look');
  await fontButton(page, t).click();
  const recent = dialog.getByRole('group', { name: new RegExp(`^${t('fontPicker.recent')}`) });
  // Both picks are in the theme now, so they are listed there, not twice.
  await expect(inTheme.getByRole('option', { name: /Courier New/ })).toHaveCount(1);
  await expect(recent).toHaveCount(0);
  await expectAccessible(page);
  await page.keyboard.press('Escape');
  expect(errors).toEqual([]);
});

test('a theme font the machine lacks is flagged with the font drawn instead', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await page.getByRole('tab', { name: t('library.themes') }).click();
  await page.locator('#panel-themes').getByRole('button', { name: t('themesPanel.filters'), exact: true }).click();
  await page.getByRole('group', { name: t('themes.scopeLabel') }).getByRole('button', { name: t('collection.filter.all') }).click();
  await page.locator('#theme-grid .theme-card').filter({ hasText: 'TURZX' }).click();
  await expect(page.locator('#theme-name')).toHaveValue('TURZX 3.5"');
  await page.getByRole('tab', { name: t('library.layers') }).click();
  await layers(page).filter({ hasText: 'Clock' }).first().click();
  await openInspectorTab(page, t, 'inspector.tab.look');
  const button = fontButton(page, t);
  await expect(button).toContainText('Digital-7');
  await expect(button).toContainText(t('fontPicker.missing'));
  const note = t('fontPicker.missingNote', { family: 'Digital-7', fallback: 'Inter' });
  // The note of this font button (a clock's parts have a font picker of their own).
  await expect(fontBlock(page).locator('.font-missing')).toHaveText(note);
  await expect(button).toHaveAccessibleDescription(new RegExp(note.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  // The name is drawn in the fallback, as the screen draws it.
  expect(await familyOf(button.locator('.font-button-name'))).toMatch(/Inter/);
  // No family to read faces from: only the theme's own face.
  await expect.poll(() => faceValues(page, t)).toHaveLength(1);
  await expectAccessible(page);

  await button.click();
  const inTheme = picker(page, t).getByRole('group', { name: new RegExp(`^${t('fontPicker.inTheme')}`) });
  await expect(inTheme.getByRole('option', { name: /Digital-7/ })).toContainText(t('fontPicker.missing'));
  await picker(page, t).getByRole('combobox', { name: t('fontPicker.search'), exact: true }).fill('Inter');
  await page.keyboard.press('Enter');
  await expect(button).toBeFocused();
  await expect(fontButton(page, t)).toContainText('Inter');
  await expect(page.locator('#inspector .font-missing')).toHaveCount(0);
  await expect.poll(() => faceShown(page, t)).toBe('700');
  expect(errors).toEqual([]);
});


test('user fonts, licence cancellation, theme assets, face preservation and removal with undo', async ({ page, t }) => {
  const errors = watchErrors(page);
  await addText(page, t);
  await fontButton(page, t).click();
  let dialog = picker(page, t);
  await dialog.getByRole('button', { name: t('fontPicker.openFolder'), exact: true }).click();
  await dialog.getByRole('button', { name: t('fontPicker.reload'), exact: true }).click();
  await dialog.getByRole('combobox', { name: t('fontPicker.search'), exact: true }).fill('Bezel Local Demo');
  const user = dialog.getByRole('group', { name: new RegExp(`^${t('fontPicker.user')}`) });
  await expect(user.getByRole('option')).toHaveCount(1);
  await user.getByRole('option').click();
  await expect(fontButton(page, t)).toContainText('Bezel Local Demo');
  await fontButton(page, t).click();
  await dialog.getByRole('button', { name: t('fontPicker.include'), exact: true }).click();
  const licence = page.locator('dialog.confirm-dialog');
  await expect(licence).toContainText(t('fontPicker.licence'));
  await expect(licence.getByRole('button', { name: t('dialog.cancel'), exact: true })).toBeFocused();
  await page.keyboard.press('Escape');
  await fontButton(page, t).click();
  await expect(dialog.getByRole('group', { name: new RegExp(`^${t('fontPicker.themeAssets')}`) })).toHaveCount(0);
  await dialog.getByRole('button', { name: t('fontPicker.include'), exact: true }).click();
  await licence.getByRole('button', { name: t('fontPicker.include'), exact: true }).click();
  await expect(fontButton(page, t)).toBeFocused();
  await pickFace(page, t, '400');
  await fontButton(page, t).click();
  const assets = dialog.getByRole('group', { name: new RegExp(`^${t('fontPicker.themeAssets')}`) });
  await expect(assets.getByRole('option', { name: /Bezel Local Demo/ })).toHaveCount(1);
  await expect(dialog.getByRole('button', { name: t('fontPicker.remove'), exact: true })).toBeVisible();
  await expectAccessible(page);
  await dialog.getByRole('button', { name: t('fontPicker.remove'), exact: true }).click();
  await expect(fontButton(page, t)).toContainText('Bezel Local Demo');
  await fontButton(page, t).click();
  await expect(assets).toHaveCount(0);
  await page.keyboard.press('Escape');
  await page.locator('#undo').click();
  await fontButton(page, t).click();
  await expect(assets.getByRole('option')).toHaveCount(1);
  // Embedded families never offer inclusion.
  await dialog.getByRole('combobox', { name: t('fontPicker.search'), exact: true }).fill('Inter');
  await expect(dialog.getByRole('button', { name: t('fontPicker.include'), exact: true })).toBeHidden();
  await page.keyboard.press('Escape');
  expect(errors).toEqual([]);
});
