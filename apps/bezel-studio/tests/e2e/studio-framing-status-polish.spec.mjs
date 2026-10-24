import { test, expect, openSection } from './helpers.mjs';
import path from 'node:path';
import fs from 'node:fs';

test('video framing uses the inspector neutral controls instead of orange borders and underlines', async ({ page }, testInfo) => {
  await page.goto('/index.html?demo=dragon');
  await openSection(page, 'color');
  const toggle = page.locator('#framing-canvas');
  await expect(toggle).toBeVisible();
  const borders = await page.evaluate(() => [getComputedStyle(document.querySelector('#framing-canvas')).borderColor, getComputedStyle(document.querySelector('#bg-change')).borderColor]);
  expect(borders[0]).toBe(borders[1]);
  for (const id of ['framing-rotation-auto', 'framing-fit-cover']) {
    await expect(page.locator(`#${id}`)).toHaveAttribute('aria-pressed', 'true');
    expect(await page.locator(`#${id}`).evaluate(node => getComputedStyle(node).boxShadow)).not.toContain('inset');
  }
  const directory = path.resolve('../../target/logs/import-video-polish');
  fs.mkdirSync(directory, { recursive: true });
  await page.locator('.framing').screenshot({ path: path.join(directory, `framing-${testInfo.project.name}.png`) });
});

test('the Bezel chip paints once a second and ignores isolated frame spikes', async ({ page }) => {
  await page.route('**/demo-backend.js', async route => {
    const response = await route.fetch();
    const body = (await response.text()).replaceAll('millis: performance.now() - started', 'millis: ((globalThis.frameSamples = (globalThis.frameSamples || 0) + 1) % 8 === 0 ? 120 : 5)');
    await route.fulfill({ response, body, contentType: 'application/javascript' });
  });
  await page.goto('/index.html?demo=gif');
  await expect(page.locator('#status-memory-toggle')).toHaveAttribute('data-demo', 'true');
  const result = await page.evaluate(async () => {
    const before = globalThis.frameSamples || 0;
    let changes = 0;
    const observer = new MutationObserver(records => { changes += records.length; });
    observer.observe(document.querySelector('#status-render'), { childList: true, subtree: true, characterData: true });
    await new Promise(resolve => setTimeout(resolve, 2300));
    observer.disconnect();
    return { changes, frames: (globalThis.frameSamples || 0) - before };
  });
  expect(result.frames).toBeGreaterThan(5);
  expect(result.changes).toBeLessThanOrEqual(4);
  await expect(page.locator('#status-memory-toggle')).toHaveAttribute('data-tone', 'ok');
});
