import { test, expect, openSection, clickAddWidget, expectAccessible, watchErrors } from './helpers.mjs';

const number = (page, t, key) => page.locator('#inspector').getByRole('spinbutton', { name: t(key), exact: true });
async function change(input, value) { await input.fill(String(value)); await input.press('Tab'); }

test('value parts inherit, reset and undo; digits, overflow, roll and averaging remain independent', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'value');
  const inspector = page.locator('#inspector');
  await openSection(page, 'text');
  const parts = inspector.getByRole('group', { name: t('insp.valueParts'), exact: true });
  await parts.getByRole('button', { name: t('insp.valuePart.unit'), exact: true }).click();
  const size = number(page, t, 'insp.partSize');
  await expect(size).toHaveValue('');
  await change(size, 55); await expect(size).toHaveValue('55');
  await page.locator('#undo').click(); await expect(size).toHaveValue('');
  await change(size, 70);
  await inspector.getByRole('button', { name: t('insp.inheritPart'), exact: true }).click();
  await expect(size).toHaveValue('');
  const regular = inspector.locator('[data-section="font"] > .property-section-body > .fld-grid').getByRole('button', { name: t('weight.400'), exact: true });
  await regular.focus(); await regular.press('Enter');
  await expect(regular).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#undo').click(); await expect(regular).toHaveAttribute('aria-pressed', 'false');
  const upper = inspector.getByRole('group', { name: t('textCase.label'), exact: true }).getByRole('button', { name: t('textCase.upper'), exact: true });
  await upper.focus(); await upper.press('Enter');
  await expect(upper).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#undo').click(); await expect(upper).toHaveAttribute('aria-pressed', 'false');
  const digits = inspector.getByRole('button', { name: t('insp.tabularDigits'), exact: true });
  await expect(digits).toHaveAttribute('aria-pressed', 'true');
  await digits.click(); await expect(digits).toHaveAttribute('aria-pressed', 'false');
  await openSection(page, 'behavior');
  await inspector.getByRole('button', { name: t('insp.transitionRoll'), exact: true }).click();
  await expect(inspector.getByRole('button', { name: t('insp.transitionRoll'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await change(number(page, t, 'insp.averageReadings'), 8);
  await change(number(page, t, 'insp.averageReadings'), 1);
  await expect(number(page, t, 'insp.averageReadings')).toHaveValue('1');
  await page.locator('#undo').click(); await expect(number(page, t, 'insp.averageReadings')).toHaveValue('8');
  await inspector.getByRole('button', { name: t('insp.overflow.ellipsis'), exact: true }).click();
  await expect(inspector.getByRole('button', { name: t('insp.overflow.ellipsis'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await openSection(page, 'data');
  await inspector.getByRole('combobox', { name: t('inspector.decimals'), exact: true }).selectOption('2');
  await inspector.getByRole('group', { name: t('inspector.content'), exact: true }).getByRole('button', { name: t('widget.text'), exact: true }).click();
  await expect(inspector.locator('textarea')).toHaveValue(/\d+\.\d{2}%/);
  await page.locator('#undo').click();
  await expect(inspector.getByRole('combobox', { name: t('inspector.decimals'), exact: true })).toHaveValue('2');
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('static multiline text has a real shadow, which disappears on Undo', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'text');
  await expect(page.locator('#inspector .inspector-live')).toHaveText(t('insp.fixedState'));
  await expect(page.locator('#inspector .inspector-live')).toHaveAttribute('data-live', 'false');
  await openSection(page, 'data');
  await change(page.locator('#inspector textarea'), 'First\nSecond');
  await expect(page.locator('#inspector textarea')).toHaveValue('First\nSecond');
  await openSection(page, 'color');
  const shadow = page.locator('#inspector').getByRole('button', { name: t('insp.textShadow'), exact: true });
  await expect(shadow).toHaveAttribute('aria-pressed', 'false');
  await shadow.click(); await expect(shadow).toHaveAttribute('aria-pressed', 'true');
  await change(number(page, t, 'insp.shadowBlur'), 12);
  await expect(number(page, t, 'insp.shadowBlur')).toHaveValue('12');
  await shadow.click(); await expect(number(page, t, 'insp.shadowBlur')).toBeHidden();
  await page.locator('#undo').click(); await expect(number(page, t, 'insp.shadowBlur')).toHaveValue('12');
  const rendered = await page.evaluate(async () => {
    const { createDemoBackend } = await import('./demo-backend.js');
    const { createWidget } = await import('./editor/widgets.js');
    const backend = createDemoBackend('turing88'), base = await backend.openTheme('demo://Demo');
    const made = createWidget('text', base.canvas);
    const element = { ...made, id: 998, name: 'shadow', frame: { x: 0, y: 0, width: 300, height: 100 } };
    element.kind.content.text = 'Test'; element.kind.style.size = 48;
    const theme = { ...base, elements: [element] };
    const plain = await backend.renderElement(theme, 998, { max: 300, motion: false });
    element.kind.style.shadow = { color: '#ff0000ff', blur: 8 };
    const shaded = await backend.renderElement(theme, 998, { max: 300, motion: false });
    return shaded.rgba.some((v, i) => i % 4 === 3 && v > 0 && plain.rgba[i] === 0);
  });
  expect(rendered).toBe(true);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('needle exposes zones, ticks, cap, centre parts and peak memory with Undo', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'needle');
  const inspector = page.locator('#inspector');
  await openSection(page, 'shape');
  await inspector.getByRole('group', { name: t('insp.needle.ticks'), exact: true }).getByRole('button', { name: t('insp.needle.ticks.major'), exact: true }).click();
  await change(number(page, t, 'insp.needle.length'), 75);
  await expect(number(page, t, 'insp.needle.length')).toHaveValue('75');
  await openSection(page, 'text');
  await inspector.getByRole('button', { name: t('insp.needle.center'), exact: true }).click();
  await expect(inspector.getByRole('group', { name: t('insp.valueParts'), exact: true })).toBeVisible();
  await expect(inspector.getByRole('button', { name: t('insp.tabularDigits'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await openSection(page, 'color');
  await inspector.getByRole('button', { name: t('insp.needle.addZone'), exact: true }).click();
  const from = inspector.getByRole('spinbutton', { name: t('insp.needle.zoneFrom', { number: 1 }), exact: true });
  await change(from, 25); await expect(from).toHaveValue('25');
  await page.locator('#undo').click(); await expect(from).toHaveValue('0');
  await openSection(page, 'behavior');
  await inspector.getByRole('button', { name: t('insp.needle.peak'), exact: true }).click();
  await change(number(page, t, 'insp.needle.peakDuration'), 15);
  await expect(number(page, t, 'insp.needle.peakDuration')).toHaveValue('15');
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('graph series and rounded bars combine, retaining the lower sensor across styles', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'graph');
  const pixelWidths = await page.evaluate(async () => {
    const { drawVisualization } = await import('./demo-visualization.js');
    const canvas = document.createElement('canvas'); canvas.width = 100; canvas.height = 80;
    const ctx = canvas.getContext('2d');
    const element = { frame: { x: 0, y: 0, width: 100, height: 80 }, kind: { type: 'graph', style: 'bars', color: '#ffffffff', lineWidth: 12 }, visualization: { style: 'rounded', roundedBars: false, gap: 4 } };
    return [null, 3, 11].map(width => {
      ctx.clearRect(0, 0, 100, 80); element.visualization.barWidth = width;
      drawVisualization(ctx, element, 0, [1, .5, 0, .75], []);
      const pixels = ctx.getImageData(0, 60, 100, 1).data;
      return Array.from({ length: 100 }, (_, x) => x).filter(x => pixels[x * 4 + 3] === 255);
    });
  });
  for (const [index, width] of [21, 3, 11].entries()) {
    expect(pixelWidths[index]).toEqual([0, 1, 3].flatMap(slot => Array.from({ length: width }, (_, offset) => slot * 25 + Math.ceil((25 - width) / 2) + offset)));
  }

  const inspector = page.locator('#inspector');
  await openSection(page, 'data');
  await inspector.getByRole('button', { name: t('insp.graphMirrored'), exact: true }).click();
  await expect(inspector.getByRole('combobox', { name: t('visual.secondary'), exact: true })).toBeVisible();
  await openSection(page, 'shape');
  await inspector.getByRole('button', { name: t('visual.rounded'), exact: true }).click();
  await expect(inspector.getByRole('button', { name: t('visual.rounded'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(inspector.getByRole('button', { name: t('insp.graphMirrored'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await change(number(page, t, 'insp.thickness'), 3);
  await expect(number(page, t, 'insp.thickness')).toHaveValue('3');
  await inspector.getByRole('button', { name: t('insp.graphAutoWidth'), exact: true }).click();
  await expect(inspector.getByRole('button', { name: t('insp.graphAutoWidth'), exact: true })).toHaveCount(0);
  await page.locator('#undo').click();
  await expect(number(page, t, 'insp.thickness')).toHaveValue('3');
  await change(number(page, t, 'insp.graphSeriesGap'), 12);
  await expect(number(page, t, 'insp.graphSeriesGap')).toHaveValue('12');
  await inspector.getByRole('button', { name: t('graph.line'), exact: true }).click();
  await expect(inspector.getByRole('button', { name: t('insp.graphMirrored'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(inspector.locator('[data-inspector-section="behavior"]')).toHaveCount(0);
  await page.locator('#save').click();
  await openSection(page, 'layout');
  await inspector.getByRole('button', { name: t('insp.sensorMotion'), exact: true }).click();
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));
  const rotate = inspector.getByRole('button', { name: t('insp.sensorMotionEnabled'), exact: true });
  await rotate.click(); await expect(rotate).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#undo').click(); await expect(rotate).toHaveAttribute('aria-pressed', 'false');
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('multi selection edits only common properties, shows mixed values and undoes once', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await page.locator('#grid-toggle').focus(); await page.keyboard.press('Control+a'); await page.keyboard.press('Delete');
  await clickAddWidget(page, t, 'ring');
  await clickAddWidget(page, t, 'bar');
  await openSection(page, 'data');
  await page.locator('#inspector').getByRole('combobox', { name: t('inspector.sensor'), exact: true }).click();
  await page.locator('.sensor-choice[data-key="gpu.usage"]').click();
  await clickAddWidget(page, t, 'value');
  await page.locator('#grid-toggle').focus(); await page.keyboard.press('Control+a');
  const inspector = page.locator('#inspector');
  await openSection(page, 'layout'); await openSection(page, 'data');
  await expect(inspector.getByRole('heading', { name: t('inspector.multi', { count: 3 }), exact: true })).toBeVisible();
  const width = number(page, t, 'inspector.width');
  await expect(width).toHaveValue(''); await expect(width).toHaveAttribute('placeholder', t('insp.mixed'));
  await expect(inspector).toContainText(t('insp.mixedSensors', { count: 2 }));
  await change(width, 240); await expect(width).toHaveValue('240');
  await page.locator('#undo').click(); await expect(width).toHaveValue('');
  const paint = inspector.getByRole('button', { name: t('insp.mainColor'), exact: true });
  await expect(paint).toContainText(t('insp.mixed'));
  await paint.click();
  const hex = page.getByRole('textbox', { name: t('colorPicker.hex'), exact: true });
  await change(hex, '#112233'); await page.keyboard.press('Escape');
  await expect(paint).toContainText('#112233');
  await page.locator('#undo').click(); await expect(paint).toContainText(t('insp.mixed'));
  await expect.poll(() => inspector.locator('.multi-preview canvas').evaluateAll(canvases => canvases.filter(canvas => canvas.getContext('2d').getImageData(0,0,72,72).data.some((v,i) => i % 4 === 3 && v > 0)).length)).toBe(3);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('bar and ring gradients retain direction, presets and Undo', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  const inspector = page.locator('#inspector');
  for (const widget of ['bar', 'ring']) {
    await clickAddWidget(page, t, widget);
    await openSection(page, 'shape');
    await inspector.getByRole('button', { name: t('inspector.segments'), exact: true }).click();
    await change(number(page, t, 'inspector.count'), 1000);
    await expect(number(page, t, 'inspector.count')).toHaveValue('200');
    await page.locator('#undo').click();
    await expect(number(page, t, 'inspector.count')).toHaveValue('10');
    await openSection(page, 'color');
    const gradient = inspector.getByRole('button', { name: t('insp.gradient'), exact: true });
    await gradient.click(); await expect(gradient).toHaveAttribute('aria-pressed', 'true');
    await expect(inspector.getByRole('button', { name: t('ring.endColor'), exact: true })).toBeVisible();
    await page.locator('#undo').click(); await expect(gradient).toHaveAttribute('aria-pressed', 'false');
    if (widget === 'bar') {
      const direction = inspector.getByRole('button', { name: t('dir.rightToLeft'), exact: true });
      await direction.click(); await expect(direction).toHaveAttribute('aria-pressed', 'true');
    }
    await expectAccessible(page);
  }
  expect(errors).toEqual([]);
});

test('multi Try uses isolated native previews, finishes and leaves the document saved', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.clock.install();
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await page.locator('#grid-toggle').focus(); await page.keyboard.press('Control+a'); await page.keyboard.press('Delete');
  await clickAddWidget(page, t, 'ring'); await clickAddWidget(page, t, 'bar');
  await page.locator('#grid-toggle').focus(); await page.keyboard.press('Control+a');
  await page.locator('#save').click();
  const button = page.locator('#inspector').getByRole('button', { name: t('insp.tryObject'), exact: true });
  await button.click(); await page.clock.runFor(1500);
  await expect(button).toHaveAttribute('aria-pressed', 'true');
  await page.clock.runFor(2000); await expect(button).toHaveAttribute('aria-pressed', 'false');
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));
  await page.emulateMedia({ reducedMotion: 'reduce' }); await expect(button).toBeDisabled();
  await expectAccessible(page); expect(errors).toEqual([]);
});
