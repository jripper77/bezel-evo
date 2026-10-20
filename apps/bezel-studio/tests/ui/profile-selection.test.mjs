import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createProfileSelection, profileRuleAvailable, profileRuleHolds } from '../../src/profile-selection.js';
import { createDemoBackend } from '../../src/demo-backend.js';

const cases = JSON.parse(readFileSync(new URL('../fixtures/profile-selection.json', import.meta.url)));
for (const scenario of cases) test(`native/demo parity: ${scenario.name}`, () => {
  const selection = createProfileSelection();
  for (const step of scenario.steps) {
    const sample = { ...step, readings: Object.fromEntries(Object.entries(step.readings ?? {}).map(([key, value]) => [key, { value }])) };
    assert.deepEqual(selection.evaluate(scenario.config, sample, step.now, step.time), step.expected, `at ${step.now}`);
  }
});

const rule = (source, extra = {}) => ({ source, priority: 10, returnSeconds: 2, ...extra });
const config = (...rules) => ({ automatic: true, profiles: [{ id: 'p', rules }] });
test('old app/media rules match normalized apps and unavailable data stays unknown', () => {
  const sample = { activity: { available: true, foreground: 'CODE.EXE', applications: ['code'], media: [{ source: 'Spotify.exe', playing: true }] } };
  for (const r of [rule('foreground',{app:'code.exe'}),rule('process',{app:'Code'}),rule('mediaPlaying',{app:'spotify.exe'}),rule('mediaPlaying')]) {
    assert.equal(profileRuleHolds(r,sample),true);
    assert.equal(profileRuleHolds(r,{}),false);
  }
  assert.equal(profileRuleHolds(rule('process',{app:'gone'}),sample),false);
  assert.equal(profileRuleHolds(rule('foreground',{app:'gone'}),sample),false);
  assert.equal(profileRuleHolds(rule('mediaPlaying',{app:'gone'}),sample),false);
  assert.equal(profileRuleAvailable(rule('unknown'),sample),false);
  assert.equal(profileRuleHolds(rule('sensor',{key:'x',compare:'below',value:10}),{readings:{x:{value:5}}}),true);
  assert.equal(profileRuleHolds(rule('time',{from:'bad',to:'10:00'}),{}, {weekday:0,hour:9,minute:0}),false);
});
test('priorities and equal priorities follow saved order; a lower priority waits out return', () => {
  const c = { automatic:true, profiles:[{id:'high',rules:[rule('locked',{priority:20})]}, {id:'low',rules:[rule('idle',{minutes:1})]}] };
  const selection = createProfileSelection();
  assert.equal(selection.evaluate(c,{locked:true,idleSeconds:60},0).active.profile,'high');
  assert.equal(selection.evaluate(c,{locked:false,idleSeconds:60},1).active.reason.kind,'returning');
  assert.equal(selection.evaluate(c,{locked:false,idleSeconds:60},3).active.profile,'low');
  const tied = { ...c, profiles:c.profiles.map(p=>({...p,rules:[rule('locked')]})) };
  assert.equal(selection.evaluate(tied,{locked:true},4).active.profile,'high');
});
test('editing configuration restarts sensor timers and any/all do not match empty rules', () => {
  const selection = createProfileSelection();
  const c = config(rule('sensor',{key:'x',compare:'above',value:1,forSeconds:5}));
  const sample={readings:{x:{value:2}}};
  assert.equal(selection.evaluate(c,sample,0).active,undefined);
  assert.equal(selection.evaluate(c,sample,5).active.profile,'p');
  c.profiles[0].name='Edited';
  assert.equal(selection.evaluate(c,sample,6).active,undefined);
  for (const match of ['any','all']) assert.equal(selection.evaluate({automatic:true,profiles:[{id:'p',rules:[],match}]},{},10).active,undefined);
  assert.equal(selection.evaluate({automatic:false,profiles:[]},{},11).active,undefined);
});
test('demo overview reports the same profile status and keeps saved configuration untouched', async () => {
  let now=0;
  const demo=createDemoBackend('turing88',{now:()=>now});
  const saved={automatic:true,profiles:[{id:'p',name:'High',themes:{screen:'x'},rules:[rule('sensor',{key:'cpu.usage',compare:'above',value:-1,forSeconds:2})]}]};
  await demo.saveProfiles(saved);
  assert.deepEqual((await demo.profiles()).status.rules.p,[false]);
  now=2;
  const overview=await demo.profiles();
  assert.deepEqual(overview.status.active,{profile:'p',reason:{kind:'rules',rules:[0]}});
  assert.deepEqual(overview.config,saved);
  overview.config.profiles[0].name='Mutated';
  assert.equal((await demo.profiles()).config.profiles[0].name,'High');
});
