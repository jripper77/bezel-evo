import { test, expect, clickAddWidget, openSection, choose, setColor, expectAccessible } from './helpers.mjs';

test('paint keeps the binding beside the choice, track compact, and midpoint edits undo atomically', async ({ page, t }) => {
  await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'ring');
  await openSection(page, 'color');
  const panel = page.locator('#inspector');
  const fill = panel.locator('.paint-fields');
  const row = fill.locator('.bound-field');
  await expect(row.getByRole('group', { name: t('ring.fillMode'), exact: true })).toBeVisible();
  await expect(row.getByRole('button', { name: t('insp.colorByValue'), exact: true })).toHaveAttribute('aria-pressed', 'false');
  await choose(panel, t('ring.fillMode'), 'gradient');
  const midpoint = panel.getByRole('slider', { name: t('ring.transition'), exact: true });
  await expect(midpoint).toHaveValue('50');
  await midpoint.focus(); await midpoint.press('ArrowRight');
  await expect(midpoint).toHaveValue('51');
  await expect(panel.getByRole('spinbutton', { name: t('ring.transition'), exact: true })).toHaveValue('51');
  await page.locator('#undo').click();
  await expect(midpoint).toHaveValue('50');
  await page.locator('#undo').click();
  await expect(fill.locator('.gradient-row')).toHaveCount(0);
  await choose(panel, t('ring.fillMode'), 'gradient');
  await setColor(page, t, fill, t('ring.startColor'), '#123456');
  await expect(fill.locator('.gradient-control')).toHaveCSS('background-image', /rgb\(18, 52, 86\)/);
  const track = panel.locator('[data-section="track"]');
  await expect(track.locator('.bound-field > button')).toHaveAttribute('aria-pressed', 'true');
  await expect(track.locator('.bound-field > .color-field')).toBeVisible();
  await track.getByRole('button', { name: t('inspector.showTrack'), exact: true }).click();
  await expect(track.locator('.color-field')).toHaveCount(0);
  await page.locator('#undo').click();
  await expect(track.locator('.color-field')).toBeVisible();
  await expect(track.getByRole('spinbutton', { name: t('inspector.opacity'), exact: true })).toBeVisible();
  await expectAccessible(page);
});

test('imported linear paint retains intermediate stops and angle when changing either endpoint', async ({ page, t }) => {
  await page.goto('/index.html?demo=turing88');
  await page.evaluate(async () => {
    const { gaugeFillFields } = await import('/ui/inspector/paint.js');
    const { colorPicker } = await import('/ui/fields.js');
    const { translator } = await import('/i18n/index.js');
    const { createPickers } = await import('/ui/pickers.js');
    const t = translator(document.documentElement.lang);
    const element = { id: 1, kind: { type: 'shape', fill: { angle: 37, stops: [[0, '#ff0000ff'], [.3, '#00ff00ff'], [1, '#0000ffff']] } } };
    const pickers = createPickers({ t, store: { getState: () => ({ theme: { elements: [], palette: [] } }) }, bridge: {}, storage: null });
    window.paintChanges = [];
    const fields = gaugeFillFields({ t, update: (id, patch) => window.paintChanges.push(patch), store: {}, colorField: (label, hex, change, opts) => colorPicker(label, hex, change, { ...pickers.color({ saved: false }), ...opts }) }, element);
    document.querySelector('#inspector').replaceChildren(...fields);
  });
  await setColor(page, t, page.locator('#inspector'), t('ring.startColor'), '#abcdef');
  expect(await page.evaluate(() => window.paintChanges.at(-1).kind.fill)).toEqual({ angle: 37, stops: [[0, '#abcdefff'], [.3, '#00ff00ff'], [1, '#0000ffff']] });
  await setColor(page, t, page.locator('#inspector'), t('ring.endColor'), '#654321');
  expect(await page.evaluate(() => window.paintChanges.at(-1).kind.fill)).toEqual({ angle: 37, stops: [[0, '#ff0000ff'], [.3, '#00ff00ff'], [1, '#654321ff']] });
});
