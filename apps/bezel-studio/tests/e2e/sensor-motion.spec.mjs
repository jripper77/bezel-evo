import { pickSensor, test, expect, watchErrors, expectAccessible, clickAddWidget as add, toggleButton, setToggle } from './helpers.mjs';

test('sensor motion animates a custom shape, edits its response, and survives undo', async ({page,t},info) => {
  const errors=watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await add(page,t,'shape');
  const inspector=page.locator('#inspector');
  const toggle=toggleButton(inspector, t('insp.sensorMotionEnabled'));
  await setToggle(toggle, true);
  const source=inspector.getByRole('combobox',{name:t('insp.motionSource'),exact:true});
  await pickSensor(page, t, inspector, 'cpu.usage', t('insp.motionSource'));
  const reference=inspector.getByRole('spinbutton',{name:t('insp.motionReference'),exact:true});
  await reference.fill('100'); await reference.press('Tab');
  await expect(source).toHaveAttribute('data-sensor-key', 'cpu.usage');
  const pixels=()=>inspector.locator('.inspector-preview canvas').evaluate(canvas=>canvas.toDataURL());
  await expect(inspector.locator('.inspector-preview canvas')).toBeVisible();
  const first=await pixels();
  await expect.poll(pixels).not.toBe(first);
  await setToggle(toggleButton(inspector, t('insp.motionReverse')), true);
  await expect(toggleButton(inspector, t('insp.motionReverse'))).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#undo').click();
  await expect(toggleButton(inspector, t('insp.motionReverse'))).toHaveAttribute('aria-pressed', 'false');
  await expectAccessible(page);
  await inspector.locator('[data-section="sensor-motion"]').scrollIntoViewIfNeeded();
  await page.screenshot({path:info.outputPath('sensor-motion.png')});
  await setToggle(toggle, false);
  await expect(source).toHaveCount(0);
  expect(errors).toEqual([]);
});
