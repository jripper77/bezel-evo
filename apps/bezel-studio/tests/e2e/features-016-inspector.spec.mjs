// Features of 0.1.0-dev.016 in the inspector: radial fades for images and
// shapes, the player's and weather's own appearance, and property navigation
// that fits small and large windows.
import { test, expect, watchErrors, expectAccessible, clickAddWidget as add, openInspectorTab as tab } from './helpers.mjs';
test('image and shape share radial controls; player and weather have their own appearance',async ({page,t})=>{
 const errors=watchErrors(page);await page.goto('/index.html?demo=turing88');
 for(const widget of ['shape','image']) {
  await add(page,t,widget);
  await tab(page,t,'inspector.tab.look');
  // The transparency kind is a row of choices.
  const modes=page.locator('#inspector').getByRole('group',{name:t('fade.mode'),exact:true});
  const mode={selectOption:(kind)=>modes.getByRole('button',{name:t(`fade.${kind}`),exact:true}).click()};
  await mode.selectOption('radial');
  const center=page.locator('#inspector').getByRole('spinbutton',{name:t('fade.centerX'),exact:true});
  await expect(center).toHaveValue('50');await center.fill('25');await center.press('Tab');await expect(center).toHaveValue('25');
  await mode.selectOption('linear');await expect(page.locator('#inspector').getByRole('spinbutton',{name:t('shape.fadeAngle'),exact:true})).toBeVisible();
  // A linear fade moves its halfway point; a radial one has no midpoint.
  const midpoint=page.locator('#inspector').getByRole('spinbutton',{name:t('fade.midpoint')});
  await expect(midpoint).toHaveValue('50');await midpoint.fill('30');await midpoint.press('Tab');await expect(midpoint).toHaveValue('30');
  await mode.selectOption('radial');await expect(midpoint).toHaveCount(0);
 }
 await add(page,t,'player');await page.locator('#inspector').getByRole('button',{name:t('insp.otherApp'),exact:true}).click();await expect(page.locator('#inspector').getByRole('textbox',{name:t('player.source'),exact:true})).toBeVisible();
 await tab(page,t,'inspector.tab.look');const cover=page.locator('#inspector').getByRole('button',{name:t('insp.cover'),exact:true});await expect(cover).toHaveAttribute('aria-pressed','true');await cover.click();await expect(cover).toHaveAttribute('aria-pressed','false');
 await add(page,t,'weather');await tab(page,t,'inspector.tab.look');const styles=page.locator('#inspector').getByRole('group',{name:t('weather.iconStyle'),exact:true});
 const style=(name)=>styles.getByRole('button',{name:t(`weather.${name}`),exact:true});
 await style('colored').click();await expect(style('colored')).toHaveAttribute('aria-pressed','true');await style('dimensional').click();await expect(style('dimensional')).toHaveAttribute('aria-pressed','true');await expect(style('colored')).toHaveAttribute('aria-pressed','false');
 await expectAccessible(page);expect(errors).toEqual([]);
});
test('property navigation fits small and large windows',async ({page,t})=>{
 const errors=watchErrors(page);await page.goto('/index.html?demo=turing88');
 for(const size of [{width:1000,height:700},{width:1600,height:1000}]) {
  await page.setViewportSize(size);await page.locator('#tab-widgets').click();await add(page,t,'player');
  for(const pane of ['.sidebar','.rail','.library']) expect(await page.locator(pane).evaluate(n=>n.scrollWidth<=n.clientWidth+1),pane).toBe(true);
  expect(await page.locator('#inspector').evaluate(n=>n.scrollWidth<=n.clientWidth+1)).toBe(true);
 }
 await page.screenshot({path:'../../target/features-016-ui.png'});await expectAccessible(page);expect(errors).toEqual([]);
});
