import { test, expect, watchErrors, expectAccessible, openSection } from './helpers.mjs';

const object = (id, frame, extra = {}) => ({ id, name: `Selection ${id}`, frame, opacity: 1, visible: true, locked: false,
  kind: { type: 'shape', shape: 'rect', radius: 0, fill: '#ff9248ff', stroke: null, strokeWidth: 0 }, ...extra });
const first = () => object(1, { x: 40, y: 40, width: 80, height: 60 });
const second = () => object(2, { x: 180, y: 100, width: 40, height: 30 });
async function open(page, elements = [first(), second()]) {
  const theme = { schema: 1, name: 'Selection', canvas: { width: 480, height: 320 }, orientation: 'portrait', refreshSeconds: 1,
    background: { type: 'color', color: '#000000ff' }, elements };
  await page.route('**/demo-theme.js', route => route.fulfill({ contentType: 'text/javascript', body: `export const DEMO_THEME = ${JSON.stringify(theme)};` }));
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Selection');
  await page.locator('#tab-layers').click();
}
async function select(page, ids) {
  for (const [index, id] of ids.entries()) await page.locator(`.layer-row[data-element-id="${id}"] .layer-name`).click({ modifiers: index ? ['Control'] : [] });
}
const squares = page => page.locator('#overlay .handle');
const boxes = page => page.locator('#overlay .sel-box[role="img"]').evaluateAll(nodes => nodes.map(n =>
  Object.fromEntries(['left', 'top', 'width', 'height'].map(key => [key, parseFloat(n.style[key])]))));

test('union resize scales positions and sizes proportionally with one Undo', async ({ page }) => {
  const errors = watchErrors(page); await open(page); await select(page, [1, 2]);
  await expect(squares(page)).toHaveCount(8); await expect(page.locator('.selection-union')).toHaveCount(1);
  const before = await boxes(page);
  const handle = await page.locator('#overlay [data-h="se"]').boundingBox();
  await page.mouse.move(handle.x + handle.width / 2, handle.y + handle.height / 2); await page.mouse.down();
  await page.keyboard.down('Alt');
  await page.mouse.move(handle.x + handle.width / 2 + 70, handle.y + handle.height / 2 + 35, { steps: 6 });
  await expect(page.locator('.handle-readout')).toBeVisible();
  await page.mouse.up(); await page.keyboard.up('Alt');
  const after = await boxes(page), factor = after[0].width / before[0].width;
  expect(factor).toBeGreaterThan(1.1);
  expect(Math.abs(after[0].height / before[0].height - factor)).toBeLessThan(.03);
  expect(Math.abs(after[1].width / before[1].width - factor)).toBeLessThan(.04);
  expect(Math.abs((after[1].left - after[0].left) / (before[1].left - before[0].left) - factor)).toBeLessThan(.03);
  await page.locator('#undo').click(); expect(await boxes(page)).toEqual(before);
  await page.locator('#redo').click(); expect(await boxes(page)).toEqual(after);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('common rotation uses the inspector command and keyboard resize keeps focus', async ({ page, t }) => {
  const errors = watchErrors(page); await open(page); await select(page, [1, 2]);
  const before = await boxes(page), east = page.locator('#overlay [data-h="e"]');
  await east.focus(); await east.press('Shift+ArrowRight'); await expect(east).toBeFocused();
  expect((await boxes(page))[0].width).toBeGreaterThan(before[0].width);
  await page.locator('#undo').click(); expect(await boxes(page)).toEqual(before);
  const rotation = page.locator('#overlay [data-rotate]'); await rotation.focus(); await rotation.press('Shift+ArrowRight');
  await expect(rotation).toBeFocused(); await openSection(page, 'layout');
  await expect(page.locator('#inspector').getByRole('spinbutton', { name: t('insp.rotation'), exact: true })).toHaveValue('15');
  for (const box of await page.locator('#overlay .sel-box[role="img"]').all()) await expect(box).toHaveCSS('transform', /matrix\(0.965926, 0.258819/);
  await page.locator('#undo').click();
  for (const box of await page.locator('#overlay .sel-box[role="img"]').all()) await expect(box).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, 0)');
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('selected descendants transform once and inherited locks suppress all handles', async ({ page, t }) => {
  const errors = watchErrors(page);
  const group = { ...first(), isGroup: true }, child = object(3, { x: 40, y: 40, width: 80, height: 60 }, { groupParent: 1 });
  await open(page, [group, child, second()]); await select(page, [1, 3, 2]);
  await expect(squares(page)).toHaveCount(8); await expect(page.locator('#overlay [data-rotate]')).toHaveCount(0);
  const before = await boxes(page); const east = page.locator('#overlay [data-h="e"]');
  await east.focus(); await east.press('Shift+ArrowRight');
  const after = await boxes(page); expect(after[0].width).toBe(after[1].width); expect(after[0].left).toBe(after[1].left);
  expect(after[0].width).toBeGreaterThan(before[0].width);
  await page.locator('#undo').click(); expect(await boxes(page)).toEqual(before);
  await page.locator('.layer-row[data-element-id="1"]').getByRole('button', { name: t('common.lock'), exact: true }).click();
  await select(page, [3]); await expect(squares(page)).toHaveCount(0); await expect(page.locator('#overlay .property-handle')).toHaveCount(0);
  await select(page, [3, 2]); await expect(squares(page)).toHaveCount(0);
  await expectAccessible(page); expect(errors).toEqual([]);
});


test('alignment guides show the real gap and disappear when the gesture ends', async ({ page, t }) => {
  const errors = watchErrors(page); await open(page); await select(page, [1]);
  const box = await page.locator('#overlay .sel-box[role="img"]').boundingBox();
  const zoom = (await page.locator('#canvas-box').boundingBox()).width / 480;
  const before = await boxes(page);
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2 + 60 * zoom, { steps: 5 });
  const distance = page.locator('.guide-distance.y');
  await expect(distance).toHaveText('60 px');
  await expect(distance).toHaveAttribute('aria-label', t('shell.guideDistance', { value: '60 px' }));
  await expect(page.locator('.guide.y')).toHaveCount(1); await page.mouse.up();
  await expect(page.locator('.guide-distance, .guide')).toHaveCount(0);
  await page.locator('#undo').click(); expect(await boxes(page)).toEqual(before);
  await expectAccessible(page); expect(errors).toEqual([]);
});
