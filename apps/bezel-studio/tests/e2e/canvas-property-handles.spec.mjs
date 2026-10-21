import { test, expect, clickAddWidget, openSection, expectAccessible, watchErrors } from './helpers.mjs';
const property = (page, id) => page.locator(`#overlay [data-property-handle="${id}"]`);
const field = (page, t, key) => page.locator('#inspector').getByRole('spinbutton', { name: t(key), exact: true });

test('ring measure drags highlight the actual inspector field and undo once', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'ring'); await openSection(page, 'shape');
  const before = Number(await field(page, t, 'insp.thickness').inputValue());
  const handle = property(page, 'thickness'); const box = await handle.boundingBox();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2 + 16, { steps: 5 });
  await expect(page.locator('.handle-readout')).toBeVisible();
  await expect(page.locator('#inspector .canvas-field-active')).toHaveCount(1);
  expect(Number(await field(page, t, 'insp.thickness').inputValue())).toBeGreaterThan(before);
  await page.mouse.up(); await expect(page.locator('.handle-readout')).toHaveCount(0);
  await page.locator('#undo').click(); await expect(field(page, t, 'insp.thickness')).toHaveValue(String(before));
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('property handles and the eight square handles are keyboard controls', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'shape'); await openSection(page, 'shape');
  const corners = field(page, t, 'inspector.radius'); const before = Number(await corners.inputValue());
  await property(page, 'corner-0').focus(); await property(page, 'corner-0').press('ArrowRight');
  expect(Number(await corners.inputValue())).toBeGreaterThan(before);
  await expect(property(page, 'corner-0')).toBeFocused();
  await page.locator('#undo').click(); await expect(corners).toHaveValue(String(before));
  await openSection(page, 'layout'); const width = field(page, t, 'inspector.width');
  const original = Number(await width.inputValue());
  await page.locator('#overlay .handle[data-h="e"]').focus(); await page.keyboard.press('ArrowRight');
  await expect(width).toHaveValue(String(original + 1));
  await expect(page.locator('#overlay .handle[data-h="e"]')).toBeFocused();
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('needle pivot supports coarse keyboard movement and one-step undo', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'needle'); await openSection(page, 'shape');
  await property(page, 'pivot').focus(); await property(page, 'pivot').press('Shift+ArrowRight');
  expect(Number(await field(page, t, 'inspector.pivotX').inputValue())).toBeGreaterThan(50);
  await expect(property(page, 'pivot')).toBeFocused();
  await page.locator('#undo').click(); await expect(field(page, t, 'inspector.pivotX')).toHaveValue('50');
  await expectAccessible(page); expect(errors).toEqual([]);
});


test('Ctrl resizing changes text size without changing its frame and undoes once', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'text'); await openSection(page, 'text');
  const size = field(page, t, 'inspector.size'), before = Number(await size.inputValue());
  const frame = await page.locator('#overlay .sel-box').getAttribute('style');
  const handle = page.locator('#overlay [data-h="se"]'), box = await handle.boundingBox();
  await page.keyboard.down('Control'); await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2 + 40, box.y + box.height / 2 + 20, { steps: 5 }); await page.mouse.up(); await page.keyboard.up('Control');
  expect(Number(await size.inputValue())).toBeGreaterThan(before);
  await expect(page.locator('#overlay .sel-box')).toHaveAttribute('style', frame);
  await page.locator('#undo').click(); await expect(size).toHaveValue(String(before));
  await handle.focus(); await handle.press('Control+Shift+ArrowRight');
  expect(Number(await size.inputValue())).toBeGreaterThan(before); await expect(handle).toBeFocused();
  await expect(page.locator('#overlay .sel-box')).toHaveAttribute('style', frame);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('double click edits multiline static text, Escape cancels and acceptance is one Undo', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'text'); await openSection(page, 'data');
  const text = page.locator('#inspector').getByRole('textbox', { name: t('common.text'), exact: true });
  const before = await text.inputValue();
  const bounds = await page.locator('#overlay .sel-box').boundingBox();
  const enter = async () => page.mouse.dblclick(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2);
  await enter(); const editor = page.locator('.canvas-text-editor'); await expect(editor).toBeFocused(); await expect(editor).toHaveValue(before);
  await editor.fill('Discarded'); await editor.press('Escape'); await expect(editor).toHaveCount(0); await expect(text).toHaveValue(before);
  await enter(); await editor.fill('First line\nSecond line'); await editor.press('Control+Enter');
  await expect(editor).toHaveCount(0); await expect(text).toHaveValue('First line\nSecond line');
  await page.locator('#undo').click(); await expect(text).toHaveValue(before);
  await page.locator('#redo').click(); await expect(text).toHaveValue('First line\nSecond line');
  await enter(); await editor.fill('Commit on blur');
  await editor.dblclick(); await expect(editor).toHaveValue('Commit on blur');
  await expectAccessible(page);
  await page.locator('#theme-name').click(); await expect(editor).toHaveCount(0); await expect(text).toHaveValue('Commit on blur');
  await page.locator('#tab-layers').click();
  await page.locator('.layer-row.selected').getByRole('button', { name: t('common.lock'), exact: true }).click();
  await enter(); await expect(editor).toHaveCount(0); await expect(text).toHaveValue('Commit on blur');
  expect(errors).toEqual([]);
});

