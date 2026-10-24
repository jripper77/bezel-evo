import { test, expect, watchErrors } from './helpers.mjs';

async function importedVideo(page, { stored = false, local = true } = {}) {
  await page.route('**/demo-backend.js', async route => {
    const response = await route.fetch();
    let body = await response.text();
    const asset = stored ? 'dragon' : 'nebula';
    body = body.replace("theme: { ...structuredClone(DEMO_THEME), name: 'Imported' }", `theme: { ...structuredClone(DEMO_THEME), name: 'Imported', canvas: { width: 1920, height: 480 }, orientation: 'landscape', background: { type: 'video', asset: 'assets/${asset}.mp4' } }`);
    body = body.replace('checkThemeVideo: () => {', `checkThemeVideo: () => {
      globalThis.checkedImports = (globalThis.checkedImports || 0) + 1;
      ${local ? '' : "if (globalThis.checkedImports === 1) return Promise.resolve({ present: false, local: false, name: 'missing.mp4', path: 'internal/video/missing.mp4' });"}`);
    body = body.replace('runUpload: async (ticket, overwrite) => {', 'runUpload: async (ticket, overwrite) => { globalThis.importUploads = (globalThis.importUploads || 0) + 1;');
    await route.fulfill({ response, body, contentType: 'application/javascript' });
  });
  await page.goto(`/index.html?demo=${stored ? 'dragon' : 'turing88'}`);
}

async function importFile(page, t) {
  await page.getByRole('tab', { name: t('library.themes') }).click();
  await page.getByRole('button', { name: t('themes.import') }).click();
  await expect(page.locator('#theme-name')).toHaveValue('Imported');
}

test('import uses a video already on device without a copy prompt or enabling Live', async ({ page, t }) => {
  const errors = watchErrors(page);
  await importedVideo(page, { stored: true });
  await importFile(page, t);
  await expect.poll(() => page.evaluate(() => globalThis.checkedImports || 0)).toBeGreaterThan(0);
  await expect(page.getByRole('dialog')).toHaveCount(0);
  await expect(page.locator('#live')).not.toBeChecked();
  expect(await page.evaluate(() => globalThis.importUploads || 0)).toBe(0);
  expect(errors).toEqual([]);
});

test('import asks before copying its included video and completes conversion and upload offline', async ({ page, t }) => {
  const errors = watchErrors(page);
  await importedVideo(page);
  await importFile(page, t);
  const summary = page.getByRole('dialog');
  await expect(summary).toContainText('nebula');
  expect(await page.evaluate(() => globalThis.importUploads || 0)).toBe(0);
  await summary.getByRole('button', { name: t('storage.confirmUploadAction'), exact: true }).click();
  await expect(page.locator('#toast')).toHaveText(t('storage.uploaded', { name: 'nebula_90.mp4' }));
  await expect(page.locator('#live')).not.toBeChecked();
  expect(await page.evaluate(() => globalThis.importUploads)).toBe(1);
  expect(errors).toEqual([]);
});

test('declining the import copy keeps the theme open without uploading', async ({ page, t }) => {
  await importedVideo(page);
  await importFile(page, t);
  const summary = page.getByRole('dialog');
  await summary.getByRole('button', { name: t('dialog.cancel'), exact: true }).click();
  await expect(summary).toHaveCount(0);
  await expect(page.locator('#theme-name')).toHaveValue('Imported');
  expect(await page.evaluate(() => globalThis.importUploads || 0)).toBe(0);
});

test('an import without a local video asks for its file before preparing the copy', async ({ page, t }) => {
  const errors = watchErrors(page);
  await importedVideo(page, { local: false });
  await importFile(page, t);
  const choose = page.getByRole('dialog', { name: t('import.videoFileTitle') });
  await expect(choose).toContainText('missing.mp4');
  expect(await page.evaluate(() => globalThis.importUploads || 0)).toBe(0);
  await choose.getByRole('button', { name: t('import.videoChoose'), exact: true }).click();
  const summary = page.getByRole('dialog');
  await expect(summary).toContainText('ferias');
  await summary.getByRole('button', { name: t('storage.confirmUploadAction'), exact: true }).click();
  await expect(page.locator('#toast')).toHaveText(t('storage.uploaded', { name: 'ferias_90.mp4' }));
  await expect(page.locator('#live')).not.toBeChecked();
  expect(errors).toEqual([]);
});
