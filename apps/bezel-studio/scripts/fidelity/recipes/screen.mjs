// The screen panel and the sensor status (artboards Screen, Storage, SensorStatus).
// The legacy artboards draw the panel 360px wide: the Settings states widen the
// library panel to that width (its resizer's own key), so the two compare.
const widen = async (app) => {
  const handle = app.page.locator('#resize-library');
  await handle.focus();
  for (let i = 0; i < 11; i += 1) await handle.press('ArrowRight');
  await handle.blur();
  await app.page.waitForTimeout(300);
};

const settings = async (app) => {
  await app.tab('screen');
  await widen(app);
  await app.page.locator('label.live-toggle').click();
  await app.page.locator('#screen-panel').getByRole('button', { name: await app.page.locator('#screen-panel .toggle-title').textContent(), exact: true }).click();
  await app.click('.standby-radio[data-choice=off]');
  await app.click('.standby-dialog .dialog-actions .primary-button');
  await app.page.locator('#toast').waitFor({ state: 'hidden' });
};

const storage = async (app) => {
  await app.tab('screen');
  await app.click('#subtab-storage');
};

export default {
  area: 'screen',
  artboards: ['Screen', 'Storage', 'SensorStatus'],
  states: [
    ...['settings', 'storage'].map(state => ({ name: `ux2-${state}`, capture: '#panel-screen', run: async app => {
      await app.page.evaluate(() => localStorage.setItem('bezel.panelWidths', JSON.stringify({ library: 380, inspector: 360 })));
      await app.page.reload();
      await app.tab('screen');
      if (state === 'storage') {
        await app.click('#subtab-storage');
        await app.page.locator('#storage-panel [data-inspector-section="storage-maintenance"]').waitFor();
      }
      await app.page.setViewportSize({ width: 1280, height: 2000 });
      await app.page.locator('.statusbar').evaluate(node => { node.style.visibility = 'hidden'; });
      await app.page.locator('#panel-screen').evaluate(node => { node.style.height = `${node.scrollHeight}px`; node.style.maxHeight = 'none'; node.style.flex = 'none'; node.scrollTop = 0; });
    } })),
    { name: 'settings', run: settings },
    { name: 'settings-advanced', run: async (app) => { await settings(app); await app.page.locator('.screen-tail').scrollIntoViewIfNeeded(); } },
    { name: 'storage', run: storage },
    { name: 'storage-boot', demo: 'vendorCard', run: storage },
    { name: 'storage-boot-drop', run: async (app) => {
      await storage(app);
      await app.page.locator('.manager-column[data-medium=internal] [role=option]').first().hover();
      await app.page.mouse.down();
      await app.page.locator('.boot-slot').hover();
    } },
    { name: 'storage-sd', run: async (app) => { await storage(app); await app.click('[data-focus=place-sd]'); } },
    { name: 'sensor-ok', run: (app) => app.click('#status-libre-toggle') },
    { name: 'sensor-status', query: '&libre=partial', run: (app) => app.click('#status-libre-toggle') },
    { name: 'sensor-status-error', query: '&libre=error', run: (app) => app.click('#status-libre-toggle') },
    { name: 'sensor-chip-error', query: '&libre=error', run: async () => {} },
    { name: 'sensor-restarting', query: '&libre=error', run: async (app) => {
      // Freeze timers before invoking the real demo bridge, so its pending
      // restart can be photographed without extending production delays.
      await app.page.clock.install({ time: new Date('2026-10-09T10:00:00Z') });
      await app.page.clock.pauseAt(new Date('2026-10-09T10:00:01Z'));
      await app.page.locator('#status-libre-restart').click();
    } },
  ].map(state => ({ ...state, ...(state.name.startsWith('settings') ? { capture: '#screen-device' } : state.name.startsWith('storage') ? { capture: '#storage-panel' } : {}) })),
};
