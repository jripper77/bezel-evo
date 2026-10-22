// The card's inspector (artboard Card, D-2026-10-20-studio-ux-v2-03): the
// Faces section with the face list, its ⋮ menu, the shared background row
// and "Move"; the Behavior section with the Card's one transition, the
// timed triggers, the rule sentences, their order and the common return.
// Every edit is one Undo step.
import { test, expect, watchErrors, expectAccessible, clickAddWidget as add, openInspectorTab as tab, openSection, setPanelWidths, cardFaceList, faceAction } from './helpers.mjs';

/** The Card's Faces and Behavior sections. */
const facesRegion = (inspector, t) => inspector.getByRole('region', { name: t('card.tab.faces'), exact: true });
const behaviorRegion = (inspector, t) => inspector.getByRole('region', { name: t('insp.section.behavior'), exact: true });

/** A card with two faces: a text in the base, a ring in face 1, a bar and a graph in face 2 (in view). */
async function twoFaceCard(page, t) {
  await page.goto('/index.html?demo=turing88');
  await expect(page.locator('#theme-name')).toHaveValue('Demo');
  const inspector = page.locator('#inspector');
  await add(page, t, 'card');
  const cardName = await inspector.getByLabel(t('common.name'), { exact: true }).inputValue();
  const selectCard = async () => {
    await page.locator('#tab-layers').click();
    await page.locator('.layer-name').filter({ hasText: cardName }).first().click();
    await page.locator('#tab-widgets').click();
  };
  await add(page, t, 'ring');
  await selectCard();
  await inspector.getByRole('button', { name: t('card.addFace'), exact: true }).click();
  await add(page, t, 'bar');
  await add(page, t, 'graph');
  await selectCard();
  return { inspector, selectCard };
}

