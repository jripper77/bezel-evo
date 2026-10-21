import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createInspectorTrial, trialAt, trialDuration, TRIAL_FRAME_MS, sceneTrialAt, heldTrial, SCENE_TRIAL_MS } from '../../src/inspector-trial.js';
import { weatherIconPaths } from '../../src/weather-icon.js';
import { weatherFamily } from '../../src/weather-format.js';

const ring = { id: 1, kind: { type: 'ring' } };
const weather = { id: 2, kind: { type: 'text', content: { type: 'weather' } } };

test('trial timing matches native: full scale halfway, eight conditions, hard end', () => {
  assert.equal(TRIAL_FRAME_MS, 33);
  assert.equal(trialDuration(ring), 3000);
  assert.equal(trialDuration(weather), 8000);
  assert.equal(trialDuration(null), 3000);
  assert.equal(trialAt(ring, 0).pulse, 0);
  assert.equal(trialAt(ring, 1500).pulse, 1);
  assert.ok(trialAt(ring, 2999).pulse < .00001);
  assert.equal(trialAt(ring, 3000), null);
  assert.equal(trialAt(weather, 8000), null);
  assert.equal(trialAt(weather, 7999).weatherCode, 999);
  assert.deepEqual(Array.from({ length: 8 }, (_, i) => weatherFamily(trialAt(weather, i * 1000).weatherCode)), [0,1,2,3,4,5,6,7]);
  assert.equal(trialAt(ring, 1500).clockOffsetSeconds, 43200);
  assert.equal(trialAt(ring, 1500).elementId, 1);
  assert.equal(trialAt(ring, 1500).elapsedMs, 1500);
});

test('reduced motion and invalid clocks never start a trial', () => {
  for (const ms of [null, undefined, NaN, Infinity, -1, 1.5, '1500']) assert.equal(trialAt(ring, ms), null);
  assert.equal(trialAt(null, 0), null);
  assert.equal(trialAt(ring, 0, false), null);
});

test('Try restarts on demand, cancels on selection, completion, reduced motion or stop', () => {
  let now = 100;
  const trial = createInspectorTrial({ now: () => now });
  assert.equal(trial.at(ring), null);
  trial.start(ring); assert.equal(trial.at(ring), 0);
  now += 1500; assert.equal(trial.at(ring), 1500);
  trial.start(ring); assert.equal(trial.at(ring), 0);
  now += 3000; assert.equal(trial.at(ring), null);
  trial.start(ring); assert.equal(trial.at(weather), null); assert.equal(trial.at(ring), null);
  trial.start(ring); assert.equal(trial.at(null), null);
  trial.start(ring); assert.equal(trial.at(ring, false), null); assert.equal(trial.at(ring), null);
  trial.start(ring); trial.stop(); assert.equal(trial.at(ring), null);
  trial.start(ring); now -= 5; assert.equal(trial.at(ring), 0);
  const realClock = createInspectorTrial(); realClock.start(ring); assert.ok(realClock.at(ring) >= 0);
});

test('all static weather families have distinct native geometry in monochrome and color', () => {
  const codes = [0,2,3,45,61,71,95,999];
  for (const colored of [false, true]) {
    const icons = codes.map(code => weatherIconPaths(code, colored));
    assert.equal(new Set(icons.map(parts => parts.map(p => p.path).join(''))).size, 8);
    assert.ok(icons.every(parts => parts.every(p => p.path.startsWith('M') && !/NaN|undefined/.test(p.path))));
    if (colored) assert.ok(icons.every(parts => parts.every(p => /^#[0-9a-f]{6}$/.test(p.light) && /^#[0-9a-f]{6}$/.test(p.dark))));
    else assert.ok(icons.every(parts => parts.length === 1));
  }
  const sun = weatherIconPaths(0, true);
  assert.match(sun[0].path, /^M16 9A4 4/);
  assert.match(weatherIconPaths(0)[0].path, /^M16 10A4 4/);
  assert.equal(weatherIconPaths(3, true)[0].path, 'M5 16C-1 16 1 8 7 10C8 3 19 5 18 11C24 10 24 16 19 16L5 16');
  assert.equal(weatherIconPaths(999, true)[0].fill, false);
});

test('the canvas trial values sweep from the minimum, repeat every 24 s and hold one sensor', () => {
  const start = sceneTrialAt(0), top = sceneTrialAt(1500);
  assert.equal(start.scene, true); assert.equal(start.pulse, 0); assert.equal(top.pulse, 1);
  assert.equal(top.clockOffsetSeconds, 43200);
  assert.deepEqual(sceneTrialAt(SCENE_TRIAL_MS + 1500), top);
  assert.deepEqual(sceneTrialAt(-SCENE_TRIAL_MS + 1500), top, 'never a negative clock');
  assert.deepEqual([0, 7000, 8000, 9000].map(ms => sceneTrialAt(ms).weatherCode), [0, 999, 0, 2]);
  assert.deepEqual(heldTrial('cpu.usage', .25), { scene: true, elapsedMs: 0, hold: { key: 'cpu.usage', fraction: .25 } });
  assert.deepEqual([heldTrial('k', 2).hold.fraction, heldTrial('k', -1).hold.fraction, heldTrial('k', NaN).hold.fraction], [1, 0, 0]);
});
