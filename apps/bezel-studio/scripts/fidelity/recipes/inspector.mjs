// The inspector (artboards Main's right panel, GraphData/Look, PlayerData/Look).
import { fileURLToPath } from 'node:url';
import { openSection, openInspectorTab, setPanelWidths } from '../../../tests/e2e/helpers.mjs';
import { translator } from '../../../src/i18n/index.js';

const t = translator('it');
// Standalone inspector boards are 360px; Main keeps the shell's 320px.
async function standalone(app, kind) {
  await setPanelWidths(app.page, { inspector: 360 });
  await app.page.reload();
  if (kind) await app.add(kind);
  for (const section of await app.page.locator('#inspector > [data-inspector-section]').all()) await openSection(app.page, await section.getAttribute('data-inspector-section'));
}
const inspector = (app) => app.page.locator('#inspector');
async function linkedBoard(app, kind) {
  await standalone(app, kind);
  if (kind === 'needle') {
    await inspector(app).getByRole('button', { name: t('insp.needle.center'), exact: true }).click();
    await openSection(app.page, 'color');
    await inspector(app).getByRole('button', { name: t('insp.needle.addZone'), exact: true }).click();
    await openSection(app.page, 'behavior');
    await inspector(app).getByRole('button', { name: t('insp.needle.peak'), exact: true }).click();
  }
  if (kind === 'graph') {
    await inspector(app).getByRole('button', { name: t('insp.graphMirrored'), exact: true }).click();
    await openSection(app.page, 'shape');
    await inspector(app).getByRole('button', { name: t('insp.graphRounded'), exact: true }).click();
    const width = inspector(app).getByRole('spinbutton', { name: t('insp.thickness'), exact: true });
    await width.fill('3'); await width.press('Tab');
  }
  for (const section of await inspector(app).locator(':scope > [data-inspector-section]').all()) await openSection(app.page, await section.getAttribute('data-inspector-section'));
  await inspector(app).evaluate(node => { node.style.height = `${node.scrollHeight}px`; node.style.maxHeight = 'none'; node.style.flex = 'none'; node.scrollTop = 0; });
  await app.page.locator('.statusbar').evaluate(node => { node.style.visibility = 'hidden'; });
}
/** "Change background…", then one of its sources opened. */
async function openChooser(app, sourceKey) {
  await inspector(app).getByRole('button', { name: t('insp.bgChange'), exact: true }).click();
  const head = inspector(app).locator('.bg-source-head').filter({ hasText: t(sourceKey) });
  if ((await head.getAttribute('aria-expanded')) !== 'true') await head.click();
}

