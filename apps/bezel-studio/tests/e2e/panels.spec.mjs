// The side panels are resizable (D-2026-10-09-studio-redesign-fidelity-7):
// a separator per panel, with its width in aria-valuenow between its
// minimum and maximum, moved by the keys (Shift for bigger steps, Home and
// End), dragged, reset by a double click and remembered after a reload.
import { test, expect, watchErrors, expectAccessible, setPanelWidths } from './helpers.mjs';

const width = async (locator) => Math.round((await locator.boundingBox()).width);

test('the side panels resize from their separators, by keys and drag, and reset', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  const library = page.getByRole('separator', { name: t('shell.resizeLibrary'), exact: true });
  const inspector = page.getByRole('separator', { name: t('shell.resizeInspector'), exact: true });
  await expect(library).toHaveAttribute('aria-valuenow', '272');
  await expect(library).toHaveAttribute('aria-valuemin', '240');
  await expect(library).toHaveAttribute('aria-valuemax', '480');
  await expect(inspector).toHaveAttribute('aria-valuenow', '360');
  await expect(inspector).toHaveAttribute('aria-valuemin', '280');
  await expect(inspector).toHaveAttribute('aria-valuemax', '520');
  const sidebar = page.locator('.sidebar');
  const aside = page.locator('#inspector');
  expect(await width(sidebar)).toBe(64 + 272);
  expect(await width(aside)).toBe(360);

  // Keys: the arrow towards the stage widens, Shift by more, Home and End go to the limits.
  await library.focus();
  await page.keyboard.press('ArrowRight');
  await expect(library).toHaveAttribute('aria-valuenow', '280');
  await page.keyboard.press('Shift+ArrowRight');
  await expect(library).toHaveAttribute('aria-valuenow', '312');
  expect(await width(sidebar)).toBe(64 + 312);
  await page.keyboard.press('Home');
  await expect(library).toHaveAttribute('aria-valuenow', '240');
  await page.keyboard.press('ArrowLeft');
  await expect(library).toHaveAttribute('aria-valuenow', '240');
  await page.keyboard.press('End');
  await expect(library).toHaveAttribute('aria-valuenow', '480');
  await inspector.focus();
  await page.keyboard.press('ArrowLeft');
  await expect(inspector).toHaveAttribute('aria-valuenow', '368');
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('ArrowRight');
  await expect(inspector).toHaveAttribute('aria-valuenow', '352');
  expect(await width(aside)).toBe(352);

  // A double click gives a panel its starting width.
  await library.dblclick();
  await expect(library).toHaveAttribute('aria-valuenow', '272');

  // Dragging follows the pointer, within the limits.
  const box = await library.boundingBox();
  const x = box.x + box.width / 2;
  const y = box.y + box.height / 2;
  await page.mouse.move(x, y);
  await page.mouse.down();
  await page.mouse.move(x + 40, y, { steps: 5 });
  await page.mouse.up();
  await expect(library).toHaveAttribute('aria-valuenow', '312');
  await page.mouse.move(x + 40, y);
  await page.mouse.down();
  await page.mouse.move(x + 600, y, { steps: 5 });
  await page.mouse.up();
  await expect(library).toHaveAttribute('aria-valuenow', '480');
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('the panels keep their widths after a reload', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  const library = page.getByRole('separator', { name: t('shell.resizeLibrary'), exact: true });
  const inspector = page.getByRole('separator', { name: t('shell.resizeInspector'), exact: true });
  await library.focus();
  await page.keyboard.press('Shift+ArrowRight');
  await inspector.focus();
  await page.keyboard.press('Shift+ArrowLeft');
  await page.reload();
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await expect(library).toHaveAttribute('aria-valuenow', '304');
  await expect(inspector).toHaveAttribute('aria-valuenow', '392');
  expect(await width(page.locator('#inspector'))).toBe(392);
  expect(errors).toEqual([]);
});

test('remembered widths out of the limits are kept within them', async ({ page, t }) => {
  await setPanelWidths(page, { library: 10, inspector: 5000 });
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await expect(page.getByRole('separator', { name: t('shell.resizeLibrary'), exact: true })).toHaveAttribute('aria-valuenow', '240');
  // 1280 px wide: the inspector gives way so the stage keeps its room.
  const now = Number(await page.getByRole('separator', { name: t('shell.resizeInspector'), exact: true }).getAttribute('aria-valuenow'));
  expect(now).toBeGreaterThanOrEqual(280);
  expect(now).toBeLessThanOrEqual(520);
  expect(await width(page.locator('.stage'))).toBeGreaterThanOrEqual(320);
});
