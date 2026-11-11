// The preferences dialog (the top bar's ⋯ menu): a column of pages on the
// left (D-2026-11-11-preferences-layout-1), the chosen page on the right.
// General: the app's language, which follows the system's until the user
// picks one, and whether Bezel starts at sign-in (the same login entry as
// Setup and Screen › Settings). Screens, Audio, Sensors (the host `net.ping`
// measures), Updates (the daily check and Check now) and Diagnostics. Each
// change applies at once and the backend remembers it.
import { el, icon } from './dom.js';
import { ICONS } from './icons.js';
import { errorText } from '../messages.js';

/**
 * The pages of the dialog, in the column's order: each its id, its icon,
 * its label (in the Studio's language) and the sections it shows (by name, built when drawn).
 */
export const PREFERENCE_PAGES = Object.freeze([
  { id: 'general', icon: 'sliders', label: (t) => t('prefs.page.general'), sections: ['language', 'startup'] },
  { id: 'screens', icon: 'monitor', label: (t) => t('prefs.page.screens'), sections: ['idle'] },
  { id: 'audio', icon: 'music', label: (t) => t('sensorsPanel.group.audio'), sections: ['audio'] },
  { id: 'sensors', icon: 'sensors', label: (t) => t('common.sensors'), sections: ['sensors'] },
  { id: 'updates', icon: 'download', label: (t) => t('prefs.updateTitle'), sections: ['update'] },
  { id: 'diagnostics', icon: 'performance', label: (t) => t('prefs.page.diagnostics'), sections: ['debug', 'logs'] },
]);

/** The page after `id` moving by `step` (wrapping), for the arrow keys. */
export function nextPage(id, step) {
  const at = Math.max(0, PREFERENCE_PAGES.findIndex((page) => page.id === id));
  return PREFERENCE_PAGES[(at + step + PREFERENCE_PAGES.length) % PREFERENCE_PAGES.length].id;
}

/**
 * What Preferences › Updates says about the last check: never made, a
 * newer release, or up to date, with when it ran.
 * @param {{ updateCheckedAt?: number|null, updateLatest?: string|null, updateAvailable?: string|null, version?: string }} prefs
 * @returns {{ key: string, params: object }}
 */
export function updateStatus(prefs, formatTime) {
  if (!prefs.updateCheckedAt) return { key: 'prefs.updateNever', params: {} };
  const when = formatTime(prefs.updateCheckedAt * 1000);
  // The newer release has its own line above, with the download button.
  if (prefs.updateAvailable) return { key: 'prefs.updateLastChecked', params: { when } };
  return { key: 'prefs.updateStatusLatest', params: { when, version: prefs.updateLatest ?? prefs.version ?? '' } };
}

/** The languages the studio speaks. */
export const LANGUAGES = Object.freeze(['pt-BR', 'en', 'it']);

/**
 * The options of the language select: the system's (named) first, then
 * each language by its own name.
 * @param {(k: string, p?: object) => string} t
 * @param {string} systemLanguage
 * @returns {Array<[string, string]>} `[value, label]`, `''` for the system's
 */
export function languageOptions(t, systemLanguage) {
  return [
    ['', t('prefs.languageSystem', { language: t(`language.${systemLanguage}`) })],
    ...LANGUAGES.map((l) => [l, t(`language.${l}`)]),
  ];
}

/**
 * @param {object} deps
 * @param {(k: string, p?: object) => string} deps.t
 * @param {object} deps.bridge
 * @param {(locale: string) => void} deps.onLanguage the UI switches to `locale`
 * @param {(on: boolean) => void} [deps.onAutostart] the start at sign-in changed
 * @param {(message: string) => void} deps.notify
 */
