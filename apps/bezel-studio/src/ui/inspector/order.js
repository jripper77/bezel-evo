// The inspector's single order row, shared by every kind and multi-selection.
import { layerOrderAvailability } from '../../editor/layers.js';
import { layerFacts } from '../../inspector-helpers.js';
import { iconChoice } from '../fields.js';

export function orderField({ store, t, ids }) {
  const { theme } = store.getState();
  const available = layerOrderAvailability(theme, ids);
  const options = [
    ['front', t('inspector.front'), 'order-front', { text: t('insp.orderFront'), disabled: !available.front }],
    ['forward', t('insp.oneForward'), 'up', { iconOnly: true, disabled: !available.forward }],
    ['backward', t('insp.oneBackward'), 'down', { iconOnly: true, disabled: !available.backward }],
    ['back', t('inspector.back'), 'order-back', { text: t('insp.orderBack'), disabled: !available.back }],
  ];
  const e = ids.length === 1 ? theme.elements.find(e => e.id === ids[0]) : null;
  const facts = e && layerFacts(theme, e);
  const control = iconChoice(t('insp.order'), null, options, key => {
    if (key === 'front' || key === 'back') store.dispatch('orderSelection', { ids, front: key === 'front' });
    else store.dispatch('stepSelection', { ids, step: key === 'forward' ? 1 : -1 });
  }, { wide: true, actions: true, detail: facts ? t('insp.layer', facts) : null });
  control.dataset.section = 'order';
  return control;
}
