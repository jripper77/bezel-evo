// The shared colour picker (D-2026-10-16-studio-redesign-fidelity-29): the
// area, hue and opacity by pointer and keyboard, hex and %, one undo step
// per change and per drag, the theme's own swatches (saved, renamed,
// reordered, deleted, copied and never linked), the colours the theme uses,
// recent ones, and the eyedropper only where the system has one.
import { test, expect, watchErrors, expectAccessible, clickAddWidget, openInspectorTab, openSection, colorButton, colorPanel, setColor } from './helpers.mjs';

const inspector = (page) => page.locator('#inspector');

async function addShape(page, t) {
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  // A shape's fill is in its Color section.
  await clickAddWidget(page, t, 'shape');
  await openSection(page, 'color');
  const fill = colorButton(inspector(page), t('inspector.fill'));
  await expect(fill).toBeVisible();
  return fill;
}

test('hex, opacity and keyboard each make one undo step; focus goes back to the button', async ({ page, t }) => {
  const errors = watchErrors(page);
  const fill = await addShape(page, t);
  const before = await fill.getAttribute('data-color');
  await setColor(page, t, inspector(page), t('inspector.fill'), '#ff0000');
  await expect(fill).toHaveAttribute('data-color', `#ff0000${before.slice(7)}`);
  await expect(fill).toBeFocused();
  await expect(fill).toContainText('#ff0000');

  await fill.click();
  const panel = colorPanel(page);
  await expect(panel).toHaveAttribute('role', 'dialog');
  await expect(panel).toHaveAccessibleName(t('colorPicker.title', { name: t('inspector.fill') }));
  const area = panel.getByRole('slider', { name: t('colorPicker.area'), exact: true });
  await expect(area).toBeFocused();
  await expectAccessible(page);
  // The opacity in %: 50 % is #…80.
  const percent = panel.getByRole('spinbutton', { name: t('colorPicker.opacityPercent'), exact: true });
  await percent.fill('50');
  await percent.press('Enter');
  await expect(fill).toHaveAttribute('data-color', '#ff000080');
  // Keyboard: the area's saturation, the hue, the opacity bar.
  await area.focus();
  await page.keyboard.press('ArrowLeft');
  await expect(area).toHaveAttribute('aria-valuetext', t('colorPicker.areaValue', { saturation: 99, brightness: 100 }));
  const hue = panel.getByRole('slider', { name: t('colorPicker.hue'), exact: true });
  await hue.focus();
  await page.keyboard.press('Shift+ArrowRight');
  await expect(hue).toHaveAttribute('aria-valuenow', '10');
  const opacity = panel.getByRole('slider', { name: t('inspector.opacity'), exact: true });
  await opacity.focus();
  await page.keyboard.press('ArrowRight');
  await expect(opacity).toHaveAttribute('aria-valuenow', '51');
  await page.keyboard.press('Escape');
  await expect(panel).toHaveCount(0);
  await expect(colorButton(inspector(page), t('inspector.fill'))).toBeFocused();
  // Five changes, five steps back to the colour before.
  for (let i = 0; i < 3; i += 1) await page.locator('#undo').click();
  await expect(fill).toHaveAttribute('data-color', '#ff000080');
  await page.locator('#undo').click();
  await expect(fill).toHaveAttribute('data-color', `#ff0000${before.slice(7)}`);
  await page.locator('#undo').click();
  await expect(fill).toHaveAttribute('data-color', before);
  expect(errors).toEqual([]);
});

test('a drag in the area is one gesture and one undo step', async ({ page, t }) => {
  const errors = watchErrors(page);
  const fill = await addShape(page, t);
  const before = await fill.getAttribute('data-color');
  await fill.click();
  const area = colorPanel(page).getByRole('slider', { name: t('colorPicker.area'), exact: true });
  const box = await area.boundingBox();
  await page.mouse.move(box.x + box.width * 0.9, box.y + box.height * 0.1);
  await page.mouse.down();
  for (const f of [0.8, 0.6, 0.4, 0.2]) await page.mouse.move(box.x + box.width * f, box.y + box.height * (1 - f));
  await page.mouse.up();
  const dragged = await fill.getAttribute('data-color');
  expect(dragged).not.toBe(before);
  await expect(colorPanel(page)).toHaveCount(1);
  await page.keyboard.press('Escape');
  await page.locator('#undo').click();
  await expect(fill).toHaveAttribute('data-color', before);
  await page.locator('#redo').click();
  await expect(fill).toHaveAttribute('data-color', dragged);
  expect(errors).toEqual([]);
});

