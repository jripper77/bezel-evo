import { panels as shapePanels } from './shape.js';
import { describeSections } from './sections.js';
// Card content only: the inspector owns header, sections, geometry and preview.
import { el, icon } from '../dom.js';
import { ICONS } from '../icons.js';
import { dashedHint, fieldGrid, iconButton, listCard, pill, prefixedField, iconChoice, numberField, previewChoice, selectField, sentence as sentenceOfParts, toggleField as switchRow, textField } from '../fields.js';
import { SEPARATOR, openMenuAt } from '../menu.js';
import { widgetOf } from '../../editor/widgets.js';
import { selectionRoots } from '../../editor/cards.js';
import { MAX_RULES, ROTATION_MIN, ROTATION_MAX, TRIGGER_SOURCES, appSuggestions, clampPriority, clampReturn, commonReturn, moveRule, newRule, newTimedRule, timedParts, timedSeconds, sentenceOf, withApp, withReturn, withSource, dropIndex } from '../../card-rules.js';
import { CARD_DURATION, cardDuration, withCardEffect } from '../../card-config.js';
import { CARD_TABS as ALL_CARD_TABS } from '../../inspector-tabs.js';
export const CARD_TABS = Object.freeze(ALL_CARD_TABS.slice(0, -1));
/** Per store and Card: the face in view when the shared background was chosen for editing. */
const baseViews = new WeakMap();
const tool = (label, name, onclick, disabled = false, extra = '') => iconButton(label, ICONS[name], onclick, { size: 16, disabled, class: extra });
/** Focus after a command: the inspector has been drawn again by then. */
const focusLater = (selector) => queueMicrotask(() => document.querySelector(selector)?.focus());

// Card content uses the same section contract as every inspector property.
function cardSection(ctx, key, label, nodes, total = null, sectionTool = null) {
  const node = ctx.section(key, label, nodes, { count: total, tool: sectionTool });
  node?.classList.add('card-section');
  return node;
}
const members = (ctx, face) => {
  const theme = ctx.store.getState().theme;
  return selectionRoots(theme, theme.elements.filter(e => e.cardMember?.parent === ctx.element.id && e.cardMember.face === face).map(e => e.id));
};

/** "Edit": shows that part of the Card (on the canvas, its objects below); pressed while it is the one edited. */
const editButton = (ctx, label, pressed, onclick) => iconButton(label, ICONS.rename, onclick, { class: 'quiet face-edit', 'aria-pressed': String(pressed) });

/**
 * The faces as a numbered list (D-24): name, "in view", Edit and a ⋮ menu
 * (rename, duplicate, move, delete), then the shared background on its own row.
 */
function faceList(ctx, base, views) {
  const { element: e, t, store } = ctx, c = e.card;
  const scope = `.face-list[data-card-id="${e.id}"]`;
  const row = (face) => `${scope} [data-face="${face}"]`;
  const show = (face) => {
    views.delete(e.id);
    if (face === c.activeFace) store.select([e.id]);
    else store.dispatch('cardFace', { id: e.id, face });
  };
  const command = (action, face, args = {}) => { views.delete(e.id); store.dispatch(action, { id: e.id, face, ...args }); };
  const faceRow = (name, i) => {
    const label = t('cardInsp.faceActions', { name });
    const more = iconButton(label, ICONS.more, () => openMenuAt(more, { label, items: [
      { label: t('common.rename'), run: () => { show(i); focusLater(`.face-editor[data-card-id="${e.id}"] input`); } },
      { label: t('card.duplicateFace'), disabled: c.faces.length >= 16, run: () => { command('duplicateCardFace', i); focusLater(`${row(c.faces.length)} .face-edit`); } },
      SEPARATOR,
      { label: t('card.previous'), disabled: i === 0, run: () => { command('reorderCardFace', i, { direction: -1 }); focusLater(`${row(i - 1)} .face-more`); } },
      { label: t('card.next'), disabled: i === c.faces.length - 1, run: () => { command('reorderCardFace', i, { direction: 1 }); focusLater(`${row(i + 1)} .face-more`); } },
      SEPARATOR,
      { label: t('card.removeFace'), disabled: c.faces.length <= 1, run: () => { command('removeCardFace', i); focusLater(`${row(Math.min(i, c.faces.length - 2))} .face-edit`); } },
    ] }), { size: 16, class: 'quiet face-more', 'aria-haspopup': 'menu', 'aria-expanded': 'false' });
    const inView = i === c.activeFace;
    return el('li', { class: 'face-row', dataset: { face: String(i), editing: String(!base && inView) }, 'aria-current': inView ? 'true' : null }, [
      el('span', { class: 'face-number', text: String(i + 1) }),
      el('span', { class: 'face-name', text: name, title: name }),
      inView && pill(t('layers.inView'), { tone: 'ok', dot: true, attrs: { title: t('layers.inViewHint') } }),
      editButton(ctx, t('cardInsp.editFace', { name }), !base && inView, () => show(i)),
      more,
    ]);
  };
  const shared = el('div', { class: 'face-row shared-background', dataset: { face: 'base', editing: String(base) } }, [
    icon(ICONS.layers, 16),
    el('span', { class: 'list-card-text' }, [el('span', { class: 'face-name', text: t('cardInsp.base') }), el('span', { class: 'face-sub', text: t('cardInsp.sharedHint') })]),
    editButton(ctx, t('cardInsp.editShared'), base, () => { views.set(e.id, c.activeFace); store.select([e.id]); }),
  ]);
  const add = el('button', { type: 'button', class: 'text-button quiet face-add', 'aria-label': t('card.addFace'), title: t('card.addFace'), disabled: c.faces.length >= 16,
    onclick: () => { views.delete(e.id); store.dispatch('addCardFace', { id: e.id, name: t('card.faceName', { number: c.faces.length + 1 }) }); focusLater(`${row(c.faces.length)} .face-edit`); } }, [icon(ICONS.plus, 16), t('cardInsp.add')]);
  return cardSection(ctx, 'faces', t('card.tab.faces'), [
    el('div', { class: 'face-list', dataset: { cardId: String(e.id) } }, [
      shared,
      el('ol', { class: 'face-rows', 'aria-label': t('card.tab.faces') }, c.faces.map(faceRow)),
    ]),
    el('p', { class: 'hint', text: t('cardInsp.editHint') }),
  ], c.faces.length, add);
}

