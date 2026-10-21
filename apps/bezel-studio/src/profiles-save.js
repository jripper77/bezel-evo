// Serial autosave for valid profile drafts. Newer drafts always win; failures remain retryable.
export function createProfilesSave({ save, state = () => {}, wait = (fn, ms) => setTimeout(fn, ms), cancel = clearTimeout, delay = 300 }) {
  let timer = null, queued = null, running = null, invalid = false;
  const clear = () => { if (timer !== null) cancel(timer); timer = null; };
  async function flush() {
    clear();
    if (running) { await running; return queued ? flush() : !invalid; }
    if (!queued) return !invalid;
    const next = queued; queued = null; state('saving');
    running = Promise.resolve().then(() => save(next)).then(() => state(invalid ? 'incomplete' : queued ? 'pending' : 'saved'), error => {
      if (!invalid) queued ??= next; state('error', error); throw error;
    }).finally(() => { running = null; });
    await running;
    return queued ? flush() : !invalid;
  }
  return {
    update(config, valid = true) {
      clear(); invalid = !valid; queued = valid ? structuredClone(config) : null;
      state(valid ? 'pending' : 'incomplete');
      if (valid) timer = wait(() => { timer = null; void flush().catch(() => {}); }, delay);
    },
    flush,
    dispose() { clear(); queued = null; },
  };
}
