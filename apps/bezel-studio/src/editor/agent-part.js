// The part of an agent widget being edited: Studio state, never saved in the theme.

let focus = null;
const listeners = new Set();

/** The part of element `id` being edited, or null. */
export function agentPart(id) {
  return focus && focus.id === id ? focus.part : null;
}

/** Edits `part` of element `id`; a null part returns to the whole widget. */
export function focusAgentPart(id, part) {
  const next = part ? { id, part } : null;
  if (JSON.stringify(next) === JSON.stringify(focus)) return;
  focus = next;
  for (const listener of listeners) listener(focus);
}

/** Calls `listener` on every change; returns the unsubscribe function. */
export function onAgentPart(listener) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** The element whose part is being edited, or null. */
export function agentPartOwner() {
  return focus?.id ?? null;
}
