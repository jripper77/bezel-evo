import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createProfilesSave } from '../../src/profiles-save.js';
const deferred = () => { let resolve, reject; const promise = new Promise((a,b) => {resolve=a;reject=b;}); return {promise,resolve,reject}; };
test('autosave snapshots the latest valid draft and flush serializes concurrent writes', async () => {
  const gate = deferred(), writes = [], states = [];
  const saver = createProfilesSave({ save: async draft => { writes.push(draft); if (draft.id === 1) await gate.promise; }, state: s => states.push(s), wait: () => 1, cancel: () => {} });
  const draft = {id:1}; saver.update(draft); draft.id=99;
  const saving = saver.flush(); await Promise.resolve();
  saver.update({id:2}); saver.update({id:3}); const closing = saver.flush();
  assert.deepEqual(writes,[{id:1}]); gate.resolve(); await Promise.all([saving,closing]);
  assert.deepEqual(writes,[{id:1},{id:3}]); assert.equal(states.at(-1),'saved'); saver.dispose();
});
test('incomplete edits cancel queued saves and a late valid save does not mark them saved', async () => {
  const gate = deferred(), states = [], writes = [];
  const saver = createProfilesSave({save: async draft => {writes.push(draft); await gate.promise;}, state:s=>states.push(s), wait:()=>1,cancel:()=>{}});
  saver.update({id:1}); const saving=saver.flush(); await Promise.resolve();
  saver.update({id:2},false); gate.resolve(); assert.equal(await saving,false);
  assert.equal(await saver.flush(),false); assert.equal(states.at(-1),'incomplete'); assert.deepEqual(writes,[{id:1}]); saver.dispose();
});
test('failed saves retain the draft for retry and newer edits replace that retry', async () => {
  let fail=true; const writes=[]; const states=[];
  const saver=createProfilesSave({save:async d=>{if(fail)throw Error('offline');writes.push(d);},state:s=>states.push(s),wait:()=>1,cancel:()=>{}});
  saver.update({id:1}); await assert.rejects(saver.flush(),/offline/); assert.equal(states.at(-1),'error');
  saver.update({id:2}); fail=false; assert.equal(await saver.flush(),true); assert.deepEqual(writes,[{id:2}]); saver.dispose();
});
test('the debounce schedules only the latest valid update and dispose cancels it', async () => {
  let scheduled, cancelled=0; const writes=[];
  const saver=createProfilesSave({save:async d=>writes.push(d),wait:fn=>{scheduled=fn;return 1;},cancel:()=>cancelled++});
  saver.update({id:1}); saver.update({id:2}); scheduled(); await saver.flush(); assert.deepEqual(writes,[{id:2}]);
  saver.update({id:3}); saver.dispose(); assert.equal(cancelled,2); await saver.flush(); assert.equal(writes.length,1);
});