export function createPreferences({ t, bridge, onLanguage, onAutostart = () => {}, notify }) {
  let dialog = null;
  let prefs = null;
  // The page shown; kept while Studio runs, so the dialog reopens there.
  let page = PREFERENCE_PAGES[0].id;
  // Check now is asking GitHub.
  let checking = false;
  // Whether Bezel starts at sign-in; null when the host cannot tell.
  let autostart = null;
  // Why the last sensor change was refused (shown under its field), and the
  // host typed then (kept in the field to fix it).
  let sensorError = null;
  let typedHost = null;

  function languageField() {
    const select = el('select', { id: 'prefs-language', 'aria-describedby': 'prefs-language-hint' });
    for (const [value, label] of languageOptions(t, prefs.systemLanguage)) {
      select.append(el('option', { value, text: label, selected: value === (prefs.language ?? '') }));
    }
    select.addEventListener('change', () => chooseLanguage(select.value || null));
    return el('section', { class: 'prefs-section', 'aria-labelledby': 'prefs-language-title' }, [
      el('h3', { id: 'prefs-language-title', text: t('prefs.languageTitle') }),
      el('label', { class: 'field', for: 'prefs-language' }, [el('span', { text: t('prefs.language') }), select]),
      el('p', { id: 'prefs-language-hint', class: 'hint', text: t('prefs.languageHint') }),
    ]);
  }

  /** Start at sign-in: one switch for the user, though it is Light that starts. */
  function startupSection() {
    if (autostart === null) return null;
    const input = el('input', { id: 'prefs-autostart', type: 'checkbox', role: 'switch', checked: autostart, 'aria-describedby': 'prefs-autostart-hint' });
    input.addEventListener('change', async () => {
      input.disabled = true;
      try { await bridge.setAutostart(input.checked); autostart = input.checked; onAutostart(autostart); }
      catch (error) { input.checked = autostart; notify(errorText(t, error)); }
      finally { input.disabled = false; }
    });
    return el('section', { class: 'prefs-section', 'aria-labelledby': 'prefs-startup-title' }, [
      el('h3', { id: 'prefs-startup-title', text: t('prefs.startupTitle') }),
      el('label', { class: 'check', for: 'prefs-autostart' }, [input, el('span', { text: t('prefs.autostart') })]),
      el('p', { id: 'prefs-autostart-hint', class: 'hint', text: t('prefs.autostartHint') }),
    ]);
  }

  function audioSection() {
    const input = el('input', { id: 'prefs-audio-capture', type: 'checkbox', role: 'switch', checked: !!prefs.audioCapture, 'aria-describedby': 'prefs-audio-hint' });
    input.addEventListener('change', async () => {
      input.disabled = true;
      try { await bridge.setAudioCapture(input.checked); prefs.audioCapture = input.checked; }
      catch (error) { input.checked = !!prefs.audioCapture; notify(errorText(t, error)); }
      finally { input.disabled = false; }
    });
    return el('section', { class: 'prefs-section', 'aria-labelledby': 'prefs-audio-title' }, [
      el('h3', { id: 'prefs-audio-title', text: t('spectrum.capture') }),
      el('label', { class: 'check', for: 'prefs-audio-capture' }, [input, el('span', { text: t('spectrum.consent') })]),
      el('p', { id: 'prefs-audio-hint', class: 'hint', text: t('spectrum.privacy') }),
    ]);
  }

  function idleSection() {
    const input = el('input', { id: 'prefs-idle-saver', type: 'checkbox', role: 'switch', checked: prefs.idleSaver !== false, 'aria-describedby': 'prefs-idle-hint' });
    input.addEventListener('change', async () => {
      input.disabled = true;
      try { await bridge.setIdleSaver(input.checked); prefs.idleSaver = input.checked; }
      catch (error) { input.checked = prefs.idleSaver !== false; notify(errorText(t, error)); }
      finally { input.disabled = false; }
    });
    return el('section', { class: 'prefs-section', 'aria-labelledby': 'prefs-idle-title' }, [
      el('h3', { id: 'prefs-idle-title', text: t('prefs.idleTitle') }),
      el('label', { class: 'check', for: 'prefs-idle-saver' }, [input, el('span', { text: t('prefs.idleSaver') })]),
      el('p', { id: 'prefs-idle-hint', class: 'hint', text: t('prefs.idleHint') }),
    ]);
  }

  /** A time as the Studio's language writes a date and an hour. */
  const formatTime = (ms) => new Date(ms).toLocaleString(document.documentElement.lang || undefined, { dateStyle: 'medium', timeStyle: 'short' });

  /**
   * Bezel Light's daily look at the latest release, the newer one found,
   * when the last check ran, and Check now (asks at once).
   */
  function updateSection() {
    const input = el('input', { id: 'prefs-update-check', type: 'checkbox', role: 'switch', checked: prefs.updateCheck !== false, 'aria-describedby': 'prefs-update-hint' });
    const found = prefs.updateCheck !== false && prefs.updateAvailable
      ? el('p', { id: 'prefs-update-found', class: 'prefs-update-found' }, [
        el('span', { text: t('prefs.updateFound', { version: prefs.updateAvailable }) }), ' ',
        el('button', { type: 'button', class: 'text-button', id: 'prefs-open-releases', text: t('prefs.updateOpen'),
          onclick: () => bridge.openLink('releases').catch((e) => notify(errorText(t, e))) }),
      ])
      : null;
    input.addEventListener('change', async () => {
      input.disabled = true;
      try { await bridge.setUpdateCheck(input.checked); prefs.updateCheck = input.checked; if (found) found.hidden = !input.checked; }
      catch (error) { input.checked = prefs.updateCheck !== false; notify(errorText(t, error)); }
      finally { input.disabled = false; }
    });
    const status = updateStatus(prefs, formatTime);
    const checkNow = el('button', { type: 'button', class: 'text-button', id: 'prefs-check-now', disabled: checking,
      'aria-describedby': 'prefs-update-status', text: t(checking ? 'prefs.updateChecking' : 'prefs.updateCheckNow'), onclick: () => void checkUpdateNow() });
    return el('section', { class: 'prefs-section', 'aria-labelledby': 'prefs-update-title' }, [
      el('h3', { id: 'prefs-update-title', text: t('prefs.updateTitle') }),
      el('p', { class: 'prefs-version', text: t('app.title', { version: prefs.version ?? '' }) }),
      found,
      el('p', { id: 'prefs-update-status', class: 'hint', role: 'status', text: t(status.key, status.params) }),
      el('div', { class: 'button-row' }, [checkNow]),
      el('label', { class: 'check', for: 'prefs-update-check' }, [input, el('span', { text: t('prefs.updateCheck') })]),
      el('p', { id: 'prefs-update-hint', class: 'hint', text: t('prefs.updateHint') }),
    ]);
  }

  /** Asks for the latest release now; the page then says what it found. */
  async function checkUpdateNow() {
    if (checking) return;
    checking = true;
    render();
    try { prefs = await bridge.checkUpdateNow(); }
    catch (error) { notify(t('prefs.updateFailed', { reason: errorText(t, error) })); }
    finally { checking = false; render(); }
  }

  function debugSection() {
    const input = el('input', { id: 'prefs-debug', type: 'checkbox', role: 'switch', checked: prefs.debug });
    const overlay = el('input', { id: 'prefs-debug-overlay', type: 'checkbox', checked: prefs.debugShowReadings, disabled: !prefs.debug });
    const corner = el('select', { id: 'prefs-debug-corner', disabled: !prefs.debug || !prefs.debugShowReadings });
    for (const value of ['top-left', 'top-right', 'bottom-left', 'bottom-right']) {
      corner.append(el('option', { value, text: t(`prefs.debugCorner.${value}`), selected: value === (prefs.debugCorner ?? 'bottom-left') }));
    }
    corner.addEventListener('change', save);
    async function save() {
      input.disabled = true; overlay.disabled = true; corner.disabled = true;
      try {
        await bridge.setDebug(input.checked, overlay.checked, corner.value);
        prefs.debug = input.checked; prefs.debugShowReadings = overlay.checked; prefs.debugCorner = corner.value;
      } catch (e) {
        input.checked = prefs.debug; overlay.checked = prefs.debugShowReadings; corner.value = prefs.debugCorner ?? "bottom-left";
        notify(errorText(t, e));
      } finally { input.disabled = false; overlay.disabled = !prefs.debug; corner.disabled = !prefs.debug || !prefs.debugShowReadings; }
    }
    input.addEventListener('change', save); overlay.addEventListener('change', save);
    return el('section', { class: 'prefs-section', 'aria-labelledby': 'prefs-debug-title' }, [
      el('h3', { id: 'prefs-debug-title', text: t('prefs.debugTitle') }),
      el('label', { class: 'check', for: 'prefs-debug' }, [input, el('span', { text: t('prefs.debugEnable') })]),
      el('div', { style: { marginLeft: '24px' } }, [
        el('label', { class: 'check', for: 'prefs-debug-overlay' }, [overlay, el('span', { text: t('prefs.debugOverlay') })]),
        el('label', { class: 'field', for: 'prefs-debug-corner' }, [el('span', { text: t('prefs.debugCorner') }), corner]),
        el('p', { class: 'hint', text: t('prefs.debugOverlayHint') }),
      ]),
      el('p', { class: 'hint', text: t('prefs.debugHint') }),
    ]);
  }

  /** Always shown: where Studio and Light write their logs, and a click opens that folder. */
  function logsSection() {
    const open = el('button', { type: 'button', class: 'text-button', id: 'prefs-open-logs', text: t('prefs.openLogs'),
      'aria-describedby': 'prefs-logs-hint', onclick: () => bridge.openLogsFolder().catch((e) => notify(errorText(t, e))) });
    return el('section', { class: 'prefs-section', 'aria-labelledby': 'prefs-logs-title' }, [
      el('h3', { id: 'prefs-logs-title', text: t('prefs.logsTitle') }),
      el('p', { class: 'folder-path', text: prefs.debugLogPath }),
      el('p', { id: 'prefs-logs-hint', class: 'hint', text: t('prefs.logsHint') }),
      el('div', { class: 'button-row' }, [open]),
    ]);
  }

  function pingField() {
    const input = el('input', {
      id: 'prefs-ping-host', type: 'text', value: typedHost ?? prefs.pingHost, spellcheck: false, autocomplete: 'off',
      'aria-describedby': sensorError ? 'prefs-ping-hint prefs-sensor-error' : 'prefs-ping-hint',
      'aria-invalid': sensorError ? 'true' : null,
    });
    input.addEventListener('change', () => saveSensors({ pingHost: input.value }));
    const error = sensorError
      ? el('p', { id: 'prefs-sensor-error', class: 'field-error', role: 'alert', text: sensorError })
      : null;
    return [
      el('label', { class: 'field', for: 'prefs-ping-host' }, [el('span', { text: t('prefs.pingHost') }), input]),
      error,
      el('p', { id: 'prefs-ping-hint', class: 'hint', text: t('prefs.pingHint', { host: prefs.defaultPingHost }) }),
    ];
  }



  function sensorsSection() {
    return el('section', { class: 'prefs-section', 'aria-labelledby': 'prefs-sensors-title' }, [
      el('h3', { id: 'prefs-sensors-title', text: t('common.sensors') }),
      ...pingField(),
    ]);
  }



  /** Saves the sensor options with `change`; a refusal is shown in place. */
  async function saveSensors(change) {
    const next = { pingHost: prefs.pingHost, ...change };
    try {
      await bridge.setSensorOptions(next.pingHost);
      prefs = await bridge.preferences();
      sensorError = null;
      typedHost = null;
      notify(t('prefs.sensorsSaved'));
    } catch (e) {
      sensorError = errorText(t, e);
      typedHost = change.pingHost ?? null;
    }
    render();
  }

  async function chooseLanguage(language) {
    try {
      await bridge.setLanguage(language);
      prefs = { ...prefs, language };
      onLanguage(language ?? prefs.systemLanguage);
      render();
    } catch (e) {
      notify(errorText(t, e));
    }
  }

  const SECTIONS = { language: languageField, startup: startupSection, idle: idleSection, audio: audioSection, sensors: sensorsSection, update: updateSection, debug: debugSection, logs: logsSection };

  /** Shows page `id`; `focus` moves the focus to its entry in the column. */
  function showPage(id, { focus = false } = {}) {
    page = id;
    render();
    if (focus) dialog?.querySelector(`#prefs-tab-${id}`)?.focus();
  }

  /** The column of pages: tabs, the arrow keys move between them. */
  function pageList() {
    const list = el('div', { class: 'prefs-nav', role: 'tablist', 'aria-orientation': 'vertical', 'aria-label': t('prefs.pages') });
    for (const entry of PREFERENCE_PAGES) {
      const chosen = entry.id === page;
      list.append(el('button', {
        type: 'button', role: 'tab', id: `prefs-tab-${entry.id}`, class: 'prefs-nav-item',
        'aria-selected': String(chosen), 'aria-controls': 'prefs-page', tabindex: chosen ? '0' : '-1',
        onclick: () => showPage(entry.id),
      }, [icon(ICONS[entry.icon], 16), el('span', { text: entry.label(t) }),
        entry.id === 'updates' && prefs.updateCheck !== false && prefs.updateAvailable ? el('span', { class: 'prefs-nav-dot', 'aria-hidden': 'true' }) : null].filter(Boolean)));
    }
    list.addEventListener('keydown', (event) => {
      const step = { ArrowDown: 1, ArrowRight: 1, ArrowUp: -1, ArrowLeft: -1 }[event.key];
      const edge = { Home: PREFERENCE_PAGES[0].id, End: PREFERENCE_PAGES[PREFERENCE_PAGES.length - 1].id }[event.key];
      if (!step && !edge) return;
      event.preventDefault();
      showPage(edge ?? nextPage(page, step), { focus: true });
    });
    return list;
  }

  /** Draws the dialog again; the focused control keeps the focus. */
  function render() {
    if (!dialog) return;
    const focused = dialog.contains(document.activeElement) ? document.activeElement.id : null;
    const close = el('button', { type: 'button', class: 'icon-button dialog-close', title: t('dialog.close'), 'aria-label': t('dialog.close'), onclick: () => dialog.close() }, [icon(ICONS.close, 16)]);
    const shown = PREFERENCE_PAGES.find((entry) => entry.id === page) ?? PREFERENCE_PAGES[0];
    dialog.replaceChildren(
      el('div', { class: 'dialog-head' }, [el('h2', { id: 'prefs-title', text: t('prefs.title') }), close]),
      el('div', { class: 'dialog-body prefs-layout' }, [
        pageList(),
        el('div', { class: 'prefs-page', id: 'prefs-page', role: 'tabpanel', 'aria-labelledby': `prefs-tab-${shown.id}` },
          shown.sections.map((name) => SECTIONS[name]()).filter(Boolean)),
      ]),
      el('div', { class: 'dialog-actions' }, [el('button', { type: 'button', class: 'primary-button', text: t('common.done'), onclick: () => dialog.close() })]),
    );
    if (focused) dialog.querySelector(`#${focused}`)?.focus();
  }

  /** Opens the dialog; focus returns to the opener when it closes. */
  async function open() {
    if (dialog) return;
    try {
      prefs = await bridge.preferences();
    } catch (e) {
      notify(errorText(t, e));
      return;
    }
    autostart = await bridge.getAutostart().catch(() => null);
    const opener = document.activeElement;
    sensorError = null;
    typedHost = null;
    dialog = el('dialog', { class: 'confirm-dialog prefs-dialog', 'aria-labelledby': 'prefs-title' });
    dialog.addEventListener('close', () => {
      dialog.remove();
      dialog = null;
      if (opener?.isConnected) opener.focus();
    }, { once: true });
    render();
    document.body.append(dialog);
    dialog.showModal();
    dialog.querySelector(`#prefs-tab-${page}`).focus();
  }

  /** Opens the dialog on page `id` (the tray's and the status bar's shortcuts). */
  async function openAt(id) {
    if (PREFERENCE_PAGES.some((entry) => entry.id === id)) page = id;
    if (dialog) render(); else await open();
  }

  return { open, openAt };
}