function facesForm(ctx) {
  const { element: e, t, store, update } = ctx, c = e.card;
  if (!baseViews.has(store)) baseViews.set(store, new Map());
  const views = baseViews.get(store);
  const base = views.get(e.id) === c.activeFace;
  const face = base ? null : c.activeFace;
  const objects = members(ctx, face);
  const name = base ? t('cardInsp.base') : c.faces[face];
  const editor = base ? null : el('div', { class: 'face-editor', dataset: { cardId: String(e.id) } }, [
    textField(t('card.faceTitle'), name, value => {
      const clean = Array.from(value.trim()).slice(0, 32).join('');
      if (!clean) return;
      const names = [...c.faces]; names[face] = clean; update(e.id, { card: { faces: names } });
    }),
  ]);
  const rows = objects.map(item => {
    const card = listCard({ icon: widgetOf(item), title: item.name, subtitle: t(`widget.${widgetOf(item)}`), label: t('cardInsp.selectObject', { name: item.name }), onClick: () => store.select([item.id]) });
    card.querySelector('.list-card-chevron').remove();
    const move = selectField(t('cardInsp.moveObject', { name: item.name }), face === null ? 'base' : String(face), [['base', t('cardInsp.base')], ...c.faces.map((n, i) => [String(i), n])], value => store.dispatch('attachCard', { ids: [item.id], parent: e.id, face: value === 'base' ? null : Number(value) }));
    move.querySelector('select').setAttribute('aria-label', t('cardInsp.moveObject', { name: item.name }));
    return el('div', { class: 'face-object' }, [card, move]);
  });
  return [faceList(ctx, base, views), editor,
    cardSection(ctx, 'objects', t('cardInsp.objectsIn', { face: name }), rows.length ? rows : [el('p', { class: 'hint', text: t('cardInsp.emptyFace') })], objects.length), dashedHint([t('cardInsp.dropHint')])];
}

const EFFECT_ART = { none: ['M3 5h7v14H3z', 'M14 5h7v14h-7z'], fade: ['M3 5h10v14H3z', 'M10 5h11v14H10z'], slide: ['M3 5h7v14H3z', 'M17 5h4v14h-4z', 'M11 12h5', 'M14 10l2 2-2 2'], flip: ['M7 5h10v14H7z', 'M3 9v6', 'M21 9v6', 'M10 3h4', 'M10 21h4'] };
const DIRECTIONS = Object.freeze([['left', 'arrowLeft'], ['right', 'arrowRight'], ['up', 'up'], ['down', 'down']]);
/** Only a slide or a flip goes somewhere. */
const directed = settings => ['slide', 'flip'].includes(settings?.effect);

/** The direction as four arrow buttons (named for assistive tech). */
function directionControl(t, transition, set) {
  return iconChoice(t('common.direction'), transition.direction ?? 'up', DIRECTIONS.map(([value, name]) => [value, t(`card.direction.${value}`), name]), direction => set({ direction }), { iconOnly: true, wide: true });
}

