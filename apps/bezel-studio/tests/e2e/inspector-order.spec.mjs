import { test, expect, openSection, openInspectorMenu, expectAccessible, watchErrors } from './helpers.mjs';
import { DEMO_THEME } from '../../src/demo-theme.js';
import { createWidget } from '../../src/editor/widgets.js';

const object = (id, extra = {}, kind = 'shape') => {
  const { width, height, ...widget } = createWidget(kind, DEMO_THEME.canvas);
  return { ...widget, id, name: `Object ${id}`, frame: { x: 20, y: 20, width, height }, opacity: 1, visible: true, locked: false, ...extra };
};
const row = (page, id) => page.locator(`.layer-row[data-element-id="${id}"]`);
const paintOrder = page => page.locator('#layer-list .layer-row').evaluateAll(rows => rows.map(n => Number(n.dataset.elementId)).reverse());
const order = page => page.locator('#inspector [data-section="order"]');
async function open(page, elements) {
  const theme = { ...structuredClone(DEMO_THEME), name: 'Order fixture', elements };
  await page.route('**/demo-theme.js', route => route.fulfill({ contentType: 'text/javascript', body: `export const DEMO_THEME = ${JSON.stringify(theme)};` }));
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Order fixture');
  await page.locator('#tab-layers').click();
}
async function select(page, id, options) {
  await row(page, id).locator('.layer-name').click(options);
  await openSection(page, 'layout');
}

test('order is in Layout for every object kind and uses one Undo per command', async ({ page, t }) => {
  const errors = watchErrors(page);
  const kinds = ['value', 'text', 'ring', 'bar', 'needle', 'graph', 'clock', 'weather', 'player', 'spectrum', 'shape', 'image', 'card'];
  await open(page, kinds.map((kind, i) => object(i + 1, { name: kind }, kind)));
  for (let i = 0; i < kinds.length; i++) {
    await select(page, i + 1);
    const panel = page.locator('#inspector');
    await expect(order(page)).toBeVisible();
    expect(await order(page).evaluate(n => n.previousElementSibling.dataset.section)).toBe('membership');
    await expect(order(page).getByRole('button')).toHaveCount(4);
    await expect(order(page).locator('.hint')).toHaveText(t('insp.layer', { level: i + 1, total: kinds.length }));
    await expect(panel.getByRole('button', { name: t('common.duplicate'), exact: true })).toHaveCount(0);
    await expect(panel.getByRole('button', { name: t('common.delete'), exact: true })).toHaveCount(0);
    const before = await paintOrder(page);
    const forward = order(page).getByRole('button', { name: t('insp.oneForward'), exact: true });
    if (i === kinds.length - 1) await expect(forward).toBeDisabled();
    else {
      await forward.click();
      const expected = [...before]; [expected[i], expected[i + 1]] = [expected[i + 1], expected[i]];
      expect(await paintOrder(page)).toEqual(expected);
      await page.locator('#undo').click(); expect(await paintOrder(page)).toEqual(before);
    }
  }
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('multi-selection preserves relative order, hides the level and shares flat menu commands', async ({ page, t }) => {
  const errors = watchErrors(page); await open(page, [1, 2, 3, 4, 5].map(id => object(id)));
  await select(page, 2); await select(page, 4, { modifiers: ['Control'] });
  await expect(order(page).locator('.hint')).toHaveCount(0);
  await order(page).getByRole('button', { name: t('insp.oneForward'), exact: true }).click();
  expect(await paintOrder(page)).toEqual([1, 3, 2, 5, 4]);
  await page.locator('#undo').click(); expect(await paintOrder(page)).toEqual([1, 2, 3, 4, 5]);
  const menu = await openInspectorMenu(page);
  const labels = await menu.getByRole('menuitem').allTextContents();
  const start = labels.indexOf(t('inspector.front'));
  expect(labels.slice(start, start + 4)).toEqual([t('inspector.front'), t('insp.oneForward'), t('insp.oneBackward'), t('inspector.back')]);
  await expect(menu.getByRole('menuitem', { name: t('insp.oneBackward'), exact: true })).not.toHaveAttribute('aria-haspopup');
  await menu.getByRole('menuitem', { name: t('insp.oneBackward'), exact: true }).click();
  expect(await paintOrder(page)).toEqual([2, 1, 4, 3, 5]);
  await page.locator('#undo').click(); expect(await paintOrder(page)).toEqual([1, 2, 3, 4, 5]);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('members stay inside their group or Card face and locks disable every order command', async ({ page, t }) => {
  const errors = watchErrors(page);
  await open(page, [object(1, { isGroup: true }), object(2, { groupParent: 1 }), object(3, { groupParent: 1 }),
    object(4, { card: { faces: ['A', 'B'], activeFace: 0 } }), object(5, { cardMember: { parent: 4, face: 0 } }),
    object(6, { cardMember: { parent: 4, face: 0 } }), object(7, { cardMember: { parent: 4, face: 1 } }), object(8, { locked: true })]);
  await select(page, 2);
  await order(page).getByRole('button', { name: t('inspector.front'), exact: true }).click();
  await expect(page.locator('[data-group-id="1"] .layer-row').nth(1)).toHaveAttribute('data-element-id', '2');
  await expect(order(page).getByRole('button', { name: t('insp.oneForward'), exact: true })).toBeDisabled();
  await page.locator('#undo').click();
  await select(page, 1);
  await order(page).getByRole('button', { name: t('inspector.front'), exact: true }).click();
  await expect(page.locator('#layer-list > .layer-card').first()).toHaveAttribute('data-group-id', '1');
  await page.locator('#undo').click();
  await (await openInspectorMenu(page)).getByRole('menuitem', { name: t('common.lock'), exact: true }).click();
  for (const id of [1, 2]) {
    await select(page, id);
    for (const button of await order(page).getByRole('button').all()) await expect(button).toBeDisabled();
  }
  await select(page, 5);
  await order(page).getByRole('button', { name: t('insp.oneForward'), exact: true }).click();
  const face = page.locator('.layer-card[data-card-id="4"] [data-face="0"] .layer-row');
  expect(await face.evaluateAll(rows => rows.map(n => n.dataset.elementId))).toEqual(['5', '6']);
  await expect(order(page).getByRole('button', { name: t('inspector.front'), exact: true })).toBeDisabled();
  await page.locator('#undo').click();
  await select(page, 8);
  for (const button of await order(page).getByRole('button').all()) await expect(button).toBeDisabled();
  await expectAccessible(page); expect(errors).toEqual([]);
});
