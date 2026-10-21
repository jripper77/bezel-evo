import { test, expect, clickAddWidget, expectAccessible, watchErrors, layers } from './helpers.mjs';

/** The colours of the canvas at 5% and 95% along the selected bar, mid-height. */
const ends = page => page.evaluate(() => {
  const canvas = document.querySelector('#preview'), box = canvas.getBoundingClientRect();
  const bar = document.querySelector('#overlay .sel-box').getBoundingClientRect();
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  const at = share => {
    const x = Math.round((bar.left + bar.width * share - box.left) * canvas.width / box.width);
    const y = Math.round((bar.top + bar.height / 2 - box.top) * canvas.height / box.height);
    return [...ctx.getImageData(x, y, 1, 1).data].join();
  };
  return [at(.05), at(.95)];
});

test('Trial values sweep every bar from empty to full on the canvas and change nothing', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'bar'); const count = await layers(page).count();
  const toggle = page.getByRole('button', { name: t('shell.trialValues'), exact: true });
  await expect(toggle).toHaveAttribute('aria-pressed', 'false');
  await toggle.click(); await expect(toggle).toHaveAttribute('aria-pressed', 'true');
  // Over one 3 s sweep the bar is seen both empty (its two ends alike, track)
  // and full (alike again, fill): two different shared colours.
  const seen = new Set();
  for (let i = 0; i < 40 && seen.size < 2; i++) {
    const [start, end] = await ends(page);
    if (start === end) seen.add(start);
    await page.waitForTimeout(80);
  }
  expect(seen.size).toBe(2);
  await toggle.click(); await expect(toggle).toHaveAttribute('aria-pressed', 'false');
  // Nothing was recorded: one Undo removes the bar itself.
  await page.locator('#undo').click(); await expect(layers(page)).toHaveCount(count - 1);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('a bar trial handle holds the bar where it is dragged, by pointer and keys, and saves nothing', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'bar'); const count = await layers(page).count();
  const handle = page.locator('#overlay [data-property-handle="trial-value"]');
  await expect(handle).toHaveAttribute('aria-label', t('shell.trialHold'));
  const frame = await page.locator('#overlay .sel-box').boundingBox(), box = await handle.boundingBox();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.down();
  await page.mouse.move(frame.x + frame.width + 20, box.y + box.height / 2, { steps: 5 });
  await expect(page.locator('.handle-readout')).toContainText('100');
  await expect.poll(async () => { const [a, b] = await ends(page); return a === b; }).toBe(true);
  const full = (await ends(page))[1];
  await page.mouse.move(frame.x - 20, box.y + box.height / 2, { steps: 5 });
  await expect(page.locator('.handle-readout')).toContainText('0');
  await expect.poll(async () => { const [a, b] = await ends(page); return a === b && a !== full; }).toBe(true);
  await page.mouse.up(); await expect(page.locator('.handle-readout')).toHaveCount(0);
  // The handle stays where it was let go, at the start of the bar.
  const after = await handle.boundingBox();
  expect(Math.abs(after.x + after.width / 2 - frame.x)).toBeLessThan(3);
  await handle.focus(); await handle.press('ArrowRight');
  await expect(page.locator('.handle-readout')).toContainText('5');
  await expect(handle).toBeFocused();
  await page.locator('#undo').click(); await expect(layers(page)).toHaveCount(count - 1);
  await expectAccessible(page); expect(errors).toEqual([]);
});
