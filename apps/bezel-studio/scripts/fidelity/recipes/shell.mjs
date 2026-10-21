// The shell (artboard Main): topbar, rail, stage and statusbar.
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
