import { test, expect, clickAddWidget, openSection, expectAccessible, watchErrors, setPanelWidths } from './helpers.mjs';
const inspector = page => page.locator('#inspector');
async function change(field, value) { await field.fill(String(value)); await field.press('Tab'); }

test('Card has one transition and complete face names at narrow widths', async ({ page, t }) => {
  const errors = watchErrors(page);
  await setPanelWidths(page, { inspector: 280 });
  await page.goto('/index.html?demo=turing88');
  await clickAddWidget(page, t, 'card');
  await expect(inspector(page).locator(':scope > .accordion-section')).toHaveCount(5);
  await expect(inspector(page).locator('[data-section="animation"]')).toHaveCount(1);
  await expect(inspector(page).locator('[data-section="transition"]')).toHaveCount(0);
  const name = 'Metriche di sistema applicazioni';
  await change(inspector(page).getByRole('textbox', { name: t('card.faceTitle'), exact: true }), name);
  await expect(inspector(page).locator('.face-rows .face-name')).toHaveText(name);
  expect(await inspector(page).evaluate(node => node.getBoundingClientRect().width)).toBe(280);
  expect(await inspector(page).locator('.face-rows .face-name').evaluate(node => node.scrollWidth <= node.clientWidth && node.scrollHeight <= node.clientHeight)).toBe(true);
  await openSection(page, 'behavior');
  const effect = inspector(page).getByRole('group', { name: t('card.effect'), exact: true });
  await effect.getByRole('button', { name: t('card.effect.slide'), exact: true }).click();
  await change(inspector(page).getByRole('spinbutton', { name: t('card.duration'), exact: true }), 450);
  await expect(inspector(page).getByRole('spinbutton', { name: t('card.duration'), exact: true })).toHaveValue('450');
  await page.locator('#undo').click();
  await expect(effect.getByRole('button', { name: t('card.effect.slide'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('Theme has a preview, disclosures, orientation and saved status', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await expect(inspector(page).locator('.theme-preview')).toBeVisible();
  await expect(inspector(page).locator(':scope > .accordion-section')).toHaveCount(2);
  await inspector(page).getByRole('button', { name: t('axis.horizontal'), exact: true }).click();
  const turned = await inspector(page).getByRole('button', { name: t('insp.theme.turn'), exact: true }).getAttribute('aria-pressed');
  await inspector(page).getByRole('button', { name: t('insp.theme.turn'), exact: true }).click();
  await expect(inspector(page).getByRole('button', { name: t('insp.theme.turn'), exact: true })).toHaveAttribute('aria-pressed', String(turned !== 'true'));
  await expect(page.locator('#orient-turn')).toHaveAttribute('aria-pressed', String(turned !== 'true'));
  await change(inspector(page).getByRole('textbox', { name: t('top.themeName'), exact: true }), 'Inspector test');
  await expect(inspector(page).locator('.theme-save-state')).toHaveText(t('status.unsaved'));
  await openSection(page, 'behavior');
  await inspector(page).getByRole('button', { name: '2 s', exact: true }).click();
  await expect(inspector(page).getByRole('button', { name: '2 s', exact: true })).toHaveAttribute('aria-pressed', 'true');
  await inspector(page).getByRole('button', { name: t('insp.theme.actions'), exact: true }).click();
  await expect(page.getByRole('menuitem', { name: t('themes.saveAs'), exact: true })).toBeVisible();
  await page.keyboard.press('Escape');
  await openSection(page, 'color'); await inspector(page).locator('#bg-change').click();
  await expect(inspector(page).locator('#bg-chooser-back')).toBeVisible();
  await inspector(page).locator('#bg-chooser-back').click();
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('Screen menu confirms release and storage expands only on request', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await page.locator('#tab-screen').click();
  const header = page.locator('#panel-screen > .screen-header');
  await expect(header.locator('.screen-card')).toBeVisible();
  const autostart = page.locator('#screen-panel').getByRole('button', { name: t('screenPanel.autostart'), exact: true });
  const wasAutostart = await autostart.getAttribute('aria-pressed');
  await autostart.click();
  await expect(autostart).toHaveAttribute('aria-pressed', String(wasAutostart !== 'true'));
  await autostart.press('Space');
  await expect(autostart).toHaveAttribute('aria-pressed', wasAutostart);
  const brightness = page.locator('#screen-panel').getByRole('spinbutton', { name: t('screen.brightness'), exact: true });
  await change(brightness, 65);
  await expect(page.locator('#screen-panel').getByRole('slider', { name: t('screen.brightness'), exact: true })).toHaveValue('65');
  await header.getByRole('button', { name: t('screenPanel.actions'), exact: true }).click();
  await page.getByRole('menuitem', { name: t('screen.release'), exact: true }).click();
  await expect(page.getByRole('dialog')).toContainText(t('screenPanel.releaseBody'));
  await page.getByRole('dialog').getByRole('button', { name: t('dialog.cancel'), exact: true }).click();
  await header.getByRole('button', { name: t('screenPanel.actions'), exact: true }).click();
  await page.getByRole('menuitem', { name: t('screen.restart'), exact: true }).click();
  await expect(page.getByRole('dialog')).toContainText(t('restart.confirmBody'));
  await page.getByRole('dialog').getByRole('button', { name: t('dialog.cancel'), exact: true }).click();
  await page.locator('#subtab-storage').click();
  await expect(page.locator('#storage-panel [data-inspector-section]')).toHaveCount(4);
  await expect(page.locator('.workspace')).not.toHaveClass(/storage-wide/);
  await expect(header.locator('.screen-card')).toBeVisible();
  const fileMenu = page.locator('#storage-panel .file-menu').first();
  const list = page.locator('#storage-panel [role="listbox"]').first();
  await list.focus(); await expect(list).toBeFocused();
  await list.press('Home'); await list.press('Space');
  await list.press('F2');
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.getByRole('dialog').getByRole('button', { name: t('dialog.cancel'), exact: true }).click();
  await fileMenu.click();
  await expect(page.getByRole('menuitem', { name: t('storage.action.rename'), exact: true })).toBeVisible();
  await page.keyboard.press('Escape'); await expect(fileMenu).toBeFocused();
  await page.locator('#storage-panel .manager-filter-details > summary').click();
  await expect(page.locator('#storage-panel [data-focus="filter-kind"]')).toBeVisible();
  await page.locator('#storage-panel [data-focus="filter-kind"]').selectOption('video');
  await page.locator('#storage-panel .manager-filter-details > summary').click();
  await page.locator('#storage-panel [data-focus="tool-expand"]').click();
  await expect(page.locator('.workspace')).toHaveClass(/storage-wide/);
  await page.locator('#storage-panel [data-focus="tool-expand"]').click();
  await expect(page.locator('.workspace')).not.toHaveClass(/storage-wide/);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('Theme menu saves a named copy and leaves it marked as saved', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  const original = await inspector(page).getByRole('textbox', { name: t('top.themeName'), exact: true }).inputValue();
  await inspector(page).getByRole('button', { name: t('insp.theme.actions'), exact: true }).click();
  await page.getByRole('menuitem', { name: t('common.duplicate'), exact: true }).click();
  await expect(inspector(page).getByRole('textbox', { name: t('top.themeName'), exact: true })).toHaveValue(t('layers.copyOf', { name: original }));
  await expect(inspector(page).locator('.theme-save-state')).toHaveText(t('status.saved'));
  await expectAccessible(page); expect(errors).toEqual([]);
});


test('multi rotation appears only for a fully eligible selection and undoes once', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  await page.locator('#grid-toggle').focus(); await page.keyboard.press('Control+a'); await page.keyboard.press('Delete');
  await clickAddWidget(page, t, 'ring'); await clickAddWidget(page, t, 'bar');
  await page.locator('#grid-toggle').focus(); await page.keyboard.press('Control+a');
  await openSection(page, 'layout');
  const rotation = inspector(page).getByRole('spinbutton', { name: t('insp.rotation'), exact: true });
  await expect(rotation).toHaveValue('0'); await change(rotation, 30);
  await expect(rotation).toHaveValue('30'); await page.locator('#undo').click();
  await expect(rotation).toHaveValue('0');
  await clickAddWidget(page, t, 'card'); await openSection(page, 'layout');
  await expect(rotation).toHaveCount(0);
  await page.locator('#grid-toggle').focus(); await page.keyboard.press('Control+a');
  await openSection(page, 'layout'); await expect(rotation).toHaveCount(0);
  await expectAccessible(page); expect(errors).toEqual([]);
});
