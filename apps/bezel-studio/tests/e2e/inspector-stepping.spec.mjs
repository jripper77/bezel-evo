import { openSection, openInspectorTab, test, expect, watchErrors, expectAccessible } from './helpers.mjs';

// Stepping a number with its arrow keys edits the object at every step, and
// the inspector does not jump (D-2026-10-15-studio-redesign-fidelity-22):
// the same input keeps focus, the panel its scroll and the sections stays open.
const STEPS = 12;

async function holdArrowUp(page) {
  for (let i = 0; i < STEPS; i += 1) await page.keyboard.down('ArrowUp');
  await page.keyboard.up('ArrowUp');
}

test('holding ArrowUp on a number field keeps its focus, the scroll and the sections', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await page.locator('#tab-widgets').click();
  await page.getByRole('button', { name: t('library.addWidget', { name: t('widget.ring') }), exact: true }).click();
  const inspector = page.locator('#inspector');
  for (const id of ['layout', 'data', 'shape', 'color', 'behavior']) await openSection(page, id);
  const look = inspector.locator('[data-inspector-section="color"] > .accordion-heading > button');
  // At the bottom of the panel, where a shorter form would pull the scroll up.
  await inspector.evaluate((node) => { node.scrollTop = node.scrollHeight; });
  const scrollTop = await inspector.evaluate((node) => node.scrollTop);
  expect(scrollTop).toBeGreaterThan(0);

  for (const field of [inspector.getByRole('spinbutton', { name: t('inspector.sweep'), exact: true }), inspector.getByRole('spinbutton', { name: t('inspector.x'), exact: true })]) {
    await field.focus();
    const name = await field.getAttribute('aria-label');
    const start = Number(await field.inputValue());
    await field.evaluate((node) => { window.steppedInput = node; });
    const before = await inspector.evaluate((node) => node.scrollTop);
    await holdArrowUp(page);
    const stepped = inspector.getByRole('spinbutton', { name, exact: true });
    await expect(stepped).toHaveValue(String(Math.min(start + STEPS, Number(await stepped.getAttribute('max') ?? Infinity))));
    await expect(stepped).toBeFocused();
    expect(await stepped.evaluate((node) => node === window.steppedInput && node.isConnected)).toBe(true);
    expect(await inspector.evaluate((node) => node.scrollTop)).toBe(before);
    await expect(look).toHaveAttribute('aria-expanded', 'true');
  }
  // Each step was an edit: Undo takes back the last one.
  const x = inspector.getByRole('spinbutton', { name: t('inspector.x'), exact: true });
  const last = Number(await x.inputValue());
  await page.locator('#undo').click();
  await expect(x).toHaveValue(String(last - 1));
  await expectAccessible(page);
  expect(errors).toEqual([]);
});
