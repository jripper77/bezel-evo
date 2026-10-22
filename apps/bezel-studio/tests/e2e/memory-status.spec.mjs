import { test, expect, watchErrors, expectAccessible } from './helpers.mjs';

const ROWS = [['studio', 100, 90], ['webview', 101, 110], ['webview', 102, 48], ['light', 103, 40], ['sensors', 104, 24]];
// Each process: its name, then its memory (the PID in the row's title).
const expectedRows = t => ROWS.map(([role, , value]) => `${t(`shell.memoryRole.${role}`)}${t('shell.memoryMB', { value })}`);

test('the Bezel chip shows render time and memory, and its dialog the breakdown, with keyboard access', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  const toggle = page.locator('#status-memory-toggle');
  // "Bezel 11 ms · 312 MB" (board StatusBarNew): render time first, then memory.
  await expect(toggle).toContainText(t('shell.bezel'));
  await expect(toggle).toContainText(t('shell.memoryMB', { value: 312 }));
  await expect(page.locator('#status-render')).toHaveText(/^\d+ ms$/);
  await expect(page.getByRole('status').locator('#status-memory-toggle')).toHaveCount(1);
  await expect(toggle).toHaveAttribute('data-demo', 'true');
  await expect(toggle).toHaveAttribute('data-tone', 'ok');
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await toggle.focus();
  await page.keyboard.press('Enter');
  const popover = page.getByRole('dialog', { name: t('shell.bezelTitle') });
  await expect(popover).toBeVisible();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await expect(popover.getByRole('listitem')).toHaveText(expectedRows(t));
  await expect(popover.getByRole('listitem').first()).toHaveAttribute('title', t('shell.memoryProcess', { name: t('shell.memoryRole.studio'), pid: 100, value: t('shell.memoryMB', { value: 90 }) }));
  await expect(popover).toContainText(t('shell.memoryDemo'));
  await expect(popover).toContainText(t('shell.bezelCaption'));
  await expect(popover.locator('.render-spark path')).toHaveAttribute('d', /^M/);
  await expect(popover.getByRole('button', { name: t('shell.copyDiagnostics'), exact: true })).toBeVisible();
  await expect(popover.getByRole('button', { name: t('dialog.close') })).toBeFocused();
  await expectAccessible(page);
  await page.keyboard.press('Escape');
  await expect(popover).toBeHidden();
  await expect(toggle).toBeFocused();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await toggle.click();
  await expect(popover).toBeVisible();
  await page.locator('#theme-name').click();
  await expect(popover).toBeHidden();
  // The statusbar remains available while Storage owns the wide workspace.
  await page.locator('#tab-screen').click();
  await page.getByRole('tab', { name: t('screen.storageTab'), exact: true }).click();
  // Storage takes the wide workspace when opened full width (T-7c).
  await page.getByRole('button', { name: t('storagePanel.expand'), exact: true }).click();
  await expect(page.locator('.workspace')).toHaveClass(/storage-wide/);
  await toggle.click();
  await expect(popover).toBeVisible();
  await expect(popover.getByRole('listitem')).toHaveCount(5);
  await popover.evaluate(node => Promise.all(node.getAnimations().map(animation => animation.finished)));
  await expectAccessible(page);
  await popover.getByRole('button', { name: t('dialog.close') }).click();
  await expect(popover).toBeHidden();
  expect(errors).toEqual([]);
});

test('unavailable backend memory is announced without a fabricated zero or demo label', async ({ page, t }) => {
  const errors = watchErrors(page);
  // An unavailable diagnostic has no bearing on sensors or the live screen.
  await page.route('**/demo-backend.js', async route => {
    const response = await route.fetch();
    const source = await response.text();
    const body = source.replace(/memoryUsage: \(\) => Promise\.resolve\(\{ demo: true,[\s\S]*?\] \}\),/, "memoryUsage: () => Promise.resolve({ totalBytes: null, processes: [] }),");
    expect(body).not.toBe(source);
    await route.fulfill({ response, body });
  });
  await page.goto('/index.html?demo=turing88');
  const toggle = page.locator('#status-memory-toggle');
  await expect(toggle).toContainText(t('shell.memoryUnavailable'));
  await expect(toggle).toHaveAttribute('data-demo', 'false');
  await toggle.click();
  const popover = page.getByRole('dialog', { name: t('shell.bezelTitle') });
  await expect(popover).toContainText(t('shell.memoryMethod'));
  await expect(popover.getByRole('listitem')).toHaveCount(0);
  await expect(page.locator('#status-libre-toggle')).toBeVisible();
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('the render chip keeps its width from one to three digits', async ({ page, t }) => {
  await page.goto('/index.html?demo=turing88');
  const chip = page.locator('#status-render');
  await expect(chip).toContainText('ms');
  const widths = [];
  for (const ms of [5, 15, 150]) {
    await chip.evaluate((node, text) => { node.textContent = text; }, t('shell.ms', { ms }));
    widths.push((await chip.boundingBox()).width);
  }
  expect(new Set(widths).size).toBe(1);
});
