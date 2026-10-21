import { test } from 'node:test';
import assert from 'node:assert/strict';
import { editedFill, scaledText, frameHandles, propertyHandles, editPropertyHandle, scaledFrames, resetPropertyHandle, partPieces, textPartAt } from '../../src/editor/handles.js';
import { createWidget } from '../../src/editor/widgets.js';
import { centerOf, rotatePoint } from '../../src/editor/rotation.js';
import { translator } from '../../src/i18n/index.js';

const made = widget => ({ ...createWidget(widget, { width: 480, height: 1920 }), id: 1, frame: { x: 10, y: 20, width: 200, height: 100 } });
const near = (actual, expected) => assert.ok(Math.abs(actual - expected) < 1e-7, `${actual} != ${expected}`);
const edit = (e, id, p, options) => editPropertyHandle(e, id, p, options);
const polar = (cx, cy, r, deg) => ({ x: cx + r * Math.sin(deg * Math.PI / 180), y: cy - r * Math.cos(deg * Math.PI / 180) });

test('frame handles follow local rotation and exclude locked or ineligible rotations', () => {
  assert.deepEqual(frameHandles([]), []);
  assert.deepEqual(frameHandles([{ ...made('shape'), locked: true }]), []);
  const e = { ...made('shape'), rotation: 90 };
  const handles = frameHandles([e], 2);
  assert.equal(handles.length, 9);
  near(handles[0].x, 160); near(handles[0].y, -30);
  const r = handles.at(-1); near(r.x, 172); near(r.y, 70);
  const other = { ...made('bar'), id: 2, frame: { x: 250, y: 30, width: 100, height: 50 } };
  assert.equal(frameHandles([e, other]).length, 9);
  const excluded = [made('card'), { ...made('shape'), isGroup: true }, { ...e, groupParent: 4 }, { ...e, cardMember: { parent: 4, face: 0 } }, { ...made('shape'), kind: { type: 'shape', videoWindow: true } }];
  for (const item of excluded) {
    assert.equal(frameHandles([item]).length, 8);
    assert.equal(frameHandles([e, item]).length, 8);
  }
});

test('common resize preserves proportions, minimum sizes and source frames', () => {
  const elements = [made('ring'), { ...made('bar'), id: 2, frame: { x: 110, y: 70, width: 4, height: 4 } }];
  const original = structuredClone(elements);
  const patches = scaledFrames(elements, { x: 10, y: 20, width: 200, height: 100 }, { x: 20, y: 40, width: 100, height: 200 });
  assert.deepEqual(patches[0].patch.frame, { x: 20, y: 40, width: 100, height: 200 });
  assert.deepEqual(patches[1].patch.frame, { x: 70, y: 140, width: 4, height: 8 });
  assert.deepEqual(elements, original);
  for (const box of [{ width: 0, height: 10 }, { width: 10, height: 0 }]) assert.deepEqual(scaledFrames(elements, box, { width: 20, height: 20 }), []);
  assert.deepEqual(scaledFrames(elements, { width: 10, height: 10 }, { width: 3, height: 3 }), []);
});

test('ring endpoints preserve the opposite endpoint, snap and handle wrap and reverse direction', () => {
  const e = made('ring'); e.frame = { x: 0, y: 0, width: 200, height: 200 };
  Object.assign(e.kind, { startAngle: 0, sweep: 180, thickness: 10 });
  const start = edit(e, 'arc-start', polar(100, 100, 95, 31), { shift: true });
  near(start.patch.kind.startAngle, 30); near(start.patch.kind.sweep, 150);
  near(edit(e, 'arc-end', polar(100, 100, 95, 120)).patch.kind.sweep, 120);
  e.kind.clockwise = false;
  near(edit(e, 'arc-end', polar(100, 100, 95, -60), { shift: true }).value, 60);
  near(edit(e, 'arc-start', polar(100, 100, 95, -15)).patch.kind.sweep, 165);
  Object.assign(e.kind, { clockwise: true, startAngle: 350, sweep: 10 });
  near(edit(e, 'arc-end', polar(100, 100, 95, 5)).value, 15);
  const h = propertyHandles(e).find(h => h.id === 'thickness');
  near(edit(e, h.id, h).value, 10);
  near(edit(e, h.id, { x: 100, y: 80 }).value, 80);
  near(edit(e, h.id, { x: 1000, y: 80 }).value, 1);
  const rotated = { ...e, rotation: 37 };
  near(edit(rotated, h.id, rotatePoint({ x: 100, y: 80 }, centerOf(e.frame), 37)).value, 80);
});

