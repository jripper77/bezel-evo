import { test, expect, watchErrors, expectAccessible, setPanelWidths, clickAddWidget, openSection } from './helpers.mjs';

test('Theme metadata stays inside a narrow inspector and all three orientation commands undo once', async ({ page, t }) => {
  const errors = watchErrors(page); await setPanelWidths(page, { inspector: 320 });
  await page.goto('/index.html?demo=turing88'); await expect(page.locator('#theme-name')).toHaveValue('Demo'); const root = page.locator('#inspector');
  await expect(root.locator('.theme-facts .mono')).toHaveAttribute('title', /480.*1920/);
  await expect(root.locator('.theme-save-state .pill-dot')).toHaveCount(1);
  expect(await root.locator('.theme-size').evaluate(n => n.scrollWidth <= n.clientWidth + 1)).toBe(true);
  await expect(root.locator('.theme-size')).toContainText('480');
  expect(await root.evaluate(n => n.scrollWidth <= n.clientWidth + 1)).toBe(true);
  const group = root.getByRole('group', { name: t('inspector.orientation'), exact: true });
  await expect(group.getByRole('button')).toHaveCount(3);
  const turn = group.getByRole('button', { name: t('insp.theme.turn'), exact: true });
  const wasTurned = await turn.getAttribute('aria-pressed');
  await turn.click(); await expect(turn).toHaveAttribute('aria-pressed', String(wasTurned !== 'true'));
  await page.locator('#undo').click(); await expect(turn).toHaveAttribute('aria-pressed', wasTurned);
  expect(await root.evaluate(n => n.scrollWidth <= n.clientWidth + 1)).toBe(true);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('multi selection uses compact distribution and grouping commands with complete accessible names', async ({ page, t }) => {
  const errors = watchErrors(page); await setPanelWidths(page, { inspector: 320 });
  await page.goto('/index.html?demo=turing88');
  await page.locator('#grid-toggle').focus(); await page.keyboard.press('Control+a'); await page.keyboard.press('Delete');
  for (const kind of ['ring', 'bar', 'value']) await clickAddWidget(page, t, kind);
  await page.locator('#grid-toggle').focus(); await page.keyboard.press('Control+a');
  const root = page.locator('#inspector'); await openSection(page, 'layout');
  await expect(root.getByRole('heading', { name: t('inspector.multi', { count: 3 }), exact: true })).toHaveCSS('text-transform', 'none');
  const distribution = root.getByRole('group', { name: t('insp.distribute'), exact: true });
  const boxes = await distribution.getByRole('button').evaluateAll(nodes => nodes.map(n => n.getBoundingClientRect().y));
  expect(Math.max(...boxes) - Math.min(...boxes)).toBeLessThan(2);
  await expect(distribution.getByRole('button', { name: t('align.distributeX'), exact: true })).toBeEnabled();
  await root.getByRole('button', { name: t('group.create'), exact: true }).click();
  await expect(root.getByRole('button', { name: t('group.ungroup'), exact: true })).toHaveCount(0); // Selection is now the one group.
  await page.locator('#undo').click();
  await expect(page.locator('.layer-name')).toHaveCount(3);
  await page.locator('#grid-toggle').focus(); await page.keyboard.press('Control+a');
  await expect(root.getByRole('heading', { name: t('inspector.multi', { count: 3 }), exact: true })).toBeVisible();
  expect(await root.evaluate(n => n.scrollWidth <= n.clientWidth + 1)).toBe(true);
  await expectAccessible(page); expect(errors).toEqual([]);
});
