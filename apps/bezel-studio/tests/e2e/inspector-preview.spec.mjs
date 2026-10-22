// The inspector of one object (artboards Main, GraphData/Look,
// PlayerData/Look): its head, its preview rendered alone by the backend
// (D-2026-10-09-studio-redesign-fidelity-8), the graph's scale and window,
// and the player's apps, all from the demo's real data.
import { openGraphPrecision, openSection, openInspectorTab, test, expect, watchErrors, expectAccessible, clickAddWidget as add, openInspectorTab as tab, setColor } from './helpers.mjs';
import { joinWords, sentenceParts } from '../../src/sentence.js';

/** Whether the preview's canvas has a pixel drawn (`any`) or a strong red one (`red`). */
const anyPixel = (preview, kind) => preview.evaluate((canvas, which) => {
  const d = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data;
  for (let i = 0; i < d.length; i += 4) {
    if (which === 'any' ? d[i + 3] > 0 : d[i + 3] > 200 && d[i] > 200 && d[i + 1] < 80 && d[i + 2] < 80) return true;
  }
  return false;
}, kind);

/** The alpha of the preview's pixel at the fraction `fx, fy` of its size. */
const alphaAt = (preview, fx, fy) => preview.evaluate((canvas, [x, y]) => {
  const px = Math.min(canvas.width - 1, Math.floor(canvas.width * x));
  const py = Math.min(canvas.height - 1, Math.floor(canvas.height * y));
  return canvas.getContext('2d').getImageData(px, py, 1, 1).data[3];
}, [fx, fy]);

