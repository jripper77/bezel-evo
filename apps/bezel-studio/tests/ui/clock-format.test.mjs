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

test('an analog dial: defaults, hands, frames, numbers and date like the backend', async () => {
  const { analogOf, dialDate, dialDue, dialNumerals, handAngles } = await import('../../src/clock-format.js');
  const defaults = analogOf(undefined);
  assert.deepEqual([defaults.ticks, defaults.numbers, defaults.handShape, defaults.border, defaults.sweep, defaults.date], ['hours', 'none', 'rounded', 2, false, 'none']);
  assert.ok(defaults.hourHand && defaults.minuteHand && defaults.secondHand);
  assert.equal(analogOf({ numbers: 'roman' }).numbers, 'roman');
  const at = (h, m, s, ms = 0) => new Date(2026, 9, 20, h, m, s, ms);
  assert.deepEqual(handAngles({}, at(15, 0, 0)), { hour: 90, minute: 0, second: 0 });
  assert.deepEqual(handAngles({}, at(6, 30, 0)), { hour: 195, minute: 180, second: 0 });
  const late = handAngles({}, at(21, 45, 30, 500));
  assert.ok(Math.abs(late.hour - 292.75) < 1e-9 && Math.abs(late.minute - 273) < 1e-9 && late.second === 180);
  assert.equal(handAngles({ sweep: true }, at(21, 45, 30, 500)).second, 183);
  assert.equal(handAngles({ sweep: true }, at(21, 45, 30, 500), false).second, 180, 'reduced motion steps');
  assert.equal(dialDue({}, at(1, 2, 3, 250)), 750);
  assert.equal(dialDue({ sweep: true }, at(1, 2, 3, 250)), 33);
  assert.equal(dialDue({ sweep: true }, at(1, 2, 3, 250), false), 750);
  assert.equal(dialDue({ secondHand: false }, at(1, 2, 3, 250)), null);
  assert.deepEqual(dialNumerals({}), []);
  assert.deepEqual(dialNumerals({ numbers: 'quarters' }).map(([, t]) => t), ['3', '6', '9', '12']);
  assert.equal(dialNumerals({ numbers: 'all' }).length, 12);
  assert.deepEqual(dialNumerals({ numbers: 'roman' })[3], [4, 'IV']);
  assert.equal(dialDate({}, at(1, 0, 0), 'it'), '');
  assert.equal(dialDate({ date: 'day' }, at(1, 0, 0), 'it'), '20');
  assert.equal(dialDate({ date: 'dayMonth' }, at(1, 0, 0), 'it'), '20 ott');
});


test('hand length settings normalize like native and original lengths stay absent', async () => {
  const { analogOf, withHandLength } = await import('../../src/clock-format.js');
  for (const hand of ['hour', 'minute', 'second']) {
    const key = `${hand}Length`;
    assert.equal(analogOf({})[key], 1);
    const source = { sweep: true };
    const changed = withHandLength(source, hand, .5);
    assert.deepEqual(changed, { sweep: true, [key]: .5 }); assert.deepEqual(source, { sweep: true });
    assert.deepEqual(withHandLength(changed, hand, 1), source);
    assert.deepEqual(withHandLength(undefined, hand, 1), {});
    for (const value of [null, undefined, NaN, Infinity]) {
      assert.equal(analogOf({ [key]: value })[key], 1);
      assert.deepEqual(withHandLength(changed, hand, value), source);
    }
    assert.equal(analogOf({ [key]: 0 })[key], .1);
    assert.equal(analogOf({ [key]: 3 })[key], 2);
    assert.equal(withHandLength({}, hand, 0)[key], .1);
    assert.equal(withHandLength({}, hand, 3)[key], 2);
  }
});
