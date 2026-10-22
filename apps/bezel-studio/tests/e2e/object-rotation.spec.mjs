import { readFileSync } from 'node:fs';
import { createWidget } from '../../src/editor/widgets.js';
import { test, expect, watchErrors, expectAccessible, copyAndPaste, openSection } from './helpers.mjs';
const fixture = JSON.parse(readFileSync(new URL('../../../../crates/bezel-core/tests/fixtures/rotation-parity.json', import.meta.url)));
const object = (id = 1, patch = {}) => ({ id, name: `Rotation ${id}`, frame: { ...fixture.frame }, opacity: 1, visible: true, locked: false, kind: { type: 'shape', shape: 'rect', radius: 0, fill: '#ff0000ff', stroke: null, strokeWidth: 0 }, ...patch });
const theme = (elements = [object()]) => ({ schema: 1, name: 'Rotation', canvas: fixture.canvas, orientation: 'portrait', refreshSeconds: 1, background: { type: 'color', color: '#00000000' }, elements });
async function open(page, value = theme()) {
  await page.route('**/demo-theme.js', route => route.fulfill({ contentType: 'text/javascript', body: `export const DEMO_THEME = ${JSON.stringify(value)};` }));
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Rotation');
  await page.locator('#tab-layers').click();
}
const field = (page, t) => page.locator('#inspector').getByRole('spinbutton', { name: t('insp.rotation'), exact: true });
// Rotation is in Layout: open it, so a missing field means there is none.
async function select(page, id = 1) { await page.locator(`.layer-row[data-element-id="${id}"] .layer-name`).click(); await openSection(page, 'layout'); }
async function setAngle(page, t, angle) { await field(page,t).fill(String(angle)); await field(page,t).press('Tab'); }
async function canvasPoint(page, x, y) {
  const b = await page.locator('#canvas-box').boundingBox();
  return { x: b.x + b.width * x / fixture.canvas.width, y: b.y + b.height * y / fixture.canvas.height };
}

