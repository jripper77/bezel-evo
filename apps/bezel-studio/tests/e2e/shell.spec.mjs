// The window's shell: the vertical rail of panels, the floating bars over the
// stage and the status bar, by keyboard and pointer, in every scheme and language.
import { test, expect, watchErrors, expectAccessible, libraryTabKey } from './helpers.mjs';

const PANELS = ['themes', 'widgets', 'sensors', 'layers', 'media', 'screen'];

test('the panel rail is a vertical tab list the arrows walk', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  const rail = page.getByRole('tablist', { name: t('library.title'), exact: true });
  await expect(rail).toHaveAttribute('aria-orientation', 'vertical');
  for (const name of PANELS) {
    await expect(rail.getByRole('tab', { name: t(libraryTabKey(name)), exact: true })).toHaveAttribute('id', `tab-${name}`);
  }
  const tab = (name) => page.locator(`#tab-${name}`);
  await tab('widgets').click();
  await page.keyboard.press('ArrowDown');
  await expect(tab('sensors')).toBeFocused();
  await expect(tab('sensors')).toHaveAttribute('aria-selected', 'true');
  await expect(page.locator('#panel-sensors')).toBeVisible();
  await expect(page.locator('#panel-widgets')).toBeHidden();
  const ring = await tab('sensors').evaluate((n) => getComputedStyle(n).outlineStyle);
  expect(ring).toBe('solid');
  await page.keyboard.press('ArrowUp');
  await expect(tab('widgets')).toBeFocused();
  await page.keyboard.press('ArrowUp');
  await expect(tab('themes')).toBeFocused();
  await page.keyboard.press('ArrowUp');
  await expect(tab('screen')).toBeFocused();
  await expect(page.locator('#panel-screen')).toBeVisible();
  await page.keyboard.press('Home');
  await expect(tab('themes')).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('End');
  await expect(tab('screen')).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('Home');
  await expect(tab('themes')).toHaveAttribute('tabindex', '0');
  await expect(tab('sensors')).toHaveAttribute('tabindex', '-1');
  expect(await page.evaluate(() => getComputedStyle(document.body).fontFamily)).toContain('IBM Plex Sans');
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('the floating bars over the stage edit and zoom', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  const stage = page.locator('#stage');
  for (const id of ['undo', 'redo', 'grid-toggle', 'zoom-in', 'zoom-out', 'zoom-fit', 'orient-vertical', 'orient-horizontal', 'orient-turn']) {
    await expect(stage.locator(`#${id}`)).toHaveCount(1);
  }
  await expect(page.locator('.topbar #undo')).toHaveCount(0);
  const box = await stage.boundingBox();
  for (const bar of ['.stage-tools', '.stage-zoom']) {
    const b = await page.locator(bar).boundingBox();
    expect(b.x).toBeGreaterThanOrEqual(box.x);
    expect(b.x + b.width).toBeLessThanOrEqual(box.x + box.width + 1);
  }

  await page.getByRole('button', { name: t('library.addWidget', { name: t('common.text') }), exact: true }).click();
  await expect(page.locator('#undo')).toBeEnabled();
  await page.locator('#undo').click();
  await expect(page.locator('#redo')).toBeEnabled();
  await page.locator('#redo').click();
  await expect(page.locator('#undo')).toBeEnabled();

  const label = page.locator('#zoom-label');
  const fitted = await label.textContent();
  await page.locator('#zoom-in').click();
  await expect(label).not.toHaveText(fitted);
  await page.locator('#zoom-fit').click();
  await expect(label).toHaveText(fitted);
  await expect(page.getByRole('toolbar', { name: t('top.edit'), exact: true })).toBeVisible();
  await expect(page.getByRole('toolbar', { name: t('framing.zoom'), exact: true })).toBeVisible();
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('the status bar keeps its regions', async ({ page }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88&libre=partial');
  // The save state moved beside the theme's name (D-2026-10-20-studio-ux-v2-11).
  await expect(page.locator('.topbar #status-main')).toHaveCount(1);
  const bar = page.locator('footer.statusbar');
  await expect(bar).toHaveAttribute('role', 'status');
  // Left the selection; then three chips in a fixed order: Sensors, Bezel, Screen.
  for (const id of ['status-selection', 'status-libre', 'status-libre-toggle', 'status-libre-dot', 'status-libre-label', 'status-memory-toggle', 'status-screen-toggle']) {
    await expect(bar.locator(`#${id}`)).toHaveCount(1);
  }
  // Restart actions belong to each restartable probe, outside the live region.
  await expect(bar.locator('.probe-restart')).toHaveCount(0);
  await page.locator('#status-libre-toggle').click();
  await expect(page.locator('#libre-popover .probe-row[data-probe="LibreHardwareMonitor"] .probe-restart')).toHaveCount(1);
  await page.locator('#status-libre-toggle').click();
  await expect(page.locator('#status-libre-toggle')).toHaveAttribute('aria-controls', 'libre-popover');
  const order = await bar.locator('#status-libre-toggle, #status-memory-toggle, #status-screen-toggle').evaluateAll(nodes => nodes.map(n => n.id));
  expect(order).toEqual(['status-libre-toggle', 'status-memory-toggle', 'status-screen-toggle']);
  // What the screen is doing, in words, is in the Screen chip's dialog.
  await expect(page.locator('#screen-popover #status-device')).not.toBeEmpty();
  await expectAccessible(page);
  expect(errors).toEqual([]);
});
