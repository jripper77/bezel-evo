// The library panels as in the artboards (Main, Sensors, Layers, Media,
// Themes), with the demo's real data: grouped and searchable widgets with the
// Group tile, sparklines and the update rate, a sensor dropped on a widget
// binding it, the Libre notice, Layers names in full, its filter and the
// multiple-selection bar, the "New theme" card, the theme search and
// Duplicate, and the Media totals.
import { test, expect, watchErrors, expectAccessible, dragTo, layers, libraryTabKey } from './helpers.mjs';

const tab = (page, t, name) => page.getByRole('tab', { name: t(libraryTabKey(name)), exact: true }).click();

test('widgets come in four foldable groups with a count, a search and the hint at the foot', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  const panel = page.locator('#panel-widgets');
  await expect(panel.locator('.panel-head .count')).toHaveText(t('widgets.count', { count: 14 }));
  await expect(panel.locator('.accordion-title')).toHaveText(['values', 'timeMedia', 'textShapes', 'containers'].map(id => t(`widgets.group.${id}`)));
  await expect(panel.locator('.accordion-summary')).toHaveText(['5', '4', '3', '2']);
  await expect(panel.locator('.widget-tile')).toHaveCount(14);
  await expect(panel.locator('.dashed-hint kbd')).toHaveText(t('widgets.enterKey'));
  await expectAccessible(page);

  const search = panel.getByRole('searchbox', { name: t('widgets.search'), exact: true });
  await search.fill(t('widget.needle').toUpperCase());
  await expect(panel.locator('.widget-tile')).toHaveCount(1);
  await expect(panel.locator('.accordion-title')).toHaveText([t('widgets.group.values')]);
  await expect(panel.locator('.panel-head .count')).toHaveText(t('widgets.count', { count: 1 }));
  await search.fill('zzzz');
  await expect(panel.locator('.widget-tile')).toHaveCount(0);
  await expect(panel).toContainText(t('widgets.none'));
  await search.fill('');
  await expect(panel.locator('.widget-tile')).toHaveCount(14);

  // A group folds with its count still in sight, and stays folded after a reload.
  const values = panel.getByRole('button', { name: t('widgets.group.values'), exact: true });
  await values.click(); await expect(values).toHaveAttribute('aria-expanded', 'false');
  await expect(panel.locator('.widget-tile:visible')).toHaveCount(9);
  await expect(panel.locator('.accordion-summary').first()).toBeVisible();
  await expectAccessible(page);
  await page.reload();
  await expect(values).toHaveAttribute('aria-expanded', 'false');
  // A search opens every group it finds.
  await search.fill(t('widget.ring'));
  await expect(values).toHaveAttribute('aria-expanded', 'true');
  await search.fill(''); await values.click(); await expect(values).toHaveAttribute('aria-expanded', 'true');

  // Enter on a focused tile adds it at the centre, as the hint says.
  await panel.getByRole('button', { name: t('library.addWidget', { name: t('common.text') }), exact: true }).focus();
  await page.keyboard.press('Enter');
  await tab(page, t, 'layers');
  await expect(layers(page)).toHaveCount(4);
  expect(errors).toEqual([]);
});

test('the Group tile groups the selection and says why it cannot otherwise', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  const group = page.getByRole('button', { name: t('widgets.groupTile'), exact: true });
  await expect(group).toHaveAttribute('aria-disabled', 'true');
  await expect(group).toHaveAttribute('title', t('group.scope'));
  await group.click({ force: true });
  await tab(page, t, 'layers');
  await expect(page.locator('#layer-list [data-group-id]')).toHaveCount(0);

  await page.locator('.layer-row').filter({ hasText: 'CPU' }).locator('.layer-name').click();
  await page.locator('.layer-row').filter({ hasText: 'Clock' }).locator('.layer-name').click({ modifiers: ['Control'] });
  await tab(page, t, 'widgets');
  await expect(group).toHaveAttribute('aria-disabled', 'false');
  await group.click();
  // The inspector's heading names the new group (T-0 made it an editable name).
  await expect(page.locator('#inspector .inspector-name')).toHaveValue(t('widget.group'));
  await tab(page, t, 'layers');
  await expect(page.locator('#layer-list [data-group-id]')).toHaveCount(1);
  expect(errors).toEqual([]);
});

