import { test } from 'node:test';
import assert from 'node:assert/strict';
import { sectionFor, summaryParts, resizedFrame } from '../../src/inspector-model.js';

test('existing groups map to canonical sections and unknown controls stay visible', () => {
  for (const [id,key] of Object.entries({layout:'position',data:'source',shape:'arc',text:'font',color:'fill',behavior:'transition'})) assert.equal(sectionFor(key),id);
  assert.equal(sectionFor('future'), 'data');
  assert.equal(sectionFor('future','behavior'), 'behavior');
});
test('summaries contain only document values, with missing values omitted by the formatter', () => {
  const e={frame:{width:140,height:100},rotation:30,opacity:.8,kind:{binding:{key:'gpu.usage'},style:'solid',shape:'rectangle',thickness:24,font:{family:'Inter',size:22},fill:'#ff0000ff'},valueTransition:{mode:'smooth',durationMs:400},card:{faces:['A','B']}};
  assert.deepEqual(summaryParts('layout',e),['140 × 100',{value:30,unit:'°'}]);
  assert.equal(summaryParts('data',e)[0],'gpu.usage');
  assert.deepEqual(summaryParts('shape',e),[null,null,null,null,{value:24,unit:'px'}]);
  assert.deepEqual(summaryParts('text',e),['Inter',{value:22,unit:'px'}]);
  assert.deepEqual(summaryParts('color',e),['#ff0000ff',{value:80,unit:'%'}]);
  assert.deepEqual(summaryParts('behavior',e),[{value:400,unit:'ms'}]);
  assert.deepEqual(summaryParts('faces',e),['A','B']);
  assert.deepEqual(summaryParts('shape',{kind:{type:'graph',style:'line',shape:'rect',startAngle:-135,sweep:270}}),[{key:'graph.line'},{key:'shape.rect'},{value:-135,unit:'\u00b0'},{value:270,unit:'\u00b0'},null]);
  assert.deepEqual(summaryParts('text',{kind:{style:{font:{family:'Inter'},size:16}}}),['Inter',{value:16,unit:'px'}]);
  assert.deepEqual(summaryParts('layout',{}),[]);
  for(const id of ['layout','data','shape','text','color','behavior','faces','motion']) assert.ok(Array.isArray(summaryParts(id,null)));
  assert.equal(summaryParts('layout',{frame:e.frame})[1].value,0);
  assert.equal(summaryParts('data',{kind:{content:{binding:{key:'cpu.usage'}}}})[0],'cpu.usage');
  assert.equal(summaryParts('data',{kind:{source:{key:'cpu.usage'}}})[0],'cpu.usage');
  assert.equal(summaryParts('color',{kind:{fill:{}}})[0],null);
});
test('linked resize preserves aspect ratio without storing it in the theme', () => {
 const frame={width:80,height:40};
 assert.deepEqual(resizedFrame(frame,{width:120},true),{width:120,height:60});
 assert.deepEqual(resizedFrame(frame,{height:80},true),{height:80,width:160});
 assert.deepEqual(resizedFrame(frame,{width:120},false),{width:120});
 assert.deepEqual(resizedFrame(frame,{x:10},true),{x:10});
 assert.deepEqual(resizedFrame({width:0,height:40},{width:10},true),{width:10});
 assert.deepEqual(resizedFrame({width:40,height:0},{height:10},true),{height:10});
});
