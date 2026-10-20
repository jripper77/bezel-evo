// Existing theme controls and the shared background chooser.
import { el, icon } from '../dom.js';
import { ICONS } from '../icons.js';
import { chipButton, propertySection as section, segmented, textField } from '../fields.js';
import { isHorizontal, isTurned, orientationOf } from '../../editor/geometry.js';
import { refreshChoices } from '../../inspector-tabs.js';

export function panel({ theme, assets, t, store, background, minRefresh }) {
  /** Vertical | Horizontal, keeping the 180° turn (like the stage's buttons). */
  function orientationField(theme) {
    const axis = isHorizontal(theme.orientation) ? 'horizontal' : 'vertical';
    const field = segmented(t('inspector.orientation'), axis, [['vertical', t('axis.vertical')], ['horizontal', t('axis.horizontal')]], (next) => {
      if (next !== axis) store.dispatch('setOrientation', { orientation: orientationOf(next, isTurned(theme.orientation)) });
    }, { bare: true });
    field.classList.add('theme-orientation');
    const [vertical, horizontal] = field.children;
    vertical.prepend(icon(ICONS.vertical, 14));
    horizontal.prepend(icon(ICONS.horizontal, 14));
    return field;
  }

  /** How often the theme reads its sensors: the quick values, and the theme's own when it is another. */
  function refreshSection(theme) {
    const set = (refreshSeconds) => store.dispatch('setTheme', { patch: { refreshSeconds } });
    return section('refresh', t('inspector.refresh'), [
      el('div', { class: 'chip-group', role: 'group', 'aria-label': t('inspector.refresh') }, refreshChoices(theme.refreshSeconds, minRefresh())
        .map(({ seconds, pressed }) => chipButton(t('unit.seconds', { value: seconds }), pressed, () => !pressed && set(seconds)))),
    ]);
  }

  function themeForm(theme, assets) {
    if (background.isOpen()) return background.chooser(theme.background, assets);
    return [
      el('h2', { text: t('inspector.theme') }),
      textField(t('common.name'), theme.name, (v) => store.dispatch('setTheme', { patch: { name: v } })),
      orientationField(theme),
      el('p', { class: 'hint theme-canvas', text: t('inspector.canvas', { width: theme.canvas.width, height: theme.canvas.height }) }),
      background.section(theme.background, assets),
      refreshSection(theme),
    ];
  }
  return themeForm(theme, assets);
}