test('sensors draw their recent values, say how often they update and bind when dropped on a widget', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await tab(page, t, 'sensors');
  const panel = page.locator('#panel-sensors');
  await expect(panel.getByRole('searchbox', { name: t('library.searchSensors'), exact: true })).toHaveAttribute('placeholder', t('sensorsPanel.search'));
  await expect(panel.locator('.panel-head .pill')).toHaveText(t('sensorsPanel.every', { seconds: 1 }), { timeout: 10_000 });
  await expect(panel.locator('svg[data-spark-for="cpu.usage"] path')).toHaveAttribute('d', /^M0 [\d.]+ L/, { timeout: 10_000 });
  await expect(panel.locator('.sensor-group .section-title').first()).toHaveText(t('category.cpu'));
  await expectAccessible(page);

  // Dropped on the ring (which reads cpu.usage), the sensor binds it: no new object.
  await tab(page, t, 'layers');
  const ring = page.locator('.layer-row').filter({ hasText: 'CPU' });
  await expect(ring.locator('.layer-meta')).toHaveText('cpu.usage');
  await ring.locator('.layer-name').click();
  const box = page.locator('.sel-box');
  await expect(box).toHaveCount(1);
  await tab(page, t, 'sensors');
  await panel.getByRole('searchbox', { name: t('library.searchSensors'), exact: true }).fill('gpu.temperature');
  await dragTo(page, panel.locator('.sensor-row[data-key="gpu.temperature"]'), box);
  await tab(page, t, 'layers');
  await expect(layers(page)).toHaveCount(3);
  await expect(ring.locator('.layer-meta')).toHaveText('gpu.temperature');
  await page.locator('#undo').click();
  await expect(ring.locator('.layer-meta')).toHaveText('cpu.usage');
  expect(errors).toEqual([]);
});

test('the Sensors panel says when LibreHardwareMonitor reads nothing and restarts it', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88&libre=partial');
  await tab(page, t, 'sensors');
  const notice = page.locator('#sensor-libre .notice');
  await expect(notice).toBeVisible({ timeout: 10_000 });
  await expect(notice.getByRole('heading')).toContainText(t('status.libreMissing', { hardware: '' }).trim());
  await expect(notice).toContainText(t('sensorsPanel.libreBody'));
  await expectAccessible(page);
  await notice.getByRole('button', { name: t('sensorsPanel.startLibre'), exact: true }).click();
  await expect(page.locator('#toast')).toHaveText(t('status.libreRestarted'));
  expect(errors).toEqual([]);
});

