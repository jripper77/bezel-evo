import { test, expect, clickAddWidget, openSection, expectAccessible, watchErrors } from './helpers.mjs';
const panel = page => page.locator('#inspector');
const button = (page, t, key) => panel(page).getByRole('button', { name: t(key), exact: true });
const number = (page, t, key) => panel(page).getByRole('spinbutton', { name: t(key), exact: true });
async function change(input, value) { await input.fill(String(value)); await input.press('Tab'); }

test('clock parts, blinking separators and analog controls keep the original digital pattern', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88'); await clickAddWidget(page, t, 'clock');
  await openSection(page, 'shape');
  await panel(page).getByRole('group', { name: t('insp.clock.parts'), exact: true }).getByRole('button', { name: t('insp.clockPart.minutes'), exact: true }).click();
  await change(number(page, t, 'insp.partSize'), 55); await expect(number(page, t, 'insp.partSize')).toHaveValue('55');
  await page.locator('#undo').click(); await expect(number(page, t, 'insp.partSize')).toHaveValue('');
  await openSection(page, 'behavior'); await button(page, t, 'insp.clock.blink').click(); await expect(button(page, t, 'insp.clock.blink')).toHaveAttribute('aria-pressed', 'true');
  await button(page, t, 'insp.clock.analog').click(); await expect(panel(page).locator('[data-inspector-section="text"]')).toHaveCount(0);
  await openSection(page, 'data'); await button(page, t, 'insp.clock.date.day').click();
  await button(page, t, 'insp.clock.numbers.roman').click(); await panel(page).getByRole('group', { name: t('insp.clock.hands'), exact: true }).getByRole('button', { name: t('insp.clockPart.seconds'), exact: true }).click();
  await expect(panel(page).locator('[data-inspector-section="behavior"]')).toHaveCount(0);
  await page.locator('#undo').click(); await openSection(page, 'behavior'); await button(page, t, 'insp.transitionSmooth').focus(); await button(page, t, 'insp.transitionSmooth').press('Enter');
  await button(page, t, 'insp.clock.digital').click(); await openSection(page, 'data');
  await expect(panel(page).getByRole('combobox', { name: t('inspector.pattern'), exact: true })).toHaveValue('%H:%M');
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('weather animation and own icon color remain available only where applicable', async ({ page, t }) => {
  const errors = watchErrors(page); await page.emulateMedia({ reducedMotion: 'reduce' }); await page.goto('/index.html?demo=turing88'); await clickAddWidget(page, t, 'weather');
  await openSection(page, 'behavior'); await button(page, t, 'insp.weather.motion.lively').click();
  await panel(page).getByRole('group', { name: t('insp.playback.speed'), exact: true }).getByRole('button', { name: '2×', exact: true }).click();
  await openSection(page, 'color'); await expect(button(page, t, 'insp.weather.iconColor')).toBeVisible();
  await openSection(page, 'shape'); await button(page, t, 'weather.colored').click(); await expect(button(page, t, 'insp.weather.iconColor')).toHaveCount(0);
  await button(page, t, 'icons.filled').click(); await expect(button(page, t, 'insp.weather.iconColor')).toBeVisible();
  await button(page, t, 'weather.icon').click(); await expect(panel(page).locator('[data-inspector-section="behavior"]')).toHaveCount(0);
  await page.locator('#undo').click(); await openSection(page, 'text'); await expect(button(page, t, 'insp.tabularDigits')).toHaveAttribute('aria-pressed', 'true');
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('player stopped controls stay in Data and only its cover receives transparency', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88'); await clickAddWidget(page, t, 'player');
  await expect(panel(page).locator('[data-inspector-section="behavior"]')).toHaveCount(0);
  await openSection(page, 'data'); await button(page, t, 'insp.stoppedHide').click(); await expect(panel(page).getByRole('textbox', { name: t('player.empty'), exact: true })).toHaveCount(0);
  await button(page, t, 'insp.stoppedText').click(); await expect(panel(page).getByRole('textbox', { name: t('player.empty'), exact: true })).toBeVisible();
  await openSection(page, 'shape'); const cover = panel(page).locator('[data-section="cover-fade"]');
  await cover.getByRole('button', { name: t('fade.radial'), exact: true }).click(); await expect(cover.getByRole('spinbutton', { name: t('fade.centerX'), exact: true })).toBeVisible();
  await page.locator('#undo').click(); await expect(cover.getByRole('button', { name: t('fade.none'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await openSection(page, 'color'); await expect(button(page, t, 'insp.player.progressFill')).toBeVisible(); await expectAccessible(page); expect(errors).toEqual([]);
});

test('spectrum bands, style and movement follow their respective sections', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88'); await clickAddWidget(page, t, 'spectrum');
  await openSection(page, 'data'); await change(number(page, t, 'spectrum.bands'), 40); await expect(number(page, t, 'spectrum.bands')).toHaveValue('40');
  await openSection(page, 'shape'); await button(page, t, 'graph.line').click(); await expect(number(page, t, 'spectrum.thickness')).toHaveCount(0); await expect(number(page, t, 'spectrum.stroke')).toBeVisible();
  await openSection(page, 'behavior'); await button(page, t, 'spectrum.snappy').click(); await expect(button(page, t, 'spectrum.snappy')).toHaveAttribute('aria-pressed', 'true');
  await openSection(page, 'color'); await button(page, t, 'insp.gradient').click(); await expect(button(page, t, 'ring.endColor')).toBeVisible(); await expectAccessible(page); expect(errors).toEqual([]);
});

test('GIF has playback, independent image corners and nine cover anchors; still images have no playback', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=gif'); await page.locator('#tab-layers').click(); await page.locator('.layer-name').filter({ hasText: 'Spinner' }).click();
  await openSection(page, 'shape'); await button(page, t, 'fit.cover').click(); await button(page, t, 'insp.image.anchor.topLeft').click();
  await expect(button(page, t, 'insp.image.anchor.topLeft')).toHaveAttribute('aria-pressed', 'true');
  await change(number(page, t, 'inspector.radius'), 12); await button(page, t, 'insp.cornersEach').click(); await change(number(page, t, 'inspector.topLeft'), 3);
  await page.locator('#undo').click(); await expect(number(page, t, 'inspector.topLeft')).toHaveValue('12');
  await openSection(page, 'behavior'); await button(page, t, 'insp.playback.once').click(); await expect(button(page, t, 'insp.playback.once')).toHaveAttribute('aria-pressed', 'true');
  await panel(page).getByRole('group', { name: t('insp.playback.speed'), exact: true }).getByRole('button', { name: '2×', exact: true }).click();
  await page.locator('#tab-widgets').click(); await clickAddWidget(page, t, 'image'); await expect(panel(page).locator('[data-inspector-section="behavior"]')).toHaveCount(0); await expectAccessible(page); expect(errors).toEqual([]);
});

test('shape reuses corners, paint and transparency without a Behavior section', async ({ page, t }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88'); await page.locator('#tab-media').click(); await page.getByRole('tab', { name: t('media.icons'), exact: true }).click(); await page.locator('#icon-search').fill('cpu'); await page.locator('[data-icon="cpu"]').click(); await openSection(page, 'data'); const source = await panel(page).getByRole('combobox', { name: t('common.image'), exact: true }).inputValue(); await page.locator('#tab-widgets').click(); await clickAddWidget(page, t, 'shape');
  await openSection(page, 'shape'); await change(number(page, t, 'inspector.radius'), 18);
  await openSection(page, 'color'); await button(page, t, 'insp.gradient').click(); await expect(button(page, t, 'ring.endColor')).toBeVisible();
  await panel(page).getByRole('combobox', { name: t('common.image'), exact: true }).selectOption(source);
  await button(page, t, 'shape.repeat.tile').click(); await expect(number(page, t, 'shape.imageScale')).toBeVisible(); await change(number(page, t, 'shape.imageOpacity'), 60); await expect(number(page, t, 'shape.imageOpacity')).toHaveValue('60');
  const fade = panel(page).locator('[data-section="transparency"]'); await fade.getByRole('button', { name: t('fade.linear'), exact: true }).click(); await expect(fade.getByRole('spinbutton', { name: t('shape.fadeAngle'), exact: true })).toBeVisible();
  await page.locator('#undo').click(); await expect(fade.getByRole('button', { name: t('fade.none'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(panel(page).locator('[data-inspector-section="behavior"]')).toHaveCount(0); await expectAccessible(page); expect(errors).toEqual([]);
});

test('demo pixels match corner clipping, isolated cover fades and monochrome icon overrides', async ({ page }) => {
  const errors = watchErrors(page); await page.goto('/index.html?demo=turing88');
  const result = await page.evaluate(async () => {
    const { renderApprox, drawClippedMedia, drawPlayerCover } = await import('./demo-render.js');
    const { createDemoBackend } = await import('./demo-backend.js');
    const { createWidget } = await import('./editor/widgets.js');
    const base = await createDemoBackend('turing88').openTheme('demo://Demo');
    const widget = (name, frame) => ({ ...createWidget(name, base.canvas), id: 1, name, frame, opacity: 1, visible: true });
    const scene = e => ({ ...base, canvas: { width: e.frame.width, height: e.frame.height }, background: { type: 'color', color: '#000000ff' }, elements: [e] });
    const weather = widget('weather', { x: 0, y: 0, width: 160, height: 40 }); weather.kind.style.size = 8; weather.kind.content.iconSize = 32; weather.kind.content.iconGap = 2;
    const icons = ['outline', 'filled', 'colored', 'dimensional'].map(style => {
      weather.kind.content.iconStyle = style; weather.kind.content.iconColor = null;
      const plain = renderApprox(scene(weather), 0, null, new Map(), 1, false).rgba;
      weather.kind.content.iconColor = '#ff0000ff'; const tinted = renderApprox(scene(weather), 0, null, new Map(), 1, false).rgba;
      return { changed: plain.some((byte, i) => byte !== tinted[i]), textSame: plain.every((byte, i) => Math.floor(i / 4) % 160 < 34 || byte === tinted[i]) };
    });
    const source = new OffscreenCanvas(10, 10), sourceContext = source.getContext('2d'); sourceContext.fillStyle = 'white'; sourceContext.fillRect(0,0,10,10);
    const png = await createImageBitmap(await source.convertToBlob({ type: 'image/png' }));
    const canvas = new OffscreenCanvas(10,10), ctx = canvas.getContext('2d');
    drawClippedMedia(ctx, { x:0,y:0,width:10,height:10 }, { corners:[5,0,0,0] }, target => target.drawImage(png,0,0));
    const image = ctx.getImageData(0,0,10,10).data; png.close();
    const gif = widget('image', { x:0,y:0,width:10,height:10 }); gif.kind.asset = 'test.gif'; gif.kind.fit = 'fill'; gif.kind.corners = [5,0,0,0];
    const first = renderApprox(scene(gif),0).rgba, second = renderApprox(scene(gif),.15).rgba;
    const covers = [null, { angle:0,start:1,end:0,midpoint:.5 }, { radial:[.5,.5,.5],angle:0,start:1,end:0,midpoint:.5 }].map(coverFade => {
      const canvas = new OffscreenCanvas(40,20), ctx = canvas.getContext('2d'); ctx.fillStyle = 'lime'; ctx.fillRect(20,0,20,20);
      drawPlayerCover(ctx, { x:0,y:0,width:20,height:20 }, { coverFade }, target => { target.fillStyle = 'white'; target.fillRect(0,0,20,20); });
      const pixels = ctx.getImageData(0,0,40,20).data;
      const alpha = (x,y) => pixels[(y*40+x)*4+3];
      return { left: alpha(1,10), right: alpha(18,10), center: alpha(10,10), edge: alpha(1,1), outside: [...pixels.slice((10*40+25)*4,(10*40+25)*4+4)] };
    });
    return { icons, pngCorner:image[3], pngSquare:image[9*4+3], gifCorner:[...first.slice(0,4)], gifMoves:first[(5*10+5)*4] !== second[(5*10+5)*4] || first[(5*10+5)*4+1] !== second[(5*10+5)*4+1], covers };
  });
  expect(result.icons).toEqual([{changed:true,textSame:true},{changed:true,textSame:true},{changed:false,textSame:true},{changed:false,textSame:true}]);
  expect(result.pngCorner).toBe(0); expect(result.pngSquare).toBe(255); expect(result.gifCorner).toEqual([0,0,0,255]); expect(result.gifMoves).toBe(true);
  expect(result.covers[0].left).toBe(255); expect(result.covers[1].left).toBeGreaterThan(result.covers[1].right); expect(result.covers[2].center).toBeGreaterThan(result.covers[2].edge);
  expect(result.covers.every(cover => JSON.stringify(cover.outside) === JSON.stringify([0,255,0,255]))).toBe(true);
  await expectAccessible(page); expect(errors).toEqual([]);
});
