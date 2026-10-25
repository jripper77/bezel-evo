import { test, expect, watchErrors, clickAddWidget as add } from './helpers.mjs';

async function fpsFixture(page, permissions = false) {
  const catalog = [
    { key: 'gpu.fps', category: 'gpu', label: 'Game FPS', quantity: 'number', source: 'Intel PresentMon (on demand)' },
    { key: 'gpu.fps_low_1p', category: 'gpu', label: 'Game 1% low', quantity: 'number', source: 'Intel PresentMon (on demand)' },
    { key: 'gpu.frametime', category: 'gpu', label: 'Frame time', quantity: 'number', source: 'Intel PresentMon (on demand)' },
    { key: 'gpu.fps_max', category: 'gpu', label: 'Game maximum FPS', quantity: 'number', source: 'Intel PresentMon (on demand)' },
  ];
  const readings = {
    'gpu.fps': permissions ? { display: '—', unavailable: 'fps.permissions:access denied' } : { value: 160, display: '160' },
    'gpu.fps_low_1p': { display: '—', unavailable: 'fps.statistics:needs more frames' },
    'gpu.frametime': { value: 6.25, display: '6.25' },
    'gpu.fps_max': { value: 165, display: '165' },
  };
  await page.route('**/demo-backend.js', async route => {
    const response = await route.fetch();
    const body = (await response.text())
      .replace('catalog: () => Promise.resolve([', `catalog: () => Promise.resolve([...${JSON.stringify(catalog)},`)
      .replace('const readings = sampleReadings(t);', `const readings = { ...sampleReadings(t), ...${JSON.stringify(readings)} };`);
    await route.fulfill({ response, body, contentType: 'application/javascript' });
  });
  await page.goto('/index.html?demo=turing88');
}

test('FPS picker localizes names, identifies PresentMon and explains statistics warmup', async ({ page, t }) => {
  const errors = watchErrors(page);
  await fpsFixture(page);
  await add(page, t, 'value');
  await page.locator('#inspector').getByRole('combobox', { name: t('inspector.sensor'), exact: true }).click();
  const popup = page.getByRole('dialog', { name: t('inspector.sensor'), exact: true });
  await popup.getByRole('searchbox').fill('gpu.fps_low_1p');
  const low = popup.getByRole('option');
  await expect(low).toContainText(t('sensor.gpu.fps_low_1p'));
  await expect(low.locator('.sensor-choice-probe')).toHaveText('PresentMon');
  await expect(low).toHaveAttribute('title', new RegExp(t('fps.reason.statistics').replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  await popup.getByRole('searchbox').fill('gpu.fps');
  const fps = popup.getByRole('option').filter({ has: page.locator('[class="mono"]', { hasText: /^gpu\.fps$/ }) });
  await expect(fps).toContainText('160');
  expect(errors).toEqual([]);
});

test('FPS permission failures keep their localized action and the diagnostic detail', async ({ page, t }) => {
  await fpsFixture(page, true);
  await add(page, t, 'value');
  await page.locator('#inspector').getByRole('combobox', { name: t('inspector.sensor'), exact: true }).click();
  const popup = page.getByRole('dialog', { name: t('inspector.sensor'), exact: true });
  await popup.getByRole('searchbox').fill('gpu.fps');
  const fps = popup.getByRole('option').filter({ has: page.locator('[class="mono"]', { hasText: /^gpu\.fps$/ }) });
  await expect(fps).toHaveAttribute('data-state', 'unavailable');
  await expect(fps).toHaveAttribute('title', new RegExp('Performance Log Users'));
  await expect(fps).toHaveAttribute('title', /access denied/);
});

test('FPS maximum identifies its one-second averaging interval', async ({ page, t }) => {
  await fpsFixture(page);
  await add(page, t, 'value');
  await page.locator('#inspector').getByRole('combobox', { name: t('inspector.sensor'), exact: true }).click();
  const popup = page.getByRole('dialog', { name: t('inspector.sensor'), exact: true });
  await popup.getByRole('searchbox').fill('gpu.fps_max');
  const option = popup.getByRole('option');
  await expect(option).toContainText(t('sensor.gpu.fps_max'));
  await expect(option).toContainText('(1 s)');
  await expect(option).toContainText('165');
});

test.describe('Italian FPS sensor messages', () => {
  test.use({ locale: 'it-IT' });
  test('warmup is translated in the Italian picker', async ({ page, t }, info) => {
    await fpsFixture(page);
    await add(page, t, 'value');
    await page.locator('#inspector').getByRole('combobox', { name: t('inspector.sensor'), exact: true }).click();
    const popup = page.getByRole('dialog', { name: t('inspector.sensor'), exact: true });
    await popup.getByRole('searchbox').fill('gpu.fps_low_1p');
    const option = popup.getByRole('option');
    await expect(option).toContainText('FPS gioco: 1% low');
    await expect(option).toHaveAttribute('title', /Raccolta di frame recenti/);
    await popup.screenshot({ path: `../../target/logs/fps-integration/fps-picker-italian-${info.project.name}.png` });
  });
});
