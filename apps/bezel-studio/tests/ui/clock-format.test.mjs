import test from 'node:test';
import assert from 'node:assert/strict';
import { CLOCK_PATTERNS, formatClock } from '../../src/clock-format.js';
const date = new Date(2026, 9, 5, 14, 30, 45);
test('date format, explicit language, system language and Unicode case', () => {
  assert.equal(formatClock('%A %e %B', date, 'it'), 'lunedì 5 ottobre');
  assert.equal(formatClock('%A %B', date, null, 'normal', 'it-IT'), 'lunedì ottobre');
  assert.equal(formatClock('%a %b', date, 'it', 'upper'), 'LUN OTT');
  assert.equal(formatClock('%A %e %B', date, 'it', 'title'), 'Lunedì 5 Ottobre');
  assert.equal(formatClock('%A %e %B', date, 'en'), 'Monday 5 October');
  assert.equal(formatClock('%A %B', date, 'pt-BR'), 'segunda-feira outubro');
  assert.equal(formatClock('%H:%M:%S %I %p %d/%m/%Y %y %% %q %', date, 'it'), '14:30:45 02 PM 05/10/2026 26 % %q %');
  assert.equal(formatClock('%I %p', new Date(2026, 0, 1, 0), 'it'), '12 AM');
  assert.equal(formatClock('%A', date, null, 'normal', 'fr'), 'Monday');
  assert.ok(CLOCK_PATTERNS.includes('%A %e %B'));
  assert.ok(formatClock('%H:%M'));
});
test('a clock is cut into hours, separators, minutes and seconds like the native one', async () => {
  const { clockParts } = await import('../../src/clock-format.js');
  const at = new Date(2026, 8, 30, 21, 5, 9);
  assert.deepEqual(clockParts('%H:%M:%S', at, 'en'), [
    { part: 'hours', text: '21' }, { part: 'separators', text: ':' }, { part: 'minutes', text: '05' },
    { part: 'separators', text: ':' }, { part: 'seconds', text: '09' },
  ]);
  assert.deepEqual(clockParts('at %I.%M %p', at, 'en'), [
    { part: null, text: 'at ' }, { part: 'hours', text: '09' }, { part: 'separators', text: '.' },
    { part: 'minutes', text: '05' }, { part: null, text: ' PM' },
  ]);
  assert.deepEqual(clockParts('%H%a%M', at, 'en'), [{ part: 'hours', text: '21' }, { part: null, text: 'Wed' }, { part: 'minutes', text: '05' }]);
  assert.deepEqual(clockParts('%d/%m', at, 'en'), [{ part: null, text: '30/09' }]);
  assert.deepEqual(clockParts('', at, 'en'), []);
  assert.deepEqual(clockParts('%a', at, 'it', 'upper'), [{ part: null, text: 'MER' }]);
  for (const pattern of ['%H:%M:%S', '%A %e %B, %I:%M %p', '100%% %q %']) {
    assert.equal(clockParts(pattern, at, 'en').map((p) => p.text).join(''), formatClock(pattern, at, 'en'));
  }
  assert.ok(clockParts('%H:%M').length === 3);
});