test('the player divider follows the drawn cover, opens its field and undoes once', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'player');
  const handle = property(page, 'cover-gap'); const box = await handle.boundingBox();
  const frame = await page.locator('#overlay .sel-box').boundingBox();
  // The divider sits after the cover square (as tall as the player, at most 40% of it), mid-height.
  expect(box.x + box.width / 2).toBeGreaterThan(frame.x + Math.min(frame.height, frame.width * .4));
  expect(Math.abs(box.y + box.height / 2 - (frame.y + Math.min(frame.height, frame.width * .4) / 2))).toBeLessThan(2);
  const gap = field(page, t, 'player.coverGap');
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2 + 30, box.y + box.height / 2, { steps: 5 });
  await expect(page.locator('#inspector .canvas-field-active')).toHaveCount(1);
  await expect(gap).toBeVisible();
  await page.mouse.up();
  const after = Number(await gap.inputValue());
  // One Undo for the whole drag back to the narrower default gap; Redo restores the drag.
  await page.locator('#undo').click(); await expect.poll(async () => Number(await gap.inputValue())).toBeLessThan(after);
  await page.locator('#redo').click(); await expect(gap).toHaveValue(String(after));
  await handle.focus(); await handle.press('Shift+ArrowLeft'); await expect(handle).toBeFocused();
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('the image anchor handle snaps between the nine positions by drag and keyboard', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'image'); await expect(property(page, 'image-anchor')).toHaveCount(0);
  await openSection(page, 'shape');
  await page.locator('#inspector').getByRole('button', { name: t('fit.cover'), exact: true }).click();
  const anchor = name => page.locator('#inspector').getByRole('button', { name: t(`insp.image.anchor.${name}`), exact: true });
  await expect(anchor('center')).toHaveAttribute('aria-pressed', 'true');
  const handle = property(page, 'image-anchor'), frame = await page.locator('#overlay .sel-box').boundingBox();
  const box = await handle.boundingBox();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.down();
  await page.mouse.move(frame.x + 4, frame.y + 4, { steps: 6 });
  await expect(page.locator('#inspector .canvas-field-active')).toHaveCount(1);
  await page.mouse.up();
  await expect(anchor('topLeft')).toHaveAttribute('aria-pressed', 'true');
  await handle.focus(); await handle.press('ArrowRight'); await expect(handle).toBeFocused();
  await expect(anchor('top')).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#undo').click(); await expect(anchor('topLeft')).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#undo').click(); await expect(anchor('center')).toHaveAttribute('aria-pressed', 'true');
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('a click on the prefix of the selected value picks that part in the inspector', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'value'); await openSection(page, 'data');
  await page.locator('#inspector').getByRole('textbox', { name: t('inspector.prefix'), exact: true }).fill('CPU LOAD ');
  await page.keyboard.press('Tab');
  const part = name => page.locator('#inspector').getByRole('button', { name: t(`insp.valuePart.${name}`), exact: true });
  const frame = await page.locator('#overlay .sel-box').boundingBox();
  await page.mouse.click(frame.x + 6, frame.y + frame.height / 2);
  await expect(part('prefix')).toHaveAttribute('aria-pressed', 'true');
  await expect(part('number')).toHaveAttribute('aria-pressed', 'false');
  await expect(part('prefix')).toBeInViewport();
  await page.mouse.click(frame.x + frame.width + 40, frame.y + frame.height / 2);
  await expect(page.locator('#overlay .sel-box')).toHaveCount(0);
  await expectAccessible(page); expect(errors).toEqual([]);
});
