import { test, expect, watchErrors, expectAccessible, clickAddWidget, openInspectorTab, choose, chosen } from './helpers.mjs';

test('demo pixels and fit measurement use display case without rewriting content', async ({ page }) => {
  await page.goto('/index.html?demo=turing88');
  const matches = await page.evaluate(async () => {
    const { renderApprox, measureApprox } = await import('/demo-render.js');
    const { createWidget } = await import('/editor/widgets.js');
    const source = 'mÚSICA já TOCA';
    const made = createWidget('text', { width: 480, height: 100 });
    return ['upper','lower','sentence','title'].map((casing, i) => {
      const expected = ['MÚSICA JÁ TOCA','música já toca','Música já toca','Música Já Toca'][i];
      const element = { ...made, id: 1, visible: true, opacity: 1, frame: { x: 0, y: 0, width: 480, height: 100 },
        kind: { ...made.kind, content: { type: 'static', text: source }, style: { ...made.kind.style, size: 26, casing } } };
      const theme = { canvas: { width: 480, height: 100 }, background: { type: 'color', color: '#000000ff' }, refreshSeconds: 1, elements: [element] };
      const reference = structuredClone(theme);
      reference.elements[0].kind.content.text = expected;
      reference.elements[0].kind.style.casing = 'normal';
      const a = renderApprox(theme, 0).rgba;
      const b = renderApprox(reference, 0).rgba;
      return a.every((byte,index) => byte === b[index])
        && JSON.stringify(measureApprox(theme,1,0)) === JSON.stringify(measureApprox(reference,1,0))
        && theme.elements[0].kind.content.text === source;
    });
  });
  expect(matches).toEqual([true,true,true,true]);
});

test('letter case preserves source text and supports Undo across text widgets', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'text');
  const inspector = page.locator('#inspector');
  const source = inspector.getByRole('textbox', { name: t('common.text'), exact: true });
  await source.fill('è GIÀ una BELLA giornata');
  await source.press('Tab');
  await openInspectorTab(page, t, 'inspector.tab.look');
  const label = t('textCase.label');
  for (const mode of ['upper', 'lower', 'sentence', 'title']) {
    await choose(inspector, label, mode);
    await expect.poll(() => chosen(inspector, label)).toBe(mode);
  }
  await page.locator('#undo').click();
  await expect.poll(() => chosen(inspector, label)).toBe('sentence');
  await expectAccessible(page);
  await openInspectorTab(page, t, 'inspector.tab.data');
  await expect(inspector.getByRole('textbox', { name: t('common.text'), exact: true })).toHaveValue('è GIÀ una BELLA giornata');
  for (const widget of ['value', 'player', 'weather']) {
    await clickAddWidget(page, t, widget);
    await openInspectorTab(page, t, 'inspector.tab.look');
    await choose(inspector, t('textCase.label'), 'lower');
    await expect.poll(() => chosen(inspector, t('textCase.label')), widget).toBe('lower');
  }
  expect(errors).toEqual([]);
});
