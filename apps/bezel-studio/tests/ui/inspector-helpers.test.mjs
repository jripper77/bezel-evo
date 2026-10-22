import { test } from 'node:test';
import assert from 'node:assert/strict';
import { DURATION_UNITS, GRAPH_INTERVAL_MAX, HISTORY_MAX, HISTORY_MIN, INTERVAL_UNITS, QUICK_DURATIONS, QUICK_REFRESHES, RULER_TICKS_MAX, TIME_UNITS, appChips, appName, arcDrawing, durationParts, graphRuler, graphSentence, graphTiming, graphTimingPatch, graphWindowRange, intervalParts, pluralOf, layerFacts, quickDurations, refreshChoices } from '../../src/inspector-helpers.js';

const theme = {
  elements: [
    { id: 1, name: 'Background' },
    { id: 2, name: 'Net', isGroup: true },
    { id: 3, name: 'Download', groupParent: 2 },
    { id: 4, name: 'Faces', card: {} },
    { id: 5, name: 'Title', cardMember: { parent: 4, face: 0 } },
    { id: 6, name: 'Lost', groupParent: 99 },
  ],
};

test('the head tells the level among all objects, back to front, and the group or card', () => {
  assert.deepEqual(layerFacts(theme, theme.elements[0]), { level: 1, total: 6, group: null });
  assert.deepEqual(layerFacts(theme, theme.elements[2]), { level: 3, total: 6, group: 'Net' });
  assert.deepEqual(layerFacts(theme, theme.elements[4]), { level: 5, total: 6, group: 'Faces' });
  assert.deepEqual(layerFacts(theme, theme.elements[5]), { level: 6, total: 6, group: null });
});

test('a duration is told in its largest whole unit', () => {
  assert.deepEqual(durationParts(30), { value: 30, unit: 's' });
  assert.deepEqual(durationParts(60), { value: 1, unit: 'min' });
  assert.deepEqual(durationParts(900), { value: 15, unit: 'min' });
  assert.deepEqual(durationParts(3600), { value: 1, unit: 'h' });
  assert.deepEqual(durationParts(90), { value: 90, unit: 's' });
  assert.deepEqual(durationParts(7200), { value: 2, unit: 'h' });
  assert.deepEqual(durationParts(1.256), { value: 1.26, unit: 's' });
});

test('a graph times itself by its own interval, else by the theme refresh (D-16)', () => {
  assert.deepEqual(graphTiming({ history: 120, intervalSeconds: 30 }, 1), { interval: 30, own: true, points: 120, duration: 3600 });
  assert.deepEqual(graphTiming({ history: 60 }, 2), { interval: 2, own: false, points: 60, duration: 120 });
  assert.deepEqual(graphTiming({ history: 60, intervalSeconds: 0 }, 0), { interval: 1, own: false, points: 60, duration: 60 });
  assert.equal(graphTiming({ history: 2, intervalSeconds: 99999 }, 1).interval, GRAPH_INTERVAL_MAX);
});

test('a duration and an interval make the points, bounded and said so', () => {
  // "1 hour, one point every 30 s".
  assert.deepEqual(graphTimingPatch(3600, 30), { history: 120, intervalSeconds: 30, duration: 3600, clamped: null });
  assert.deepEqual(graphTimingPatch(7200, 1), { history: HISTORY_MAX, intervalSeconds: 1, duration: 3600, clamped: 'max' });
  assert.deepEqual(graphTimingPatch(30, 60), { history: HISTORY_MIN, intervalSeconds: 60, duration: 120, clamped: 'min' });
  assert.deepEqual(graphTimingPatch(90, 0.4), { history: 90, intervalSeconds: 1, duration: 90, clamped: null }, 'whole seconds, at least one');
  assert.equal(graphTimingPatch(60, 1e9).intervalSeconds, GRAPH_INTERVAL_MAX);
  assert.deepEqual(graphTimingPatch(Number.NaN, Number.NaN), { history: HISTORY_MIN, intervalSeconds: 1, duration: 2, clamped: 'min' });
  assert.deepEqual(graphWindowRange(30), { min: 60, max: 108000 });
  assert.deepEqual(graphWindowRange(0), { min: 2, max: 3600 });
  assert.deepEqual(TIME_UNITS, { s: 1, min: 60, h: 3600 });
});

