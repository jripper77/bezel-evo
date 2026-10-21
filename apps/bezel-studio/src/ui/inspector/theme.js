// Theme controls use the same primitives and disclosure memory as objects.
import { el } from '../dom.js';
import { ICONS } from '../icons.js';
import { iconButton, iconChoice } from '../fields.js';
import { openMenuAt } from '../menu.js';
import { isHorizontal, isTurned, orientationOf } from '../../editor/geometry.js';
import { refreshChoices } from '../../inspector-helpers.js';
import { backgroundName } from '../../editor/background.js';

export function header({ theme, t, store, video, host }) {
  const screen = video.context().screen;
  const model = screen?.models?.length === 1 ? screen.models[0].name : screen?.key;
  const name = el('input', { type: 'text', class: 'inspector-name', value: theme.name, 'aria-label': t('top.themeName'), onchange: event => {
    const value = event.target.value.trim();
    if (value) store.dispatch('setTheme', { patch: { name: value } });
    else event.target.value = theme.name;
  } });
  const more = iconButton(t('insp.theme.actions'), ICONS.more, () => openMenuAt(more, { label: t('insp.theme.actions'), items: [
    { label: t('common.save'), run: () => host.saveTheme?.(false) },
    { label: t('themes.saveAs'), run: () => host.saveTheme?.(true) },
    { label: t('common.duplicate'), run: () => host.saveTheme?.(true, t('layers.copyOf', { name: theme.name })) },
  ] }), { class: 'quiet', 'aria-haspopup': 'menu', 'aria-expanded': 'false' });
  return el('div', { class: 'theme-heading' }, [
    el('canvas', { class: 'theme-preview', width: 320, height: 82, role: 'img', 'aria-label': t('stage.preview') }),
    el('div', { class: 'theme-name-row' }, [name, more]),
    el('div', { class: 'theme-facts' }, [el('span', { class: 'mono', text: [model, `${theme.canvas.width} × ${theme.canvas.height}`].filter(Boolean).join(' · ') }),
      el('span', { class: 'theme-save-state', dataset: { dirty: String(store.isDirty()) }, text: t(store.isDirty() ? 'status.unsaved' : 'status.saved') }),
    ]),
  ]);
}

export function orientationField({ theme, t, store }) {
  const horizontal = isHorizontal(theme.orientation), turned = isTurned(theme.orientation);
  const axis = horizontal ? 'horizontal' : 'vertical';
  const set = (nextAxis, nextTurned) => store.dispatch('setOrientation', { orientation: orientationOf(nextAxis, nextTurned) });
  return el('div', { class: 'theme-orientation' }, [
    el('span', { text: t('inspector.orientation') }),
    iconChoice(t('inspector.orientation'), axis, ['vertical', 'horizontal'].map(value => [value, t(`axis.${value}`), value]), value => set(value, turned), { iconOnly: true, bare: true }),
    iconButton(t('insp.theme.turn'), ICONS.rotate, () => set(axis, !turned), { 'aria-pressed': String(turned) }),
  ]);
}

export function panel({ theme, assets, background }) {
  return background.chooser(theme.background, assets);
}

export function sections(ctx) {
  const { theme, assets, t, store, background, minRefresh, video } = ctx;
  const locale = video.context().locale;
  const seconds = new Intl.NumberFormat(locale, { maximumFractionDigits: 2 }).format(theme.refreshSeconds);
  return [
    { id: 'color', title: t('common.background'), icon: 'background', summary: () => theme.background.type === 'color' ? `${t('insp.bgSolid')} · ${theme.background.color}` : backgroundName(theme.background), render: body => body.append(background.section(theme.background, assets)) },
    { id: 'behavior', title: t('inspector.refresh'), icon: 'refresh', summary: () => t('insp.theme.every', { seconds }), render: body => body.append(
      iconChoice(t('insp.theme.readEvery'), theme.refreshSeconds, refreshChoices(theme.refreshSeconds, minRefresh()).map(({ seconds }) => [seconds, `${new Intl.NumberFormat(locale).format(seconds)} s`]), value => store.dispatch('setTheme', { patch: { refreshSeconds: value } }), { wide: true }),
      el('p', { class: 'hint', text: t('insp.theme.refreshHint') }),
    ) },
  ];
}