test('Layers shows names in full, filters by name or sensor and counts what it shows (fidelity-13)', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await tab(page, t, 'layers');
  const panel = page.locator('#panel-layers');
  await expect(layers(page)).toHaveCount(3);
  await expect(panel.locator('#layer-front .section-title')).toHaveText(t('layersPanel.front'));
  await expect(panel.locator('#layer-front .count')).toHaveText(t('layers.count', { count: 3 }));
  // Every name fits at the default width: the tools never take its room.
  const labels = panel.locator('.layer-label');
  await expect(labels).toHaveText(['Background card', 'CPU', 'Clock']);
  for (const label of await labels.all()) {
    const fits = await label.evaluate((n) => n.scrollWidth <= n.clientWidth);
    expect(fits, await label.textContent()).toBe(true);
  }
  await expectAccessible(page);

  const filter = panel.getByRole('searchbox', { name: t('layersPanel.filter'), exact: true });
  await filter.fill('cpu.usage');
  await expect(layers(page)).toHaveCount(1);
  await expect(panel.locator('#layer-front .count')).toHaveText(t('layers.countOne'));
  await filter.fill('CLOCK');
  await expect(panel.locator('.layer-label')).toHaveText(['Clock']);
  await filter.fill('zzzz');
  await expect(layers(page)).toHaveCount(0);
  await expect(panel).toContainText(t('layersPanel.noMatch'));
  await filter.fill('');
  await expect(layers(page)).toHaveCount(3);
  const type = panel.getByRole('combobox', { name: t('layersPanel.type'), exact: true });
  await expect(type).toHaveValue('all');
  await type.selectOption('clock');
  await expect(panel.locator('.layer-label')).toHaveText(['Clock']);
  await expect(panel.locator('#layer-front .count')).toHaveText(t('layers.countOne'));
  await filter.fill('cpu.usage');
  await expect(layers(page)).toHaveCount(0);
  await expect(panel).toContainText(t('layersPanel.noMatch'));
  await type.selectOption('ring');
  await expect(panel.locator('.layer-label')).toHaveText(['CPU']);
  await filter.fill('');
  await type.selectOption('player');
  await expect(layers(page)).toHaveCount(0);
  await type.selectOption('all');
  await expect(layers(page)).toHaveCount(3);
  await expect(page.locator('#status-main')).toHaveText(t('status.saved'));
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('two selected layers get the bar: Group (Ctrl G), Make card and Deselect', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await tab(page, t, 'layers');
  const bar = page.locator('#layer-selection');
  await expect(bar).toBeHidden();
  const select = async () => {
    await page.locator('.layer-row').filter({ hasText: 'CPU' }).locator('.layer-name').click();
    await page.locator('.layer-row').filter({ hasText: 'Clock' }).locator('.layer-name').click({ modifiers: ['Control'] });
  };
  await select();
  await expect(bar).toBeVisible();
  await expect(bar.locator('strong')).toHaveText(t('layersPanel.selected', { count: 2 }));
  await expect(bar.locator('kbd')).toHaveText(t('layersPanel.groupKeys'));
  await expectAccessible(page);

  await bar.getByRole('button', { name: t('layersPanel.groupLabel'), exact: true }).click();
  await expect(page.locator('#layer-list [data-group-id]')).toHaveCount(1);
  await expect(bar).toBeHidden();
  const group = page.locator('#layer-list [data-group-id]');
  await group.getByRole('button', { name: t('layers.collapse'), exact: true }).click();
  await expect(group.locator('.layer-children')).toBeHidden();
  const filter = page.getByRole('searchbox', { name: t('layersPanel.filter'), exact: true });
  await filter.fill('cpu.usage');
  await expect(group.locator('.layer-label').filter({ hasText: 'CPU' })).toBeVisible();
  await expect(group.locator('.layer-children')).toBeVisible();
  await filter.fill('');
  await expect(group.locator('.layer-children')).toBeHidden();
  const type = page.getByRole('combobox', { name: t('layersPanel.type'), exact: true });
  await type.selectOption('ring');
  await expect(group.locator('.layer-children')).toBeVisible();
  await expect(group.locator('.layer-label')).toHaveText([t('widget.group'), 'CPU']);
  await type.selectOption('all');
  await expect(group.locator('.layer-children')).toBeHidden();
  await group.getByRole('button', { name: t('layers.expand'), exact: true }).click();
  await expect(group.locator('.layer-children')).toBeVisible();
  await page.locator('#undo').click();
  await expect(page.locator('#layer-list [data-group-id]')).toHaveCount(0);

  await select();
  await bar.getByRole('button', { name: t('layersPanel.cardLabel'), exact: true }).click();
  await expect(page.locator('#layer-list [data-card-id]')).toHaveCount(1);
  await page.locator('#undo').click();
  await expect(page.locator('#layer-list [data-card-id]')).toHaveCount(0);

  await select();
  await bar.getByRole('button', { name: t('layersPanel.clear'), exact: true }).click();
  await expect(bar).toBeHidden();
  await expect(page.locator('.layer-row.selected')).toHaveCount(0);
  expect(errors).toEqual([]);
});