export default {
  area: 'inspector',
  mockDesigns: [{
    root: fileURLToPath(new URL('../../../../../.jdi/phases/studio-redesign-fidelity/design-bg/project/', import.meta.url)),
    artboards: ['BgCurrent', 'BgChooser', 'BgStates'],
  }],
  artboards: ['Main', 'GraphData', 'GraphLook', 'PlayerData', 'PlayerLook'],
  states: [
    ...['value', 'text', 'bar', 'needle', 'graph'].map(kind => ({ name: `ux2-linked-${kind}`, capture: '#inspector', run: app => linkedBoard(app, kind) })),
    { name: 'ux2-linked-multi', capture: '#inspector', run: async app => {
      await standalone(app, null);
      await app.page.locator('#grid-toggle').focus(); await app.page.keyboard.press('Control+a'); await app.page.keyboard.press('Delete');
      await app.add('ring'); await app.add('bar'); await app.add('value');
      await app.page.locator('#grid-toggle').focus(); await app.page.keyboard.press('Control+a');
      await openSection(app.page, 'layout'); await openSection(app.page, 'data');
      await inspector(app).evaluate(node => { node.style.height = `${node.scrollHeight}px`; node.style.maxHeight = 'none'; node.style.flex = 'none'; node.scrollTop = 0; });
      await app.page.locator('.statusbar').evaluate(node => { node.style.visibility = 'hidden'; });
    } },
    { name: 'ux2-ring-closed', run: async app => { await standalone(app, 'ring'); await app.page.locator('#inspector [data-inspector-section="data"] .accordion-toggle').click({ modifiers: ['Alt'] }); } },
    { name: 'ux2-ring-open', run: async app => {
      await standalone(app, 'ring');
      for (const id of ['layout', 'data', 'shape', 'color', 'behavior']) await openSection(app.page, id);
      await inspector(app).evaluate(node => { node.style.height = `${node.scrollHeight}px`; node.style.maxHeight = 'none'; node.style.flex = 'none'; node.scrollTop = 0; });
      await app.page.locator('.statusbar').evaluate(node => { node.style.visibility = 'hidden'; });
    } },
    { name: 'ux2-main', run: app => app.add('ring') },
    { name: 'ux2-theme', run: async app => {
      await standalone(app, null);
      await openSection(app.page, 'color'); await openSection(app.page, 'behavior');
    } },

    { name: 'theme', run: async () => {} },
    // The background in use and its chooser (D-2026-10-16-studio-redesign-fidelity-25, artboards BgCurrent, BgChooser, BgStates).
    { name: 'bg-computer-image', run: async (app) => {
      await standalone(app, null);
      await app.page.locator('#tab-media').click();
      await app.page.locator('#media-add').click();
      await app.page.locator('.media-item').getByRole('button', { name: t('collection.useBackground'), exact: true }).click();
    } },
    { name: 'bg-computer-video', run: async (app) => {
      await standalone(app, null);
      await openChooser(app, 'insp.bgComputer');
      await inspector(app).getByRole('button', { name: t('insp.bgPickFile'), exact: true }).click();
      await inspector(app).getByRole('button', { name: t('insp.bgUseVideo'), exact: true }).click();
      await app.page.locator('#bg-change').waitFor({ state: 'visible' });
    } },
    { name: 'bg-screen', demo: 'screenVideoMissing', run: async (app) => {
      await standalone(app, null);
      await openChooser(app, 'insp.bgScreenVideo');
      await inspector(app).getByRole('radio', { name: /^AMD\.mp4/ }).check();
      await inspector(app).getByRole('button', { name: t('insp.bgUseVideo'), exact: true }).click();
      await app.page.locator('#bg-change').waitFor({ state: 'visible' });
    } },
    { name: 'bg-chooser', demo: 'screenVideoMissing', run: async (app) => {
      await standalone(app, null);
      await openChooser(app, 'insp.bgScreenVideo');
      await inspector(app).getByRole('radio', { name: /^AMD\.mp4/ }).check();
      await inspector(app).getByRole('button', { name: t('insp.bgUseVideo'), exact: true }).click();
      await app.page.locator('#bg-change').waitFor({ state: 'visible' });
      await openChooser(app, 'insp.bgScreenVideo');
      await inspector(app).getByRole('radio', { name: /^NVI\.mp4 / }).check();
    } },
    { name: 'bg-missing', demo: 'screenVideoMissing', run: (app) => standalone(app, null) },
    { name: 'bg-gone', demo: 'screenVideoGone', run: (app) => standalone(app, null) },
    { name: 'bg-no-playback', demo: 'noPlayback', run: async (app) => {
      await standalone(app, null);
      await inspector(app).getByRole('button', { name: t('insp.bgChange'), exact: true }).click();
    } },
    { name: 'ring', run: (app) => app.add('ring') },
    { name: 'ring-look', run: async (app) => { await app.add('ring'); await openInspectorTab(app.page, t, 'inspector.tab.look'); } },
    { name: 'graph', run: (app) => standalone(app, 'graph') },
    { name: 'graph-auto', run: async (app) => {
      await standalone(app, 'graph');
      await app.page.locator('#inspector').getByRole('button', { name: t('insp.automatic'), exact: true }).click();
    } },
    { name: 'graph-look', run: async (app) => { await standalone(app, 'graph'); await openInspectorTab(app.page, t, 'inspector.tab.look'); } },
    { name: 'player', run: (app) => standalone(app, 'player') },
    { name: 'player-look', run: async (app) => { await standalone(app, 'player'); await openInspectorTab(app.page, t, 'inspector.tab.look'); } },
    { name: 'player-corners', run: async (app) => {
      await standalone(app, 'player');
      await openInspectorTab(app.page, t, 'inspector.tab.look');
      await app.page.locator('#inspector').getByRole('button', { name: t('insp.cornersEach'), exact: true }).click();
    } },
    // The player's parts and its bar's options (D-2026-10-16-studio-redesign-fidelity-26).
    { name: 'player-progress', run: async (app) => {
      await standalone(app, 'player');
      await openInspectorTab(app.page, t, 'inspector.tab.look');
      const section = inspector(app).locator('[data-section="progress"]');
      await inspector(app).getByRole('button', { name: t('insp.partArtist'), exact: true }).click();
      await section.getByRole('button', { name: t('insp.patternDots'), exact: true }).click();
      await section.getByRole('button', { name: t('insp.progressKnob'), exact: true }).click({ force: true });
      await section.getByRole('button', { name: t('insp.timesSides'), exact: true }).click();
      await section.scrollIntoViewIfNeeded();
    } },
    { name: 'shape-corners', run: async (app) => {
      await standalone(app, 'shape');
      await app.page.locator('#inspector').getByRole('button', { name: t('insp.cornersEach'), exact: true }).click();
      await app.page.locator('#inspector .corner-radii').scrollIntoViewIfNeeded();
    } },
    { name: 'shape-corners-large', run: async (app) => {
      await standalone(app, 'shape');
      const inspector = app.page.locator('#inspector');
      const set = async (name, value) => { const field = inspector.getByRole('spinbutton', { name, exact: true }); await field.fill(String(value)); await field.press('Tab'); };
      await set(t('inspector.width'), 342); await set(t('inspector.height'), 432); await set(t('inspector.radius'), 24);
      await inspector.getByRole('button', { name: t('insp.cornersEach'), exact: true }).click();
      for (const [name, value] of [['topLeft', 0], ['topRight', 8], ['bottomRight', 24], ['bottomLeft', 16]]) await set(t(`inspector.${name}`), value);
      await inspector.getByRole('spinbutton', { name: t('inspector.topRight'), exact: true }).focus();
      await app.page.locator('#inspector .corner-radii').scrollIntoViewIfNeeded();
    } },
    { name: 'shape-radius-24', run: async (app) => {
      await standalone(app, 'shape');
      const inspector = app.page.locator('#inspector');
      const set = async (name, value) => { const field = inspector.getByRole('spinbutton', { name, exact: true }); await field.fill(String(value)); await field.press('Tab'); };
      await set(t('inspector.width'), 342); await set(t('inspector.height'), 432); await set(t('inspector.radius'), 24);
      await app.page.locator('#inspector .corner-radii').scrollIntoViewIfNeeded();
    } },
    { name: 'bar-look', run: async (app) => { await standalone(app, 'bar'); await openInspectorTab(app.page, t, 'inspector.tab.look'); } },
    { name: 'player-custom', run: async (app) => {
      await standalone(app, 'player');
      await app.page.locator('#inspector').getByRole('button', { name: t('insp.otherApp'), exact: true }).click();
    } },
    { name: 'player-stopped', run: async (app) => {
      await standalone(app, 'player');
      await app.page.locator('#inspector').getByRole('radio', { name: new RegExp(t('insp.stoppedHide')) }).check();
    } },
    // The shared pickers (D-2026-10-16-studio-redesign-fidelity-28, -29), open: the detail is the popover.
    { name: 'font-picker', capture: '.picker-popover', run: async (app) => {
      await standalone(app, 'text');
      await openInspectorTab(app.page, t, 'inspector.tab.look');
      await inspector(app).getByRole('button', { name: t('inspector.fontFamily'), exact: true }).click();
    } },
    { name: 'color-picker', capture: '.picker-popover', run: async (app) => {
      await standalone(app, 'shape');
      await inspector(app).getByRole('button', { name: t('inspector.fill'), exact: true }).click();
      await app.page.locator('.picker-popover [data-section="theme"]').getByRole('button', { name: t('common.save'), exact: true }).click();
    } },
  ].map(state => ({ ...state, capture: state.capture ?? '#inspector' })),
};
