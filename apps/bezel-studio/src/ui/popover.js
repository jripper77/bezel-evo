// The one anchored popover of the pickers (D-14): a dialog-like panel under
// the button that opened it, kept on screen, closed by Escape (focus back
// to the button), a pointer outside it or opening another. The look is
// `.picker-popover` in base.css; the content is the caller's.

let closeOpen = null;

/** Closes the popover that is open, if any (as a dismissal). */
export function closePopover() {
  closeOpen?.('dismiss');
}

/**
 * Keeps `node` (position: fixed) at `x`, `y` (CSS px) inside the window,
 * 4 px from its edges. Also used by the popup menu (ui/menu.js).
 */
export function keepOnScreen(node, x, y) {
  node.style.left = `${Math.max(4, Math.min(x, innerWidth - node.offsetWidth - 4))}px`;
  node.style.top = `${Math.max(4, Math.min(y, innerHeight - node.offsetHeight - 4))}px`;
}

/**
 * Places `node` under `anchor`, or above it when it does not fit below.
 * @param {HTMLElement} node
 * @param {Element} anchor
 */
export function placeBy(node, anchor) {
  const box = anchor.getBoundingClientRect();
  const below = box.bottom + 4;
  const y = below + node.offsetHeight <= innerHeight - 4 ? below : box.top - 4 - node.offsetHeight;
  keepOnScreen(node, box.left, y);
}

/**
 * Opens `node` as a popover by `anchor`, replacing any open one.
 * `onClose(reason)` says how it ended: 'escape' (focus goes back by
 * `refocus`, the anchor by default), 'dismiss' (a pointer outside, the window resized, closePopover())
 * or 'done' (the returned `close('done')`, the caller having picked).
 * A pointer on the anchor is left to the anchor (it toggles).
 * @param {{node: HTMLElement, anchor: HTMLElement, label: string, onClose?: (reason: string) => void, refocus?: () => void}} opts
 * @returns {(reason?: string) => void} closes it
 */
export function openPopover({ node, anchor, label, onClose = () => {}, refocus = () => anchor.focus() }) {
  closePopover();
  const close = (reason = 'done') => {
    if (closeOpen !== close) return;
    closeOpen = null;
    resize.disconnect();
    node.remove();
    anchor.setAttribute('aria-expanded', 'false');
    document.removeEventListener('pointerdown', outside, true);
    window.removeEventListener('resize', dismiss);
    onClose(reason);
  };
  const outside = (evt) => { if (!node.contains(evt.target) && !anchor.contains(evt.target)) close('dismiss'); };
  const dismiss = () => close('dismiss');
  // Swatches and font results can grow after the panel opens. Keep the
  // finished panel in the viewport, including after its anchor is redrawn.
  const resize = new ResizeObserver(() => keepOnScreen(node, parseFloat(node.style.left), parseFloat(node.style.top)));
  node.classList.add('picker-popover');
  node.setAttribute('role', 'dialog');
  node.setAttribute('aria-label', label);
  node.addEventListener('keydown', (evt) => {
    if (evt.key !== 'Escape' || evt.defaultPrevented) return;
    evt.preventDefault();
    evt.stopPropagation();
    close('escape');
    refocus();
  });
  closeOpen = close;
  anchor.setAttribute('aria-expanded', 'true');
  // Inside a modal dialog the rest of the page is inert: the panel lives in it.
  (anchor.closest('dialog[open]') ?? document.body).append(node);
  placeBy(node, anchor);
  resize.observe(node);
  document.addEventListener('pointerdown', outside, true);
  window.addEventListener('resize', dismiss);
  return close;
}

/** Whether a popover opened by `anchor` is open. */
export const isOpenBy = (anchor) => closeOpen !== null && anchor.getAttribute('aria-expanded') === 'true';
