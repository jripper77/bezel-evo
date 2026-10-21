import { addMedia, setPanelWidths } from '../../../tests/e2e/helpers.mjs';
import { translator } from '../../../src/i18n/index.js';
// The library panels (artboards Main's left panel, Sensors, Layers, Media, Themes).
// The standalone artboards use 360px panels. Use the existing resizer
// preference helper, then reload so the normal host applies that width.
const panel = async (app, name) => {
  await setPanelWidths(app.page, { library: 360, inspector: 320 });
  await app.page.reload();
  await app.tab(name);
};
const sensors = async (app) => {
  // Let the real demo bridge produce a deterministic, changing history.
  await app.page.clock.install({ time: new Date('2026-10-09T10:00:00Z') });
  await panel(app, 'sensors');
  await app.page.clock.runFor(6000);
};
const themes = async (app) => { await app.tab('themes'); await app.click('#theme-gallery'); };
export default {
  area: 'library',
  artboards: ['Main', 'Sensors', 'Layers', 'Media', 'Themes'],
  states: [
    { name: 'widgets', run: (app) => app.tab('widgets') },
    { name: 'sensors', run: sensors },
    { name: 'sensors-libre', query: '&libre=partial', run: sensors },
    { name: 'sensors-libre-detail', query: '&libre=partial', run: async (app) => { await sensors(app); await app.page.locator('#sensor-libre').scrollIntoViewIfNeeded(); } },
    { name: 'layers', run: (app) => panel(app, 'layers') },
    { name: 'layers-default', run: (app) => app.tab('layers') },
    { name: 'layers-selection', run: async (app) => {
      await panel(app, 'layers');
      const names = app.page.locator('#layer-list button.layer-name');
      await names.nth(1).click();
      await names.nth(2).click({ modifiers: ['Control'] });
    } },
    { name: 'media', run: (app) => panel(app, 'media') },
    { name: 'media-filled', run: async (app) => {
      await panel(app, 'media');
      const t = translator('it');
      await addMedia(app.page, t, 'image');
      await addMedia(app.page, t, 'image');
      await addMedia(app.page, t, 'video');
      await app.page.locator('.media-item').nth(2).waitFor({ state: 'visible' });
      await app.page.locator('#toast').waitFor({ state: 'hidden' });
    } },
    { name: 'media-collection', run: async (app) => { await app.tab('media'); await app.click('#subtab-media-collection'); } },
    { name: 'media-icons', run: async (app) => { await app.tab('media'); await app.click('#subtab-media-icons'); } },
    { name: 'gif-search', run: async (app) => { await app.tab('media'); await app.click('#subtab-media-collection'); await app.click('#gif-search-open'); } },
    { name: 'themes-panel', run: (app) => app.tab('themes') },
    { name: 'themes', run: themes },
    { name: 'themes-all', run: async (app) => { await themes(app); await app.click('#theme-scope [data-scope=all]'); } },
  ].map(state => ({ ...state, capture: state.name.startsWith('themes') && state.name !== 'themes-panel' ? '.themes-dialog' : state.name === 'gif-search' ? '.gif-dialog' : '.sidebar' })),
};
