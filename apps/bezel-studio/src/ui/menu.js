// The one popup menu of the Studio (D-14): the object's context menu and the
// ⋮ menus of list rows. A list of actions at a point, kept on screen,
// operable from the keyboard (arrows, Home/End, Escape, Tab). The look is
// `.object-menu` in shell.css.
import { el } from './dom.js';
import { keepOnScreen } from './popover.js';

/** A rule between groups of items. */
export const SEPARATOR = 'separator';

let closeOpen = null;

/** Closes the menu that is open, if any (as a dismissal). */
export function closeMenu() {
  closeOpen?.('dismiss');
}

/**
 * Opens a menu of `items` at `x`, `y` (CSS px), replacing any open one, and
 * focuses its first enabled item. An item is `{label, run, disabled?}` or
 * SEPARATOR. `onClose(reason)` reports how it ended: 'item' (after the
 * item's `run`), 'escape', 'tab' or 'dismiss' (a pointer outside it, the
 * window losing focus, closeMenu()). A pointer on `anchor` (the button
 * that opened it) is left to that button.
 * @param {{label: string, items: Array<{label: string, run: () => void, disabled?: boolean}|string>, x: number, y: number, anchor?: Element|null, onClose?: (reason: string) => void}} opts
 * @returns {HTMLElement} the menu
 */
export function openMenu({ label, items, x, y, anchor = null, onClose = () => {} }) {
  closeMenu();
  const close = (reason) => {
    if (closeOpen !== close) return;
    closeOpen = null;
    menu.remove();
    document.removeEventListener('pointerdown', outside, true);
    window.removeEventListener('blur', dismiss);
    if (reason) onClose(reason);
  };
  const outside = (evt) => { if (!menu.contains(evt.target) && !anchor?.contains(evt.target)) close('dismiss'); };
  const dismiss = () => close('dismiss');
  const menu = el('div', { class: 'object-menu', role: 'menu', 'aria-label': label }, items.map((item) => item === SEPARATOR
    ? el('hr', { role: 'separator' })
    : el('button', { type: 'button', role: 'menuitem', disabled: Boolean(item.disabled), text: item.label, onclick: () => { close(null); item.run(); onClose('item'); } })));
  closeOpen = close;
  (anchor?.closest('dialog[open]') ?? document.body).append(menu);
  keepOnScreen(menu, x, y);
  menu.querySelector('button:not(:disabled)')?.focus();
  menu.addEventListener('keydown', (evt) => {
    const buttons = [...menu.querySelectorAll('button:not(:disabled)')];
    const index = buttons.indexOf(document.activeElement);
    if (evt.key === 'Escape') { evt.preventDefault(); evt.stopPropagation(); close('escape'); }
    else if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(evt.key)) {
      evt.preventDefault(); evt.stopPropagation();
      const next = evt.key === 'Home' ? 0 : evt.key === 'End' ? buttons.length - 1 : (index + (evt.key === 'ArrowDown' ? 1 : -1) + buttons.length) % buttons.length;
      buttons[next]?.focus();
    } else if (evt.key === 'Tab') close('tab');
  });
  document.addEventListener('pointerdown', outside, true);
  window.addEventListener('blur', dismiss);
  return menu;
}

/**
 * A ⋮ button's menu: opened under the button, aligned to its right edge;
 * the button again closes it, and Escape gives the focus back to it.
 * @returns {HTMLElement|null} the menu, null when the call closed it
 */
export function openMenuAt(button, { label, items, onClose = () => {} }) {
  if (button.getAttribute('aria-expanded') === 'true') { closeMenu(); return null; }
  const box = button.getBoundingClientRect();
  button.setAttribute('aria-expanded', 'true');
  const menu = openMenu({ label, items, x: box.right, y: box.bottom + 2, anchor: button, onClose: (reason) => {
    button.setAttribute('aria-expanded', 'false');
    if (reason === 'escape') button.focus();
    onClose(reason);
  } });
  menu.style.left = `${Math.max(4, Math.min(box.right - menu.offsetWidth, innerWidth - menu.offsetWidth - 4))}px`;
  return menu;
}