/** Every face change uses the same transition; edit it once in Behavior. */
function transitionSection(ctx) {
  const { element: e, t, update } = ctx;
  const settings = e.card.transition ?? null;
  const set = patch => update(e.id, { card: { transition: patch } });
  const transition = settings?.effect && settings.effect !== 'none' ? settings : null;
  return cardSection(ctx, 'animation', t('cardInsp.transitionTitle'), [
    previewChoice(t('card.effect'), Object.keys(EFFECT_ART).map(effect => ({
      text: t(`card.effect.${effect}`), art: EFFECT_ART[effect] ?? EFFECT_ART.none,
      pressed: (settings?.effect ?? 'none') === effect,
      onClick: () => set(withCardEffect(settings, effect)),
    }))),
    transition && numberField(t('card.duration'), transition.durationMs, durationMs => set({ durationMs: cardDuration(durationMs) }), { ...CARD_DURATION, unit: 'ms', iconName: 'clock' }),
    directed(transition) && directionControl(t, transition, set),
    transition && switchRow(t('cardInsp.animateShared'), transition.includeBase, includeBase => set({ includeBase })),
  ]);
}

function motionForm(ctx) { return [transitionSection(ctx)]; }

function triggerForm(ctx) {
  const { element: e, t, store, update } = ctx, c = e.card;
  const rules = c.triggers ?? [];
  const activity = () => ctx.host?.lastSample()?.activity ?? null;
  const set = next => update(e.id, { card: { triggers: next } });
  const change = (i, next) => set(rules.map((r, n) => n === i ? next : r));
  const faceOptions = c.faces.map((name, i) => [String(i), name]);
  const list = el('div', { class: 'trigger-list' });
  const refreshers = [];
  const reorder = (from, to) => {
    const next = moveRule(rules, from, to);
    if (next === rules) return;
    set(next);
    queueMicrotask(() => document.querySelector(`[data-card-id="${e.id}"][data-rule-index="${to}"] .rule-handle`)?.focus());
  };
  rules.forEach((rule, i) => {
    const handle = tool(t('cardInsp.reorder', { value: i + 1 }), 'drag', () => {});
    handle.classList.add('rule-handle');
    handle.addEventListener('keydown', evt => {
      if (!['ArrowUp', 'ArrowDown', 'Home', 'End'].includes(evt.key)) return;
      evt.preventDefault(); evt.stopPropagation(); reorder(i, evt.key === 'Home' ? 0 : evt.key === 'End' ? rules.length - 1 : i + (evt.key === 'ArrowUp' ? -1 : 1));
    });
    handle.addEventListener('pointerdown', evt => {
      if (evt.button !== 0) return;
      evt.preventDefault(); handle.setPointerCapture(evt.pointerId);
      const row = handle.closest('.trigger-rule'); row.classList.add('dragging');
      const finish = end => {
        handle.removeEventListener('pointerup', finish); handle.removeEventListener('pointercancel', cancel);
        row.classList.remove('dragging');
        const mids = Array.from(list.children).filter(node => node !== row).map(node => { const b = node.getBoundingClientRect(); return b.y + b.height / 2; });
        reorder(i, dropIndex(mids, end.clientY));
      };
      const cancel = () => { handle.removeEventListener('pointerup', finish); handle.removeEventListener('pointercancel', cancel); row.classList.remove('dragging'); };
      handle.addEventListener('pointerup', finish); handle.addEventListener('pointercancel', cancel);
    });
    const sourceField = selectField(t('trigger.condition'), rule.source, TRIGGER_SOURCES.map(value => [value, t(value === 'timer' ? 'inspector.refreshSeconds' : `trigger.${value}`)]), value => change(i, withSource(rule, value, activity())));
    const source = sourceField.querySelector('select');
    source.setAttribute('aria-label', t('trigger.condition'));
    const suggestions = el('datalist', { id: `card-app-${e.id}-${i}` });
    const appField = textField(t('trigger.app'), rule.app ?? '', value => { const next = withApp(rule, value); if (next !== rule) change(i, next); }, { list: suggestions.id, placeholder: t('cardInsp.anyApp') });
    const input = appField.querySelector('input');
    input.setAttribute('aria-label', t('trigger.app'));
    const refresh = () => suggestions.replaceChildren(...appSuggestions(activity(), rule.source, rule.app).map(value => el('option', { value })));
    refresh(); refreshers.push(refresh);
    const faceField = selectField(t('trigger.face'), rule.nextFace ? 'next' : String(rule.face), [...(rule.source === 'timer' ? [['next', t('cardInsp.nextFace')]] : []), ...faceOptions], value => change(i, { ...rule, face: value === 'next' ? 0 : Number(value), ...(rule.source === 'timer' ? { nextFace: value === 'next' } : {}) }));
    const face = faceField.querySelector('select'); face.setAttribute('aria-label', t('trigger.face'));
    const timing = timedParts(rule);
    const intervalField = prefixedField('', t('cardInsp.interval'), timing.value, value => change(i, { ...rule, intervalSeconds: timedSeconds(value, timing.unit) }), { min: timing.unit === 'min' ? ROTATION_MIN / 60 : ROTATION_MIN, max: timing.unit === 'min' ? ROTATION_MAX / 60 : ROTATION_MAX, step: timing.unit === 's' ? 1 : 'any' });
    const unitField = selectField(t('cardInsp.intervalUnit'), timing.unit, ['s','min'].map(unit => [unit, t(`unit.${unit === 'min' ? 'minutes' : 'seconds'}`, { value: '' }).trim()]), unit => change(i, { ...rule, intervalSeconds: timedSeconds(timing.value, unit) }));
    unitField.querySelector('select').setAttribute('aria-label', t('cardInsp.intervalUnit'));
    const tokens = { source: sourceField, app: appField, face: faceField, interval: intervalField, unit: unitField };
    const sentence = sentenceOfParts(sentenceOf(rule).map(part => part.word ? t(`cardInsp.${part.word}`) : tokens[part.token]));
    list.append(el('div', { class: 'trigger-rule', role: 'group', 'aria-label': t('trigger.rule', { value: i + 1 }), dataset: { ruleIndex: String(i), cardId: String(e.id) } }, [
      el('div', { class: 'trigger-rule-head' }, [handle, el('span', { class: 'mono', text: t('trigger.rule', { value: i + 1 }) }), tool(t('trigger.remove'), 'trash', () => set(rules.filter((_, n) => n !== i)), false, 'danger')]), sentence, rule.source !== 'timer' && suggestions,
      rule.source !== 'timer' && fieldGrid([prefixedField(t('cardInsp.priority'), t('trigger.priority'), rule.priority ?? 0, value => change(i, { ...rule, priority: clampPriority(value) }), { min: 0, max: 100 }), prefixedField(t('cardInsp.return'), t('trigger.return'), rule.returnSeconds ?? 0, value => change(i, { ...rule, returnSeconds: clampReturn(value) }), { min: 0, max: 300, unit: 's' })]),
    ]));
  });
  // Stop polling when this render is replaced; no synthetic application names.
  if (ctx.host) queueMicrotask(function refreshActivity() { if (!list.isConnected) return; refreshers.forEach(refresh => refresh()); setTimeout(refreshActivity, 1000); });
  const appRules = rules.filter(r => r.source !== 'timer');
  const common = commonReturn(appRules);
  const returnTo = selectField(t('trigger.returnFace'), common === null ? 'previous' : String(common), [['previous', t('trigger.previous')], ...(common === 'mixed' ? [['mixed', t('cardInsp.mixed')]] : []), ...faceOptions], value => value !== 'mixed' && set(withReturn(rules, value === 'previous' ? value : Number(value))));
  returnTo.querySelector('select').disabled = !appRules.length;
  return [cardSection(ctx, 'triggers', t('cardInsp.triggersTitle'), [el('p', { class: 'hint', text: t('cardInsp.triggersIntro') }), list,
    el('button', { type: 'button', class: 'text-button trigger-add', disabled: rules.length >= MAX_RULES || c.faces.length < 2, onclick: () => set([...rules, newRule(c)]) }, [icon(ICONS.plus, 16), t('trigger.add')]),
    el('button', { type: 'button', class: 'text-button trigger-add', disabled: rules.length >= MAX_RULES || c.faces.length < 2, onclick: () => set([...rules, newTimedRule(c)]) }, [icon(ICONS.plus, 16), t('cardInsp.addTimed')]),
    rules.some(r => r.source === 'timer') && el('p', { class: 'hint', text: t('cardInsp.paused') }),
    c.faces.length < 2 && el('p', { class: 'hint', text: t('cardInsp.needsFaces') }), el('p', { class: 'hint', text: t('cardInsp.priorityHint') }),
  ], rules.length), cardSection(ctx, 'return', t('cardInsp.whenEnds'), [returnTo, el('p', { class: 'hint', text: t('cardInsp.appHint') })])];
}

export function renderCardPanel(tab, ctx) {
  if (tab === 'faces') return facesForm(ctx);
  if (tab === 'motion') return motionForm(ctx);
  if (tab === 'triggers') return triggerForm(ctx);
  return [];
}

export function sections(ctx) {
  const panels = ctx.completePanels(shapePanels(ctx));
  const descriptors = describeSections(ctx, { ...panels, faces: facesForm(ctx), behavior: [...motionForm(ctx), ...triggerForm(ctx)] });
  // Faces precede the common appearance controls in the Card artboard.
  const order = ['layout', 'faces', 'shape', 'color', 'behavior'];
  return descriptors.sort((a, b) => order.indexOf(a.id) - order.indexOf(b.id));
}