test('an interval is told in seconds or whole minutes', () => {
  assert.deepEqual(intervalParts(30), { value: 30, unit: 's' });
  assert.deepEqual(intervalParts(120), { value: 2, unit: 'min' });
  assert.deepEqual(intervalParts(3600), { value: 60, unit: 'min' });
  assert.deepEqual(intervalParts(90), { value: 90, unit: 's' });
  assert.deepEqual(intervalParts(0.5), { value: 0.5, unit: 's' });
});

test('the window sentence tells the duration and the interval in whole units, the words agreeing with them', () => {
  assert.deepEqual(DURATION_UNITS, ['s', 'min', 'h']);
  assert.deepEqual(INTERVAL_UNITS, ['s', 'min']);
  assert.equal(pluralOf(1), 'one');
  assert.equal(pluralOf(2), 'other');
  assert.equal(pluralOf(1.5), 'other');
  // One hour, a point every 30 s: "the last [1] [hour], one point every [30] [seconds]".
  assert.deepEqual(graphSentence({ duration: 3600, interval: 30 }), {
    last: 'insp.lastHour', duration: { value: 1, unit: 'h', plural: 'one' }, interval: { value: 30, unit: 's', plural: 'other' },
  });
  assert.deepEqual(graphSentence({ duration: 7200, interval: 60 }), {
    last: 'insp.lastHours', duration: { value: 2, unit: 'h', plural: 'other' }, interval: { value: 1, unit: 'min', plural: 'one' },
  });
  assert.equal(graphSentence({ duration: 60, interval: 1 }).last, 'insp.lastOne');
  assert.deepEqual(graphSentence({ duration: 60, interval: 1 }).duration, { value: 1, unit: 'min', plural: 'one' });
  assert.equal(graphSentence({ duration: 300, interval: 1 }).last, 'insp.lastMany');
  // Hours are no unit of the interval: an hour between two points reads as 60 minutes.
  assert.deepEqual(graphSentence({ duration: 7200, interval: 3600 }).interval, { value: 60, unit: 'min', plural: 'other' });
  assert.deepEqual(graphSentence({ duration: 90, interval: 1.5 }).duration, { value: 90, unit: 's', plural: 'other' });
});

test('the time ruler marks every point up to its limit, then one in so many, counted from now', () => {
  assert.equal(RULER_TICKS_MAX, 60);
  assert.deepEqual(graphRuler(4), { ticks: [0.25, 0.5, 0.75, 1], every: 1 });
  const hour = graphRuler(120);
  assert.equal(hour.every, 2);
  assert.equal(hour.ticks.length, 60);
  assert.deepEqual([hour.ticks[0], hour.ticks.at(-1)], [0.0167, 1]);
  const most = graphRuler(3600);
  assert.equal(most.every, 60);
  assert.equal(most.ticks.length, 60);
  assert.equal(graphRuler(61).ticks.length, 31);
  assert.ok(graphRuler(61).ticks.every((x, i, all) => i === 0 || x > all[i - 1]), 'ascending');
  assert.deepEqual(graphRuler(10, 5), { ticks: [0.2, 0.4, 0.6, 0.8, 1], every: 2 });
  assert.deepEqual(graphRuler(0), { ticks: [1], every: 1 });
  assert.deepEqual(graphRuler(3, 0), { ticks: [1], every: 3 });
});

