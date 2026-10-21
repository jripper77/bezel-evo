// The library panels in demo mode, in pt-BR and en, light and dark: widgets
// and sensors drag onto the canvas, new themes take either axis, imports
// list what had no equivalent, the sensor list tells the app what it shows,
// and an animated GIF moves in the preview by itself.
import { test, expect, watchErrors, expectAccessible, dragTo, literally, layers, themeSize, sizeText } from './helpers.mjs';

const addWidget = (t, widget) => t('library.addWidget', { name: t(`widget.${widget}`) });

test('drag a widget onto the canvas', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await dragTo(page, page.getByRole('button', { name: addWidget(t, 'bar') }), page.locator('#canvas-box'), { x: 0.5, y: 0.6 });
  await expect(page.locator('.sel-box')).toHaveCount(1);
  await expect(page.locator('#status-main')).toHaveText(t('status.unsaved'));
  await expect(page.locator('#inspector')).toContainText(t('inspector.position'));
  await expect(page.locator('#inspector').getByRole('textbox', { name: t('common.name'), exact: true })).toHaveValue(t('widget.bar'));
  await page.getByRole('tab', { name: t('library.layers') }).click();
  await expect(layers(page)).toHaveCount(4);
  await expectAccessible(page);

  await page.getByRole('tab', { name: t('common.sensors') }).click();
  const gpuTemperature = t('sensor.gpu.temperature');
  await page.getByRole('searchbox', { name: t('library.searchSensors') }).fill(gpuTemperature.toLowerCase());
  await dragTo(page, page.getByRole('button', { name: literally(gpuTemperature) }), page.locator('#canvas-box'), { x: 0.3, y: 0.8 });
  await expect(page.locator('#inspector').getByRole('combobox', { name: t('inspector.sensor'), exact: true })).toHaveAttribute('data-sensor-key', 'gpu.temperature');
  await expect(page.locator('#inspector').getByRole('textbox', { name: t('common.name'), exact: true })).toHaveValue(gpuTemperature);
  await page.getByRole('tab', { name: t('library.layers') }).click();
  await expect(layers(page)).toHaveCount(5);

  // A drop outside the canvas adds nothing; a plain click adds at the center.
  await page.getByRole('tab', { name: t('library.widgets') }).click();
  await dragTo(page, page.getByRole('button', { name: addWidget(t, 'text') }), page.locator('#inspector'));
  await page.getByRole('button', { name: addWidget(t, 'ring') }).click();
  await page.getByRole('tab', { name: t('library.layers') }).click();
  await expect(layers(page)).toHaveCount(6);
  expect(errors).toEqual([]);
});

test('new vertical and horizontal themes', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await page.getByRole('tab', { name: t('library.themes') }).click();
  const cards = page.locator('#theme-grid .theme-card');
  await expect(cards).toHaveCount(1);
  await expect(cards.first()).toContainText(t('axis.vertical'));

  await page.getByRole('button', { name: t('themes.newHorizontal') }).click();
  await expect(page.locator('#theme-name')).toHaveValue(t('themes.untitled'));
  const topBar = page.getByRole('group', { name: t('top.orientation') });
  await expect(topBar.getByRole('button', { name: t('axis.horizontal'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  const box = await page.locator('#canvas-box').boundingBox();
  expect(box.width).toBeGreaterThan(box.height);
  await page.keyboard.press('Control+s');
  await expect(cards).toHaveCount(2);
  // The gallery lists the newest first: the saved theme is found by its name.
  const saved = cards.filter({ hasText: t('themes.untitled') });
  await expect(saved).toContainText(t('axis.horizontal'));
  await expect(saved).toContainText('1920×480');

  await page.getByRole('button', { name: t('themes.newVertical') }).click();
  await expect(topBar.getByRole('button', { name: t('axis.vertical'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(themeSize(page)).toHaveText(new RegExp(`${sizeText({ width: 480, height: 1920 })}$`));
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('an import lists what had no equivalent', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await page.getByRole('tab', { name: t('library.themes') }).click();
  await page.getByRole('button', { name: t('themes.import') }).click();
  await expect(page.locator('#theme-name')).toHaveValue('Imported');
  await expect(page.locator('#toast')).toHaveText(t('toast.importedWithWarnings', { count: 2 }));
  const report = page.getByRole('region', { name: t('import.title') });
  await expect(report).toBeVisible();
  await expect(report.getByRole('listitem')).toHaveCount(2);
  await expect(report).toContainText(t('importWarning.backplateLed'));
  await expect(report).toContainText(t('importWarning.cpuFanGuessed', { name: 'CPU.FAN_SPEED.TEXT' }));
  await expectAccessible(page);
  await report.getByRole('button', { name: t('import.dismiss') }).click();
  await expect(report).toHaveCount(0);
  await expect(page.getByRole('button', { name: t('themes.import') })).toBeFocused();
  expect(errors).toEqual([]);
});

// The app measures what is shown: `net.ping` sends packets only while the
// theme or this list shows it (D-2026-09-30-release-polish-11).
test('the sensor list tells the app which sensors it shows', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  const html = page.locator('html');
  await expect(html).toHaveAttribute('data-demo-sensors', '');
  await page.getByRole('tab', { name: t('common.sensors') }).click();
  await expect(html).toHaveAttribute('data-demo-sensors', /^cpu\.usage .*net\.down net\.up/);
  await page.getByRole('searchbox', { name: t('library.searchSensors') }).fill('net.');
  await expect(html).toHaveAttribute('data-demo-sensors', 'net.down net.up');
  await page.getByRole('tab', { name: t('library.layers') }).click();
  await expect(html).toHaveAttribute('data-demo-sensors', '');
  expect(errors).toEqual([]);
});

test('an animated GIF moves in the preview at its own pace', async ({ page }) => {
  // T-7.11: the theme refreshes every 5 s, its GIF every 100 ms; the
  // preview draws the GIF's frames in between, at most 15 a second.
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=gif');
  await expect(page.locator('#theme-name')).toHaveValue('GIF');
  const colors = await page.evaluate(async () => {
    const ctx = document.getElementById('preview').getContext('2d');
    const seen = new Set();
    const end = performance.now() + 900;
    while (performance.now() < end) {
      seen.add(ctx.getImageData(300, 240, 1, 1).data.slice(0, 3).join(','));
      await new Promise((resolve) => { setTimeout(resolve, 30); });
    }
    return [...seen];
  });
  expect(colors.length).toBeGreaterThanOrEqual(3);
  expect(errors).toEqual([]);
});
