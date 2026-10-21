import { test } from 'node:test';
import assert from 'node:assert/strict';
import { translator } from '../../src/i18n/index.js';
import {
  PROFILE_EFFECTS, directed, firstIncomplete, minuteOf, validProfileRule, modeOf, newProfileRule, orientationOf, profileIcon, profileSummary, ruleSummary, triedThemes, withMode, withoutProfile,
} from '../../src/profiles-model.js';

const t = translator('it');
const music = { id: 'music', name: 'Music Player', themes: { COM6: 'r.bezeltheme' }, rules: [{ source: 'mediaPlaying', app: '', priority: 10, returnSeconds: 3 }] };
const gaming = { id: 'gaming', name: 'Gaming', themes: { COM6: 'g.bezeltheme' }, rules: [{ source: 'foreground', app: 'steam.exe', priority: 30, returnSeconds: 5 }] };
const config = (extra = {}) => ({ transition: { effect: 'wipe', direction: 'down', durationMs: 600 }, profiles: [music, gaming], automatic: true, manual: null, defaultProfile: null, ...extra });

test('the two modes read the saved selection', () => {
  assert.deepEqual(modeOf(config()), { mode: 'auto', manual: null, fallback: null });
  assert.deepEqual(modeOf(config({ defaultProfile: 'music' })), { mode: 'auto', manual: null, fallback: 'music' });
  // The former Default: manual, no profile.
  assert.deepEqual(modeOf(config({ automatic: false })), { mode: 'manual', manual: null, fallback: null });
  assert.deepEqual(modeOf(config({ automatic: false, manual: 'gaming' })), { mode: 'manual', manual: 'gaming', fallback: null });
});

test('choosing a mode writes automatic, manual and defaultProfile', () => {
  const manual = withMode(config({ defaultProfile: 'music' }), { mode: 'manual', manual: 'gaming' });
  assert.deepEqual([manual.automatic, manual.manual, manual.defaultProfile], [false, 'gaming', 'music']);
  const back = withMode(manual, { mode: 'auto' });
  assert.deepEqual([back.automatic, back.manual, back.defaultProfile], [true, null, 'music']);
  const fallback = withMode(back, { mode: 'auto', fallback: null });
  assert.equal(fallback.defaultProfile, null);
  // Unknown profiles are never written.
  const unknown = withMode(config(), { mode: 'manual', manual: 'gone', fallback: 'gone' });
  assert.deepEqual([unknown.manual, unknown.defaultProfile], [null, null]);
  assert.equal(unknown.transition.effect, 'wipe');
});

test('removing a profile clears the choices that named it', () => {
  const next = withoutProfile(config({ automatic: false, manual: 'gaming', defaultProfile: 'gaming' }), 'gaming');
  assert.deepEqual(next.profiles.map((p) => p.id), ['music']);
  assert.deepEqual([next.manual, next.defaultProfile], [null, null]);
});

test('a rule reads as a sentence and the list shows the first one', () => {
  assert.equal(ruleSummary(t, music.rules[0]), "quando c'è musica in riproduzione");
  assert.equal(ruleSummary(t, { ...music.rules[0], app: 'spotify.exe' }), 'quando spotify.exe riproduce musica');
  assert.equal(ruleSummary(t, gaming.rules[0]), 'quando steam.exe è in primo piano');
  assert.equal(ruleSummary(t, { source: 'process', app: '' }), 'quando … è in esecuzione');
  assert.equal(profileSummary(t, gaming), 'quando steam.exe è in primo piano');
  assert.equal(profileSummary(t, { ...gaming, rules: [...gaming.rules, music.rules[0], music.rules[0]] }), 'quando steam.exe è in primo piano · altre 2');
  assert.equal(profileSummary(t, { ...gaming, rules: [] }), 'nessuna regola · solo manuale');
  assert.equal(profileSummary(translator('en'), music), 'when music is playing');
});

test('icons, orientations and directions follow the profile and the theme', () => {
  assert.deepEqual([profileIcon(music), profileIcon(gaming), profileIcon({ rules: [] })], ['player', 'chip', 'themes']);
  assert.equal(orientationOf({ canvas: { width: 1920, height: 480 } }), 'landscape');
  assert.equal(orientationOf({ canvas: { width: 320, height: 480 } }), 'portrait');
  assert.equal(orientationOf(undefined), 'portrait');
  assert.deepEqual(PROFILE_EFFECTS.filter(directed), ['curtain', 'wipe', 'blinds']);
});

test('▶ tries the selected profile, or the base themes', () => {
  const displays = [{ key: 'COM6', theme: 'base-wide.bezeltheme' }, { key: 'COM3', theme: null }];
  assert.deepEqual(triedThemes(config(), 'music', displays), { COM6: 'r.bezeltheme' });
  assert.deepEqual(triedThemes(config(), 'base', displays), { COM6: 'base-wide.bezeltheme' });
  assert.deepEqual(triedThemes(config({ profiles: [{ ...music, themes: { COM6: '' } }] }), 'music', displays), {});
});

