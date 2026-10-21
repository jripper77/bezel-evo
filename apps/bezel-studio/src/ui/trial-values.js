// The canvas toolbar's Trial values (D-2026-10-20-studio-ux-v2-10, board
// MainNew): the canvas draws trial inputs instead of the sensors, every
// sensor sweeping its scale (render_preview's trialMs). A bar's trial handle
// holds one sensor instead (trialHold) while it is dragged. Nothing is
// saved, and the screens keep showing the sensors.
import { el, icon } from './dom.js';
import { ICONS } from './icons.js';

/**
 * @param {object} opts
 * @param {HTMLElement} opts.after the toolbar control the button follows
 * @param {(key: string) => string} opts.t
 * @param {() => boolean} opts.motion whether motion is allowed (the sweep needs it)
 * @param {() => void} opts.onChange the canvas must be drawn again
 */
export function createTrialValues({ after, t, motion, onChange, now = () => performance.now() }) {
  let start = null;
  let hold = null;
  const button = el('button', { type: 'button', class: 'icon-button quiet', id: 'trial-values', 'aria-pressed': 'false', onclick: () => set(start === null) }, [icon(ICONS['test-values'], 18)]);
  after.after(button);

  function set(on) {
    const next = on && motion() ? now() : null;
    if ((next === null) === (start === null)) return;
    start = next;
    button.setAttribute('aria-pressed', String(start !== null));
    onChange();
  }

  function retranslate() {
    button.title = t('shell.trialValues');
    button.setAttribute('aria-label', t('shell.trialValues'));
    button.disabled = !motion();
    if (!motion()) set(false);
  }
  retranslate();

  return {
    /** The render options of the canvas now: `{trialMs?, trialHold?}`. */
    options: () => ({ ...(start === null ? {} : { trialMs: Math.max(0, Math.floor(now() - start)) }), ...(hold ? { trialHold: hold } : {}) }),
    active: () => start !== null || hold !== null,
    /** One element's sensor at `fraction` of its scale (`null`: let go). */
    hold(next) {
      const changed = JSON.stringify(next) !== JSON.stringify(hold);
      hold = next;
      if (changed) onChange();
    },
    set,
    /** The language or the motion preference changed. */
    retranslate,
  };
}