test('needle pivot and length use gauge geometry; zone edits join both adjacent intervals', () => {
  const e = made('needle'); e.frame = { x: 0, y: 0, width: 200, height: 200 };
  e.kind.startAngle = -90; e.kind.sweep = 180;
  assert.deepEqual(edit(e, 'pivot', { x: -20, y: 300 }).patch.kind.pivot, [0, 1]);
  const legacyTip = propertyHandles(e).find(h => h.id === 'needle-length');
  near(edit(e, 'needle-length', legacyTip).value, 92);
  delete e.kind.pivot;
  assert.equal(propertyHandles(e).find(h => h.id === 'pivot').value[0], 50);
  e.kind.look = { zones: [{ from: 0, to: 40, color: '#00ff00ff' }, { from: 40, to: 80, color: '#ffff00ff' }, { from: 80, to: 100, color: '#ff0000ff' }] };
  const original = structuredClone(e);
  const boundary = edit(e, 'zone-0', { x: 100, y: 0 });
  assert.deepEqual(boundary.fieldParams, { number: 1 });
  near(boundary.value, 50); near(boundary.patch.kind.look.zones[0].to, 50); near(boundary.patch.kind.look.zones[1].from, 50);
  assert.equal(boundary.patch.kind.look.zones[2].to, 100);
  assert.deepEqual(e, original);
  assert.equal(propertyHandles(e).filter(h => h.id.startsWith('zone-')).length, 2);
  assert.equal(edit(e, 'needle-length', { x: 100, y: 100 }).value, 10);
  e.kind.sweep = 0;
  near(edit(e, 'zone-0', { x: 0, y: 0 }).value, 40);
});

test('corners share one radius; Alt changes only the chosen corner', () => {
  for (const widget of ['shape', 'bar', 'image', 'card']) {
    const e = made(widget); e.kind.radius = 10;
    assert.equal(propertyHandles(e).filter(h => h.id.startsWith('corner-')).length, 4);
    const start = propertyHandles(e).find(h => h.id === 'corner-0');
    const all = edit(e, 'corner-0', { x: start.x + 10, y: start.y + 10 });
    assert.deepEqual(all.patch.kind, { radius: 20, corners: null });
    const bottom = propertyHandles(e).find(h => h.id === 'corner-2');
    const one = edit(e, 'corner-2', { x: bottom.x - 30, y: bottom.y - 30 }, { alt: true });
    assert.deepEqual(one.patch.kind.corners, [10, 10, 40, 10]);
    e.kind.corners = [1, 2, 3, 4];
    const topRight = propertyHandles(e).find(h => h.id === 'corner-1');
    assert.deepEqual(edit(e, 'corner-1', { x: topRight.x - 18, y: topRight.y + 18 }, { alt: true }).patch.kind.corners, [1, 20, 3, 4]);
  }
  const ellipse = made('shape'); ellipse.kind.shape = 'ellipse';
  assert.deepEqual(propertyHandles(ellipse), []);
});

test('bar testing is transient for every direction and clamps to the scale', () => {
  const e = made('bar');
  for (const [direction, point, expected] of [
    ['leftToRight', { x: 60, y: 70 }, .25], ['rightToLeft', { x: 60, y: 70 }, .75],
    ['topToBottom', { x: 110, y: 45 }, .25], ['bottomToTop', { x: 110, y: 45 }, .75],
  ]) {
    e.kind.direction = direction;
    const original = structuredClone(e);
    const next = edit(e, 'trial-value', point, { fraction: .3 });
    near(next.trial, expected); assert.equal(next.patch, undefined);
    assert.deepEqual(e, original);
  }
  e.kind.direction = 'leftToRight';
  assert.equal(edit(e, 'trial-value', { x: 1000, y: 70 }).trial, 1);
});

