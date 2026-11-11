import test from 'node:test';
import assert from 'node:assert/strict';
import { PREFERENCE_PAGES, nextPage, updateStatus } from '../../src/ui/preferences.js';

test('the pages of the Preferences come in order and the arrows wrap around', () => {
  assert.deepEqual(PREFERENCE_PAGES.map((page) => page.id), ['general', 'screens', 'audio', 'sensors', 'updates', 'diagnostics']);
  assert.equal(nextPage('general', 1), 'screens');
  assert.equal(nextPage('general', -1), 'diagnostics');
  assert.equal(nextPage('diagnostics', 1), 'general');
  assert.equal(nextPage('unknown', 1), 'screens', 'an unknown page counts as the first');
  // Every section is on exactly one page.
  const sections = PREFERENCE_PAGES.flatMap((page) => page.sections);
  assert.equal(new Set(sections).size, sections.length);
});

test('the update status says never, newer or up to date, with when it ran', () => {
  const when = (ms) => `at ${ms}`;
  assert.deepEqual(updateStatus({ updateCheckedAt: null }, when), { key: 'prefs.updateNever', params: {} });
  assert.deepEqual(updateStatus({ updateCheckedAt: 2, updateAvailable: '0.3.0', updateLatest: '0.3.0', version: '0.2.28' }, when),
    { key: 'prefs.updateLastChecked', params: { when: 'at 2000' } });
  assert.deepEqual(updateStatus({ updateCheckedAt: 2, updateAvailable: null, updateLatest: '0.2.28', version: '0.2.28' }, when),
    { key: 'prefs.updateStatusLatest', params: { when: 'at 2000', version: '0.2.28' } });
  assert.equal(updateStatus({ updateCheckedAt: 2, updateLatest: null, version: '0.2.28' }, when).params.version, '0.2.28',
    'an answer without a version names the running one');
});
