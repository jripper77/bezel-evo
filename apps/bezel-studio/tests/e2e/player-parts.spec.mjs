// The player's parts and progress bar (D-2026-10-16-studio-redesign-fidelity-26):
// title, artist and time turn on and off like cover, progress and app
// name, the fit measures only what shows, and the bar's look (colours,
// thickness, ends, knob, pattern, where the times go) is set in Look.
import { test, expect, watchErrors, expectAccessible, clickAddWidget as add, openInspectorTab as tab, colorButton, setColor, toggleButton } from './helpers.mjs';

/** Whether the preview has a strong red pixel. */
const hasRed = (preview) => preview.evaluate((canvas) => {
  const d = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data;
  for (let i = 0; i < d.length; i += 4) if (d[i + 3] > 200 && d[i] > 200 && d[i + 1] < 80 && d[i + 2] < 80) return true;
  return false;
});

async function addPlayer(page, t) {
  await page.goto('/index.html?demo=turing88');
  await add(page, t, 'player');
  const inspector = page.locator('#inspector');
  await tab(page, t, 'inspector.tab.look');
  const show = inspector.getByRole('group', { name: t('insp.show'), exact: true });
  return { inspector, part: (key) => show.getByRole('button', { name: t(key), exact: true }) };
}

test('each part of the player turns on and off alone and the fit measures only what shows', async ({ page, t }) => {
  const errors = watchErrors(page);
  const { inspector, part } = await addPlayer(page, t);
  const parts = ['insp.cover', 'insp.partTitle', 'insp.partArtist', 'insp.partTime', 'insp.progress', 'insp.appName'];
  const pressed = [true, true, true, true, true, false];
  for (const [i, key] of parts.entries()) await expect(part(key)).toHaveAttribute('aria-pressed', String(pressed[i]));

  // The bar off: the times stay (they no longer follow it).
  await part('insp.progress').click();
  await expect(part('insp.progress')).toHaveAttribute('aria-pressed', 'false');
  await expect(part('insp.partTime')).toHaveAttribute('aria-pressed', 'true');
  await expect(inspector.locator('[data-section="progress"]')).toHaveCount(0);

  // Only the title: one line of text, no cover.
  for (const key of ['insp.cover', 'insp.partArtist', 'insp.partTime']) await part(key).click();
  for (const key of ['insp.cover', 'insp.partArtist', 'insp.partTime']) await expect(part(key)).toHaveAttribute('aria-pressed', 'false');
  await expect(inspector.locator('[data-section="corners"]')).toHaveCount(0);
  const size = Number(await inspector.getByRole('spinbutton', { name: t('inspector.size'), exact: true }).inputValue());
  const height = inspector.getByRole('spinbutton', { name: t('inspector.height'), exact: true });
  await inspector.getByRole('button', { name: t('insp.fitContent'), exact: true }).click();
  await expect(height).toHaveValue(String(Math.ceil(size * 1.2)));
  // The artist too: two lines.
  await part('insp.partArtist').click();
  await inspector.getByRole('button', { name: t('insp.fitContent'), exact: true }).click();
  await expect(height).toHaveValue(String(2 * Math.ceil(size * 1.2)));

  // Keyboard: Space toggles a tile; one Undo per change.
  await part('insp.partTitle').focus();
  await page.keyboard.press('Space');
  await expect(part('insp.partTitle')).toHaveAttribute('aria-pressed', 'false');
  await page.locator('#undo').click();
  await expect(part('insp.partTitle')).toHaveAttribute('aria-pressed', 'true');

  // The time format is data: shown only while the time is on.
  await tab(page, t, 'inspector.tab.data');
  await expect(inspector.getByRole('combobox', { name: t('insp.timeFormat'), exact: true })).toHaveCount(0);
  await tab(page, t, 'inspector.tab.look');
  await part('insp.partTime').click();
  await tab(page, t, 'inspector.tab.data');
  const format = inspector.getByRole('combobox', { name: t('insp.timeFormat'), exact: true });
  await expect(format).toHaveValue('elapsedTotal');
  await format.selectOption('remaining');
  await expect(format).toHaveValue('remaining');
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('the progress bar has its colours, thickness, ends, knob, pattern and times placement', async ({ page, t }) => {
  const errors = watchErrors(page);
  const { inspector, part } = await addPlayer(page, t);
  const section = inspector.locator('[data-section="progress"]');
  await expect(section.getByRole('heading', { name: t('insp.progressBar'), exact: true })).toBeVisible();

  // Fill colour: the preview turns red.
  const name = await inspector.getByLabel(t('common.name'), { exact: true }).inputValue();
  const preview = inspector.getByRole('img', { name: t('inspector.previewOf', { name }), exact: true });
  await setColor(page, t, section, t('inspector.fill'), '#ff0000');
  await expect.poll(() => hasRed(preview)).toBe(true);
  await expect(colorButton(section, t('inspector.track'))).toHaveAttribute('data-color', /^#646464/);

  // Thickness by keyboard, ends and knob as switches.
  const thickness = section.getByRole('slider', { name: t('insp.thickness'), exact: true });
  await expect(thickness).toHaveValue('4');
  await thickness.focus();
  await page.keyboard.press('ArrowRight');
  await expect(section.getByRole('slider', { name: t('insp.thickness'), exact: true })).toHaveValue('5');
  const round = toggleButton(section, t('inspector.roundCaps'));
  await expect(round).toHaveAttribute('aria-pressed', 'true');
  await round.click({ force: true });
  await expect(toggleButton(section, t('inspector.roundCaps'))).toHaveAttribute('aria-pressed', 'false');
  const knob = toggleButton(section, t('insp.progressKnob'));
  await expect(knob).toHaveAttribute('aria-pressed', 'false');
  await knob.click({ force: true });
  await expect(toggleButton(section, t('insp.progressKnob'))).toHaveAttribute('aria-pressed', 'true');

  // Pattern: continuous, segments (with a count) or dots.
  const style = section.getByRole('group', { name: t('inspector.style'), exact: true });
  const pattern = (key) => style.getByRole('button', { name: t(key), exact: true });
  await expect(pattern('insp.patternContinuous')).toHaveAttribute('aria-pressed', 'true');
  await expect(section.getByRole('spinbutton', { name: t('inspector.count'), exact: true })).toHaveCount(0);
  await pattern('insp.patternSegments').click();
  await expect(pattern('insp.patternSegments')).toHaveAttribute('aria-pressed', 'true');
  const count = section.getByRole('spinbutton', { name: t('inspector.count'), exact: true });
  await expect(count).toHaveValue('12');
  await count.fill('20');
  await count.press('Tab');
  await expect(section.getByRole('spinbutton', { name: t('inspector.count'), exact: true })).toHaveValue('20');
  await pattern('insp.patternDots').focus();
  await page.keyboard.press('Enter');
  await expect(pattern('insp.patternDots')).toHaveAttribute('aria-pressed', 'true');
  await expect(section.getByRole('spinbutton', { name: t('inspector.count'), exact: true })).toHaveCount(0);

  // Where the times go: only while the time shows.
  const place = section.getByRole('group', { name: t('insp.timesPlace'), exact: true });
  await expect(place.getByRole('button', { name: t('insp.timesText'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await place.getByRole('button', { name: t('insp.timesSides'), exact: true }).click();
  await expect(section.getByRole('group', { name: t('insp.timesPlace'), exact: true }).getByRole('button', { name: t('insp.timesSides'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  // A bar with its times at the sides alone: the fit is one row tall.
  for (const key of ['insp.cover', 'insp.partTitle', 'insp.partArtist']) await part(key).click();
  const size = Number(await inspector.getByRole('spinbutton', { name: t('inspector.size'), exact: true }).inputValue());
  await inspector.getByRole('button', { name: t('insp.fitContent'), exact: true }).click();
  await expect(inspector.getByRole('spinbutton', { name: t('inspector.height'), exact: true })).toHaveValue(String(Math.max(Math.ceil(size * 1.2), 11)));
  await part('insp.partTime').click();
  await expect(section.getByRole('group', { name: t('insp.timesPlace'), exact: true })).toHaveCount(0);

  // Undo walks back one change at a time.
  await page.locator('#undo').click();
  await expect(section.getByRole('group', { name: t('insp.timesPlace'), exact: true })).toBeVisible();
  await expectAccessible(page);
  expect(errors).toEqual([]);
});
