import { test, expect, clickAddWidget, expectAccessible, watchErrors } from './helpers.mjs';

const painted = canvas => canvas.evaluate(node => {
  const pixels = node.getContext('2d').getImageData(0, 0, node.width, node.height).data;
  return pixels.reduce((n, value, i) => n + (i % 4 === 3 && value > 128 ? 1 : 0), 0);
});

test('Try takes a gauge to full scale, returns to live, and never dirties the document', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.clock.install();
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await clickAddWidget(page, t, 'ring');
  await page.locator('#save').click();
  const saved = page.locator('#status-main');
  await expect(saved).toHaveText(t('status.saved'));
  const inspector = page.locator('#inspector');
  const button = inspector.getByRole('button', { name: t('insp.tryObject'), exact: true });
  const canvas = inspector.locator('.inspector-preview canvas');
  await expect.poll(() => painted(canvas)).toBeGreaterThan(0);
  await button.click();
  await page.clock.runFor(1500);
  await expect(button).toHaveAttribute('aria-pressed', 'true');
  const full = await painted(canvas);
  await page.clock.runFor(2200);
  await expect(button).toHaveAttribute('aria-pressed', 'false');
  expect(await painted(canvas)).toBeLessThan(full);
  await expect(saved).toHaveText(t('status.saved'));
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('a selection change cancels Try; reduced motion keeps it disabled', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.clock.install();
  await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'ring');
  const button = page.locator('#inspector .inspector-try');
  await button.click(); await page.clock.runFor(100);
  await expect(button).toHaveAttribute('aria-pressed', 'true');
  await clickAddWidget(page, t, 'bar');
  await expect(button).toHaveAttribute('aria-pressed', 'false');
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await expect(button).toBeDisabled();
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('every inspector trial renders, and weather visits distinct native shapes in each style', async ({ page }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  const results = await page.evaluate(async () => {
    const { createDemoBackend } = await import('./demo-backend.js');
    const { createWidget } = await import('./editor/widgets.js');
    const backend = createDemoBackend('turing88');
    const base = await backend.openTheme('demo://Demo');
    const results = [];
    for (const widget of ['value','ring','bar','needle','graph','clock','weather','spectrum','player','shape','image']) {
      const widgetData = createWidget(widget, base.canvas);
      const made = { ...widgetData, id: 901, name: widget, frame: { x: 0, y: 0, width: widgetData.width, height: widgetData.height } };
      const theme = { ...base, elements: [made] };
      const before = JSON.stringify(theme);
      const frame = await backend.renderElement(theme, made.id, { max: 72, trialMs: 1500 });
      results.push({ widget, valid: frame.width > 0 && frame.nextMs <= 33 && JSON.stringify(theme) === before });
    }
    const data = createWidget('weather', base.canvas);
    const weather = { ...data, id: 901, name: 'weather', frame: { x: 0, y: 0, width: data.width, height: data.height } };
    for (const style of ['outline','filled','colored','dimensional']) {
      weather.kind.content.iconStyle = style;
      const signatures = [];
      for (let i = 0; i < 8; i++) {
        const frame = await backend.renderElement({ ...base, elements: [weather] }, 901, { max: 120, trialMs: i * 1000 });
        // Hash the complete rendered condition, independent of the palette.
        signatures.push(frame.rgba.reduce((hash, byte) => (Math.imul(hash, 31) + byte) | 0, 0));
      }
      results.push({ widget: `weather-${style}`, valid: new Set(signatures).size === 8 });
    }
    const group = { id: 1, name: 'group', isGroup: true, kind: { type: 'shape' }, frame: { x: 0, y: 0, width: 20, height: 20 } };
    const gif = { id: 2, name: 'gif', groupParent: 1, kind: { type: 'image', asset: 'assets/spin.gif', fit: 'contain' }, frame: { x: 0, y: 0, width: 20, height: 20 } };
    const replay = await createDemoBackend('turing88', { now: () => 123.15 }).renderElement({ ...base, elements: [group, gif] }, 1, { trialMs: 0 });
    results.push({ widget: 'group-gif-replay', valid: [...replay.rgba.slice((10 * replay.width + 10) * 4, (10 * replay.width + 10) * 4 + 4)].join(',') === '228,87,46,255' });
    return results;
  });
  expect(results.filter(result => !result.valid)).toEqual([]);
  expect(errors).toEqual([]);
});
