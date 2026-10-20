import { test, expect, expectAccessible, watchErrors } from './helpers.mjs';

// Exercise the shared primitive in the real WebView-sized browser, independent
// of the inspector migration. Its caller owns state; controls remain mounted.
async function mountSections(page, t) {
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await page.evaluate(async labels => {
    const { accordionSection, numberField } = await import('./ui/fields.js');
    const { createSectionMemory } = await import('./inspector-sections.js');
    const memory = createSectionMemory({ storage: null });
    const host = document.createElement('div');
    host.id = 'primitive-fixture';
    const sections = [];
    for (const [id, title, summary] of labels) {
      const section = accordionSection({
        id, title, summary, expanded: memory.isOpen('ring', id),
        onToggle: options => {
          memory.toggle('ring', id, options);
          for (const [key, control] of sections) control.setExpanded(memory.isOpen('ring', key));
        },
        render: body => {
          body.append(numberField(`${title} px`, 24, value => { body.dataset.committed = String(value); }));
        },
      });
      sections.push([id, section]);
      host.append(section.element);
    }
    document.querySelector('#inspector').replaceChildren(host);
  }, [
    ['layout', t('inspector.position'), '90, 300 · 300 × 300'],
    ['data', t('inspector.tab.data'), 'cpu.usage'],
    ['shape', t('common.shape'), '−135° · 270° · 24 px'],
  ]);
  return page.locator('#primitive-fixture');
}

test('accordion titles expose independent state, summaries, Alt+click and retained fields', async ({ page, t }) => {
  const errors = watchErrors(page);
  const fixture = await mountSections(page, t);
  const data = fixture.locator('[data-section="data"]');
  const shape = fixture.locator('[data-section="shape"]');
  const toggle = shape.getByRole('button', { name: t('common.shape'), exact: true });
  await expect(data.locator('.accordion-toggle')).toHaveAttribute('aria-expanded', 'true');
  await expect(shape.locator('.accordion-summary')).toHaveText('−135° · 270° · 24 px');
  await expect(shape.locator('.accordion-summary')).toBeVisible();
  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await expect(shape.locator('.accordion-summary')).toBeHidden();
  await expect(data.getByRole('spinbutton')).toBeVisible();
  const input = shape.getByRole('spinbutton');
  await input.fill('31');
  await input.press('Tab');
  await expect(shape.locator('.accordion-body')).toHaveAttribute('data-committed', '31');
  await toggle.click();
  await expect(shape.getByRole('region', { includeHidden: true })).toBeHidden();
  await toggle.click({ modifiers: ['Alt'] });
  await expect(input).toHaveValue('31');
  await expect(data.locator('.accordion-toggle')).toHaveAttribute('aria-expanded', 'false');
  // Exclusive opening an already open section leaves it open.
  await toggle.click({ modifiers: ['Alt'] });
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  const box = await toggle.boundingBox();
  expect(box.height).toBe(48);
  expect(await fixture.evaluate(root => root.scrollWidth <= root.clientWidth)).toBe(true);
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('accordion native buttons support Space/Enter and arrows visit titles without toggling', async ({ page, t }) => {
  const errors = watchErrors(page);
  const fixture = await mountSections(page, t);
  const titles = fixture.locator('.accordion-toggle');
  await titles.first().focus();
  await page.keyboard.press('Space');
  await expect(titles.first()).toHaveAttribute('aria-expanded', 'true');
  await page.keyboard.press('Enter');
  await expect(titles.first()).toHaveAttribute('aria-expanded', 'false');
  await page.keyboard.press('ArrowUp');
  await expect(titles.last()).toBeFocused();
  await expect(titles.last()).toHaveAttribute('aria-expanded', 'false');
  await page.keyboard.press('ArrowDown');
  await expect(titles.first()).toBeFocused();
  await page.keyboard.press('End');
  await expect(titles.last()).toBeFocused();
  await page.keyboard.press('Home');
  await expect(titles.first()).toBeFocused();
  await expectAccessible(page);
  expect(errors).toEqual([]);
});
