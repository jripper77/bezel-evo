import { test } from 'node:test';
import assert from 'node:assert/strict';
import { visualization, visualStyles, visualArt, matrixFrame } from '../../src/visualization.js';
test('presets preserve edited settings and never invent a second source',()=>{
  assert.equal(visualization('standard'),null);
  const v=visualization('matrix');
  assert.equal(v.secondary,null);
  assert.equal(visualization('matrix',{...v,columns:12}).columns,12);
  assert.equal(visualization('mirrored',v).style,'mirrored');
  assert.deepEqual(visualStyles('text'),[]);
  assert.deepEqual(visualStyles('ring'),['standard','pixel','dial']);
});
test('every selectable preset has a schematic and matrix fitting preserves position and width',()=>{
  for(const kind of ['bar','ring','graph']) for(const style of visualStyles(kind)) assert.ok(visualArt(kind,style).length);
  assert.deepEqual(visualArt('text','standard'),[]);
  const frame={x:20,y:100,width:320,height:18};
  assert.deepEqual(matrixFrame(frame,visualization('matrix'),{height:480}),{...frame,height:120});
  assert.equal(frame.height,18);
  assert.equal(matrixFrame({...frame,y:470},visualization('matrix'),{height:480}).height,10);
  assert.equal(matrixFrame({...frame,y:500},visualization('matrix'),{height:480}).height,4);
});
