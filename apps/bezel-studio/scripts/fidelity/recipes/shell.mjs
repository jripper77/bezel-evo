// The shell (artboard Main): topbar, rail, stage and statusbar.
import { translator } from '../../../src/i18n/index.js';
import { openSection } from '../../../tests/e2e/helpers.mjs';
export default {
  area: 'shell',
  artboards: ['Main', 'Logo'],
  states: [
    { name: 'ux2-handles-ring', capture: '.workspace', run: async app => {
      await app.add('ring'); await openSection(app.page, 'shape');
      const handle = await app.page.locator('#overlay [data-property-handle="thickness"]').boundingBox();
      await app.page.mouse.move(handle.x + handle.width / 2, handle.y + handle.height / 2);
      await app.page.mouse.down();
      await app.page.mouse.move(handle.x + handle.width / 2, handle.y + handle.height / 2 + 8, { steps: 4 });
    } },
    { name: 'ux2-handles-selection', capture: '.workspace', run: async app => {
      const t = translator('it');
      await app.add('shape'); await openSection(app.page, 'layout');
      const x = app.page.locator('#inspector').getByRole('spinbutton', { name: t('inspector.x'), exact: true });
      await x.fill('100'); await x.press('Tab');
      await app.tab('layers');
      const first = await app.page.locator('.layer-row.selected').getAttribute('data-element-id');
      await app.add('bar'); await app.tab('layers');
      await app.page.locator(`.layer-row[data-element-id="${first}"] .layer-name`).click({ modifiers: ['Control'] });
      await app.page.locator('#overlay [data-h="e"]').focus();
      await app.page.keyboard.press('Shift+ArrowRight');
    } },
    { name: 'ux2-text-inline', capture: '.workspace', run: async app => {
      await app.add('text');
      const bounds = await app.page.locator('#overlay .sel-box').boundingBox();
      await app.page.mouse.dblclick(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2);
    } },
    { name: 'ux2-card-entry', capture: '.workspace', run: async app => {
      await app.add('card'); await app.add('text');
      const bounds = await app.page.locator('#overlay .sel-box[role="img"]').boundingBox();
      await app.page.mouse.dblclick(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2);
    } },
    { name: 'ux2-shape-image-entry', capture: '.workspace', run: async app => {
      const t = translator('it'), inspector = app.page.locator('#inspector');
      await app.tab('media');
      await app.page.getByRole('tab', { name: t('media.icons'), exact: true }).click();
      await app.page.locator('#icon-search').fill('cpu');
      await app.page.locator('[data-icon="cpu"]').click();
      const asset = await inspector.getByRole('combobox', { name: t('common.image'), exact: true }).inputValue();
      await app.add('shape'); await openSection(app.page, 'color');
      await inspector.getByRole('combobox', { name: t('common.image'), exact: true }).selectOption(asset);
      await inspector.getByRole('button', { name: t('shape.repeat.tile'), exact: true }).click();
      const bounds = await app.page.locator('#overlay .sel-box[role="img"]').boundingBox();
      await app.page.mouse.dblclick(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2);
      await app.page.keyboard.press('Shift+ArrowRight');
    } },
    { name: 'ux2-clock-length', capture: '.workspace', run: async app => {
      const t = translator('it'), inspector = app.page.locator('#inspector');
      await app.add('clock'); await openSection(app.page, 'layout');
      for (const key of ['inspector.width', 'inspector.height']) {
        const field = inspector.getByRole('spinbutton', { name: t(key), exact: true });
        await field.fill('360'); await field.press('Tab');
      }
      await openSection(app.page, 'shape');
      await inspector.getByRole('button', { name: t('insp.clock.analog'), exact: true }).click();
      const frame = await app.page.locator('#overlay .sel-box[role="img"]').boundingBox();
      const handle = await app.page.locator('#overlay [data-property-handle="clock-hour-length"]').boundingBox();
      const x = handle.x + handle.width / 2, y = handle.y + handle.height / 2;
      const dx = x - frame.x - frame.width / 2, dy = y - frame.y - frame.height / 2, length = Math.hypot(dx, dy);
      await app.page.mouse.move(x, y); await app.page.mouse.down();
      await app.page.mouse.move(x + dx / length * 12, y + dy / length * 12, { steps: 4 });
    } },
    { name: 'ux2-alignment-distance', capture: '.workspace', run: async app => {
      const t = translator('it'), inspector = app.page.locator('#inspector');
      await app.page.locator('#grid-toggle').focus();
      await app.page.keyboard.press('Control+a'); await app.page.keyboard.press('Delete');
      const place = async (x, y) => {
        await app.add('shape'); await openSection(app.page, 'layout');
        for (const [key, value] of [['inspector.x', x], ['inspector.y', y], ['inspector.width', 120], ['inspector.height', 60]]) {
          const field = inspector.getByRole('spinbutton', { name: t(key), exact: true });
          await field.fill(String(value)); await field.press('Tab');
        }
        await app.tab('layers');
        return app.page.locator('.layer-row.selected').getAttribute('data-element-id');
      };
      const first = await place(100, 500); await place(300, 600);
      await app.page.locator(`.layer-row[data-element-id="${first}"] .layer-name`).click();
      const bounds = await app.page.locator('#overlay .sel-box[role="img"]').boundingBox();
      const zoom = (await app.page.locator('#canvas-box').boundingBox()).width / 480;
      await app.page.mouse.move(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2); await app.page.mouse.down();
      await app.page.mouse.move(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2 + 100 * zoom, { steps: 4 });
    } },
    { name: 'ux2-object-bar', capture: '.workspace', run: async app => {
      await app.add('ring');
      await app.page.locator('#overlay [data-object-bar="duplicate"]').hover();
    } },
    { name: 'ux2-player-cover', capture: '.workspace', run: async app => {
      await app.add('player');
      const handle = await app.page.locator('#overlay [data-property-handle="cover-gap"]').boundingBox();
      await app.page.mouse.move(handle.x + handle.width / 2, handle.y + handle.height / 2); await app.page.mouse.down();
      await app.page.mouse.move(handle.x + handle.width / 2 + 12, handle.y + handle.height / 2, { steps: 4 });
    } },
    { name: 'ux2-value-part', capture: '.workspace', run: async app => {
      const t = translator('it'), inspector = app.page.locator('#inspector');
      await app.add('value'); await openSection(app.page, 'data');
      await inspector.getByRole('textbox', { name: t('inspector.prefix'), exact: true }).fill('CPU ');
      await app.page.keyboard.press('Tab');
      const frame = await app.page.locator('#overlay .sel-box').boundingBox();
      await app.page.mouse.click(frame.x + 6, frame.y + frame.height / 2);
    } },
    { name: 'main', run: async () => {} },
    { name: 'logo', capture: '.brand', run: async () => {} },
    // An edit: "Unsaved" in the breadcrumb and the statusbar.
    { name: 'unsaved', run: (app) => app.add('ring') },
    { name: 'live', run: (app) => app.click('label.live-toggle') },
    { name: 'grid', run: (app) => app.click('#grid-toggle') },
    { name: 'preferences', run: (app) => app.click('#preferences') },
    // No screen: the Connect card over the stage (outside the gallery, D-5 revised).
    { name: 'no-screen', demo: 'empty', run: async () => {} },
  ],
};