test('rotation field and handle turn an object, Shift snaps to 15\u00b0 and one drag is one Undo', async ({ page, t }) => {
  const errors = watchErrors(page); await open(page); await select(page);
  await setAngle(page,t,200.5); await expect(field(page,t)).toHaveValue('-159.5');
  await setAngle(page,t,30);
  await expect(page.locator('.handle')).toHaveCount(8);
  await expect(page.locator('.sel-box')).toHaveCSS('transform', /matrix\(0.866025, 0.5, -0.5, 0.866025/);
  const handle = page.getByRole('button', {name:t('insp.rotateHandle'),exact:true});
  const b = await handle.boundingBox(), center = await canvasPoint(page,80,60);
  const start = {x:b.x+b.width/2,y:b.y+b.height/2};
  const radius = Math.hypot(start.x-center.x,start.y-center.y);
  await page.mouse.move(start.x,start.y); await page.mouse.down(); await page.keyboard.down('Shift');
  for (const angle of [38,46,54]) await page.mouse.move(center.x+radius*Math.sin(angle*Math.PI/180),center.y-radius*Math.cos(angle*Math.PI/180),{steps:3});
  await page.mouse.up(); await page.keyboard.up('Shift');
  await expect(field(page,t)).toHaveValue('60');
  await page.locator('#undo').click(); await expect(field(page,t)).toHaveValue('30');
  await page.locator('#redo').click(); await expect(field(page,t)).toHaveValue('60');
  await handle.focus(); await page.keyboard.press('ArrowRight'); await expect(field(page,t)).toHaveValue('61');
  // Resize in local coordinates: the diagonally opposite handle stays still.
  await setAngle(page,t,90);
  const fixed = await page.locator('[data-h="se"]').boundingBox();
  const moving = await page.locator('[data-h="nw"]').boundingBox();
  await page.mouse.move(moving.x+moving.width/2,moving.y+moving.height/2); await page.mouse.down();
  await page.mouse.move(moving.x+moving.width/2+20,moving.y+moving.height/2+14,{steps:5}); await page.mouse.up();
  const after = await page.locator('[data-h="se"]').boundingBox();
  expect(Math.abs(after.x-fixed.x)).toBeLessThan(0.05); expect(Math.abs(after.y-fixed.y)).toBeLessThan(0.05);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('a click hits the rotated box, not the unrotated corners', async ({page,t}) => {
  const errors=watchErrors(page); await open(page,theme([object(1,{rotation:30})]));
  const inside=await canvasPoint(page,...fixture.ink[1]); await page.mouse.click(inside.x,inside.y);
  await openSection(page,'layout'); await expect(field(page,t)).toHaveValue('30');
  const outside=await canvasPoint(page,...fixture.empty[0]); await page.mouse.click(outside.x,outside.y);
  await expect(page.locator('.sel-box')).toHaveCount(0);
  await expect(field(page,t)).toHaveCount(0);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('no rotation field or handle for cards, their members, groups, group members and video windows', async ({page,t}) => {
  const errors=watchErrors(page);
  await open(page,theme([
    object(1,{card:{faces:['Face'],activeFace:0}}),object(2,{cardMember:{parent:1,face:0}}),
    object(3,{isGroup:true}),object(4,{groupParent:3}),object(5,{kind:{...object().kind,videoWindow:true}}),object(6,{locked:true}),
  ]));
  for (const id of [1,2,3,4,5]) {
    await select(page,id); await expect(field(page,t)).toHaveCount(0); await expect(page.locator('[data-rotate]')).toHaveCount(0);
  }
  await select(page,6); await expect(field(page,t)).toHaveCount(1); await expect(page.locator('[data-rotate]')).toHaveCount(0);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('copy, paste and duplicate keep the angle', async ({page,t}) => {
  const errors=watchErrors(page); await open(page,theme([object(1,{rotation:30})])); await select(page);
  await copyAndPaste(page); await expect(field(page,t)).toHaveValue('30');
  await expect(page.locator('.layer-row')).toHaveCount(2);
  await page.locator('#inspector').getByRole('button',{name:t('insp.more'),exact:true}).click();
  await page.locator('#inspector').getByRole('button',{name:t('common.duplicate'),exact:true}).click();
  await expect(page.locator('.layer-row')).toHaveCount(3); await expect(field(page,t)).toHaveValue('30');
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('demo render and inspector preview match the Rust parity fixture at 30\u00b0', async ({page,t}) => {
  const errors=watchErrors(page); await open(page,theme([object(1,{rotation:30})])); await select(page);
  const samples = await page.evaluate(async ({fixture,theme}) => {
    const {renderApprox}=await import('./demo-render.js');
    const {createDemoBackend,demoIsolate}=await import('./demo-backend.js');
    const backend=createDemoBackend('turing88',{now:()=>0}); const results=[];
    for (const opacity of fixture.opacities) {
      theme.elements[0].opacity=opacity;
      const frame=renderApprox(theme,0), isolated=demoIsolate(theme,1), preview=await backend.renderElement(theme,1,{max:512,motion:false});
      const dx=theme.elements[0].frame.x-isolated.elements[0].frame.x,dy=theme.elements[0].frame.y-isolated.elements[0].frame.y;
      const pixel=(f,x,y)=>Array.from(f.rgba.slice((y*f.width+x)*4,(y*f.width+x)*4+4));
      results.push({opacity,ink:fixture.ink.map(([x,y])=>[pixel(frame,x,y),pixel(preview,x-dx,y-dy)]),empty:fixture.empty.map(([x,y])=>[pixel(frame,x,y),pixel(preview,x-dx,y-dy)])});
    }
    return results;
  },{fixture,theme:theme([object(1,{rotation:30})])});
  for (const sample of samples) {
    for (const pair of sample.ink) for (const [r,g,b,a] of pair) { expect(r).toBeGreaterThanOrEqual(252);expect(g).toBeLessThanOrEqual(3);expect(b).toBeLessThanOrEqual(3);expect(Math.abs(a-255*sample.opacity)).toBeLessThanOrEqual(3); }
    for (const pair of sample.empty) for (const pixel of pair) expect(pixel[3]).toBeLessThanOrEqual(3);
  }
  await expect(page.locator('#inspector canvas')).toBeVisible();
  const canvas = {width:640,height:360};
  const elements = ['text','graph','spectrum'].map((widget,index) => {
    const kind = createWidget(widget,canvas).kind;
    if (widget === 'text') kind.content = {type:'static',text:'Rotation 30'};
    if (widget === 'spectrum') kind.content.demo = true;
    return object(index+1,{name:widget,rotation:30,frame:{x:50+index*190,y:120,width:150,height:90},kind});
  });
  const sample = {...theme(elements),canvas,background:{type:'color',color:'#101827ff'}};
  await page.route('**/demo-theme.js',route=>route.fulfill({contentType:'text/javascript',body:`export const DEMO_THEME = ${JSON.stringify(sample)};`}));
  await page.reload(); await expect(page.locator('#theme-name')).toHaveValue('Rotation');
  await expect.poll(() => page.locator('#preview').evaluate(c=>c.getContext('2d').getImageData(320,180,1,1).data[3])).toBe(255);
  const screenshot = await page.locator('#canvas-box').screenshot({path:test.info().outputPath('object-rotation-30.png')});
  await test.info().attach('Text, graph and spectrum at 30 degrees',{body:screenshot,contentType:'image/png'});
  await expectAccessible(page); expect(errors).toEqual([]);
});
