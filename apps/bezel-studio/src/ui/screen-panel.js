// The Screen panel's Settings (artboard Screen): the card of the screen
// chosen at the top (glyph, name, port, size and protocol in mono, its
// state) with its brightness, "Start with the computer", the panels the
// vendor app left in desktop mode and, at the end of the tab, its advanced
// actions. The other screens show only once chosen at the top.
import { el, icon } from './dom.js';
import { ICONS } from './icons.js';
import { accordionSection, iconButton, numberField, rangeField, screenThumb, sectionTitle, toggleField } from './fields.js';

import { openMenuAt } from './menu.js';
import { createConfirm } from './storage.js';

/** The protocol letter of each screen family ("protocol C"). */
const PROTOCOLS = Object.freeze({ 'turing-rev-a': 'A', 'turing-rev-b': 'B', 'turing-rev-c': 'C', 'turing-usb': 'USB' });

/**
 * The screen panel (Screen › Settings): the chosen screen's card (glyph,
 * name, size, protocol, state), its brightness, "Start with the computer",
 * the panels in desktop mode and its advanced actions.
 * @param {{
 *   root: HTMLElement,
 *   t: (k: string, p?: object) => string,
 *   actions: {
 *     autostart: () => boolean, setAutostart: (on: boolean) => void,
 *     setBrightness: (screen: string, percent: number) => void, release: (screen: string) => void,
 *     leaveDesktopMode: (panel: object) => void, restart: (screen: string) => void,
 *   },
 * }} deps
 */
