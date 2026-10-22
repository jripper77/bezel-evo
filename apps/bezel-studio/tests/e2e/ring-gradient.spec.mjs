import { test, expect, watchErrors, expectAccessible, copyAndPaste, colorButton, setColor, openInspectorTab, toggleButton, setToggle, choose } from './helpers.mjs';

test('ring arc gradient, full-scale test, copy and Undo', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await page.getByRole('button', { name: t('library.addWidget', { name: t('widget.ring') }), exact: true }).click();
  const detail = page.locator('#inspector');
  await openInspectorTab(page, t, 'inspector.tab.look');
  const checkbox = toggleButton(detail, t('ring.testFull'));
  await expect(checkbox).toHaveAttribute('aria-pressed', 'false');
  await setToggle(checkbox, true);
  // The arc gradient is the ring's Gradient choice.
  await choose(detail, t('ring.fillMode'), 'gradient');
  const colorInput = (key) => colorButton(detail, t(key));
  await setColor(page, t, detail, t('ring.startColor'), '#0000ff');
  await setColor(page, t, detail, t('ring.endColor'), '#ff0000');
  await detail.getByRole('spinbutton', { name: t('ring.transition'), exact: true }).fill('25');
  await detail.getByRole('spinbutton', { name: t('ring.transition'), exact: true }).press('Tab');
  await copyAndPaste(page);
  await expect(checkbox).toHaveAttribute('aria-pressed', 'true');
  await expect(colorInput('ring.startColor')).toHaveAttribute('data-color', /^#0000ff/);
  await expect(colorInput('ring.endColor')).toHaveAttribute('data-color', /^#ff0000/);
  await expect(detail.getByRole('spinbutton', { name: t('ring.transition'), exact: true })).toHaveValue('25');
  await setToggle(checkbox, false);
  await page.locator('#undo').click();
  await expect(checkbox).toHaveAttribute('aria-pressed', 'true');
  await choose(detail, t('ring.fillMode'), 'solid');
  await expect(detail.getByRole('spinbutton', { name: t('ring.transition'), exact: true })).toHaveCount(0);
  await expect(checkbox).toHaveAttribute('aria-pressed', 'true');
  await expectAccessible(page);
  expect(errors).toEqual([]);
});
