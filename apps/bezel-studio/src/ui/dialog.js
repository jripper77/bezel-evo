// A modal choice dialog: focus stays inside, Esc and the close button
// cancel, and focus returns to the element that opened it.
import { el, icon } from './dom.js';
import { ICONS } from './icons.js';

/** The class of each kind of button; any other kind is a text button. */
const BUTTON_CLASS = { primary: 'primary-button', danger: 'danger-button' };

/**
 * Asks a question; resolves with the `id` of the chosen action, or
 * `'cancel'`.
 * @param {(k: string, p?: object) => string} t
 * @param {{title: string, body: string, actions: Array<{id: string, label: string, kind?: 'primary'|'danger'|'text'}>, initial: string}} opts
 *   `actions` in reading order after Cancel; `initial` gets the focus.
 * @returns {Promise<string>}
 */
export function askChoice(t, { title, body, actions, initial }) {
  return new Promise((resolve) => {
    const opener = document.activeElement;
    const cancel = el('button', { type: 'button', class: 'text-button', text: t('dialog.cancel') });
    const buttons = actions.map((a) => el('button', { type: 'button', class: BUTTON_CLASS[a.kind] ?? 'text-button', dataset: { choice: a.id }, text: a.label }));
    const close = el('button', { type: 'button', class: 'icon-button dialog-close', title: t('dialog.close'), 'aria-label': t('dialog.close') }, [icon(ICONS.close, 16)]);
    const dialog = el('dialog', { class: 'confirm-dialog', 'aria-labelledby': 'choice-title', 'aria-describedby': 'choice-body' }, [
      el('div', { class: 'dialog-head' }, [el('h2', { id: 'choice-title', text: title }), close]),
      el('div', { id: 'choice-body', class: 'dialog-body' }, [el('p', { text: body })]),
      el('div', { class: 'dialog-actions' }, [cancel, ...buttons]),
    ]);
    cancel.addEventListener('click', () => dialog.close('cancel'));
    close.addEventListener('click', () => dialog.close('cancel'));
    for (const b of buttons) b.addEventListener('click', () => dialog.close(b.dataset.choice));
    dialog.addEventListener('close', () => {
      const answer = dialog.returnValue || 'cancel';
      dialog.remove();
      if (opener?.isConnected) opener.focus();
      resolve(answer);
    }, { once: true });
    document.body.append(dialog);
    dialog.showModal();
    (buttons.find((b) => b.dataset.choice === initial) ?? cancel).focus();
  });
}

/**
 * Asks for a name ("Save as"): resolves with the trimmed name, or `null`
 * when cancelled. An empty name cannot be confirmed.
 * @param {(k: string, p?: object) => string} t
 * @param {{title: string, body: string, label: string, value: string, confirm: string}} opts
 * @returns {Promise<string|null>}
 */
export function askName(t, { title, body, label, value, confirm }) {
  return new Promise((resolve) => {
    const opener = document.activeElement;
    const input = el('input', { type: 'text', id: 'name-input', class: 'text-input', value, autocomplete: 'off', spellcheck: 'false' });
    const ok = el('button', { type: 'submit', class: 'primary-button', text: confirm });
    const cancel = el('button', { type: 'button', class: 'text-button', text: t('dialog.cancel') });
    const close = el('button', { type: 'button', class: 'icon-button dialog-close', title: t('dialog.close'), 'aria-label': t('dialog.close') }, [icon(ICONS.close, 16)]);
    const form = el('form', { method: 'dialog' }, [
      el('div', { class: 'dialog-head' }, [el('h2', { id: 'name-title', text: title }), close]),
      el('div', { class: 'dialog-body' }, [el('p', { id: 'name-body', text: body }), el('label', { for: 'name-input', text: label }), input]),
      el('div', { class: 'dialog-actions' }, [cancel, ok]),
    ]);
    const dialog = el('dialog', { class: 'confirm-dialog name-dialog', 'aria-labelledby': 'name-title', 'aria-describedby': 'name-body' }, [form]);
    const valid = () => { ok.disabled = !input.value.trim(); };
    input.addEventListener('input', valid);
    form.addEventListener('submit', (evt) => { evt.preventDefault(); if (input.value.trim()) dialog.close('ok'); });
    cancel.addEventListener('click', () => dialog.close('cancel'));
    close.addEventListener('click', () => dialog.close('cancel'));
    dialog.addEventListener('close', () => {
      const name = dialog.returnValue === 'ok' ? input.value.trim() : null;
      dialog.remove();
      if (opener?.isConnected) opener.focus();
      resolve(name);
    }, { once: true });
    document.body.append(dialog);
    dialog.showModal();
    valid(); input.focus(); input.select();
  });
}