test('manual graph limits preserve the opposite bound and turn off automatic scale', () => {
  const e = made('graph');
  const max = edit(e, 'scale-max', { x: 210, y: 10 });
  near(max.value, 110); assert.deepEqual(max.patch.kind, { autoscale: false, binding: { max: 110 } });
  near(edit(e, 'scale-min', { x: 210, y: 130 }).value, -10);
  assert.ok(edit(e, 'scale-min', { x: 210, y: -1000 }).value < e.kind.binding.max);
  assert.ok(edit(e, 'scale-max', { x: 210, y: 1000 }).value > e.kind.binding.min);
  assert.deepEqual(resetPropertyHandle(e, 'scale-min'), { kind: { autoscale: true } });
  assert.deepEqual(resetPropertyHandle(e, 'scale-max'), { kind: { autoscale: true } });
  assert.equal(resetPropertyHandle(e, 'removed'), null);
  assert.equal(resetPropertyHandle({ ...e, locked: true }, 'scale-min'), null);
  assert.equal(resetPropertyHandle(made('ring'), 'scale-min'), null);
});

test('text sizing and spectrum counts use their existing property ranges', () => {
  for (const widget of ['text', 'value', 'clock']) {
    const e = made(widget), h = propertyHandles(e).find(h => h.id === 'text-size');
    near(edit(e, h.id, h).value, e.kind.style.size);
    assert.equal(edit(e, h.id, { x: 10, y: 20 }).value, 4);
    assert.equal(edit(e, h.id, { x: 100000, y: 100000 }).value, 800);
  }
  const clock = made('clock'); clock.kind.content.analog = {};
  assert.deepEqual(propertyHandles(clock).map(h => h.id), ['clock-hour-length', 'clock-minute-length', 'clock-second-length']);
  const spectrum = made('spectrum');
  assert.equal(edit(spectrum, 'bands', { x: 1000, y: 70 }).value, 64);
  assert.equal(edit(spectrum, 'bands', { x: -1000, y: 70 }).value, 8);
  assert.equal(edit(spectrum, 'bar-gap', { x: 35, y: 120 }).value, 13);
});

test('linear and radial fades preserve untouched parameters and use the supplied cover bounds', () => {
  const e = made('image'); e.kind.fade = { angle: 0, start: 1, end: 0 };
  let out = edit(e, 'fade-direction', { x: 110, y: 120 });
  near(out.value, 90); near(out.patch.kind.fade.midpoint, .5);
  assert.equal(out.patch.kind.fade.start, undefined);
  e.kind.fade.midpoint = .7;
  near(edit(e, 'fade-direction', propertyHandles(e).find(h => h.id === 'fade-direction')).patch.kind.fade.midpoint, .7);
  e.kind.fade.radial = [.5, .5, .5];
  out = edit(e, 'fade-center', { x: 60, y: 45 });
  assert.deepEqual(out.patch.kind.fade.radial, [.25, .25, .5]);
  assert.deepEqual(out.value, [25, 25]);
  near(edit(e, 'fade-radius', { x: 160, y: 70 }).value, 25);
  const player = made('player'); player.kind.content.coverFade = e.kind.fade;
  assert.deepEqual(propertyHandles(player).map(h => h.id), ['cover-gap', 'fade-center', 'fade-radius']);
  // Without supplied bounds the fade follows the drawn cover: 40% of 200 px wide, top left.
  const centre = propertyHandles(player).find(h => h.id === 'fade-center');
  near(centre.x, 50); near(centre.y, 60);
  const coverFrame = { x: 10, y: 20, width: 40, height: 40 };
  near(edit(player, 'fade-radius', { x: 50, y: 40 }, { coverFrame }).value, 50);
  assert.ok(edit(player, 'fade-center', { x: 30, y: 40 }, { coverFrame }).patch.kind.content.coverFade);
  player.kind.content.coverFade = { angle: 0, start: 1, end: 0 };
  near(edit(player, 'fade-direction', { x: 50, y: 40 }, { coverFrame }).value, 0);
});

test('the player divider moves the text after the drawn cover and is absent when the cover is alone or off', () => {
  const player = made('player');
  const gap = player.kind.style.size * .3;
  const divider = propertyHandles(player).find(h => h.id === 'cover-gap');
  near(divider.x, 10 + 80 + gap); near(divider.y, 60);
  assert.equal(edit(player, 'cover-gap', divider).value, Math.round(gap));
  assert.deepEqual(edit(player, 'cover-gap', { x: 110, y: 60 }).patch, { kind: { content: { coverGap: 20 } } });
  assert.equal(edit(player, 'cover-gap', { x: 0, y: 60 }).value, 0);
  assert.equal(edit(player, 'cover-gap', { x: 900, y: 60 }).value, 200);
  const alone = { ...player, kind: { ...player.kind, content: { ...player.kind.content, showTitle: false, showArtist: false, showTime: false, showProgress: false } } };
  assert.equal(propertyHandles(alone).some(h => h.id === 'cover-gap'), false);
  const hidden = { ...player, kind: { ...player.kind, content: { ...player.kind.content, showCover: false, coverFade: { angle: 0, start: 1, end: 0 } } } };
  assert.deepEqual(propertyHandles(hidden), []);
});

