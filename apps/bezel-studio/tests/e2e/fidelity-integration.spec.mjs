import { test, expect, expectAccessible, watchErrors, clickAddWidget as add, openInspectorTab as tab } from './helpers.mjs';

test('activity requests follow Player and triggered Card editors and stop in Storage', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  const requests = page.locator('html');
  const activity = /activity\.processes.*media\.sessions/;
  const anyActivity = /activity\.processes|media\.sessions/;
  await expect(requests).not.toHaveAttribute('data-demo-sensors', anyActivity);
  await add(page, t, 'player');
  await expect(requests).toHaveAttribute('data-demo-sensors', activity);
  await add(page, t, 'ring');
  await expect(requests).not.toHaveAttribute('data-demo-sensors', anyActivity);
  await add(page, t, 'card');
  await expect(requests).not.toHaveAttribute('data-demo-sensors', anyActivity);
  await expect(page.locator('.face-row').first()).toBeVisible();
  await page.locator('#inspector').getByRole('button', { name: t('card.addFace'), exact: true }).click();
  await tab(page, t, 'card.tab.triggers');
  await page.locator('#inspector').getByRole('button', { name: t('trigger.add'), exact: true }).click();
  await expect(requests).toHaveAttribute('data-demo-sensors', activity);
  await page.locator('#undo').click();
  await expect(requests).not.toHaveAttribute('data-demo-sensors', anyActivity);
  await page.locator('#redo').click();
  await expect(requests).toHaveAttribute('data-demo-sensors', activity);
  await page.locator('#tab-screen').click();
  await page.locator('#subtab-storage').click();
  // The storage manager takes the editor's place only when opened full width (T-7c).
  await page.getByRole('button', { name: t('storagePanel.expand'), exact: true }).click();
  await expect(requests).not.toHaveAttribute('data-demo-sensors', anyActivity);
  await page.locator('#tab-widgets').click();
  await expect(requests).toHaveAttribute('data-demo-sensors', activity);
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

for (const [kind, width] of [['wide', 720], ['video-preview', 850]]) {
  test(`${kind} dialog has its D-15 width and stays within the window`, async ({ page, t }) => {
    await page.goto('/index.html?demo=vendorCard');
    if (kind === 'wide') {
      await page.locator('#tab-screen').click();
      await page.locator('#subtab-storage').click();
      await page.getByRole('button', { name: t('storage.cleanup.open') }).click();
    } else {
      await page.evaluate(async kind => {
        const { translator } = await import('/i18n/index.js');
        const t = translator(document.documentElement.lang);
          const { openVideoPreview } = await import('/ui/video-preview.js');
          openVideoPreview({ t, name: 'clip.mp4', load: async () => null });
      }, kind);
    }
    const dialog = page.locator(`dialog.${kind}-dialog`);
    await expect(dialog).toBeVisible();
    await expect.poll(async () => (await dialog.boundingBox()).width).toBe(width);
    await expectAccessible(page);
    for (const windowWidth of [640, 480]) {
      await page.setViewportSize({ width: windowWidth, height: 800 });
      await expect.poll(async () => (await dialog.boundingBox()).width).toBe(Math.min(width, windowWidth - 32));
      const box = await dialog.boundingBox();
      expect(box.x).toBeGreaterThanOrEqual(16);
      expect(box.x + box.width).toBeLessThanOrEqual(windowWidth - 16);
    }
    await page.keyboard.press('Escape');
    await expect(dialog).toHaveCount(0);
  });
}
