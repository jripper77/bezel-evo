import { test, expect, watchErrors, expectAccessible, pickSensor, clickAddWidget as add, toggleButton, setToggle } from './helpers.mjs';

test('pointer selection survives native focus moving from search into a result',async({page,t})=>{
  const errors=watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await add(page,t,'value');
  const detail=page.locator('#inspector');
  const source=detail.getByRole('combobox',{name:t('inspector.sensor'),exact:true});
  await pickSensor(page,t,detail,'cpu.temperature');
  await expect(source).toHaveAttribute('data-sensor-key','cpu.temperature');
  await pickSensor(page,t,detail,'cpu.usage');
  await expect(source).toHaveAttribute('data-sensor-key','cpu.usage');
  expect(errors).toEqual([]);
});

test('shared sensor picker searches names, keys and probes and supports keyboard selection', async ({page,t},info)=>{
  const errors=watchErrors(page);
  await page.goto('/index.html?demo=turing88&libre=partial');
  await add(page,t,'value');
  const source=page.locator('#inspector').getByRole('combobox',{name:t('inspector.sensor'),exact:true});
  await source.click();
  const popup=page.getByRole('dialog',{name:t('inspector.sensor'),exact:true});
  const search=popup.getByRole('searchbox',{name:t('insp.sensorPickerSearch'),exact:true});
  await expect(search).toBeFocused();
  await search.fill('LibreHardwareMonitor');
  await expect(popup.getByRole('option')).toHaveCount(3);
  await expect(popup.locator('[data-state="unavailable"]')).toHaveCount(1);
  await expectAccessible(page);
  await popup.screenshot({path:info.outputPath('sensor-picker.png')});
  await search.fill('no-such-sensor');
  await expect(popup.getByRole('option')).toHaveCount(0);
  await expect(popup).toContainText(t('insp.sensorPickerEmpty'));
  await search.fill('GPU TEMPERATURE');
  await expect(popup.getByRole('option')).toHaveCount(1);
  await search.press('ArrowDown');
  await expect(popup.getByRole('option')).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(popup).toHaveCount(0);
  await expect(source).toHaveAttribute('data-sensor-key','gpu.temperature');
  await expect(source).toBeFocused();
  await source.press('ArrowDown');
  await expect(search).toBeFocused();
  await search.fill('system.hostname');
  await expect(popup.getByRole('option')).toHaveCount(1);
  await search.press('Escape');
  await expect(source).toHaveAttribute('data-sensor-key','gpu.temperature');
  await expect(source).toBeFocused();
  await page.locator('#undo').click();
  await expect(source).toHaveAttribute('data-sensor-key','cpu.usage');
  expect(errors).toEqual([]);
});

test('motion uses the same picker with numeric scope and practical controls',async({page,t})=>{
  const errors=watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await add(page,t,'shape');
  const section=page.locator('[data-section="sensor-motion"]');
  await setToggle(toggleButton(section, t('insp.sensorMotionEnabled')), true);
  await expect(section.getByRole('spinbutton',{name:t('insp.motionPeriod'),exact:true})).toHaveValue('1');
  await section.getByRole('button',{name:t('insp.motionImmediate'),exact:true}).click();
  await expect(section.getByRole('button',{name:t('insp.motionImmediate'),exact:true})).toHaveAttribute('aria-pressed','true');
  const period=section.getByRole('spinbutton',{name:t('insp.motionPeriod'),exact:true});
  await period.fill('0.5');await period.press('Tab');
  await section.getByRole('combobox',{name:t('insp.motionSource'),exact:true}).click();
  const popup=page.getByRole('dialog',{name:t('insp.motionSource'),exact:true});
  await popup.getByRole('searchbox').fill('system.hostname');
  await expect(popup.getByRole('option')).toHaveCount(0);
  await page.keyboard.press('Escape');
  await expect(period).toHaveValue('0.5');
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('hundreds of sensors scroll vertically and live readings preserve keyboard focus',async({page,t})=>{
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  await page.evaluate(async()=>{
    const {sensorPicker}=await import('/ui/sensor-picker.js');
    const {translator}=await import('/i18n/index.js');
    const fixture={readings:{},chosen:null};
    const catalog=Array.from({length:700},(_,i)=>({key:`fixture.${i}`,label:`Sensor ${i}`,category:['cpu','gpu','memory'][i%3],source:'Windows',quantity:'rpm'}));
    const wrap=document.createElement('div');wrap.id='sensor-picker-fixture';wrap.style.cssText='position:fixed;left:350px;top:80px;width:300px;z-index:25';
    wrap.append(sensorPicker({t:translator(document.documentElement.lang),key:'fixture.400',catalog:()=>catalog,readings:()=>fixture.readings,onChange:key=>{fixture.chosen=key;},label:'Fixture'}));
    document.body.append(wrap);window.sensorPickerFixture=fixture;
  });
  await page.locator('#sensor-picker-fixture [role="combobox"]').click();
  const popup=page.getByRole('dialog',{name:'Fixture',exact:true});
  const list=popup.getByRole('listbox');
  expect(await list.evaluate(node=>node.scrollHeight>node.clientHeight && node.scrollWidth<=node.clientWidth)).toBe(true);
  const search=popup.getByRole('searchbox');
  await search.fill('fixture.600');await search.press('ArrowDown');
  const row=popup.getByRole('option');
  await expect(row).toBeFocused();
  await page.evaluate(async()=>{
    window.sensorPickerFixture.readings={'fixture.600':{value:0,display:'0 RPM'}};
    (await import('/ui/sensor-picker.js')).refreshSensorPicker();
  });
  await expect(row).toBeFocused();
  await expect(row).toHaveAttribute('data-state','available');
  await expect(row).toContainText('0 RPM');
  await expectAccessible(page);
  await page.keyboard.press('Enter');
  expect(await page.evaluate(()=>window.sensorPickerFixture.chosen)).toBe('fixture.600');
});