test('the image anchor snaps to the nearest of the nine discrete positions, only when the image fills', () => {
  const image = made('image');
  assert.equal(propertyHandles(image).some(h => h.id === 'image-anchor'), false);
  image.kind.fit = 'cover';
  const anchor = propertyHandles(image).find(h => h.id === 'image-anchor');
  near(anchor.x, 110); near(anchor.y, 70); assert.deepEqual(anchor.keyStep, { x: 100, y: 50 });
  assert.deepEqual(edit(image, 'image-anchor', { x: 20, y: 25 }), { field: 'insp.image.anchor', fieldParams: undefined, unit: '%', patch: { kind: { anchor: 'topLeft' } }, value: [0, 0] });
  assert.deepEqual(edit(image, 'image-anchor', { x: 900, y: 64 }).patch, { kind: { anchor: 'right' } });
  assert.deepEqual(edit(image, 'image-anchor', { x: 104, y: 76 }).patch, { kind: { anchor: null } }, 'center is stored as absent');
  image.kind.anchor = 'bottomRight';
  const corner = propertyHandles(image).find(h => h.id === 'image-anchor');
  near(corner.x, 210); near(corner.y, 120);
  assert.deepEqual(edit(image, 'image-anchor', { x: corner.x - anchor.keyStep.x, y: corner.y }).value, [50, 100]);
});

test('inactive and stale handles cannot edit documents; every field has localized text', () => {
  const e = made('shape'); delete e.kind.radius;
  assert.equal(propertyHandles(e).length, 4);
  assert.deepEqual(propertyHandles({ ...e, locked: true }), []);
  assert.deepEqual(propertyHandles({ ...e, isGroup: true }), []);
  assert.equal(edit(e, 'removed', { x: 0, y: 0 }), null);
  assert.equal(edit(e, 'corner-0', { x: NaN, y: 0 }), null);
  const fixtures = ['ring', 'needle', 'shape', 'bar', 'image', 'graph', 'text', 'value', 'clock', 'weather', 'player', 'spectrum', 'card'].map(made);
  for (const fixture of fixtures) for (const handle of propertyHandles(fixture)) for (const locale of ['en', 'it', 'pt-BR']) {
    assert.notEqual(translator(locale)(handle.field, handle.fieldParams), handle.field);
  }
});


test('Ctrl resize scales only eligible typography and clamps its size', () => {
  for (const type of ['text', 'value', 'clock']) {
    const e = made(type), original = structuredClone(e);
    assert.deepEqual(scaledText(e, { width: 400, height: 200 }), { kind: { style: { size: e.kind.style.size * 2 } } });
    assert.deepEqual(e, original);
    assert.equal(scaledText(e, { width: 0, height: 0 }).kind.style.size, 4);
    assert.equal(scaledText(e, { width: 100000, height: 100000 }).kind.style.size, 800);
  }
  assert.equal(scaledText(made('shape'), {}), null);
  assert.equal(scaledText(made('player'), {}), null);
  const analog = made('clock'); analog.kind.content.analog = {};
  assert.equal(scaledText(analog, {}), null);
});


test('shape image edits keep the source unchanged and share inspector limits', () => {
  const image = { asset: 'image.png', scale: 2, offsetX: 5, offsetY: -10 }, original = structuredClone(image);
  assert.deepEqual(editedFill(image, { dx: 3.7, dy: -2.3, scale: 1.5 }), { offsetX: 9, offsetY: -12, scale: 3 });
  assert.deepEqual(image, original);
  assert.deepEqual(editedFill({}), { offsetX: 0, offsetY: 0, scale: 1 });
  assert.equal(editedFill(image, { scale: .001 }).scale, .05);
  assert.equal(editedFill(image, { scale: 100 }).scale, 10);
});


