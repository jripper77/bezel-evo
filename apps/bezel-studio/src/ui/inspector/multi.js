import { describeSections } from './sections.js';
// Existing multi-selection tools, without inspector navigation.
import { canGroup } from '../../editor/cards.js';
import { el, icon } from '../dom.js';
import { ICONS } from '../icons.js';
import { propertySection as section } from '../fields.js';

export function panel({ ids, t, store, alignTool, ALIGN_EDGES }) {
  function multiForm(ids) {
    return [
      el('h2', { text: t('inspector.multi', { count: ids.length }) }),
      section('position', t('inspector.alignTools'), [
        el('div', { class: 'button-row' }, ALIGN_EDGES.map(([edge, name]) => alignTool(ids, edge, ICONS[name]))),
        el('div', { class: 'button-row' }, [
          el('button', { type: 'button', class: 'text-button', disabled: ids.length < 3, onclick: () => store.dispatch('distribute', { ids, axis: 'x' }) }, [icon(ICONS.distributeX, 16), t('align.distributeX')]),
          el('button', { type: 'button', class: 'text-button', disabled: ids.length < 3, onclick: () => store.dispatch('distribute', { ids, axis: 'y' }) }, [icon(ICONS.distributeY, 16), t('align.distributeY')]),
        ]),
      ]),
      section('membership', t('inspector.membership'), [
        el('button', { type: 'button', class: 'text-button', text: t('group.create'), disabled: !canGroup(store.getState().theme, ids), title: t('group.scope'), onclick: () => store.dispatch('groupSelection', { ids }) }),
        ...(store.getState().theme.elements.some(e => ids.includes(e.id) && e.isGroup) ? [el('button', { type: 'button', class: 'text-button', text: t('group.ungroup'), onclick: () => store.dispatch('ungroup', { ids }) })] : []),
        el('button', { type: 'button', class: 'text-button', text: t('card.group'), onclick: () => store.dispatch('cardFromSelection', { ids }) }),
      ]),
      section('actions', t('inspector.actions'), [
        el('div', { class: 'actions' }, [
          el('button', { type: 'button', class: 'text-button', onclick: () => store.dispatch('duplicate', { ids }) }, [icon(ICONS.copy, 16), t('common.duplicate')]),
          el('button', { type: 'button', class: 'text-button danger', onclick: () => store.dispatch('remove', { ids }) }, [icon(ICONS.trash, 16), t('common.delete')]),
        ]),
      ]),
    ];
  }
  return multiForm(ids);
}

export function sections(ctx) { return describeSections(ctx, { layout: panel(ctx).slice(1) }); }