test('theme swatches are saved, renamed, reordered, deleted and copied, never linked', async ({ page, t }) => {
  const errors = watchErrors(page);
  const fill = await addShape(page, t);
  await setColor(page, t, inspector(page), t('inspector.fill'), '#123456');
  await fill.click();
  const panel = colorPanel(page);
  const themed = panel.locator('[data-section="theme"]');
  await expect(themed.getByText(t('colorPicker.themeEmpty'))).toBeVisible();
  await themed.getByRole('button', { name: t('common.save'), exact: true }).click();
  await expect(themed.locator('.swatch-button')).toHaveCount(1);
  // Saved in the theme: one undo step removes it.
  await page.keyboard.press('Escape');
  await page.locator('#undo').click();
  await fill.click();
  await expect(themed.locator('.swatch-button')).toHaveCount(0);
  await page.keyboard.press('Escape');
  await page.locator('#redo').click();
  await fill.click();
  await expect(themed.locator('.swatch-button')).toHaveCount(1);

  // A second swatch, then names, order and deletion.
  const hex = panel.getByRole('textbox', { name: t('colorPicker.hex'), exact: true });
  await hex.fill('#abcdef');
  await hex.press('Enter');
  await themed.getByRole('button', { name: t('common.save'), exact: true }).click();
  await expect(themed.locator('.swatch-button')).toHaveCount(2);
  await themed.getByRole('button', { name: t('colorPicker.edit'), exact: true }).click();
  const first = themed.getByRole('textbox', { name: t('colorPicker.swatchName', { n: 1 }), exact: true });
  await first.fill('Brand');
  await first.press('Enter');
  await themed.getByRole('button', { name: t('colorPicker.moveDown', { n: 1 }), exact: true }).click();
  await expect(themed.getByRole('textbox', { name: t('colorPicker.swatchName', { n: 2 }), exact: true })).toHaveValue('Brand');
  await themed.getByRole('button', { name: t('colorPicker.remove', { n: 1 }), exact: true }).click();
  await themed.getByRole('button', { name: t('common.done'), exact: true }).click();
  const brand = themed.getByRole('button', { name: 'Brand (#123456)', exact: true });
  await expect(brand).toHaveCount(1);
  await expect(themed.locator('.swatch-button')).toHaveCount(1);
  // The shape kept its colour through every swatch edit: a swatch is a copy.
  await expect(fill).toHaveAttribute('data-color', '#abcdefff');
  await expectAccessible(page);

  // Clicking the swatch copies its colour (one step).
  await brand.click();
  await expect(fill).toHaveAttribute('data-color', '#123456ff');
  await page.keyboard.press('Escape');
  await page.locator('#undo').click();
  await expect(fill).toHaveAttribute('data-color', '#abcdefff');
  expect(errors).toEqual([]);
});