test('clock length handles follow real time, local rotation and shown hands; original lengths disappear', () => {
  const clock = made('clock'); clock.kind.content.analog = {};
  clock.frame = { x: 0, y: 0, width: 256, height: 256 };
  const date = new Date(2026, 9, 21, 3, 0, 0);
  const handles = propertyHandles(clock, { date });
  const hour = handles.find(h => h.id === 'clock-hour-length');
  near(hour.x, 128 + 11.7 * 4); near(hour.y, 128);
  near(handles[1].x, 128); near(handles[1].y, 128 - 20.9 * 4);
  const next = edit(clock, hour.id, { x: 128 + 11.7 * 2, y: 128 }, { date });
  assert.equal(next.value, 50); assert.equal(next.kind.content.analog.hourLength, .5);
  const original = { ...clock, kind: { ...next.kind } };
  assert.deepEqual(edit(original, hour.id, hour, { date }).kind.content.analog, {});
  assert.equal(edit(clock, hour.id, { x: 10000, y: 0 }, { date }).value, 200);
  assert.equal(edit(clock, hour.id, { x: 128, y: 128 }, { date }).value, 10);
  clock.rotation = 90;
  const rotated = propertyHandles(clock, { date }).find(h => h.id === hour.id);
  near(rotated.x, 128); near(rotated.y, 128 + 11.7 * 4);
  clock.kind.content.analog.minuteHand = false;
  assert.equal(propertyHandles(clock).length, 2);
  clock.kind.content.analog.hourHand = false; clock.kind.content.analog.secondHand = false;
  assert.deepEqual(propertyHandles(clock), []);
  clock.kind.content.analog.secondHand = true; clock.kind.content.analog.sweep = true;
  const half = new Date(2026, 9, 21, 3, 0, 0, 500);
  assert.notEqual(propertyHandles(clock, { date: half })[0].y, propertyHandles(clock, { date: half, motion: false })[0].y);
});

test('a click picks the value part drawn under it, also on a rotated value', () => {
  const value = made('value'); value.kind.content.prefix = 'cpu '; value.kind.style.casing = 'upper';
  const { scope, pieces } = partPieces(value, { reading: { value: 38 }, quantity: 'percent' });
  assert.equal(scope, 'value');
  assert.deepEqual(pieces.map(p => [p.id, p.text]), [['prefix', 'CPU '], ['number', '38'], ['unit', '%'], ['suffix', '']]);
  // Ten pixels a character: prefix 10–50, number 50–70, unit 70–80, from the left edge, mid-height.
  const measure = text => text.length * 10;
  const at = (x, y = 70, e = value) => textPartAt(e, pieces, { x, y }, measure);
  assert.equal(at(30), 'prefix'); assert.equal(at(60), 'number'); assert.equal(at(75), 'unit');
  assert.equal(at(150), null, 'past the text'); assert.equal(at(60, 21), null, 'above the line');
  assert.equal(at(NaN), null);
  const turned = { ...value, rotation: 90 }, c = centerOf(value.frame);
  const p = rotatePoint({ x: 75, y: 70 }, c, 90);
  assert.equal(at(p.x, p.y, turned), 'unit');
  assert.equal(textPartAt(value, [{ text: '', id: 'prefix' }], { x: 30, y: 70 }, measure), null, 'nothing drawn');
  assert.deepEqual(partPieces(value).pieces[1].text, '—', 'no reading yet');
  assert.deepEqual(partPieces(value, { reading: { text: 'idle', value: 3 } }).pieces[1].text, 'IDLE');
  const bare = { ...value, kind: { ...value.kind, content: { type: 'sensor', key: 'cpu' } } };
  assert.deepEqual(partPieces(bare).pieces.map(p => p.text), ['', '—', '', ''], 'a value saved without prefix or suffix');
});

test('a digital clock offers its time parts at the current time; other kinds have none', () => {
  const clock = made('clock');
  const { scope, pieces } = partPieces(clock, { date: new Date(2026, 0, 1, 9, 5) });
  assert.equal(scope, 'clock');
  assert.deepEqual(pieces.map(p => [p.id, p.text]), [['hours', '09'], ['separators', ':'], ['minutes', '05']]);
  // Centred, fifty pixels wide in a 200 px box from x = 10: 85–105–115–135.
  const at = x => textPartAt(clock, pieces, { x, y: 70 }, text => text.length * 10);
  assert.deepEqual([at(90), at(110), at(120)], ['hours', 'separators', 'minutes']);
  assert.equal(partPieces({ ...clock, kind: { ...clock.kind, content: { ...clock.kind.content, analog: {} } } }), null);
  for (const kind of ['text', 'ring', 'player']) assert.equal(partPieces(made(kind)), null);
});
