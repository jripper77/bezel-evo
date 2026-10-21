// The statusbar's Screen chip (D-2026-10-20-studio-ux-v2-11, board
// StatusBarNew): the screen and its frame rate, and its dialog with how it is
// connected, what it shows and two actions, Screen settings and Live on or
// off. The same native popover as the Bezel chip.
import { el, icon } from './dom.js';
import { ICONS } from './icons.js';
import { iconButton, panelHead, pill } from './fields.js';
import { screenStatus } from '../screen-status.js';

export function createScreenStatus({ t, root, onSettings, onLive }) {
  const label = el('span', { class: 'status-figure-text' });
  const toggle = pill('', { as: 'button', dot: true, attrs: { id: 'status-screen-toggle', popovertarget: 'screen-popover', 'aria-haspopup': 'dialog', 'aria-controls': 'screen-popover', 'aria-expanded': 'false' } });
  toggle.append(icon(ICONS.screen, 13), label);
  const popover = el('div', { id: 'screen-popover', class: 'popover status-popover', popover: 'auto', role: 'dialog', 'aria-labelledby': 'screen-popover-title' });
  const close = iconButton(t('dialog.close'), ICONS.close, () => popover.hidePopover(), { class: 'quiet', autofocus: true });
  const head = panelHead('', { id: 'screen-popover-title', tools: [close] });
  // What the screen is doing now, said in words (also its errors): kept as #status-device.
  const state = el('p', { class: 'hint', id: 'status-device' });
  const rows = el('dl', { class: 'status-rows' });
  const settings = el('button', { type: 'button', class: 'text-button', onclick: () => { popover.hidePopover(); onSettings(); } });
  const live = el('button', { type: 'button', class: 'primary-button', onclick: () => { popover.hidePopover(); onLive(); } });
  popover.append(head, state, rows, el('div', { class: 'status-actions' }, [settings, live]));
  root.append(toggle);
  document.body.append(popover);
  let last = null;
  let said = { message: '', problem: false };

  function render() {
    const s = screenStatus(last, t, said);
    label.textContent = s.label;
    toggle.dataset.tone = s.tone;
    toggle.title = s.title;
    head.querySelector('h2').textContent = s.present ? (s.live ? t('shell.screenTitleLive', { name: last.name }) : last.name) : t('shell.noScreenChip');
    close.title = t('dialog.close');
    close.setAttribute('aria-label', t('dialog.close'));
    state.textContent = said.message || t('top.noScreen');
    rows.replaceChildren(...s.rows.flatMap(row => [el('dt', { text: row.label }), el('dd', { class: 'mono', text: row.value })]));
    settings.textContent = t('shell.screenSettings');
    live.textContent = t(s.live ? 'shell.stopLive' : 'shell.startLive');
    live.disabled = !s.present;
  }

  function place() {
    const rect = toggle.getBoundingClientRect();
    popover.style.right = `${Math.max(8, window.innerWidth - rect.right)}px`;
    popover.style.bottom = `${window.innerHeight - rect.top + 8}px`;
  }
  popover.addEventListener('beforetoggle', evt => {
    toggle.setAttribute('aria-expanded', String(evt.newState === 'open'));
    if (evt.newState === 'open') place();
  });
  window.addEventListener('resize', () => { if (popover.matches(':popover-open')) place(); });
  render();
  return {
    /** The screen in use (`null`: none) as screenStatus reads it, and what it is doing in words. */
    update(screen, status = {}) { last = screen; said = { message: status.message ?? '', problem: Boolean(status.problem) }; render(); },
    retranslate: render,
  };
}
