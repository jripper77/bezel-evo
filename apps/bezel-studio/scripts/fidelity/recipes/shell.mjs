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