export function createScreenPanel({ root, t, actions }) {
  let args = [[], null, false, {}, [], {}];
  const confirm = createConfirm(t);
  const panel = root.closest('#panel-screen');
  const header = panel?.querySelector(':scope > .panel-head');
  let settingsOpen = true;
  // Status stays readable after "When the computer shuts down".
  const tail = el('div', { class: 'screen-tail' });
  root.parentElement?.append(tail);

  /** "Start with the computer" as a toggle row: its name, what it does, and the switch. */
  function autostartField() {
    const hint = el('span', { id: 'autostart-hint', class: 'toggle-hint', 'aria-hidden': 'true', text: t('screenPanel.autostartHint') });
    const row = toggleField(t('screenPanel.autostart'), actions.autostart(), on => {
      const button = row.querySelector('button');
      button.disabled = true;
      const refresh = () => {
        const focused = row.contains(document.activeElement);
        const next = autostartField(); row.replaceWith(next);
        if (focused) next.querySelector('button').focus();
      };
      Promise.resolve(actions.setAutostart(on)).then(refresh, refresh);
    }, { iconName: 'power' });
    row.classList.add('toggle-row');
    const label = row.querySelector('span');
    label.className = 'toggle-label';
    label.replaceChildren(el('span', { class: 'toggle-title', text: label.textContent }), hint);
    row.querySelector('button').setAttribute('aria-describedby', hint.id);
    return row;
  }

  /**
   * The panels the vendor app left in desktop mode: listed, labelled "not
   * validated on hardware", and switched back only after a confirmation
   * (D-2026-09-30-release-polish-8).
   */
  function desktopSection(panels) {
    if (!panels.length) return null;
    return el('section', { class: 'desktop-mode', 'aria-labelledby': 'desktop-mode-title' }, [
      sectionTitle(t('desktop.title'), { id: 'desktop-mode-title' }),
      ...panels.map((p) => el('div', { class: 'screen-card', role: 'group', 'aria-label': t('desktop.cardLabel', { address: p.key }) }, [
        el('strong', { text: t('desktop.name') }),
        el('span', { class: 'meta', text: `${p.key} · ${p.usb}` }),
        el('span', { class: 'badge not-validated' }, [icon(ICONS.warning, 14), t('desktop.notValidated')]),
        el('p', { class: 'hint', text: t('desktop.hint') }),
        el('p', { class: 'hint', text: t('desktop.models', { models: p.models.map((m) => m.name).join(', ') }) }),
        el('div', { class: 'button-row' }, [
          el('button', { type: 'button', class: 'text-button', text: t('desktop.leave'), onclick: () => actions.leaveDesktopMode(p) }),
        ]),
      ])),
    ]);
  }

  /**
   * What a screen that can be restarted without a replug offers: the
   * Restart status, and why to use the menu when the screen hung
   * (D-2026-09-30-release-polish-13).
   * @param {{key: string, restartable?: boolean}} s
   * @param {{restarting?: string|null, hung?: string|null}} restart
   */
  function restartParts(s, { restarting = null, hung = null }) {
    if (!s.restartable) return { note: null };
    const running = restarting === s.key;
    let note = null;
    if (running) note = el('p', { class: 'hint', role: 'status', text: t('restart.running') });
    else if (hung === s.key) note = el('p', { class: 'dialog-warning', role: 'status' }, [icon(ICONS.warning, 18), el('span', { text: t('restart.hung') })]);
    return { note };
  }

  /** The screen's card: a glyph in its native shape, its name, size and protocol in mono, and its state. */
  function screenCard(s, model, liveHere) {
    const shape = model ?? s.models[0];
    const thumb = screenThumb({ width: shape?.width ?? 1, height: shape?.height ?? 1 });
    thumb.classList.add('screen-glyph');
    const longest = Math.max(shape?.width ?? 1, shape?.height ?? 1);
    thumb.style.width = `${54 * (shape?.width ?? 1) / longest}px`;
    thumb.style.height = `${54 * (shape?.height ?? 1) / longest}px`;
    const protocol = PROTOCOLS[s.family];
    const facts = [
      model ? `${model.width}×${model.height}` : null,
      protocol ? t('screenPanel.protocol', { name: protocol }) : null,
    ].filter(Boolean);
    let tone = 'idle';
    if (s.state === 'awake') tone = liveHere ? 'live' : 'awake';
    return el('div', { class: 'screen-card' }, [
      thumb,
      el('span', { class: 'screen-id' }, [
        el('strong', { text: model ? model.name : s.models.map((m) => m.name).join(' / ') }),
        facts.length ? el('span', { class: 'screen-size', text: facts.join(' · ') }) : null,
        el('span', { class: 'screen-state', dataset: { tone } }, [
          el('span', { class: 'status-dot', 'aria-hidden': 'true' }),
          el('span', { class: 'meta mono', text: `${s.key} · ${t(`screen.state.${s.state}`)}${liveHere ? ` · ${t('status.live')}` : ''}` }),
        ]),
      ]),
    ]);
  }

  /** Brightness: its title and the level in mono over a slider between a dim and a bright sun. */
  function brightnessField(s, level) {
    const apply = value => actions.setBrightness(s.key, value);
    const field = rangeField(t('screen.brightness'), level, apply, { format: (value) => `${value}%` });
    field.classList.add('brightness');
    const head = sectionTitle(t('screen.brightness'));
    field.querySelector('.slider-head').replaceWith(head);
    const slider = field.querySelector('input');
    slider.setAttribute('aria-label', t('screen.brightness'));
    const amount = numberField(t('screen.brightness'), level, value => { slider.value = String(value); apply(value); }, { min: 0, max: 100, unit: '%', iconName: 'sunDim' });
    slider.addEventListener('input', () => { amount.querySelector('input').value = slider.value; });
    // Two native inputs have their own labels; neither label contains the other.
    return el('div', { class: 'brightness' }, [head, el('div', { class: 'brightness-row' }, [slider, amount])]);
  }

  function screenMenu(s, restart) {
    const more = iconButton(t('screenPanel.actions'), ICONS.more, () => openMenuAt(more, { label: t('screenPanel.actions'), items: [
      { label: t('screen.release'), run: async () => {
        if (await confirm({ title: t('screenPanel.releaseTitle'), body: [el('p', { text: t('screenPanel.releaseBody') })], action: t('screen.release') })) actions.release(s.key);
      } },
      ...(s.restartable ? [{ label: t(restart.restarting === s.key ? 'restart.running' : 'screen.restart'), disabled: Boolean(restart.restarting), run: () => actions.restart(s.key) }] : []),
    ] }), { class: 'quiet', 'aria-haspopup': 'menu', 'aria-expanded': 'false' });
    return more;
  }

  function settings(s, brightness) {
    const control = accordionSection({ id: 'screen-settings', title: t('library.screen'), icon: 'sunBright', expanded: settingsOpen,
      onToggle: () => { settingsOpen = !settingsOpen; control.setExpanded(settingsOpen); },
      render: body => body.append(brightnessField(s, brightness[s.key] ?? 70), autostartField()),
    });
    return control.element;
  }

  /**
   * @param {object[]} screens the screens listed
   * @param {string|null} current the screen chosen at the top, the only one shown
   * @param {boolean} live whether the screen in use is live
   * @param {Record<string, number>} brightness the level set on each screen in this session
   * @param {object[]} desktopMode the panels in desktop mode
   * @param {{restarting?: string|null, hung?: string|null}} restart the screen restarting, and the one that hung
   */
  function render(screens, current, live, brightness = {}, desktopMode = [], restart = {}) {
    args = [screens, current, live, brightness, desktopMode, restart];
    const desktop = [desktopSection(desktopMode)].filter(Boolean);
    const chosen = screens.find((s) => s.key === current) ?? null;
    if (!chosen) {
      root.replaceChildren(el('p', { class: 'empty-note', text: t('screen.none') }), autostartField(), ...desktop);
      header?.replaceChildren(el('h2', { text: t('library.screen') }));
      tail.replaceChildren();
      return;
    }
    const model = chosen.models.length === 1 ? chosen.models[0] : null;
    const card = screenCard(chosen, model, live);
    card.append(screenMenu(chosen, restart));
    if (header) { header.classList.add('screen-header'); header.replaceChildren(card); }
    root.replaceChildren(...(header ? [] : [card]), settings(chosen, brightness), ...desktop);
    const status = restartParts(chosen, restart);
    tail.replaceChildren(...(status?.note ? [status.note] : []));
  }

  return {
    render,
    /** The UI's language changed: the panel is drawn again. */
    retranslate: () => render(...args),
  };
}
