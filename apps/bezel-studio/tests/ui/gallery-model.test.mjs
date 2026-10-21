import { test } from 'node:test';
import assert from 'node:assert/strict';
import { themeSensorKeys, missingSensors, themeUses, ruleNow, recentThemes } from '../../src/gallery-model.js';
import { themePreview } from '../../src/theme-previews.js';
import { searchThemes } from '../../src/theme-filter.js';
test('gallery facts use actual catalog and runtime status, never the selected profile', () => {
  const rule={source:'locked',priority:10,returnSeconds:3};
  const overview={config:{profiles:[{id:'a',name:'A',themes:{COM1:'a'},rules:[rule]},{id:'b',name:'B',themes:{COM1:'b'},rules:[rule]}]},displays:[{key:'COM1',connected:true,theme:'base',name:'Screen'}],status:{active:{profile:'b'},rules:{a:[false],b:[true]}}};
  assert.deepEqual(themeUses({location:'a'},overview),[{kind:'profile',name:'A',key:'a'}]);
  assert.deepEqual(themeUses({location:'b'},overview).map(u=>u.kind),['screen','profile']);
  assert.equal(ruleNow(overview,'a',0,rule),false); assert.equal(ruleNow(overview,'b',0,rule),true);
  assert.equal(ruleNow(overview,'b',0,{...rule,priority:11}),null); assert.equal(ruleNow({...overview,status:null},'b',0,rule),null);
  overview.status.preview={themes:{COM1:'a'}}; assert.equal(themeUses({location:'a'},overview)[0].kind,'screen');
  assert.deepEqual(themeUses({location:'base'},{...overview,status:null}).map(u=>u.kind), ['base']);
  const entry={sensors:['cpu.load','gone','media.sessions','weather.1:2.code']};
  assert.deepEqual(missingSensors(entry,[{key:'cpu.load'}]),['gone']); assert.deepEqual(missingSensors(entry,null),[]);
});
test('sensor facts ignore hidden elements and include sampled inactive Card faces', () => {
  const value=(id,key,extra={})=>({id,kind:{type:'text',content:{type:'sensor',key}},...extra});
  const card={id:3,visible:true,card:{activeFace:0,triggers:[{source:'foreground'}]},kind:{type:'shape'}};
  const theme={elements:[value(1,'cpu.load'),value(2,'hidden',{visible:false}),card,value(4,'gpu.load',{cardMember:{parent:3,face:1}})]};
  assert.deepEqual(themeSensorKeys(theme),['activity.processes','cpu.load','gpu.load']);
  card.card.triggers=[]; assert.deepEqual(themeSensorKeys(theme),['cpu.load']);
});
test('search matches sensor labels and recent order is stable for unknown dates', () => {
  const entries=[{name:'A',sensors:['cpu.load'],modifiedMs:1},{name:'B',modifiedMs:3},{name:'C'}];
  assert.deepEqual(searchThemes(entries,'processor',[{key:'cpu.load',label:'Processor usage'}]),[entries[0]]);
  assert.deepEqual(recentThemes(entries).map(e=>e.name),['B','A','C']);
});
test('thumbnails share one in-flight request per backend and revision, with failures cached', async () => {
  let count=0; const bridge={themeThumbnail:async path=>{count++;if(path==='bad')throw Error(); return path;}};
  const entry={location:'a',revision:'1'}; assert.equal(await themePreview(bridge,entry),'a'); await themePreview(bridge,entry); assert.equal(count,1);
  await themePreview(bridge,{...entry,revision:'2'}); assert.equal(count,2);
  assert.equal(await themePreview(bridge,{location:'bad'}),null); await themePreview(bridge,{location:'bad'}); assert.equal(count,3);
});
