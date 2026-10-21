import { createWidget } from '../../src/editor/widgets.js';
import { test, expect, expectAccessible, watchErrors, openSection } from './helpers.mjs';
const canvas = { width: 480, height: 320 }, at = (second = 0) => new Date(2026, 9, 21, 3, 0, second);
async function open(page) {
  await page.clock.setFixedTime(at());
  const clock = { ...createWidget('clock', canvas), id: 1, name: 'Clock', frame: { x: 100, y: 32, width: 256, height: 256 } };
  clock.kind.content.analog = {};
  const theme = { schema: 1, name: 'Clock canvas', canvas, orientation: 'portrait', refreshSeconds: 1,
    background: { type: 'color', color: '#000000ff' }, elements: [clock] };
  await page.route('**/demo-theme.js', route => route.fulfill({ contentType: 'text/javascript', body: `export const DEMO_THEME = ${JSON.stringify(theme)};` }));
  await page.goto('/index.html?demo=turing88'); await expect(page.locator('#theme-name')).toHaveValue('Clock canvas');
  await page.locator('#tab-layers').click(); await page.locator('.layer-name').click();
}
const tip = (page, hand) => page.locator(`#overlay [data-property-handle="clock-${hand}-length"]`);
const length = (page, t, hand) => page.locator('#inspector').getByRole('spinbutton', { name: t(`insp.clock.${hand}Length`), exact: true });

test('clock hand tips edit real length fields with one Undo and keyboard access', async ({ page, t }) => {
  const errors = watchErrors(page); await open(page); await openSection(page, 'shape');
  for (const hand of ['hour', 'minute', 'second']) await expect(tip(page, hand)).toHaveCount(1);
  const handle = tip(page, 'hour'), box = await handle.boundingBox();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2 + 20, box.y + box.height / 2, { steps: 5 });
  await expect(page.locator('.handle-readout')).toBeVisible(); await expect(page.locator('.canvas-field-active')).toHaveCount(1);
  expect(Number(await length(page, t, 'hour').inputValue())).toBeGreaterThan(100);
  const visible = await page.locator('#inspector').boundingBox(), active = await page.locator('.canvas-field-active').boundingBox();
  expect(active.y).toBeGreaterThanOrEqual(visible.y); expect(active.y + active.height).toBeLessThanOrEqual(visible.y + visible.height + 1);
  await page.mouse.up(); await page.locator('#undo').click(); await expect(length(page, t, 'hour')).toHaveValue('100');
  await handle.focus(); await handle.press('Shift+ArrowRight'); await expect(handle).toBeFocused();
  expect(Number(await length(page, t, 'hour').inputValue())).toBeGreaterThan(100);
  await page.locator('#undo').click(); await expect(length(page, t, 'hour')).toHaveValue('100');
  const field = length(page, t, 'minute'); await field.fill('50'); await field.press('Tab'); await expect(field).toHaveValue('50');
  await field.fill('100'); await field.press('Tab'); await expect(field).toHaveValue('100');
  await page.locator('#inspector').getByRole('group', { name: t('insp.clock.hands'), exact: true }).getByRole('button', { name: t('insp.clockPart.seconds'), exact: true }).click();
  await expect(tip(page, 'second')).toHaveCount(0); await expect(length(page, t, 'second')).toHaveCount(0);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('clock tips track real time while keeping native keyboard focus', async ({ page, t }) => {
  const errors = watchErrors(page); await open(page);
  const second = tip(page, 'second'), original = await second.boundingBox();
  await second.focus(); await page.clock.setFixedTime(at(30));
  await expect.poll(async () => (await second.boundingBox()).y).toBeGreaterThan(original.y + 100);
  await expect(second).toBeFocused(); await second.press('ArrowDown');
  expect(Number(await length(page, t, 'second').inputValue())).toBeGreaterThan(100);
  await expectAccessible(page); expect(errors).toEqual([]);
});

test('demo clock pixels preserve old themes and match native shortened hand samples', async ({ page }) => {
  await page.goto('/index.html?demo=turing88');
  const result = await page.evaluate(async () => {
    const { renderApprox } = await import('./demo-render.js'); const { createWidget } = await import('./editor/widgets.js');
    const canvas = { width: 256, height: 256 }, clock = createWidget('clock', canvas); clock.id = 1; clock.frame = { x: 0, y: 0, ...canvas };
    const analog = { face: '#282828ff', hourColor: '#ff0000ff', minuteColor: '#00ff00ff', secondColor: '#0000ffff' };
    const render = patch => { clock.kind.content.analog = { ...analog, ...patch }; return renderApprox({ canvas, elements: [clock], background: { type: 'color', color: '#000000ff' } }, new Date(2026, 9, 21, 3, 0, 30).getTime() / 1000).rgba; };
    const hash = bytes => bytes.reduce((hash, byte) => Math.imul(hash ^ byte, 0x01000193) >>> 0, 0x811c9dc5);
    const legacy = ['thin', 'rounded', 'arrow'].map(handShape => hash(render({ handShape })));
    const original = render({}), pixel = (bytes, [x, y]) => [...bytes.slice((y * 256 + x) * 4, (y * 256 + x + 1) * 4)];
    const unchanged = ['hourLength', 'minuteLength', 'secondLength'].every(key => render({ [key]: 1 }).every((b, i) => b === original[i]));
    const samples = [
      ['hourLength', [168, 128], [144, 128], [128, 68]],
      ['minuteLength', [128, 68], [128, 100], [158, 128]],
      ['secondLength', [128, 198], [128, 160], [158, 128]],
    ].map(([key, beyond, inside, untouched]) => {
      const bytes = render({ [key]: .5 });
      return { beyond: pixel(bytes, beyond), inside: pixel(bytes, inside), otherSame: pixel(bytes, untouched).every((b, i) => b === pixel(original, untouched)[i]),
        tailSame: pixel(bytes, [128, 110]).every((b, i) => b === pixel(original, [128, 110])[i]) };
    });
    return { legacy, unchanged, samples };
  });
  expect(result.legacy).toEqual([4231351915, 1096893813, 1012282381]);
  expect(result.unchanged).toBe(true);
  const colors = [[255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255]];
  for (const [index, sample] of result.samples.entries()) {
    expect(sample.beyond).toEqual([40, 40, 40, 255]);
    for (let component = 0; component < 4; component++) expect(Math.abs(sample.inside[component] - colors[index][component])).toBeLessThanOrEqual(8);
    expect(sample.otherSame).toBe(true); expect(sample.tailSame).toBe(true);
  }
});