test('the colours the theme uses and the recent ones are offered', async ({ page, t }) => {
  const errors = watchErrors(page);
  const fill = await addShape(page, t);
  await setColor(page, t, inspector(page), t('inspector.fill'), '#00ff00');
  await fill.click();
  const panel = colorPanel(page);
  const used = panel.locator('[data-section="used"]');
  // The background of the demo theme and the new fill are in use.
  await expect(used.getByRole('button', { name: '#0c0e16', exact: true })).toHaveCount(1);
  await expect(used.getByRole('button', { name: '#00ff00', exact: true })).toHaveCount(1);
  const recent = panel.locator('[data-section="recent"]');
  await expect(recent.getByRole('button', { name: '#00ff00', exact: true })).toHaveCount(1);
  await used.getByRole('button', { name: '#0c0e16', exact: true }).click();
  await expect(fill).toHaveAttribute('data-color', '#0c0e16ff');
  await page.keyboard.press('Escape');
  // Recent colours are the user's: another object offers them too.
  await clickAddWidget(page, t, 'text');
  await openInspectorTab(page, t, 'inspector.tab.look');
  await colorButton(inspector(page), t('inspector.color')).click();
  await expect(colorPanel(page).locator('[data-section="recent"]').getByRole('button', { name: '#0c0e16', exact: true })).toHaveCount(1);
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('the eyedropper is offered only where the system has one', async ({ page, t }) => {
  await page.addInitScript(() => { delete window.EyeDropper; });
  const fill = await addShape(page, t);
  await fill.click();
  await expect(colorPanel(page)).toHaveCount(1);
  await expect(colorPanel(page).getByRole('button', { name: t('colorPicker.eyedropper'), exact: true })).toHaveCount(0);
});

test('the eyedropper picks a colour from the screen', async ({ page, t }) => {
  await page.addInitScript(() => {
    window.EyeDropper = class { open() { return Promise.resolve({ sRGBHex: '#336699' }); } };
  });
  const fill = await addShape(page, t);
  await fill.click();
  await colorPanel(page).getByRole('button', { name: t('colorPicker.eyedropper'), exact: true }).click();
  await expect(fill).toHaveAttribute('data-color', /^#336699/);
});

// Exercise the desktop provider through the shared picker context. WebView2's
// EyeDropper may exist but refuse to open; Studio must prefer the native path.
async function nativePicker(page) {
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await page.evaluate(async () => {
    const { colorPicker } = await import('/ui/fields.js');
    const { createPickers } = await import('/ui/pickers.js');
    const { translator, pickLocale } = await import('/i18n/index.js');
    const t = translator(pickLocale(navigator.languages));
    window.EyeDropper = class { open() { throw new Error('WebView picker must not be used'); } };
    window.nativePicks = 0;
    window.pickedColors = [];
    const pickers = createPickers({
      t, storage: null,
      store: { getState: () => ({ theme: { elements: [] } }) },
      bridge: { pickScreenColor: () => {
        window.nativePicks++;
        return new Promise((resolve, reject) => { window.finishPick = resolve; window.failPick = reject; });
      } },
    });
    const fixture = document.createElement('div');
    fixture.id = 'native-color-fixture';
    fixture.style.cssText = 'position:fixed;top:10px;left:10px;z-index:9999;background:#222';
    fixture.append(colorPicker('Native colour', '#f9731680', (c) => window.pickedColors.push(c), pickers.color({ saved: false })));
    document.body.append(fixture);
  });
  await page.locator('#native-color-fixture .color-button').click();
}

test('desktop eyedropper takes precedence, preserves opacity and reports failure without changing colour', async ({ page, t }) => {
  const errors = watchErrors(page);
  await nativePicker(page);
  const panel = colorPanel(page);
  const pick = panel.getByRole('button', { name: t('colorPicker.eyedropper'), exact: true });
  await pick.click();
  await expect(pick).toBeDisabled();
  await expect(panel.getByRole('status')).toHaveText(t('colorPicker.eyedropperHint'));
  await page.evaluate(() => window.finishPick('#336699'));
  await expect(pick).toBeEnabled();
  expect(await page.evaluate(() => window.pickedColors)).toEqual(['#33669980']);
  await pick.click();
  await page.evaluate(() => window.finishPick(null));
  await expect(pick).toBeEnabled();
  expect(await page.evaluate(() => window.pickedColors)).toEqual(['#33669980']);
  await pick.click();
  await page.evaluate(() => window.failPick(new Error('native tool failed')));
  await expect(panel.getByRole('status')).toHaveText(t('colorPicker.eyedropperError'));
  expect(await page.evaluate(() => window.pickedColors)).toEqual(['#33669980']);
  expect(errors).toEqual([]);
});

test('a desktop pick completing after dismissal does not change the old object', async ({ page, t }) => {
  await nativePicker(page);
  await colorPanel(page).getByRole('button', { name: t('colorPicker.eyedropper'), exact: true }).click();
  await page.keyboard.press('Escape');
  await expect(colorPanel(page)).toHaveCount(0);
  await page.evaluate(() => window.finishPick('#336699'));
  expect(await page.evaluate(() => window.pickedColors)).toEqual([]);
});