test('Themes: the New theme card, the search, the screen thumbnail and Duplicate', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await tab(page, t, 'themes');
  const panel = page.locator('#panel-themes');
  await panel.locator('#theme-scope [data-scope="all"]').click();
  const cards = panel.locator('#theme-grid .theme-card');
  const total = await cards.count();
  expect(total).toBeGreaterThan(1);
  const fresh = panel.locator('.theme-new');
  await expect(fresh).toContainText(t('themesPanel.new'));
  await expect(fresh.locator('.theme-new-size')).toHaveText('480×1920');
  await expect(fresh.getByRole('button', { name: t('themes.newVertical'), exact: true })).toBeVisible();
  await expect(fresh.getByRole('button', { name: t('themes.newHorizontal'), exact: true })).toBeVisible();
  await expect(panel.locator('.theme-totals')).toHaveText(`${t('themes.countAll', { count: total })} · ${t('themesPanel.forScreen', { count: 1 })}`);
  const demo = cards.filter({ hasText: 'Demo' });
  await expect(demo.locator('.screen-thumb')).toHaveAttribute('style', /--ratio: 480 \/ 1920/);
  await expectAccessible(page);

  const search = panel.getByRole('searchbox', { name: t('themesPanel.search'), exact: true });
  await search.fill('demo');
  await expect(cards).toHaveCount(1);
  await search.fill('zzzz');
  await expect(cards).toHaveCount(0);
  await expect(panel).toContainText(t('themesPanel.noMatch'));
  await expect(fresh).toBeVisible();
  await search.fill('');
  await expect(cards).toHaveCount(total);

  await panel.getByRole('button', { name: t('themesPanel.duplicateLabel', { name: 'Demo' }), exact: true }).click();
  await expect(cards).toHaveCount(total + 1);
  await expect(panel.locator('#theme-grid')).toContainText(t('layers.copyOf', { name: 'Demo' }));
  expect(errors).toEqual([]);
});

test('Media says what the theme holds and opens the GIF search from its foot', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await tab(page, t, 'media');
  const panel = page.locator('#panel-media');
  await expect(panel.locator('.media-drop-title')).toHaveText(t('mediaPanel.dropTitle'));
  await expect(panel.locator('#media-head .section-title')).toHaveText(t('media.inTheme'));
  const open = panel.getByRole('button', { name: t('media.searchGifs') });
  await expect(open).toBeVisible();
  await expect(open).toContainText(t('mediaPanel.provider'));
  await expectAccessible(page);
  await open.click();
  await expect(page.getByRole('dialog')).toBeVisible();
  expect(errors).toEqual([]);
});


