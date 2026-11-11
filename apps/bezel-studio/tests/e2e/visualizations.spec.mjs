import { test, expect, watchErrors, expectAccessible, copyAndPaste, pickSensor, openInspectorTab, toggleButton, setToggle, openSection } from './helpers.mjs';

test('native visualization presets, scoped controls, second sensor and Undo',async({page,t},info)=>{
  const errors=watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  const detail=page.locator('#inspector');
  const add=async kind=>{
    await page.getByRole('button',{name:t('library.addWidget',{name:t(`widget.${kind}`)}),exact:true}).click();
    await openInspectorTab(page, t, 'inspector.tab.look');
    await openSection(page, 'layout');
  };
  const preset=name=>detail.locator('[data-section="visualization"]').getByRole('button',{name:t(`visual.${name}`),exact:true});
  await add('bar');await preset('matrix').click();
  await detail.getByRole('spinbutton',{name:t('visual.columns'),exact:true}).fill('12');
  await detail.getByRole('spinbutton',{name:t('visual.columns'),exact:true}).dispatchEvent('change');
  const height=detail.getByRole('spinbutton',{name:t('inspector.height'),exact:true});
  const beforeFit=await height.inputValue();
  await detail.getByRole('button',{name:t('visual.fitMatrix'),exact:true}).click();
  await expect(height).not.toHaveValue(beforeFit);
  await page.locator('#undo').click();
  await expect(height).toHaveValue(beforeFit);
  await copyAndPaste(page);
  await expect(preset('matrix')).toHaveAttribute('aria-pressed','true');
  await expect(detail.getByRole('spinbutton',{name:t('visual.columns'),exact:true})).toHaveValue('12');
  await preset('standard').click();await page.locator('#undo').click();
  await expect(preset('matrix')).toHaveAttribute('aria-pressed','true');
  await add('ring');await preset('pixel').click();
  await expect(detail.getByRole('spinbutton',{name:t('visual.rows'),exact:true})).toHaveCount(0);
  await expect(detail.getByRole('spinbutton',{name:t('visual.cells'),exact:true})).toBeVisible();
  await preset('dial').click();
  await setToggle(toggleButton(detail, t('visual.marker')), false);
  await detail.screenshot({path:info.outputPath('visualization-tiles.png')});
  // A graph's second series is its Data's "Two, mirrored" (D-18).
  await add('graph');await openInspectorTab(page, t, 'inspector.tab.data');
  await detail.getByRole('group',{name:t('insp.graphSeries'),exact:true}).getByRole('button',{name:t('insp.graphMirrored'),exact:true}).click();
  await expect(detail.getByRole('combobox',{name:t('visual.secondary'),exact:true})).toBeVisible();
  await pickSensor(page,t,detail,'gpu.usage',t('visual.secondary'));
  await expect(detail.getByRole('combobox',{name:t('visual.secondary'),exact:true})).toHaveAttribute('data-sensor-key','gpu.usage');
  await expect(detail.getByRole('spinbutton',{name:t('visual.secondaryMin'),exact:true})).toHaveCount(1);
  await expect(detail.getByRole('spinbutton',{name:t('visual.secondaryMax'),exact:true})).toHaveCount(1);
  // Each series has its own scale, side by side in Vertical scale; both
  // automatic share one peak until told otherwise.
  const scale=name=>detail.getByRole('group',{name,exact:true});
  const upperScale=scale(t('visual.primaryScale')),lowerScale=scale(t('visual.secondaryScale'));
  const pick=(group,mode)=>group.getByRole('button',{name:t(`insp.${mode}`),exact:true});
  await expect(pick(upperScale,'fixed')).toHaveAttribute('aria-pressed','true');
  await expect(pick(lowerScale,'fixed')).toHaveAttribute('aria-pressed','true');
  // Changing one series leaves the other as it was.
  await pick(upperScale,'automatic').click();
  await expect(pick(upperScale,'automatic')).toHaveAttribute('aria-pressed','true');
  await expect(pick(lowerScale,'fixed')).toHaveAttribute('aria-pressed','true');
  await expect(detail.getByRole('spinbutton',{name:t('visual.secondaryMax'),exact:true})).toBeEnabled();
  await expect(toggleButton(detail,t('visual.sharedScale'))).toHaveCount(0);
  await pick(lowerScale,'automatic').click();
  await expect(detail.getByRole('spinbutton',{name:t('visual.secondaryMax'),exact:true})).toBeDisabled();
  const shared=toggleButton(detail,t('visual.sharedScale'));
  await expect(shared).toHaveAttribute('aria-pressed','true');
  // Automatic bounds show the readings: linked, both series reach the same
  // maximum; a mirrored series starts at 0.
  const upperMax=detail.getByRole('spinbutton',{name:t('inspector.max'),exact:true}),lowerMax=detail.getByRole('spinbutton',{name:t('visual.secondaryMax'),exact:true});
  await expect.poll(async()=>(await upperMax.inputValue())!=='100'||(await lowerMax.inputValue())!=='100').toBe(true);
  await expect.poll(async()=>[await upperMax.inputValue()===await lowerMax.inputValue()]).toEqual([true]);
  await expect(detail.getByRole('spinbutton',{name:t('visual.secondaryMin'),exact:true})).toHaveValue('0');
  await setToggle(shared,false);
  await expect(detail.getByText(t('visual.ownScaleHint'),{exact:true})).toBeVisible();
  await detail.locator('[data-section="source"]').screenshot({path:info.outputPath('mirrored-source.png')});
  await detail.locator('[data-section="scale"]').screenshot({path:info.outputPath('mirrored-scales.png')});
  await page.locator('#undo').click();
  await expect(toggleButton(detail,t('visual.sharedScale'))).toHaveAttribute('aria-pressed','true');
  await page.locator('#undo').click();
  await expect(pick(lowerScale,'fixed')).toHaveAttribute('aria-pressed','true');
  await expect(pick(upperScale,'automatic')).toHaveAttribute('aria-pressed','true');
  await expectAccessible(page);expect(errors).toEqual([]);
});
