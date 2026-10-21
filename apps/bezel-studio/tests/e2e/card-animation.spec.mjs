import { test, expect, watchErrors, expectAccessible, openSection } from './helpers.mjs';

test('flip shades its source once per frame and projects distinct face pictures', async ({ page }) => {
  await page.goto('/index.html?demo=turing88');
  const frames = await page.evaluate(async () => {
    const [{ createStore }, { DEMO_THEME }, { renderApprox }] = await Promise.all([
      import('/editor/store.js'), import('/demo-theme.js'), import('/demo-render.js'),
    ]);
    const store = createStore({ ...DEMO_THEME, elements: [] });
    store.dispatch('cardDemo', { x: 240, y: 480 });
    const id = store.getState().theme.elements.find(e => e.card).id;
    store.dispatch('update', { id, patch: { card: { transition: { effect: 'flip', direction: 'right' } } } });
    const theme = store.getState().theme;
    const settings = theme.elements.find(e => e.id === id).card.transition;
    const prototype = Object.getPrototypeOf(new OffscreenCanvas(1, 1).getContext('2d'));
    const draw = prototype.drawImage;
    let filtered = 0, strips = 0;
    prototype.drawImage = function (...args) {
      if (this.filter !== 'none') filtered++;
      if (args.length === 9) strips++;
      return draw.apply(this, args);
    };
    try {
      return [0.25, 0.75].map(progress => {
        filtered = strips = 0;
        const frame = renderApprox(theme, 0, null, new Map([[id, { from: 0, to: 1, progress, settings }]]));
        let hash = 2166136261;
        for (const byte of frame.rgba) hash = Math.imul(hash ^ byte, 16777619);
        return { filtered, strips, hash };
      });
    } finally { prototype.drawImage = draw; }
  });
  expect(frames.map(frame => frame.filtered)).toEqual([1, 1]);
  for (const frame of frames) expect(frame.strips).toBeGreaterThan(20);
  expect(frames[0].hash).not.toBe(frames[1].hash);
  await expectAccessible(page);
});

test('card built in the editor previews fade, slide and flip with changing frames and reduced motion', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await expect(page.getByRole('button', { name: t('card.demo'), exact: true })).toHaveCount(0);
  const detail = page.locator('#inspector');
  const add = async widget => {
    await page.locator('#tab-widgets').click();
    await page.getByRole('button', { name: t('library.addWidget', { name: t(`widget.${widget}`) }), exact: true }).click();
  };
  await add('card');
  const cardName = await detail.getByLabel(t('common.name'), { exact: true }).inputValue();
  const selectCard = async () => {
    await page.locator('#tab-layers').click();
    await page.locator('.layer-name').filter({ hasText: cardName }).first().click();
  };
  await add('text');
  await selectCard();
  await detail.getByRole('button', { name: t('card.addFace'), exact: true }).click();
  await add('ring');
  await selectCard();
  await openSection(page, 'behavior');
  const effects = detail.getByRole('group', { name: t('card.effect'), exact: true });
  const effect = (name) => effects.getByRole('button', { name: t(`card.effect.${name}`), exact: true });
  await effect('flip').click();
  await expect(effect('flip')).toHaveAttribute('aria-pressed', 'true');
  const duration = detail.getByRole('spinbutton', { name: t('card.duration'), exact: true });
  await duration.fill('1000'); await duration.press('Tab');
  await detail.getByRole('group', { name: t('common.direction'), exact: true }).getByRole('button', { name: t('common.right'), exact: true }).click();
  for (const name of ['fade', 'slide', 'flip']) {
    await effects.getByRole('button', { name: t(`card.effect.${name}`), exact: true }).click();
    await page.waitForTimeout(150);
    const sampling = page.evaluate(async () => {
      // ▶ Try plays the next face in the inspector's own preview.
      const source = document.querySelector('#inspector .inspector-preview canvas');
      const sample = document.createElement('canvas'); sample.width = 48; sample.height = 192;
      const ctx = sample.getContext('2d', { willReadFrequently: true }), hashes = [];
      for (let i = 0; i < 10; i++) {
        ctx.drawImage(source, 0, 0, 48, 192);
        const rgba = ctx.getImageData(0, 0, 48, 192).data;
        let hash = 2166136261; for (let j = 0; j < rgba.length; j++) hash = Math.imul(hash ^ rgba[j], 16777619);
        hashes.push(hash); await new Promise(resolve => setTimeout(resolve, 140));
      }
      return hashes;
    });
    await detail.getByRole('button', { name: t('insp.tryObject'), exact: true }).click();
    const hashes = await sampling;
    expect(new Set(hashes).size, name).toBeGreaterThanOrEqual(4);
    await expect(duration).toHaveValue('1000');
  }
  const shared = detail.getByRole('button', { name: t('cardInsp.animateShared'), exact: true });
  await shared.click();
  await expect(shared).toHaveAttribute('aria-pressed', 'false');
  // With reduced motion the trial does not play.
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await expect(detail.getByRole('button', { name: t('insp.tryObject'), exact: true })).toBeDisabled();
  await expectAccessible(page);
  expect(errors).toEqual([]);
  await page.screenshot({ path: test.info().outputPath('card-animation.png') });
});