test('visible preview content stays centred when the widget text aligns left or right', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await add(page, t, 'text');
  const inspector = page.locator('#inspector');
  // ValueInspectorOpen specifies a thumbnail well for text, unlike the ring.
  expect(await inspector.locator('.inspector-preview').evaluate(figure =>
    getComputedStyle(figure).backgroundColor)).not.toBe('rgba(0, 0, 0, 0)');
  await inspector.getByRole('textbox', { name: t('common.text'), exact: true }).fill('Hi');
  await inspector.getByRole('textbox', { name: t('common.text'), exact: true }).press('Tab');
  await tab(page, t, 'inspector.tab.look');
  const preview = inspector.locator('.inspector-preview canvas');
  const alignment = inspector.getByRole('group', { name: t('inspector.align'), exact: true });
  const bounds = () => preview.evaluate(canvas => {
    const pixels = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data;
    let left = canvas.width, right = -1, top = canvas.height, bottom = -1;
    for (let y = 0; y < canvas.height; y++) for (let x = 0; x < canvas.width; x++) {
      if (!pixels[(y * canvas.width + x) * 4 + 3]) continue;
      left = Math.min(left, x); right = Math.max(right, x); top = Math.min(top, y); bottom = Math.max(bottom, y);
    }
    const box = canvas.getBoundingClientRect(), stage = canvas.parentElement.getBoundingClientRect();
    return { left, right, error: Math.max(
      Math.abs(box.left + (left + right + 1) / 2 * box.width / canvas.width - (stage.left + stage.right) / 2),
      Math.abs(box.top + (top + bottom + 1) / 2 * box.height / canvas.height - (stage.top + stage.bottom) / 2),
    ) };
  });
  await alignment.getByRole('button', { name: t('common.left'), exact: true }).click();
  await expect.poll(async () => (await bounds()).right).toBeGreaterThan(0);
  await expect.poll(async () => (await bounds()).error).toBeLessThan(1);
  const left = (await bounds()).left;
  await alignment.getByRole('button', { name: t('common.right'), exact: true }).click();
  // The actual pixels retain the object's alignment; only their display is centred.
  await expect.poll(async () => (await bounds()).left).toBeGreaterThan(left + 10);
  await expect.poll(async () => (await bounds()).error).toBeLessThan(1);
  await expect(alignment.getByRole('button', { name: t('common.right'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('the preview draws the object alone: an object under it is absent, and its color follows the edit', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  // An opaque shape, then a ring over it, both in the middle of the screen.
  await add(page, t, 'shape');
  await add(page, t, 'ring');
  const inspector = page.locator('#inspector');
  const name = await inspector.getByLabel(t('common.name'), { exact: true }).inputValue();
  const preview = inspector.getByRole('img', { name: t('inspector.previewOf', { name }), exact: true });
  await expect(preview).toBeVisible();
  await expect.poll(() => anyPixel(preview, 'any')).toBe(true);
  // The shape fills the ring's middle on the screen; alone, the middle is empty.
  await expect.poll(() => alphaAt(preview, 0.5, 0.5)).toBe(0);
  await expect.poll(() => alphaAt(preview, 0.02, 0.02)).toBe(0);
  // RingInspectorOpen has no thumbnail tile behind the isolated ring.
  await expect(inspector.locator('.inspector-preview')).toHaveCSS('background-color', 'rgba(0, 0, 0, 0)');
  await expect(inspector.locator('.inspector-preview')).toHaveCSS('border-radius', '0px');
  expect(await anyPixel(preview, 'red')).toBe(false);

  await tab(page, t, 'inspector.tab.look');
  await setColor(page, t, inspector, t('inspector.fill'), '#ff0000');
  await expect.poll(() => anyPixel(preview, 'red')).toBe(true);
  await page.locator('#undo').click();
  await expect.poll(() => anyPixel(preview, 'red')).toBe(false);
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('the head names the object, its kind and layer; the eye hides it and the lock locks it', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await add(page, t, 'ring');
  const inspector = page.locator('#inspector');
  const head = inspector.locator('.object-header');
  const level = Number(await head.getAttribute('data-layer'));
  const total = Number(await head.getAttribute('data-total'));
  expect(level).toBe(total);
  await expect(head.locator('.inspector-sub')).toContainText('CPU');
  await page.locator('#tab-layers').click();
  await expect(page.locator('#layer-list .layer-row')).toHaveCount(Number(total));

  const name = inspector.getByRole('textbox', { name: t('common.name'), exact: true });
  await name.fill('Gauge 42');
  await name.press('Tab');
  await expect(page.locator('.layer-name').filter({ hasText: 'Gauge 42' })).toHaveCount(1);

  const menu = async () => {
    await inspector.getByRole('button', { name: t('insp.more'), exact: true }).click();
    return page.getByRole('menu');
  };
  await (await menu()).getByRole('menuitem', { name: t('common.hide'), exact: true }).click();
  await expect((await menu()).getByRole('menuitem', { name: t('common.show'), exact: true })).toBeVisible();
  await page.getByRole('menuitem', { name: t('common.show'), exact: true }).click();
  await (await menu()).getByRole('menuitem', { name: t('common.lock'), exact: true }).click();
  await expect((await menu()).getByRole('menuitem', { name: t('common.unlock'), exact: true })).toBeVisible();
  await page.keyboard.press('Escape');
  await page.locator('#undo').click();
  await expect((await menu()).getByRole('menuitem', { name: t('common.lock'), exact: true })).toBeVisible();
  await page.keyboard.press('Escape');
  await openSection(page, 'shape');
  // The source card shows the sensor's key and value now over its select.
  await expect(inspector.locator('.source-card .list-card-sub')).toHaveText('cpu.usage');
  await expect(inspector.locator('.source-card .list-card-value')).toHaveText(/%$/);
  await expect(inspector.getByRole('combobox', { name: t('inspector.sensor'), exact: true })).toHaveAttribute('data-sensor-key', 'cpu.usage');
  const arc = inspector.locator('.arc-preview .arc-fill');
  const beforeArc = await arc.getAttribute('d');
  const sweep = inspector.getByRole('spinbutton', { name: t('inspector.sweep'), exact: true });
  await sweep.fill('180');
  await sweep.press('Tab');
  await expect(sweep).toHaveValue('180');
  await expect(arc).toHaveAttribute('d', 'M15.03 48.97A24 24 0 0 1 48.97 15.03');
  await page.locator('#undo').click();
  await expect(arc).toHaveAttribute('d', beforeArc);
  // Both degree values fit beside their translated prefixes at default width.
  for (const label of ['inspector.startAngle', 'inspector.sweep']) {
    const field = inspector.getByRole('spinbutton', { name: t(label), exact: true });
    expect(await field.evaluate((input) => {
      const ctx = document.createElement('canvas').getContext('2d');
      ctx.font = getComputedStyle(input).font;
      return ctx.measureText(input.value).width <= input.clientWidth;
    })).toBe(true);
  }
  const x = inspector.getByRole('spinbutton', { name: t('inspector.x'), exact: true });
  await inspector.getByRole('button', { name: t('align.left'), exact: true }).click();
  await expect(x).toHaveValue('0');
  await (await menu()).getByRole('menuitem', { name: t('common.duplicate'), exact: true }).click();
  await expect(page.locator('#layer-list .layer-row')).toHaveCount(total + 1);
  await (await menu()).getByRole('menuitem', { name: t('common.delete'), exact: true }).click();
  await expect(page.locator('#layer-list .layer-row')).toHaveCount(total);
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('the graph precision reads as a sentence over a time ruler; it scales itself and shows its peak', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await add(page, t, 'graph');
  await openGraphPrecision(page);
  const inspector = page.locator('#inspector');
  // D-2026-10-15-studio-redesign-fidelity-17: "Show the last [1] [minute], with one point every [1] [second]",
  // in the locale's own word order, then a ruler from -window to now with a mark per point.
  const sentence = inspector.locator('.graph-window');
  const duration = sentence.getByRole('spinbutton', { name: t('insp.durationLabel'), exact: true });
  const every = sentence.getByRole('spinbutton', { name: t('insp.intervalLabel'), exact: true });
  const durationUnit = sentence.getByRole('combobox', { name: t('insp.durationUnit'), exact: true });
  const everyUnit = sentence.getByRole('combobox', { name: t('insp.intervalUnit'), exact: true });
  const words = (last) => joinWords(sentenceParts(t('insp.windowSentence')).map((part) => part.word ?? (part.slot === 'last' ? t(last) : part))).filter((part) => typeof part === 'string');
  const unitWords = (units, plural) => units.map((unit) => t(`insp.unitWord.${unit}.${plural}`));
  const ruler = inspector.locator('.graph-ruler');
  const points = ruler.locator('.graph-points');
  const ticks = ruler.locator('.graph-ruler-tick');
  const range = inspector.locator('.graph-range');
  const axis = inspector.locator('.inspector-preview-axis');
  const quick = (key, value) => inspector.getByRole('group', { name: t('insp.quickDurations'), exact: true }).getByRole('button', { name: t(key, { value }), exact: true });
  const everySecond = { min: t('unit.seconds', { value: 2 }), max: t('unit.hours', { value: 1 }) };

  await expect(sentence.locator('.sentence-word')).toHaveText(words('insp.lastOne'));
  await expect(duration).toHaveValue('1');
  await expect(durationUnit).toHaveValue('min');
  await expect(durationUnit.locator('option')).toHaveText(unitWords(['s', 'min', 'h'], 'one'));
  await expect(every).toHaveValue('1');
  await expect(everyUnit).toHaveValue('s');
  await expect(everyUnit.locator('option')).toHaveText(unitWords(['s', 'min'], 'one'));
  await expect(points).toHaveText(t('insp.points', { points: 60 }));
  await expect(ticks).toHaveCount(60);
  await expect(ruler).toContainText(t('insp.ago', { duration: t('unit.minutes', { value: 1 }) }));
  await expect(ruler).toContainText(t('insp.now'));
  await expect(range).toHaveText(t('insp.windowRange', everySecond));
  await expect(quick('unit.minutes', 1)).toHaveAttribute('aria-pressed', 'true');
  // The two unit pickers line up: as wide as each other.
  expect(await durationUnit.evaluate((n) => n.offsetWidth)).toBe(await everyUnit.evaluate((n) => n.offsetWidth));

  await quick('unit.minutes', 5).click();
  await expect(duration).toHaveValue('5');
  await expect(sentence.locator('.sentence-word')).toHaveText(words('insp.lastMany'));
  await expect(durationUnit.locator('option')).toHaveText(unitWords(['s', 'min', 'h'], 'other'));
  await expect(points).toHaveText(t('insp.points', { points: 300 }));
  // 300 points: one mark in five.
  await expect(ticks).toHaveCount(60);
  await expect(quick('unit.minutes', 5)).toHaveAttribute('aria-pressed', 'true');
  await expect(axis).toContainText(t('insp.ago', { duration: t('unit.minutes', { value: 5 }) }));

  // One hour, one point every 30 s: 120 points.
  await every.fill('30');
  await every.press('Tab');
  const every30 = { min: t('unit.minutes', { value: 1 }), max: t('unit.hours', { value: 30 }) };
  await expect(points).toHaveText(t('insp.points', { points: 10 }));
  await expect(ticks).toHaveCount(10);
  await expect(range).toHaveText(t('insp.windowRange', every30));
  await expect(everyUnit.locator('option')).toHaveText(unitWords(['s', 'min'], 'other'));
  await expect(duration).toHaveValue('5');
  await duration.fill('1');
  await duration.press('Tab');
  await durationUnit.selectOption('h');
  await expect(duration).toHaveValue('1');
  await expect(durationUnit).toHaveValue('h');
  await expect(sentence.locator('.sentence-word')).toHaveText(words('insp.lastHour'));
  await expect(points).toHaveText(t('insp.points', { points: 120 }));
  await expect(ticks).toHaveCount(60);
  await expect(ruler).toContainText(t('insp.ago', { duration: t('unit.hours', { value: 1 }) }));
  await expect(quick('unit.hours', 1)).toHaveAttribute('aria-pressed', 'true');
  await expect(quick('unit.seconds', 30)).toBeDisabled();
  await expect(axis).toContainText(t('insp.ago', { duration: t('unit.hours', { value: 1 }) }));

  // The unit keeps the number: 30 minutes between two points of an hour make 2 points.
  await everyUnit.selectOption('min');
  await expect(every).toHaveValue('30');
  await expect(everyUnit).toHaveValue('min');
  await expect(points).toHaveText(t('insp.points', { points: 2 }));
  await expect(ticks).toHaveCount(2);
  await everyUnit.selectOption('s');
  await expect(points).toHaveText(t('insp.points', { points: 120 }));

  // A point every second cannot cover two hours: the window stops at 3600 points.
  await every.fill('1');
  await every.press('Tab');
  await duration.fill('2');
  await duration.press('Tab');
  await expect(points).toHaveText(t('insp.points', { points: 3600 }));
  await expect(ticks).toHaveCount(60);
  await expect(range).toHaveText(t('insp.windowRange', everySecond));
  await expect(duration).toHaveValue('1');
  await expect(axis).toContainText(t('insp.ago', { duration: t('unit.hours', { value: 1 }) }));

  await expect(inspector.locator('.graph-peak')).toHaveCount(0);
  await inspector.getByRole('button', { name: t('insp.automatic'), exact: true }).click();
  await expect(inspector.getByRole('button', { name: t('insp.automatic'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(inspector).toContainText(t('insp.peak'));
  await expect(inspector.locator('.graph-peak')).toHaveText(/%$/);
  await expect(inspector.locator('.inspector-preview-value')).toHaveText(/%$/);

  await tab(page, t, 'inspector.tab.look');
  const styles = inspector.getByRole('group', { name: t('inspector.style'), exact: true });
  await expect(styles.getByRole('button', { name: t('graph.area'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(inspector.getByRole('spinbutton', { name: t('insp.areaFill'), exact: true })).toBeVisible();
  await styles.getByRole('button', { name: t('graph.line'), exact: true }).click();
  await expect(styles.getByRole('button', { name: t('graph.line'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(inspector.getByRole('spinbutton', { name: t('insp.areaFill'), exact: true })).toHaveCount(0);
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('the player lists the apps with a media session, the playing one marked, and what shows when nothing plays', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await add(page, t, 'player');
  const inspector = page.locator('#inspector');
  const apps = inspector.getByRole('group', { name: t('insp.apps'), exact: true });
  await expect(apps.getByRole('button', { name: t('insp.automatic'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  // The demo's Spotify plays, its Chrome is paused.
  const spotify = apps.getByRole('button', { name: `Spotify, ${t('insp.playing')}`, exact: true });
  await expect(spotify.locator('.pill-dot')).toHaveAttribute('data-playing', 'true');
  await expect(apps.getByRole('button', { name: 'chrome', exact: true }).locator('.pill-dot')).toHaveAttribute('data-playing', 'false');
  await spotify.click();
  await expect(apps.getByRole('button', { name: `Spotify, ${t('insp.playing')}`, exact: true })).toHaveAttribute('aria-pressed', 'true');
  const source = inspector.getByRole('textbox', { name: t('player.source'), exact: true });
  await expect(source).toHaveCount(0);
  await apps.getByRole('button', { name: t('insp.otherApp'), exact: true }).click();
  await expect(source).toHaveValue('Spotify');
  await source.fill('vlc');
  await source.press('Tab');
  await expect(apps.getByRole('button', { name: t('insp.otherApp'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  // A named app closes the manual editor, including when the value was custom.
  await spotify.click();
  await expect(source).toHaveCount(0);
  await expect(apps.getByRole('button', { name: t('insp.otherApp'), exact: true })).toHaveAttribute('aria-pressed', 'false');
  await apps.getByRole('button', { name: t('insp.automatic'), exact: true }).click();
  await expect(apps.getByRole('button', { name: t('insp.automatic'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(source).toHaveCount(0);

  // What shows when nothing plays: two choices in Data, each one Undo step.
  const stopped = inspector.getByRole('group', { name: t('insp.stopped'), exact: true });
  const choice = (key) => stopped.getByRole('button', { name: t(key), exact: true });
  await expect(choice('insp.stoppedText')).toHaveAttribute('aria-pressed', 'true');
  await expect(inspector.getByRole('textbox', { name: t('player.empty'), exact: true })).toBeVisible();
  await choice('insp.stoppedHide').click();
  await expect(choice('insp.stoppedHide')).toHaveAttribute('aria-pressed', 'true');
  await expect(choice('insp.stoppedHide')).toBeFocused();
  await expect(inspector.getByRole('textbox', { name: t('player.empty'), exact: true })).toHaveCount(0);
  await choice('insp.stoppedText').click();
  await expect(choice('insp.stoppedText')).toHaveAttribute('aria-pressed', 'true');
  await choice('insp.stoppedHide').click();
  await expect(choice('insp.stoppedHide')).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#undo').click();
  await expect(choice('insp.stoppedText')).toHaveAttribute('aria-pressed', 'true');

  await tab(page, t, 'inspector.tab.look');
  const gap = inspector.getByRole('spinbutton', { name: t('player.coverGap'), exact: true });
  await gap.fill('14');
  await gap.press('Tab');
  await expect(gap).toHaveValue('14');
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('late isolated frames are discarded and Player chips follow newly available activity', async ({ page, t, lang }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  // A second inspector with the real demo backend: only IPC delivery is
  // delayed, so a selection change can overtake its first rendered answer.
  await page.evaluate(async (locale) => {
    const [{ createInspector }, { createStore }, { createDemoBackend }, { translator }] = await Promise.all([
      import('/ui/inspector.js'), import('/editor/store.js'), import('/demo-backend.js'), import('/i18n/index.js'),
    ]);
    const backend = createDemoBackend('turing88');
    const store = createStore((await backend.session()).theme);
    const catalog = await backend.catalog();
    const actual = await backend.sample();
    let sample = { ...actual, activity: null };
    let release;
    let calls = 0;
    const root = document.createElement('aside');
    root.id = 'inspector-probe';
    root.className = 'inspector';
    Object.assign(root.style, { position: 'fixed', inset: '52px 0 30px auto', width: '360px', zIndex: '100' });
    document.body.append(root);
    const inspector = createInspector({ root, store, t: translator(locale), minRefresh: () => 1,
      sensors: { catalog: () => catalog },
      host: {
        lastSample: () => sample,
        renderElement: async (id, opts) => {
          calls += 1;
          const frame = await backend.renderElement(store.getState().theme, id, opts);
          if (calls === 1) await new Promise((resolve) => { release = resolve; });
          return frame;
        },
      },
    });
    store.subscribe(() => inspector.render());
    store.dispatch('add', { widget: 'ring', x: 200, y: 400 });
    globalThis.inspectorProbe = { store, inspector, release: () => release(), calls: () => calls,
      activity: (available) => { sample = { ...actual, activity: available ? actual.activity : null }; inspector.frameShown(); } };
  }, lang);
  const probe = page.locator('#inspector-probe');
  await expect.poll(() => page.evaluate(() => inspectorProbe.calls())).toBe(1);
  await page.evaluate(() => inspectorProbe.store.dispatch('add', { widget: 'shape', x: 200, y: 400 }));
  expect(await page.evaluate(() => inspectorProbe.calls())).toBe(1);
  await page.evaluate(() => inspectorProbe.release());
  const canvas = probe.getByRole('img');
  await expect.poll(() => page.evaluate(() => inspectorProbe.calls())).toBe(2);
  await expect.poll(() => alphaAt(canvas, 0.5, 0.5)).toBe(255);
  await expect(canvas).toHaveAttribute('aria-label', t('inspector.previewOf', { name: await probe.getByLabel(t('common.name'), { exact: true }).inputValue() }));

  await page.evaluate(() => inspectorProbe.store.dispatch('add', { widget: 'player', x: 200, y: 400 }));
  await expect(probe.locator('.player-apps-hint')).toHaveText(t('insp.appsNone'));
  await page.evaluate(() => inspectorProbe.activity(true));
  const spotify = probe.getByRole('button', { name: `Spotify, ${t('insp.playing')}`, exact: true });
  await expect(spotify).toBeVisible();
  await expect(probe.locator('.player-apps-hint')).toHaveText(t('insp.appsHint'));
  await spotify.click();
  await spotify.focus();
  await page.evaluate(() => inspectorProbe.activity(false));
  await expect(probe.getByRole('textbox', { name: t('player.source'), exact: true })).toHaveValue('Spotify');
  await expect(probe.locator('.player-apps-hint')).toHaveText(t('insp.appsNone'));
  await page.evaluate(() => inspectorProbe.activity(true));
  await expect(spotify).toHaveAttribute('aria-pressed', 'true');
  await expect(probe.getByRole('textbox', { name: t('player.source'), exact: true })).toHaveCount(0);
  await expectAccessible(page);
  expect(errors).toEqual([]);
});