test('quick durations turn into points at the graph interval, the current one pressed', () => {
  assert.deepEqual(QUICK_DURATIONS, [30, 60, 300, 900, 3600]);
  assert.deepEqual(quickDurations(1, 60).map((d) => [d.samples, d.pressed, d.disabled]),
    [[30, false, false], [60, true, false], [300, false, false], [900, false, false], [3600, false, false]]);
  assert.deepEqual(quickDurations(0.5, 120).map((d) => [d.samples, d.pressed, d.disabled]),
    [[60, false, false], [120, true, false], [600, false, false], [1800, false, false], [7200, false, true]]);
  assert.equal(quickDurations(0, 30)[0].samples, 30);
  assert.equal(quickDurations(40, 2)[0].disabled, true);
  assert.deepEqual(quickDurations(30, 120).map((d) => [d.samples, d.pressed, d.disabled]),
    [[1, false, true], [2, false, false], [10, false, false], [30, false, false], [120, true, false]]);
  assert.equal(HISTORY_MIN, 2);
  assert.equal(HISTORY_MAX, 3600);
});

test('the arc drawing starts where the renderer starts and turns its way', () => {
  const ring = arcDrawing(-135, 270, true);
  assert.deepEqual(ring.start, { x: 15.03, y: 48.97 });
  assert.equal(ring.d, 'M15.03 48.97A24 24 0 1 1 48.97 48.97');
  assert.equal(ring.full, false);
  assert.equal(arcDrawing(0, 90, false).d, 'M32 8A24 24 0 0 0 8 32');
  assert.equal(arcDrawing(0, 90, true).d, 'M32 8A24 24 0 0 1 56 32');
  const full = arcDrawing(0, 400);
  assert.equal(full.full, true);
  assert.equal(full.d, 'M32 8A24 24 0 1 1 32 56A24 24 0 1 1 32 8');
  assert.equal(arcDrawing(0, 'x').d, 'M32 8A24 24 0 0 1 32 8');
  assert.deepEqual(arcDrawing(90, 90, true, { size: 20, radius: 10 }).start, { x: 20, y: 10 });
});

test('an application name drops its folder and .exe', () => {
  assert.equal(appName('Spotify.exe'), 'Spotify');
  assert.equal(appName('C:\\Program Files\\VLC\\vlc.EXE'), 'vlc');
  assert.equal(appName('/usr/bin/firefox'), 'firefox');
  assert.equal(appName(null), '');
});

test('the app chips list each media app once, playing ones first, the followed one pressed', () => {
  const activity = {
    media: [
      { source: 'chrome', playing: false },
      { source: 'Spotify.exe', playing: true },
      { source: 'spotify', playing: false },
      { source: 'vlc.exe', playing: false },
      { source: '', playing: true },
    ],
  };
  assert.deepEqual(appChips(activity, ''), {
    chips: [
      { name: 'Spotify', playing: true, pressed: false },
      { name: 'chrome', playing: false, pressed: false },
      { name: 'vlc', playing: false, pressed: false },
    ],
    custom: null,
  });
  assert.deepEqual(appChips(activity, 'SPOTIFY').chips.map((c) => c.pressed), [true, false, false]);
  assert.equal(appChips(activity, ' Foobar ').custom, 'Foobar');
  assert.deepEqual(appChips(null, 'foobar'), { chips: [], custom: 'foobar' });
  assert.deepEqual(appChips({ media: [{ source: 'chrome', playing: false }, { source: 'Chrome.exe', playing: true }] }).chips,
    [{ name: 'chrome', playing: true, pressed: false }]);
  assert.deepEqual(appChips(undefined), { chips: [], custom: null });
});

test('the theme refresh offers quick values and keeps a value of its own', () => {
  assert.deepEqual(QUICK_REFRESHES, [0.5, 1, 2, 5]);
  const shown = (current, min) => refreshChoices(current, min).map((c) => `${c.seconds}${c.pressed ? '*' : ''}`);
  assert.deepEqual(shown(1, 0.5), ['0.5', '1*', '2', '5']);
  assert.deepEqual(shown(1, 1), ['1*', '2', '5'], 'faster than the backend allows: not offered');
  assert.deepEqual(shown(3, 1), ['1', '2', '3*', '5'], 'another value saved in the theme stays, in order');
  assert.deepEqual(shown(10), ['0.5', '1', '2', '5', '10*']);
  assert.deepEqual(shown(0, 1), ['1', '2', '5'], 'no value: none pressed');
  assert.deepEqual(shown(undefined), ['0.5', '1', '2', '5']);
});
