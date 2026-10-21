// What every e2e spec shares: the `t` fixture (the studio's own translator
// in the project's language, so each test runs in pt-BR and en alike), the
// console watch, the axe check (recorded on the run) and pointer drags.
import { test as base, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { pickLocale, translator } from '../../src/i18n/index.js';

/** The tests, with `t` and `lang` (`pt-BR` or `en`) for the project's locale. */
export const test = base.extend({
  lang: async ({ locale }, use) => use(pickLocale([locale ?? 'en'])),
  t: async ({ lang }, use) => use(translator(lang)),
});

export { expect };

/** Console errors and page errors, collected while a test runs. */
export function watchErrors(page) {
  const errors = [];
  page.on('console', (msg) => msg.type() === 'error' && errors.push(msg.text()));
  page.on('pageerror', (err) => errors.push(err.message));
  return errors;
}

/**
 * No serious or critical accessibility violation on the page now. A check
 * that passed is recorded on the test's run (annotation `axe`), so
 * `scripts/e2e-passed.mjs` can require it of every run of a named test.
 */
export async function expectAccessible(page) {
  const { violations, passes } = await new AxeBuilder({ page }).analyze();
  const serious = violations.filter((v) => ['critical', 'serious'].includes(v.impact));
  expect(serious.map((v) => `${v.id}: ${v.help} @ ${v.nodes.map((n) => n.target.join(' ')).join(', ')}`)).toEqual([]);
  test.info().annotations.push({ type: 'axe', description: `${passes.length} rules passed, no serious or critical violation` });
}

/** Drags `source` onto `target` at a fraction of its box, like a person. */
export async function dragTo(page, source, target, at = { x: 0.5, y: 0.5 }) {
  const from = await source.boundingBox();
  const to = await target.boundingBox();
  await page.mouse.move(from.x + from.width / 2, from.y + from.height / 2);
  await page.mouse.down();
  await page.mouse.move(from.x + from.width / 2 + 20, from.y + from.height / 2 + 20, { steps: 4 });
  await page.mouse.move(to.x + to.width * at.x, to.y + to.height * at.y, { steps: 8 });
  await page.mouse.up();
}

/** `text` as a regular expression that matches it literally. */
export function literally(text) {
  return new RegExp(text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'));
}

/** A placeholder no translation contains, to cut a text around a value. */
const CUT = '\u0001';

/** The part of `t(key)` before its first parameter (params filled with a cut mark). */
export function prefixOf(t, key, params) {
  return t(key, Object.fromEntries(params.map((p) => [p, CUT]))).split(CUT)[0];
}

/** The part of `t(key)` after its last parameter. */
export function suffixOf(t, key, params) {
  return t(key, Object.fromEntries(params.map((p) => [p, CUT]))).split(CUT).pop();
}

/** The rows of the Layers panel. */
export const layers = (page) => page.locator('#layer-list .layer-row');

/** Adds `widget` with its button in the Widgets panel. */
export const clickAddWidget = async (page, t, widget) => {
  await page.getByRole('button', { name: t('library.addWidget', { name: t(`widget.${widget}`) }), exact: true }).click();
  await openSection(page, 'layout');
};

/** The key of a Library tab's label: the Sensors one reads the shared `common.sensors`. */
export const libraryTabKey = (name) => (name === 'sensors' ? 'common.sensors' : `library.${name}`);

/** Opens the inspector's tab named by the translation `key`. */
export async function openSection(page, id) {
  const button = page.locator(`#inspector [data-inspector-section="${id}"] > .accordion-heading > button`);
  if (await button.count() && await button.getAttribute('aria-expanded') !== 'true') await button.click();
}

/** Legacy callers open the same controls through the new independent sections. */
export async function openInspectorTab(page, t, key) {
  const ids = key === 'inspector.tab.look' ? ['shape', 'text', 'color', 'behavior']
    : key === 'inspector.tab.data' ? ['data', 'shape', 'behavior']
      : ['card.tab.motion', 'card.tab.triggers'].includes(key) ? ['behavior'] : [key.split('.').pop()];
  for (const id of ids) await openSection(page, id);
}

/** Copies and pastes the selection with the keyboard, from a neutral control of the stage's bar. */
export async function copyAndPaste(page) {
  await page.locator('#grid-toggle').focus();
  await page.keyboard.press('Control+c');
  await page.keyboard.press('Control+v');
}

/**
 * Starts the page with the side panels at `widths` (`{library, inspector}`,
 * CSS px), as remembered by panel-sizes.js: a spec that measures the canvas
 * does not depend on the panels' starting widths.
 */
export async function setPanelWidths(page, widths) {
  await page.addInitScript((saved) => localStorage.setItem('bezel.panelWidths', JSON.stringify(saved)), widths);
}

/** The selected Card's faces in the inspector: a numbered list, one row per face (D-24). */
export function cardFaceList(page, t) {
  return page.locator('#inspector').getByRole('list', { name: t('card.tab.faces'), exact: true });
}

/** Runs a face's ⋮ action (`action` is its i18n key, e.g. 'card.duplicateFace'). */
export async function faceAction(page, t, face, action) {
  await cardFaceList(page, t).locator(`[data-face="${face}"]`).getByRole('button', { name: t('cardInsp.faceActions', { name: await cardFaceList(page, t).locator(`[data-face="${face}"] .face-name`).textContent() }), exact: true }).click();
  await page.getByRole('menu').getByRole('menuitem', { name: t(action), exact: true }).click();
}

/** Theme background controls shared by the chooser and video regression specs. */
export const themeInspector = (page) => page.locator('#inspector');
export const backgroundCard = (page, t, key) => themeInspector(page).getByRole('group', { name: t(key), exact: true });
export const changeBackgroundButton = (page, t) => themeInspector(page).getByRole('button', { name: t('insp.bgChange'), exact: true });
export const backgroundChooser = (page, t) => themeInspector(page).getByRole('region', { name: t('insp.bgChooserTitle'), exact: true });
export const backgroundSourceButton = (page, t, key) => backgroundChooser(page, t).locator('.bg-source-head').filter({ hasText: t(key) });

/** The colour picker's button named `label` (D-2026-10-16-studio-redesign-fidelity-29) in `scope`; its `data-color` is the colour. */
export const colorButton = (scope, label) => scope.getByRole('button', { name: label, exact: true });

/** The open colour panel. */
export const colorPanel = (page) => page.locator('.picker-popover:has(.cp-area)');

/** Chooses a sensor through the same searchable popup used by every card. */
export async function pickSensor(page, t, scope, key, label = t('inspector.sensor')) {
  await scope.getByRole('combobox', { name: label, exact: true }).click();
  const popup = page.getByRole('dialog', { name: label, exact: true });
  await popup.getByRole('searchbox', { name: t('insp.sensorPickerSearch'), exact: true }).fill(key);
  await popup.locator(`[role="option"][data-key="${key}"]`).click();
  await expect(popup).toHaveCount(0);
}

/**
 * Sets the colour picker named `label` in `scope` to `hex` through the
 * panel's hex field, then closes the panel with Escape.
 */
export async function setColor(page, t, scope, label, hex) {
  await colorButton(scope, label).click();
  const field = colorPanel(page).getByRole('textbox', { name: t('colorPicker.hex'), exact: true });
  await field.fill(hex);
  await field.press('Enter');
  await page.keyboard.press('Escape');
  await expect(colorPanel(page)).toHaveCount(0);
}
