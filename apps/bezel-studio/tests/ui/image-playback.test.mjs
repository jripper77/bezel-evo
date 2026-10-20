import { test } from 'node:test';
import assert from 'node:assert/strict';
import { DEMO_GIF_FRAME_MS, demoGifDue, demoGifFrame } from '../../src/demo-render.js';
import { demoNextChange } from '../../src/demo-backend.js';

test('a demo GIF plays at its speed and once stays on its last frame, like the backend', () => {
  assert.equal(DEMO_GIF_FRAME_MS, 100);
  assert.equal(demoGifFrame({}, 150), 1);
  assert.equal(demoGifFrame({}, 450), 0, 'four frames, looping');
  assert.equal(demoGifFrame({ speed: 2 }, 150), 3);
  assert.equal(demoGifFrame({ speed: 0.5 }, 150), 0);
  assert.equal(demoGifFrame({ speed: 99 }, 30), 1, 'at most 4x');
  assert.equal(demoGifFrame({ once: true }, 450), 3, 'the last frame stays');
  assert.equal(demoGifFrame({ once: true }, 150), 1);
  assert.equal(demoGifDue({}, 150), 50);
  assert.equal(demoGifDue({ speed: 2 }, 150), 50, '300 ms in, the next frame at 400');
  assert.equal(demoGifDue({ once: true }, 350), null);
  assert.equal(demoGifDue({ once: true }, 250), 50);
});

test('the demo asks for GIF frames only while one plays', () => {
  const theme = (kind) => ({ elements: [{ id: 1, visible: true, opacity: 1, kind: { type: 'image', asset: 'a.gif', fit: 'fill', ...kind } }] });
  assert.equal(demoNextChange(theme({}), 150), 50);
  assert.equal(demoNextChange(theme({ once: true }), 450), null);
  assert.equal(demoNextChange(theme({ speed: 2 }), 120), 30, '240 ms in, the next frame at 300');
  assert.equal(demoNextChange({ elements: [] }, 150), null);
});
