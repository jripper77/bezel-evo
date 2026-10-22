import { test, expect, watchErrors, expectAccessible, clickAddWidget as add, openSection } from './helpers.mjs';

test('value transitions offer inertia with settling time and undo',async({page,t},info)=>{
  const errors=watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await add(page,t,'value');
  await openSection(page,'behavior');
  const section=page.locator('[data-section="transition"]');
  await section.getByRole('button',{name:t('insp.transitionInertia'),exact:true}).click();
  await expect(section.getByRole('button',{name:t('insp.transitionInertia'),exact:true})).toHaveAttribute('aria-pressed','true');
  await expect(section).toContainText(t('insp.transitionSettling'));
  await expect(section).toContainText(t('insp.transitionInertiaHint'));
  const settling=section.getByRole('spinbutton',{name:t('insp.transitionSettling'),exact:true});
  await settling.press('ArrowUp');
  await expect(settling).toHaveValue('450');
  await page.locator('#undo').click();
  await expect(settling).toHaveValue('400');
  await page.locator('#undo').click();
  await expect(section.getByRole('button',{name:t('insp.transitionSmooth'),exact:true})).toHaveAttribute('aria-pressed','true');
  await section.getByRole('button',{name:t('insp.transitionInertia'),exact:true}).click();
  await expectAccessible(page);
  await section.screenshot({path:info.outputPath('value-inertia.png')});
  expect(errors).toEqual([]);
});
