import { fileURLToPath } from 'node:url';
import { addMedia, setPanelWidths } from '../../../tests/e2e/helpers.mjs';
import { translator } from '../../../src/i18n/index.js';
// The library panels (artboards Main's left panel, Sensors, Layers, Media, Themes).
// LibraryPanelsNew uses 400px panels without the rail. Use the existing resizer
// preference helper, then reload so the normal host applies that width.
const panel = async (app, name) => {
  await setPanelWidths(app.page, { library: 400, inspector: 360 });
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
  mockDesigns: [{ root: fileURLToPath(new URL('../../../../../.jdi/phases/studio-ux-v2/design/boards/', import.meta.url)), artboards: ['ProfilesNew', 'ThemesNew'] }],
  artboards: ['Main', 'Sensors', 'Layers', 'Media', 'Themes'],
  states: [
    ...['active', 'ready', 'manual'].map(status => ({ name: status === 'active' ? 'ux2-profiles' : `ux2-profiles-${status}`, demo: 'two', capture: '.profiles-dialog', run: async app => {
      const t = translator('it');
      await app.page.setViewportSize({ width: 1280, height: 1500 });
      await app.tab('themes'); await app.click('#theme-profiles'); await app.click('#pf-add');
      await app.page.locator('#pf-name').fill('Browser');
      await app.page.locator('.pf-add-types').getByRole('button', { name: t('profiles.foreground'), exact: true }).click();
      await app.page.locator('.pf-rule input[type=text]').fill(status === 'ready' ? 'game.exe' : 'chrome');
      await app.page.locator('.pf-add-types').getByRole('button', { name: t('inspector.sensor'), exact: true }).click();
      await app.page.getByRole('spinbutton', { name: t('profileRules.ui.threshold'), exact: true }).fill(status === 'ready' ? '100' : '0');
      if (status === 'manual') await app.page.getByRole('button', { name: t('profiles.manual'), exact: true }).click();
      await app.page.locator('.pf-save-status').getByText(t('profileRules.ui.save.saved'), {exact:true}).waitFor();
      if (status === 'active') await app.page.locator('.pf-object-head [data-profile-active][data-active=true]').waitFor();
      if (status === 'ready') await app.page.locator('.pf-object-head [data-profile-active][data-active=false]').waitFor();
      await app.page.getByRole('group', { name: t('profiles.transition'), exact: true }).getByRole('button', { name: t('profiles.effect.curtain'), exact: true }).click();
      await app.page.locator('.pf-save-status').getByText(t('profileRules.ui.save.saved'), {exact:true}).waitFor();
      await app.page.locator('.profiles-dialog > form').evaluate(form => { form.style.height = `${form.querySelector('.pf-detail').scrollHeight + form.querySelector('.pf-head').offsetHeight + 8}px`; });
      await app.page.locator('.pf-detail').evaluate(node => { node.scrollTop = 0; });
      await app.page.locator('#pf-name').focus(); await app.page.locator('#pf-name').blur();
      await app.page.waitForFunction(() => document.querySelector('.pf-object-head .pf-thumb')?.style.backgroundImage.startsWith('url('));
    } })),
    { name: 'ux2-themes', capture: '.themes-dialog', run: async app => { await themes(app); await app.click('#theme-search .filters-toggle'); await app.click('#theme-scope [data-scope=all]'); } },
    { name: 'widgets', run: (app) => app.tab('widgets') },
    { name: 'sensors', run: sensors },
    { name: 'sensors-libre', query: '&libre=partial', run: sensors },
    { name: 'sensors-libre-detail', query: '&libre=partial', run: async (app) => { await sensors(app); await app.page.locator('#sensor-libre').scrollIntoViewIfNeeded(); } },
    { name: 'layers', run: (app) => panel(app, 'layers') },
    { name: 'layers-default', run: (app) => app.tab('layers') },
    { name: 'layers-selection', run: async (app) => {
      await panel(app, 'layers');
      const names = app.page.locator('#layer-list button.layer-name');
      await names.filter({ has: app.page.locator('.layer-label', { hasText: /^CPU$/ }) }).click();
      await names.filter({ hasText: 'Clock' }).click({ modifiers: ['Control'] });
      await app.page.keyboard.press('Control+g');
      await app.add('card'); await app.add('ring'); await app.add('bar');
      await app.tab('layers');
      await names.filter({ has: app.page.locator('.layer-label', { hasText: /^CPU$/ }) }).click();
      await names.filter({ hasText: 'Clock' }).click({ modifiers: ['Control'] });
    } },
    { name: 'media', run: (app) => panel(app, 'media') },
    { name: 'media-filled', run: async (app) => {
      await panel(app, 'media');
      const t = translator('it');
      await addMedia(app.page, t, 'image');
      await addMedia(app.page, t, 'image');
      await addMedia(app.page, t, 'video');
      await app.page.locator('.media-item').nth(2).waitFor({ state: 'visible' });
      await app.page.locator('.media-item').first().hover();
      await app.page.locator('.media-item').first().getByRole('button', { name: t('collection.useBackground'), exact: true }).click();
      await app.page.locator('#toast').waitFor({ state: 'hidden' });
    } },
    { name: 'media-collection', run: async (app) => { await app.tab('media'); await app.click('#subtab-media-collection'); } },
    { name: 'media-icons', run: async (app) => { await app.tab('media'); await app.click('#subtab-media-icons'); } },
    { name: 'gif-search', run: async (app) => { await app.tab('media'); await app.click('#subtab-media-collection'); await app.click('#gif-search-open'); } },
    { name: 'themes-panel', run: (app) => app.tab('themes') },
    { name: 'themes', run: themes },
    { name: 'themes-all', run: async (app) => { await themes(app); await app.click('#theme-search .filters-toggle'); await app.click('#theme-scope [data-scope=all]'); } },
  ].map(state => ({ ...state, capture: state.capture ?? (state.name.startsWith('themes') && state.name !== 'themes-panel' ? '.themes-dialog' : state.name === 'gif-search' ? '.gif-dialog' : '.library') })),
};