test('faces are a numbered list; the shared background is its own row; Move sends an object to another face', async ({ page, t }) => {
  const errors = watchErrors(page);
  const { inspector, selectCard } = await twoFaceCard(page, t);
  const faces = cardFaceList(page, t);
  const face = number => t('card.faceName', { number });
  await expect(faces.getByRole('listitem')).toHaveCount(2);
  await expect(faces.locator('.face-number')).toHaveText(['1', '2']);
  await expect(faces.locator('.face-name')).toHaveText([face(1), face(2)]);
  await expect(inspector.locator('[data-section="faces"] .count')).toHaveText('2');
  // The face in view: current, marked "in view", its Edit pressed; the other is not.
  const second = faces.locator('[data-face="1"]');
  await expect(second).toHaveAttribute('aria-current', 'true');
  await expect(second).toContainText(t('layers.inView'));
  await expect(faces.getByText(t('layers.inView'), { exact: true })).toHaveCount(1);
  const editSecond = second.getByRole('button', { name: t('cardInsp.editFace', { name: face(2) }), exact: true });
  await expect(editSecond).toHaveAttribute('aria-pressed', 'true');
  await expect(editSecond.locator('svg')).toHaveCount(1);
  await expect(faces.locator('[data-face="0"]')).not.toHaveAttribute('aria-current', 'true');
  // No thumbnails, no pair chips, no horizontal strip.
  await expect(inspector.locator('.face-chip, .face-transition, .screen-thumb')).toHaveCount(0);
  const objects = inspector.locator('[data-section="objects"]');
  await expect(objects.getByRole('heading')).toHaveText(t('cardInsp.objectsIn', { face: face(2) }));
  await expect(objects.locator('.count')).toHaveText('2');
  await expect(objects.locator('.face-object')).toHaveCount(2);
  // Move the bar to face 1: it leaves this list; Undo brings it back.
  await objects.getByRole('combobox', { name: t('cardInsp.moveObject', { name: t('widget.bar') }) }).selectOption('0');
  await expect(objects.locator('.face-object')).toHaveCount(1);
  await page.locator('#undo').click();
  await expect(objects.locator('.face-object')).toHaveCount(2);
  // To the shared background; its own row (outside the numbered list) edits it.
  await objects.getByRole('combobox', { name: t('cardInsp.moveObject', { name: t('widget.bar') }) }).selectOption('base');
  const shared = inspector.locator('.shared-background');
  await expect(faces.locator('.shared-background')).toHaveCount(0);
  await expect(shared).toContainText(t('cardInsp.base'));
  await expect(shared).toContainText(t('cardInsp.sharedHint'));
  const editShared = shared.getByRole('button', { name: t('cardInsp.editShared'), exact: true });
  await editShared.click();
  await expect(editShared).toHaveAttribute('aria-pressed', 'true');
  await expect(shared).toHaveAttribute('data-editing', 'true');
  await expect(editSecond).toHaveAttribute('aria-pressed', 'false');
  await expect(second).toHaveAttribute('aria-current', 'true');
  await expect(objects.getByRole('heading')).toHaveText(t('cardInsp.objectsIn', { face: t('cardInsp.base') }));
  await expect(objects.locator('.face-object')).toHaveCount(1);
  await expect(inspector.getByLabel(t('card.faceTitle'), { exact: true })).toHaveCount(0);
  // Edit on the first face shows it on the canvas and opens its name and objects.
  await faces.getByRole('button', { name: t('cardInsp.editFace', { name: face(1) }), exact: true }).click();
  await expect(faces.locator('[data-face="0"]')).toHaveAttribute('aria-current', 'true');
  await expect(editShared).toHaveAttribute('aria-pressed', 'false');
  await expect(inspector.getByLabel(t('card.faceTitle'), { exact: true })).toHaveValue(face(1));
  await expect(objects.getByRole('heading')).toHaveText(t('cardInsp.objectsIn', { face: face(1) }));
  await page.locator('#undo').click();
  await faces.getByRole('button', { name: t('cardInsp.editFace', { name: face(2) }), exact: true }).click();
  // A list card selects its object.
  await objects.getByRole('button', { name: t('cardInsp.selectObject', { name: t('widget.graph') }), exact: true }).click();
  await expect(faces).toHaveCount(0);
  await selectCard();
  await expect(inspector.getByText(t('cardInsp.dropHint'), { exact: true })).toBeVisible();
  await facesRegion(inspector, t).screenshot({ path: test.info().outputPath('faces-content.png') });
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('the ⋮ menu renames, duplicates, moves and deletes a face, from the keyboard, each one Undo step', async ({ page, t }) => {
  const errors = watchErrors(page);
  const { inspector } = await twoFaceCard(page, t);
  const faces = cardFaceList(page, t);
  const face = number => t('card.faceName', { number });
  const names = faces.locator('.face-name');
  const more = faces.getByRole('button', { name: t('cardInsp.faceActions', { name: face(1) }), exact: true });
  await expect(more).toHaveAttribute('aria-haspopup', 'menu');
  await more.focus(); await page.keyboard.press('Enter');
  const menu = page.getByRole('menu', { name: t('cardInsp.faceActions', { name: face(1) }), exact: true });
  await expect(menu.getByRole('menuitem')).toHaveText([t('common.rename'), t('card.duplicateFace'), t('card.previous'), t('card.next'), t('card.removeFace')]);
  await expect(menu.getByRole('menuitem', { name: t('card.previous'), exact: true })).toBeDisabled();
  await expect(menu.getByRole('menuitem', { name: t('common.rename'), exact: true })).toBeFocused();
  await expect(more).toHaveAttribute('aria-expanded', 'true');
  // Escape closes it and gives the focus back.
  await page.keyboard.press('Escape');
  await expect(menu).toHaveCount(0);
  await expect(more).toBeFocused();
  await expect(more).toHaveAttribute('aria-expanded', 'false');
  // Rename: shows the face and focuses its name.
  await page.keyboard.press('Enter');
  await page.keyboard.press('Enter');
  const title = inspector.getByLabel(t('card.faceTitle'), { exact: true });
  await expect(title).toBeFocused();
  await expect(faces.locator('[data-face="0"]')).toHaveAttribute('aria-current', 'true');
  await title.fill('Meteo'); await title.press('Tab');
  await expect(names).toHaveText(['Meteo', face(2)]);
  // Move later with the arrow keys: the order changes, the focus follows the face.
  await faces.getByRole('button', { name: t('cardInsp.faceActions', { name: 'Meteo' }), exact: true }).click();
  // Disabled items are skipped: Rename, Duplicate, Move later.
  await page.keyboard.press('ArrowDown'); await page.keyboard.press('ArrowDown');
  await expect(page.getByRole('menuitem', { name: t('card.next'), exact: true })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(names).toHaveText([face(2), 'Meteo']);
  await expect(faces.locator('[data-face="1"]')).toHaveAttribute('aria-current', 'true');
  await expect(faces.getByRole('button', { name: t('cardInsp.faceActions', { name: 'Meteo' }), exact: true })).toBeFocused();
  await page.locator('#undo').click();
  await expect(names).toHaveText(['Meteo', face(2)]);
  // Duplicate the face that is not in view: the copy is added and shown.
  await faceAction(page, t, 1, 'card.duplicateFace');
  await expect(faces.getByRole('listitem')).toHaveCount(3);
  await expect(faces.locator('[data-face="2"]')).toHaveAttribute('aria-current', 'true');
  await expect(faces.locator('[data-face="2"] .face-edit')).toBeFocused();
  await page.locator('#undo').click();
  await expect(faces.getByRole('listitem')).toHaveCount(2);
  // Delete: the face and its objects go; Undo restores both.
  await faceAction(page, t, 1, 'card.removeFace');
  await expect(names).toHaveText(['Meteo']);
  await faces.getByRole('button', { name: t('cardInsp.faceActions', { name: 'Meteo' }), exact: true }).click();
  await expect(page.getByRole('menuitem', { name: t('card.removeFace'), exact: true })).toBeDisabled();
  await page.keyboard.press('Escape');
  await page.locator('#undo').click();
  await expect(names).toHaveText(['Meteo', face(2)]);
  await faces.getByRole('button', { name: t('cardInsp.editFace', { name: face(2) }), exact: true }).click();
  await expect(inspector.locator('[data-section="objects"] .face-object')).toHaveCount(2);
  // The add button sits in the section head and gives the focus to the new face.
  await inspector.locator('[data-section="faces"] .section-head').getByRole('button', { name: t('card.addFace'), exact: true }).click();
  await expect(faces.locator('[data-face="2"] .face-edit')).toBeFocused();
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('one transition for the whole Card: effect, duration and direction in Behavior, with Undo', async ({ page, t }) => {
  const errors = watchErrors(page);
  const { inspector } = await twoFaceCard(page, t);
  await openSection(page, 'behavior');
  const section = inspector.locator('[data-section="animation"]');
  await expect(section.getByRole('heading')).toHaveText(t('cardInsp.transitionTitle'));
  const effects = section.getByRole('group', { name: t('card.effect'), exact: true });
  const effect = (name) => effects.getByRole('button', { name: t(`card.effect.${name}`), exact: true });
  await expect(effects.getByRole('button')).toHaveCount(4);
  await expect(effect('none')).toHaveAttribute('aria-pressed', 'true');
  await expect(section.getByRole('spinbutton', { name: t('card.duration'), exact: true })).toHaveCount(0);
  await effect('flip').click();
  const duration = section.getByRole('spinbutton', { name: t('card.duration'), exact: true });
  await expect(duration).toHaveValue('650');
  await duration.fill('437'); await duration.press('Tab');
  await expect(duration).toHaveValue('450');
  await duration.fill('9000'); await duration.press('Tab');
  await expect(duration).toHaveValue('1500');
  const up = section.getByRole('button', { name: t('card.direction.up'), exact: true });
  await up.click();
  await expect(up).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#undo').click();
  await expect(section.getByRole('button', { name: t('common.left'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#undo').click();
  await expect(duration).toHaveValue('450');
  // A fade has no direction.
  await effect('fade').click();
  await expect(section.getByRole('group', { name: t('common.direction'), exact: true })).toHaveCount(0);
  await expect(duration).toHaveValue('450');
  // Whether the shared background moves is part of the same transition.
  const shared = section.getByRole('button', { name: t('cardInsp.animateShared'), exact: true });
  await expect(shared).toHaveAttribute('aria-pressed', 'true');
  await shared.click();
  await expect(shared).toHaveAttribute('aria-pressed', 'false');
  await page.locator('#undo').click();
  await expect(shared).toHaveAttribute('aria-pressed', 'true');
  // ▶ in the header tries it; there is one editor of the transition and nothing about pairs.
  await expect(inspector.getByRole('button', { name: t('insp.tryObject'), exact: true })).toBeEnabled();
  await expect(inspector.getByRole('group', { name: t('card.effect'), exact: true })).toHaveCount(1);
  await expect(inspector.locator('[data-section*="pair"], .face-transition')).toHaveCount(0);
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('timed triggers edit intervals with bounds and Undo; the effects are four choices', async ({ page, t }) => {
  const errors = watchErrors(page);
  const { inspector } = await twoFaceCard(page, t);
  await openSection(page, 'behavior');
  await inspector.getByRole('button', { name: t('cardInsp.addTimed'), exact: true }).click();
  const seconds = inspector.getByRole('spinbutton', { name: t('cardInsp.interval'), exact: true });
  const unit = inspector.getByRole('combobox', { name: t('cardInsp.intervalUnit'), exact: true });
  await expect(seconds).toHaveValue('10');
  await seconds.focus(); await seconds.press('ArrowUp'); await seconds.press('Tab');
  await expect(seconds).toHaveValue('11');
  await seconds.focus(); await seconds.press('ArrowDown'); await seconds.press('ArrowDown'); await seconds.press('Tab');
  await expect(seconds).toHaveValue('9');
  await seconds.fill('60'); await seconds.press('Tab');
  await expect(seconds).toHaveValue('1'); await expect(unit).toHaveValue('min');
  await page.locator('#undo').click();
  await expect(seconds).toHaveValue('9'); await expect(unit).toHaveValue('s');
  await seconds.fill('2'); await seconds.press('Tab');
  await expect(seconds).toHaveValue('5'); await expect(seconds).toHaveAttribute('min', '5');
  await expect(inspector.getByText(t('cardInsp.paused'), { exact: true })).toBeVisible();
  const face = inspector.getByRole('combobox', { name: t('trigger.face'), exact: true });
  await expect(face).toHaveValue('next');
  await face.selectOption('0'); await expect(face).toHaveValue('0');
  await page.locator('#undo').click(); await expect(face).toHaveValue('next');
  // Effects: four choices; a slide has a duration and a direction, each one Undo step.
  const effects = inspector.getByRole('group', { name: t('card.effect'), exact: true });
  await expect(effects.getByRole('button')).toHaveCount(4);
  await expect(effects.getByRole('button', { name: t('card.effect.none'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(inspector.getByRole('spinbutton', { name: t('card.duration'), exact: true })).toHaveCount(0);
  await effects.getByRole('button', { name: t('card.effect.slide'), exact: true }).click();
  await expect(inspector.getByRole('spinbutton', { name: t('card.duration'), exact: true })).toHaveValue('650');
  const up = inspector.getByRole('group', { name: t('common.direction'), exact: true }).getByRole('button', { name: t('card.direction.up'), exact: true });
  await up.click();
  await expect(up).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#undo').click();
  await expect(inspector.getByRole('group', { name: t('common.direction'), exact: true }).getByRole('button', { name: t('common.left'), exact: true })).toHaveAttribute('aria-pressed', 'true');
  await behaviorRegion(inspector, t).screenshot({ path: test.info().outputPath('motion-content.png') });
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('triggers read as sentences with tokens, reorder by keyboard, and share one return', async ({ page, t }) => {
  const errors = watchErrors(page);
  const { inspector } = await twoFaceCard(page, t);
  await openSection(page, 'behavior');
  await expect(inspector).toContainText(t('cardInsp.triggersIntro'));
  const returnTo = inspector.getByRole('combobox', { name: t('trigger.returnFace'), exact: true });
  await expect(returnTo).toBeDisabled();
  const addRule = inspector.getByRole('button', { name: t('trigger.add'), exact: true });
  await addRule.click();
  await addRule.click();
  const rules = inspector.locator('.trigger-rule');
  await expect(rules).toHaveCount(2);
  const first = inspector.getByRole('group', { name: t('trigger.rule', { value: 1 }), exact: true });
  // Music in any app: the sentence says "in" before the app, empty = any app.
  await expect(first.locator('.sentence-word')).toHaveText([t('cardInsp.when'), t('cardInsp.in'), t('cardInsp.show')]);
  await expect(first.getByRole('combobox', { name: t('trigger.app') })).toHaveAttribute('placeholder', t('cardInsp.anyApp'));
  await first.getByRole('combobox', { name: t('trigger.condition'), exact: true }).selectOption('foreground');
  await expect(first.locator('.sentence-word')).toHaveText([t('cardInsp.when'), t('cardInsp.show')]);
  const app = first.getByRole('combobox', { name: t('trigger.app') });
  await expect(app).not.toHaveValue('');
  await app.fill('obs64');
  await app.press('Tab');
  await expect(app).toHaveValue('obs64');
  const priority = first.getByRole('spinbutton', { name: t('trigger.priority'), exact: true });
  await priority.fill('80');
  await priority.press('Tab');
  await first.getByRole('combobox', { name: t('trigger.face'), exact: true }).selectOption('0');
  // Rule 1 goes down with the keyboard: the second card now holds it; Undo puts it back.
  await first.getByRole('button', { name: t('cardInsp.reorder', { value: 1 }), exact: true }).press('ArrowDown');
  const second = inspector.getByRole('group', { name: t('trigger.rule', { value: 2 }), exact: true });
  await expect(second.getByRole('combobox', { name: t('trigger.app') })).toHaveValue('obs64');
  await expect(second.getByRole('spinbutton', { name: t('trigger.priority'), exact: true })).toHaveValue('80');
  await expect(inspector.getByRole('button', { name: t('cardInsp.reorder', { value: 2 }), exact: true })).toBeFocused();
  await page.locator('#undo').click();
  await expect(first.getByRole('combobox', { name: t('trigger.app') })).toHaveValue('obs64');
  // Dragging the handle below rule 2 moves it there too, one Undo step.
  const handle = first.getByRole('button', { name: t('cardInsp.reorder', { value: 1 }), exact: true });
  const from = await handle.boundingBox(), target = await second.boundingBox();
  await page.mouse.move(from.x + from.width / 2, from.y + from.height / 2);
  await page.mouse.down();
  await page.mouse.move(from.x + from.width / 2, target.y + target.height - 4, { steps: 8 });
  await page.mouse.up();
  await expect(second.getByRole('combobox', { name: t('trigger.app') })).toHaveValue('obs64');
  await page.locator('#undo').click();
  await expect(first.getByRole('combobox', { name: t('trigger.app') })).toHaveValue('obs64');
  // One return for every rule.
  await expect(returnTo).toBeEnabled();
  await expect(returnTo).toHaveValue('previous');
  await returnTo.selectOption('0');
  await expect(returnTo).toHaveValue('0');
  await page.locator('#undo').click();
  await expect(returnTo).toHaveValue('previous');
  await expect(inspector).toContainText(t('cardInsp.priorityHint'));
  await behaviorRegion(inspector, t).screenshot({ path: test.info().outputPath('triggers-content.png') });
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test('triggers stay off with one face, and a rule removed leaves the return disabled again', async ({ page, t }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  await add(page, t, 'card');
  const inspector = page.locator('#inspector');
  await tab(page, t, 'card.tab.triggers');
  await expect(inspector.getByRole('button', { name: t('trigger.add'), exact: true })).toBeDisabled();
  await expect(inspector).toContainText(t('cardInsp.needsFaces'));
  await tab(page, t, 'card.tab.faces');
  await inspector.getByRole('button', { name: t('card.addFace'), exact: true }).click();
  await tab(page, t, 'card.tab.triggers');
  await inspector.getByRole('button', { name: t('trigger.add'), exact: true }).click();
  await inspector.getByRole('button', { name: t('trigger.remove'), exact: true }).click();
  await expect(inspector.locator('.trigger-rule')).toHaveCount(0);
  await expect(inspector.getByRole('combobox', { name: t('trigger.returnFace'), exact: true })).toBeDisabled();
  await expectAccessible(page);
  expect(errors).toEqual([]);
});

test.describe('Italian mockup comparison', () => {
  test.use({ locale: 'it-IT' });
  test('populated Card panels match the standalone board width', async ({ page, t }) => {
    const errors = watchErrors(page);
    await page.setViewportSize({ width: 1440, height: 1440 });
    await page.goto('/index.html?demo=turing88');
    await setPanelWidths(page, { inspector: 360 });
    const { inspector } = await twoFaceCard(page, t);
    expect(await inspector.evaluate(node => node.getBoundingClientRect().width)).toBe(360);
    await facesRegion(inspector, t).screenshot({ path: test.info().outputPath('CardFaces-it.png') });
    await openSection(page, 'behavior');
    await inspector.getByRole('group', { name: t('card.effect'), exact: true }).getByRole('button', { name: t('card.effect.slide'), exact: true }).click();
    await inspector.getByRole('button', { name: t('trigger.add'), exact: true }).click();
    await inspector.getByRole('button', { name: t('trigger.add'), exact: true }).click();
    await inspector.getByRole('button', { name: t('cardInsp.addTimed'), exact: true }).click();
    await expect(inspector.getByRole('spinbutton', { name: t('cardInsp.interval'), exact: true })).toHaveValue('10');
    await behaviorRegion(inspector, t).screenshot({ path: test.info().outputPath('CardBehavior-it.png') });
    await expectAccessible(page);
    expect(errors).toEqual([]);
  });
});

test('the shared context supplies live trigger app choices', async ({ page, t, lang }) => {
  const errors = watchErrors(page);
  await page.goto('/index.html?demo=turing88');
  // The Card module alone, with a sample whose activity can stop.
  await page.evaluate(async locale => {
    const [{ renderCardPanel }, { createStore }, { createDemoBackend }, { translator }, { el }, { propertySection }] = await Promise.all([
      import('/ui/inspector/card.js'), import('/editor/store.js'), import('/demo-backend.js'), import('/i18n/index.js'), import('/ui/dom.js'), import('/ui/fields.js'),
    ]);
    const bridge = createDemoBackend('turing88');
    const store = createStore((await bridge.session()).theme);
    const translate = translator(locale);
    store.dispatch('add', { widget: 'card', x: 240, y: 960 });
    const id = store.getState().selection[0];
    store.dispatch('add', { widget: 'ring', x: 240, y: 960 });
    store.dispatch('addCardFace', { id, name: translate('card.faceName', { number: 2 }) });
    store.dispatch('add', { widget: 'bar', x: 240, y: 960 });
    store.select([id]);
    let sample = await bridge.sample();
    let selectedTab = 'faces';
    const root = el('aside', { id: 'card-probe', class: 'inspector', style: { position: 'fixed', inset: '52px 0 30px auto', width: '360px', zIndex: '100' } });
    document.body.append(root);
    const render = () => root.replaceChildren(...renderCardPanel(selectedTab, {
      element: store.getState().theme.elements.find(e => e.id === id), store, t: translate, bridge,
      host: { lastSample: () => sample }, update: (elementId, patch) => store.dispatch('update', { id: elementId, patch }),
      section: propertySection,
    }).filter(Boolean));
    store.subscribe(render);
    render();
    globalThis.cardProbe = { triggers: () => { selectedTab = 'triggers'; render(); }, stopActivity: () => { sample = { ...sample, activity: null }; } };
  }, lang);
  const probe = page.locator('#card-probe');
  await expect(probe.locator('.face-rows > li')).toHaveCount(2);
  await page.evaluate(() => cardProbe.triggers());
  await probe.getByRole('button', { name: t('trigger.add'), exact: true }).click();
  await probe.getByRole('combobox', { name: t('trigger.condition'), exact: true }).selectOption('foreground');
  await expect(probe.getByRole('combobox', { name: t('trigger.app'), exact: true })).toHaveValue('chrome');
  await expect(probe.locator('datalist option[value="chrome"]')).toHaveCount(1);
  await expect(probe.locator('datalist option[value="explorer"]')).toHaveCount(1);
  await page.evaluate(() => cardProbe.stopActivity());
  await expect(probe.locator('datalist option[value="explorer"]')).toHaveCount(0);
  await expect(probe.getByRole('combobox', { name: t('trigger.app'), exact: true })).toHaveValue('chrome');
  await expectAccessible(page);
  expect(errors).toEqual([]);
});