test('the first incomplete profile is found before saving', () => {
  const displays = [{ key: 'COM6', connected: true }, { key: 'COM3', connected: false }];
  assert.equal(firstIncomplete(config(), displays), null);
  assert.equal(firstIncomplete(config({ profiles: [music, { ...gaming, name: ' ' }] }), displays), 'gaming');
  assert.equal(firstIncomplete(config({ profiles: [{ ...music, themes: {} }, gaming] }), displays), 'music');
  assert.equal(firstIncomplete(config({ profiles: [music, { ...gaming, rules: [{ ...gaming.rules[0], app: '' }] }] }), displays), 'gaming');
  assert.equal(firstIncomplete(config({ profiles: [{ ...music, rules: [{ ...music.rules[0], priority: 101 }] }] }), displays), 'music');
  // A disconnected display keeps its theme or none.
  assert.equal(firstIncomplete(config(), [{ key: 'COM9', connected: false }]), null);
});

test('new rule types validate their own fields and preserve default any', () => {
  const rule = (source, extra = {}) => ({ source, priority: 10, returnSeconds: 3, ...extra });
  const sensor = rule('sensor', { key: 'cpu.temperature', compare: 'above', value: 80 });
  const time = rule('time', { from: '22:00', to: '06:00', days: [6] });
  for (const r of [sensor, time, rule('idle',{minutes:2}),rule('locked')]) {
    assert.equal(validProfileRule(r),true);
    assert.equal(firstIncomplete(config({profiles:[{...music,rules:[r],match:'all'}]}),[]),null);
    assert.equal(validProfileRule({...r,priority:101}),false);
    assert.equal(validProfileRule({...r,returnSeconds:301}),false);
  }
  for (const patch of [{key:''},{key:'bad key'},{compare:'equals'},{value:NaN},{forSeconds:-1},{forSeconds:3601},{forSeconds:null}]) assert.equal(validProfileRule({...sensor,...patch}),false);
  for (const patch of [{from:'24:00'},{to:'06:60'},{from:'6:00'},{days:[7]},{days:Array(8).fill(0)},{days:'Monday'}]) assert.equal(validProfileRule({...time,...patch}),false);
  for (const minutes of [0,1441,1.5]) assert.equal(validProfileRule(rule('idle',{minutes})),false);
  assert.equal(validProfileRule(rule('future')),false);
  assert.equal(validProfileRule(rule('locked',{app:1})),false);
  assert.equal(validProfileRule(rule('locked',{app:null})),false);
  assert.equal(validProfileRule({...time,days:null}),false);
  assert.equal(validProfileRule(rule('mediaPlaying',{app:'x'.repeat(257)})),false);
  assert.equal(firstIncomplete(config({profiles:[{...music,match:'future'}]}),[]),'music');
  assert.equal(firstIncomplete(config({profiles:[{...music,match:null}]}),[]),'music');
  assert.equal(minuteOf(null),null);
  assert.equal(minuteOf('23:59'),1439);
});

test('all locales describe the new rule types without technical fallback text', () => {
  const rules=[{source:'sensor',key:'cpu.temperature',compare:'above',value:80,forSeconds:2},
    {source:'time',days:[0,6],from:'22:00',to:'06:00'},{source:'idle',minutes:5},{source:'locked'}];
  for (const locale of ['en','it','pt-BR']) for (const rule of rules) {
    const summary=ruleSummary(translator(locale),rule);
    assert.ok(summary.length>10);
    assert.doesNotMatch(summary,/profileRules\.|\{\w+\}/);
  }
  assert.match(ruleSummary(t,{source:'time',from:'00:00',to:'23:00'}),/ogni giorno/);
  assert.equal(ruleSummary(t,{source:'sensor',key:'x',compare:'below',value:1}), 'quando x resta sotto 1 per 0 s');
  assert.deepEqual(rules.map(r=>profileIcon({rules:[r]})),['sliders','clock','clock','lock']);
});

test('new rule sources start with distinct native defaults and allow keeping other screens unchanged', () => {
  const sensors=[{key:'system.hostname',quantity:'text'},{key:'cpu.temperature',quantity:'celsius'}];
  assert.equal(newProfileRule('sensor',sensors).key,'cpu.temperature');
  for(const source of ['sensor','time','idle','locked','mediaPlaying']) assert.equal(validProfileRule(newProfileRule(source,sensors)),true);
  assert.equal(validProfileRule(newProfileRule('foreground')),false);
  const first=newProfileRule('time'); first.days.push(1); assert.deepEqual(newProfileRule('time').days,[]);
  assert.equal(firstIncomplete(config(),[{key:'COM6',connected:true},{key:'COM8',connected:true}]),null);
});