test('Background and CPU Analog stay complete beside visible layer tools at the default width', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await tab(page, t, 'layers');
  for (const [id, name] of [[3, 'Background'], [2, 'CPU Analog'], [1, 'Clock with a deliberately long layer name']]) {
    const row = page.locator(`#layer-list .layer-row[data-element-id="${id}"]`);
    await row.locator('button.layer-name').dblclick();
    const edit = row.getByRole('textbox');
    await edit.fill(name);
    await edit.press('Enter');
    await row.locator('button.layer-name').click();
    await row.hover();
    const label = row.locator('.layer-label');
    await expect(label).toHaveText(name);
    expect(await label.evaluate((node) => node.scrollWidth <= node.clientWidth)).toBe(true);
    const text = await label.boundingBox();
    for (const tool of await row.locator('.layer-tool').all()) {
      await expect(tool).toBeVisible();
      const box = await tool.boundingBox();
      // Tools cannot paint over any part of the name, even when selected.
      expect(text.y + text.height <= box.y || text.x + text.width <= box.x).toBe(true);
    }
  }
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('the Themes dialog reuses its grid, selects before opening, duplicates and restores focus', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await tab(page, t, 'themes');
  const launcher = page.getByRole('button', { name: t('themesPanel.gallery'), exact: true });
  const original = await page.locator('#theme-grid').elementHandle();
  await launcher.click();
  const dialog = page.getByRole('dialog', { name: t('library.themes'), exact: true });
  await expect(dialog).toBeVisible();
  expect(await dialog.locator('#theme-grid').evaluate((node, old) => node === old, original)).toBe(true);
  const search = dialog.getByRole('searchbox', { name: t('themesPanel.search'), exact: true });
  await expect(search).toBeFocused();
  await dialog.locator('#theme-scope [data-scope=all]').click();
  const cards = dialog.locator('.theme-card');
  const count = await cards.count();
  const chosen = cards.filter({ hasText: 'Midnight 5' });
  const name = await chosen.locator('strong').textContent();
  await chosen.click();
  await expect(chosen).toHaveAttribute('aria-pressed', 'true');
  await expect(dialog.locator('.themes-gallery-selection')).toHaveText(t('themesPanel.selected', { name }));
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await expectAccessible(page);
  await dialog.getByRole('button', { name: t('common.duplicate'), exact: true }).click();
  await expect(cards).toHaveCount(count + 1);
  await expect(dialog.locator('#theme-grid')).toContainText(t('layers.copyOf', { name }));
  await search.fill('zzzz');
  await expect(cards).toHaveCount(0);
  await expect(dialog.getByRole('button', { name: t('themesPanel.openEditor'), exact: true })).toBeDisabled();
  await page.keyboard.press('Escape');
  await expect(dialog).toHaveCount(0);
  await expect(launcher).toBeFocused();
  expect(await page.locator('#theme-grid').evaluate((node, old) => node === old, original)).toBe(true);
  await expect(page.locator('#theme-search input')).toHaveValue('zzzz');
  await launcher.click();
  await dialog.getByRole('searchbox', { name: t('themesPanel.search'), exact: true }).fill('Midnight 5');
  await dialog.locator('.theme-card').filter({ has: page.getByText(name, { exact: true }) }).click();
  await dialog.getByRole('button', { name: t('themesPanel.openEditor'), exact: true }).click();
  await expect(dialog).toHaveCount(0);
  await expect(page.locator('#theme-name')).toHaveValue(name);
  await expectAccessible(page);
  expect(errors).toEqual([]);
});


test('Media lists bridge imports, counts files without posters and applies their background', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await tab(page, t, 'media');
  const panel = page.locator('#panel-media');
  const items = panel.locator('.media-item');
  await expect(items).toHaveCount(0);
  await panel.locator('#media-add').click();
  await expect(items).toHaveCount(1);
  await expect(items.first().locator('.media-kind')).toHaveText('PNG');
  await panel.locator('#media-add-video').click();
  await expect(items).toHaveCount(2);
  await expect(panel.locator('#media-head .count')).toContainText(t('media.count', { count: 2 }));
  const video = items.filter({ has: page.getByText(t('common.video'), { exact: true }) });
  await expect(video).toHaveCount(1);
  await video.getByRole('button', { name: t('collection.useBackground'), exact: true }).click();
  await expect(video).toHaveClass(/is-background/);
  await expect(video.locator('.media-bg')).toBeVisible();
  await page.locator('#undo').click();
  await expect(video.locator('.media-bg')).toBeHidden();
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('Media removes unused files and protects an image used as background', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await tab(page, t, 'media');
  const panel = page.locator('#panel-media');
  await panel.locator('#media-add').click();
  const item = panel.locator('.media-item');
  const remove = item.getByRole('button', { name: t('mediaPanel.remove', { name: 'image-1.png' }), exact: true });
  await item.getByRole('button', { name: t('collection.useBackground'), exact: true }).click();
  await remove.click();
  await expect(page.locator('#toast')).toHaveText(t('mediaPanel.inUse'));
  await expect(item).toHaveCount(1);
  await page.locator('#undo').click();
  await remove.click();
  await expect(item).toHaveCount(0);
  await expect(panel).toContainText(t('media.empty'));
  expect(errors).toEqual([]);
});
