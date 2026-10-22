// Smooth value transitions (D-2026-10-15-studio-redesign-fidelity-18): the
// "Transition" control of a ring, a bar, a needle and a value, and the
// demo's frames while a value eases (counted by the demo backend on
// `data-demo-transition-frames`).
import { test, expect, watchErrors, expectAccessible, clickAddWidget, openSection } from './helpers.mjs';

const transition = (page, t) => page.locator('#inspector [data-section="transition"]');
const modeButton = (page, t, key) => transition(page, t).getByRole('group', { name: t('insp.onReading'), exact: true }).getByRole('button', { name: t(key), exact: true });
const durationSlider = (page, t) => transition(page, t).getByRole('spinbutton', { name: t('card.duration'), exact: true });

test('a new ring eases for 400 ms; the control turns it off, on and longer, with Undo', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await clickAddWidget(page, t, 'ring');
  // The transition is in Behavior.
  await openSection(page, 'behavior');
  await expect(transition(page, t)).toBeVisible();
  await expect(transition(page, t).getByRole('heading', { name: t('insp.onReading'), exact: true })).toBeVisible();
  await expect(modeButton(page, t, 'insp.transitionSmooth')).toHaveAttribute('aria-pressed', 'true');
  await expect(modeButton(page, t, 'insp.transitionJump')).toHaveAttribute('aria-pressed', 'false');
  await expect(durationSlider(page, t)).toHaveValue('400');
  await expect(transition(page, t).getByRole('group', { name: t('insp.onReading'), exact: true }).locator('button svg')).toHaveCount(3);

  await modeButton(page, t, 'insp.transitionJump').click();
  await expect(modeButton(page, t, 'insp.transitionJump')).toHaveAttribute('aria-pressed', 'true');
  await expect(durationSlider(page, t)).toHaveCount(0);
  await modeButton(page, t, 'insp.transitionSmooth').click();
  await expect(durationSlider(page, t)).toHaveValue('400', { timeout: 5000 });

  const slider = durationSlider(page, t);
  await slider.focus();
  await slider.press('ArrowUp');
  await expect(durationSlider(page, t)).toHaveValue('450');
  await page.locator('#undo').click();
  await expect(durationSlider(page, t)).toHaveValue('400');
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('bars, needles and values have the control; texts and graphs do not', async ({ page, t }) => {
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  for (const widget of ['bar', 'needle', 'value']) {
    await clickAddWidget(page, t, widget);
    await openSection(page, 'behavior');
    await expect(modeButton(page, t, 'insp.transitionSmooth'), widget).toHaveAttribute('aria-pressed', 'true');
  }
  for (const [widget, shown] of [['text', 'content'], ['graph', 'source']]) {
    await clickAddWidget(page, t, widget);
    await expect(page.locator(`#inspector [data-section="${shown}"]`), widget).toBeVisible();
    await expect(transition(page, t), widget).toHaveCount(0);
  }
});

/** Starts recording when each demo frame of a moving value is drawn. */
async function recordTransitionFrames(page) {
  await page.evaluate(() => {
    window.__transitionFrames = [];
    new MutationObserver(() => window.__transitionFrames.push(performance.now()))
      .observe(document.documentElement, { attributes: true, attributeFilter: ['data-demo-transition-frames'] });
  });
}

test('the canvas eases at 20 frames a second at most, and not with reduced motion', async ({ page, t }) => {
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await clickAddWidget(page, t, 'ring');
  await recordTransitionFrames(page);
  await page.waitForTimeout(3000);
  const frames = await page.evaluate(() => window.__transitionFrames);
  expect(frames.length).toBeGreaterThan(0);
  // Frames of one transition follow each other closely; the next sample
  // starts another a second later.
  const gaps = frames.slice(1).map((at, i) => at - frames[i]).filter((gap) => gap < 200);
  expect(gaps.length).toBeGreaterThan(0);
  const median = [...gaps].sort((a, b) => a - b)[Math.floor(gaps.length / 2)];
  const fps = 1000 / median;
  test.info().annotations.push({ type: 'fps', description: `${fps.toFixed(1)} frames/s while easing (${frames.length} frames in 3 s)` });
  expect(fps).toBeLessThanOrEqual(21);
  // At most one transition a second (one sample a second), 8 steps each.
  expect(frames.length).toBeLessThanOrEqual(3 * 9 + 9);

  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.waitForTimeout(300);
  await recordTransitionFrames(page);
  await page.waitForTimeout(2500);
  expect(await page.evaluate(() => window.__transitionFrames.length)).toBe(0);
});
