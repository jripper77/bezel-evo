import { test, expect, watchErrors, expectAccessible, clickAddWidget as add, openInspectorTab as tab, toggleButton, setToggle, choose, chosen, openPreferences } from './helpers.mjs';
async function setup(page,t,player=true) {
  await page.goto('/index.html?demo=turing88');
  if(player) await add(page,t,'player');
  await add(page,t,'spectrum'); return page.locator('#inspector');
}
test('spectrum follows one Player; demo, presets, gradients and band thresholds use shared controls',async({page,t})=>{
  const errors=watchErrors(page), inspector=await setup(page,t);
  const follow=inspector.getByRole('combobox',{name:t('spectrum.follow'),exact:true});
  await expect(follow).toHaveValue('');await expect(inspector.getByText(t('spectrum.playerMissing'),{exact:true})).toHaveCount(0);
  await expect(follow.locator('option')).toHaveCount(2);await follow.selectOption({index:1});await expect(follow).not.toHaveValue('');
  const count=inspector.getByRole('spinbutton',{name:t('spectrum.bands'),exact:true});await expect(count).toHaveValue('32');
  await count.fill('48');await count.press('Tab');await expect(count).toHaveValue('48');
  const gain=inspector.getByRole('spinbutton',{name:t('spectrum.gain'),exact:true});await expect(gain).toHaveValue('1');
  await gain.fill('0.5');await gain.press('Tab');await expect(gain).toHaveValue('0.5');await expect(inspector.getByText(t('spectrum.gainHint'),{exact:true})).toBeVisible();
  await inspector.getByRole('button',{name:t('spectrum.demo'),exact:true}).click();await expect(inspector.getByText(t('spectrum.demoHint'),{exact:true})).toBeVisible();
  const canvas=inspector.locator('canvas');const before=await canvas.evaluate(c=>c.toDataURL());await expect.poll(()=>canvas.evaluate(c=>c.toDataURL())).not.toBe(before);
  await tab(page,t,'inspector.tab.look');
  await expect(inspector.getByRole('button',{name:t('spectrum.balanced'),exact:true})).toHaveAttribute('aria-pressed','true');
  await expect(toggleButton(inspector, t('spectrum.peaks'))).toHaveAttribute('aria-pressed', 'true');
  await inspector.getByRole('button',{name:t('spectrum.snappy'),exact:true}).click();await expect(inspector.getByRole('button',{name:t('spectrum.snappy'),exact:true})).toHaveAttribute('aria-pressed','true');
  await page.getByRole('button',{name:t('top.undo'),exact:true}).click();await expect(inspector.getByRole('button',{name:t('spectrum.balanced'),exact:true})).toHaveAttribute('aria-pressed','true');
  await choose(inspector,t('inspector.fill'),'gradient');await expect.poll(()=>chosen(inspector,t('inspector.fill'))).toBe('gradient');
  await setToggle(toggleButton(inspector, t('insp.colorByValue')), true);await expect(inspector.getByText(t('spectrum.levelScale'),{exact:true})).toBeVisible();
  await expectAccessible(page);expect(errors).toEqual([]);
});
test('the whole PC output needs no Player and hides the Player controls (D-AS-13)',async({page,t})=>{
  const errors=watchErrors(page), inspector=await setup(page,t,false);
  await expect(inspector.getByRole('status')).toHaveText(t('spectrum.playerMissing'));
  const system=inspector.getByRole('button',{name:t('spectrum.audioSystem'),exact:true});
  await system.click();await expect(system).toHaveAttribute('aria-pressed','true');
  await expect(inspector.getByText(t('spectrum.systemHint'),{exact:true})).toBeVisible();
  await expect(inspector.getByRole('combobox',{name:t('spectrum.follow'),exact:true})).toHaveCount(0);
  await expect(inspector.getByText(t('spectrum.playerMissing'),{exact:true})).toHaveCount(0);
  await page.getByRole('button',{name:t('top.undo'),exact:true}).click();
  await expect(inspector.getByRole('button',{name:t('spectrum.audioPlayer'),exact:true})).toHaveAttribute('aria-pressed','true');
  await expectAccessible(page);expect(errors).toEqual([]);
});
test('missing Player is explicit and audio capture is on until turned off locally',async({page,t})=>{
  const errors=watchErrors(page), inspector=await setup(page,t,false);
  await expect(inspector.getByRole('status')).toHaveText(t('spectrum.playerMissing'));
  await expect(inspector.getByText(t('spectrum.browser'),{exact:true})).toBeVisible();
  await openPreferences(page,t);const dialog=page.getByRole('dialog',{name:t('prefs.title')});
  const consent=dialog.getByRole('switch',{name:t('spectrum.consent'),exact:true});await expect(consent).toBeChecked();await expect(dialog.getByText(t('spectrum.privacy'),{exact:true})).toBeVisible();
  await consent.uncheck();await expect(consent).not.toBeChecked();await expectAccessible(page);
  await dialog.getByRole('button',{name:t('common.done'),exact:true}).click();await openPreferences(page,t);
  await expect(page.getByRole('switch',{name:t('spectrum.consent'),exact:true})).not.toBeChecked();expect(errors).toEqual([]);
});
test('the bars corners switch between one radius and four, under their own title',async({page,t})=>{
  const errors=watchErrors(page), inspector=await setup(page,t);await tab(page,t,'inspector.tab.look');
  await expect(inspector.getByRole('heading',{name:t('insp.cornersTitle'),exact:true})).toBeVisible();
  await expect(inspector.getByRole('heading',{name:t('insp.corners'),exact:true})).toHaveCount(0);
  const each=inspector.getByRole('button',{name:t('insp.cornersEach'),exact:true});
  const control=inspector.locator('.corner-radii');
  await expect(each).toHaveAttribute('aria-pressed','false');await expect(control.getByRole('spinbutton')).toHaveCount(1);
  await expect(control.getByRole('button')).toHaveCount(1);
  await expect(control.locator('.corner-drawing')).toHaveCount(0);
  const radius=control.getByRole('spinbutton');
  const previous=await radius.inputValue();
  await radius.fill('7');await radius.press('Tab');await expect(radius).toHaveValue('7');
  await page.locator('#undo').click();await expect(radius).toHaveValue(previous);
  await each.focus();await page.keyboard.press('Space');await expect(each).toHaveAttribute('aria-pressed','true');await expect(control.getByRole('spinbutton')).toHaveCount(4);
  await expect(control.locator('.corner-drawing')).toBeVisible();
  for(const field of await control.getByRole('spinbutton').all())await expect(field).toHaveValue(previous);
  // Spectrum stores four radii in either mode; its chosen editing mode is UI
  // memory. Undo restores a corner value while retaining that editing mode.
  const topLeft=control.getByRole('spinbutton',{name:t('inspector.topLeft'),exact:true});
  await topLeft.fill('7');await topLeft.press('Tab');await expect(topLeft).toHaveValue('7');
  await page.locator('#undo').click();await expect(topLeft).toHaveValue(previous);await expect(each).toHaveAttribute('aria-pressed','true');
  await each.click();await expect(each).toHaveAttribute('aria-pressed','false');await expect(control.getByRole('spinbutton')).toHaveCount(1);
  expect(errors).toEqual([]);
});
test('each style shows only its own fields, one Undo step per change (D-AS-11)',async({page,t})=>{
  const errors=watchErrors(page), inspector=await setup(page,t);
  await inspector.getByRole('button',{name:t('spectrum.demo'),exact:true}).click();
  await tab(page,t,'inspector.tab.look');
  // The five styles are picture tiles named after them.
  const LABELS={mirrored:'spectrum.mirrored',bars:'graph.bars',blocks:'spectrum.blocks',line:'graph.line',radial:'fade.radial'};
  const tile=v=>inspector.getByRole('group',{name:t('spectrum.style'),exact:true}).getByRole('button',{name:t(LABELS[v]),exact:true});
  const style={selectOption:v=>tile(v).click()};await expect(tile('mirrored')).toHaveAttribute('aria-pressed','true');
  const field=name=>inspector.getByRole('spinbutton',{name:t(name),exact:true});
  const toggle=name=>toggleButton(inspector, t(name));
  await expect(toggle('spectrum.centre')).toBeVisible();await expect(field('spectrum.block')).toHaveCount(0);
  await style.selectOption('blocks');await expect(field('spectrum.block')).toHaveValue('6');
  await expect(inspector.getByText(t('spectrum.blockColors'),{exact:true})).toBeVisible();await expect(toggle('spectrum.centre')).toHaveCount(0);
  await style.selectOption('line');await expect(field('spectrum.stroke')).toHaveValue('2');await expect(toggle('spectrum.fill')).toHaveAttribute('aria-pressed', 'true');
  await expect(field('spectrum.thickness')).toHaveCount(0);await expect(toggle('spectrum.peaks')).toHaveCount(0);
  await expect(inspector.getByRole('heading',{name:t('insp.cornersTitle'),exact:true})).toHaveCount(0);
  await style.selectOption('radial');await expect(field('spectrum.inner')).toHaveValue('40');await expect(toggle('spectrum.ring')).toBeVisible();
  await style.selectOption('bars');await expect(toggle('spectrum.baseline')).toBeVisible();
  await page.getByRole('button',{name:t('top.undo'),exact:true}).click();await expect(tile('radial')).toHaveAttribute('aria-pressed','true');
  await expectAccessible(page);expect(errors).toEqual([]);
});
