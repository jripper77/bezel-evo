import { test } from 'node:test';
import assert from 'node:assert/strict';
import { IMAGE_ANCHORS, isAnimatedImage, playbackSpeeds } from '../../src/media-inspector.js';
import { applyCanvasFade, drawClippedMedia } from '../../src/demo-render.js';

test('image controls distinguish GIF metadata from videos and retain imported speeds', () => {
  assert.equal(IMAGE_ANCHORS.length, 9); assert.equal(new Set(IMAGE_ANCHORS).size, 9);
  assert.equal(isAnimatedImage('a.GIF'), true);
  assert.equal(isAnimatedImage('a.mp4', [{ ref: 'a.mp4', kind: 'video', animated: true }]), false);
  assert.equal(isAnimatedImage('a.png'), false);
  assert.equal(isAnimatedImage('asset', [{ ref: 'asset', kind: 'image', animated: true }]), true);
  assert.deepEqual(playbackSpeeds(.25), [.25,.5,1,2]);
  assert.deepEqual(playbackSpeeds(1), [.5,1,2]);
});

test('absent fade and zero corners allocate no canvas and do no drawing work', () => {
  const ctx = {}, frame = { x: 0, y: 0, width: 10, height: 10 };
  applyCanvasFade(ctx, null, frame);
  let draws = 0;
  drawClippedMedia(ctx, frame, {}, target => { assert.equal(target, ctx); draws++; });
  drawClippedMedia(ctx, frame, { radius: 0, corners: [0,0,0,0] }, () => draws++);
  assert.equal(draws, 2);
});
