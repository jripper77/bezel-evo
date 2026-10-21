// Profile drafts autosave independently of the open Studio document.
import { el, icon } from './dom.js';
import { ICONS } from './icons.js';
import { askChoice } from './dialog.js';
import { accordionSection, chipButton, iconButton, iconChoice, numberField, sentence, screenThumb } from './fields.js';
import { openMenuAt } from './menu.js';
import { createSectionMemory } from '../inspector-sections.js';
import { createProfilesSave } from '../profiles-save.js';
import { themePreview } from '../theme-previews.js';
import { ruleNow } from '../gallery-model.js';
import { errorText, sensorLabel } from '../messages.js';
import { PROFILE_EFFECTS, PROFILE_RULE_SOURCES, directed, firstIncomplete, modeOf, newProfileRule, profileIcon, profileSummary, ruleSummary, triedThemes, withMode, withoutProfile } from '../profiles-model.js';

const ADD_RULE_SOURCES = PROFILE_RULE_SOURCES.filter(source => source !== 'process');
const BASE = 'base', SECTIONS = ['themes', 'rules', 'behavior'];
export function createProfilesDialog({ t, bridge, notify, onStatus = () => {} }) {
  let dialog = null, config, overview, themes = [], devices = [], catalog = [], selected = BASE, saver, timer = null, saveState = 'saved';
  const memory = createSectionMemory({ key: 'bezel.profileSections', ids: SECTIONS, open: SECTIONS });
  const label = source => ['sensor', 'time'].includes(source) ? t(source === 'sensor' ? 'inspector.sensor' : 'widget.clock') : ['idle', 'locked'].includes(source) ? t(`profileRules.ui.${source}`) : t(`profiles.${source}`);
  const choices = () => [['', t('profiles.currentThemes')], ...config.profiles.map(p => [p.id, p.name])];
  const selectedProfile = () => config.profiles.find(p => p.id === selected);
  const entry = path => themes.find(e => e.location === path);
  function select(name, value, options, change, id) {
    return el('select', { id, 'aria-label': name, onchange: event => { change(event.target.value); changed(true); } }, options.map(([valueOf, text]) => el('option', { value: valueOf, text, selected: String(valueOf) === String(value) })));
  }
  function thumb(path) {
    const theme = entry(path), node = screenThumb(theme?.canvas ?? { width: 1920, height: 480 });
    node.classList.add('pf-thumb');
    if (theme) void themePreview(bridge, theme).then(url => { if (url) node.style.backgroundImage = `url("${url}")`; });
    return node;
  }
  function changed(redraw = false) {
    saver.update(withMode(config, modeOf(config)), firstIncomplete(config, overview.displays) === null);
    if (redraw) render(); else paintStatus();
  }
  function section(id, title, name, renderBody) {
    return accordionSection({ id, title, icon: name, expanded: memory.isOpen('profile', id), onToggle: options => {
      memory.toggle('profile', id, options); render();
    }, render: renderBody }).element;
  }
  function paintStatus() {
    if (!dialog) return;
    const status = overview.status, active = overview.config.profiles.find(p => p.id === status?.active?.profile);
    for (const node of dialog.querySelectorAll('[data-profile-active]')) {
      const on = node.dataset.profileActive === active?.id;
      node.textContent = on ? t('profileRules.ui.active') : t('profileRules.ui.ready');
      node.dataset.active = String(on);
    }
    const now = dialog.querySelector('.pf-now');
    if (now) {
      const reason = status?.active?.reason;
      const why = reason?.kind === 'rules' ? (reason.rules ?? []).map(i => active?.rules[i] && ruleSummary(t, active.rules[i])).filter(Boolean).join(' · ') : reason ? t(`profileRules.ui.reason.${reason.kind}`) : '';
      now.textContent = !status ? t('profileRules.ui.statusUnknown') : [active?.name ?? t(status?.active ? 'profileRules.ui.statusUnknown' : 'profiles.currentThemes'), why].filter(Boolean).join(' · ');
    }
    for (const node of dialog.querySelectorAll('[data-profile-rule]')) {
      const index = Number(node.dataset.profileRule), profile = selectedProfile();
      const value = profile ? ruleNow(overview, profile.id, index, profile.rules[index]) : null;
      node.textContent = value === null ? '—' : t(value ? 'profileRules.ui.trueNow' : 'profileRules.ui.falseNow');
      node.dataset.active = String(value === true);
    }
    for (const profile of config.profiles) {
      const caption = dialog.querySelector(`[data-item="${profile.id}"] .pf-item-summary`);
      if (caption) caption.textContent = profileSummary(t, profile);
    }
    const saved = dialog.querySelector('.pf-save-status');
    if (saved) saved.textContent = t(`profileRules.ui.save.${saveState}`);
  }
  async function refreshStatus() {
    if (!dialog || document.hidden) return;
    try { const next = await bridge.profiles(); if (dialog) { overview = next; paintStatus(); onStatus(next); } } catch { if (dialog) { overview = { ...overview, status: null }; paintStatus(); } }
  }
  function poll() { clearTimeout(timer); if (!dialog || document.hidden) return; timer = setTimeout(async () => { await refreshStatus(); poll(); }, 1000); }
  const visibility = () => { if (document.hidden) clearTimeout(timer); else { void refreshStatus(); poll(); } };
  async function tryProfile(profile) {
    try {
      if (!await saver.flush()) { revealIncomplete(); return; }
      const count = await bridge.tryThemes(triedThemes(config, profile?.id ?? BASE, overview.displays), profile?.id ?? null);
      if (!count) notify(t('profileRules.ui.noLiveScreen'));
      await refreshStatus();
    } catch (error) { notify(errorText(t, error)); }
  }
  function revealIncomplete() {
    const id = firstIncomplete(config, overview.displays); if (id) selected = id;
    for (const id of SECTIONS) if (!memory.isOpen('profile', id)) memory.toggle('profile', id);
    render(); dialog.querySelector('form').reportValidity();
  }
  async function close() {
    try {
      if (!await saver.flush()) {
        const answer = await askChoice(t, { title: t('profileRules.ui.incompleteTitle'), body: t('profileRules.ui.incompleteBody'), actions: [{ id: 'discard', label: t('profileRules.ui.discard'), kind: 'danger' }] });
        if (answer !== 'discard') { revealIncomplete(); return; }
      }
      dialog?.close();
    } catch { /* The error state and retry remain available. */ }
  }
  function addProfile() {
    const themes = Object.fromEntries(overview.displays.filter(d => d.theme).map(d => [d.key, d.theme]));
    const profile = { id: crypto.randomUUID(), name: t('profiles.new'), themes, rules: [] };
    config.profiles.push(profile); selected = profile.id; changed(true); dialog.querySelector('#pf-name')?.select();
  }
  async function removeProfile(profile) {
    const answer = await askChoice(t, { title: t('profiles.deleteTitle', { name: profile.name }), body: t('profiles.deleteBody'), actions: [{ id: 'delete', label: t('profiles.remove'), kind: 'danger' }], initial: 'delete' });
    if (answer !== 'delete' || !dialog) return;
    config = withoutProfile(config, profile.id); selected = BASE; changed(true);
  }
  function profileMenu(profile, anchor) {
    openMenuAt(anchor, { label: profile.name, items: [
      { label: t('common.rename'), run: () => dialog.querySelector('#pf-name')?.select() },
      { label: t('common.duplicate'), disabled: config.profiles.length >= 32, run: () => {
        const copy = { ...structuredClone(profile), id: crypto.randomUUID(), name: t('layers.copyOf', { name: profile.name }) };
        config.profiles.push(copy); selected = copy.id; changed(true);
      } },
      { label: t('profiles.remove'), run: () => void removeProfile(profile) },
    ] });
  }
  function list() {
    const choose = profile => el('button', { type: 'button', id: `pf-item-${profile?.id ?? BASE}`, class: 'pf-item', dataset: { item: profile?.id ?? BASE }, 'aria-current': selected === (profile?.id ?? BASE) ? 'true' : null,
      onclick: () => { selected = profile?.id ?? BASE; render(); } }, [
      icon(ICONS[profile ? profileIcon(profile) : 'base'], 18),
      el('span', { class: 'pf-item-text' }, [el('strong', { text: profile?.name ?? t('profiles.currentThemes') }), el('span', { class: 'pf-item-summary', text: profile ? profileSummary(t, profile) : t('profiles.baseHint') }),
        profile && el('span', { class: 'pf-miniatures' }, [...new Set(Object.values(profile.themes))].map(thumb))]),
      profile && el('span', { class: 'pf-state', dataset: { profileActive: profile.id } }),
    ]);
    const mode = modeOf(config);
    return el('aside', { class: 'pf-list' }, [
      el('p', { class: 'hint', text: t('profiles.whoDecides') }),
      iconChoice(t('profiles.whoDecides'), mode.mode, [['auto', t('profiles.automatic'), 'data'], ['manual', t('profiles.manual')]], value => { config = withMode(config, { ...mode, mode: value }); changed(true); }, { bare: true, wide: true }),
      mode.mode === 'manual' && select(t('profiles.manualShows'), mode.manual ?? '', choices(), value => { config = withMode(config, { ...mode, manual: value || null }); }, 'pf-manual'),
      el('p', { class: 'pf-now', role: 'status' }),
      el('h3', { class: 'pf-sect', text: t('profiles.count', { count: config.profiles.length }) }),
      ...config.profiles.map(choose), choose(null),
      el('button', { type: 'button', id: 'pf-add', class: 'pf-add', text: t('profiles.new'), disabled: !themes.length || !devices.length || config.profiles.length >= 32, onclick: addProfile }),
    ]);
  }
  function assignments(profile) {
    return overview.displays.filter(d => d.connected || profile?.themes[d.key]).map(display => {
      const path = profile ? profile.themes[display.key] ?? '' : display.theme;
      const device = devices.find(d => d.key === display.key);
      const compatible = themes.filter(theme => !device || device.models.some(model => theme.models.includes(model.id)));
      const options = compatible.map(theme => [theme.location, theme.name]);
      if (path && !options.some(([location]) => location === path)) options.push([path, path]);
      return el('div', { class: 'pf-theme', dataset: { screen: display.key } }, [
        icon(ICONS.screen, 16), el('span', { class: 'pf-theme-display', text: display.name }), thumb(path),
        profile ? select(t('profiles.themeFor', { display: display.name }), path, [['', t('profileRules.ui.keepCurrent')], ...options], value => {
          if (value) profile.themes[display.key] = value; else delete profile.themes[display.key];
        }, `pf-theme-${profile.id}-${display.key}`) : el('strong', { class: 'pf-theme-current', text: display.themeName ?? t('profiles.unassigned') }),
      ]);
    });
  }
  function addRule(profile, source) {
    const rule = newProfileRule(source, catalog);
    profile.rules.push(rule); changed(true);
  }
  function ruleEditor(profile, rule, index) {
    const id = `pf-rule-${profile.id}-${index}`;
    const input = (key, name, value, attrs = {}) => el('input', { id: `${id}-${key}`, type: 'text', 'aria-label': name, value: String(value ?? ''), ...attrs, oninput: event => { rule[key] = attrs.type === 'number' ? Number(event.target.value) : event.target.value; changed(); } });
    const pick = (key, name, options) => select(name, rule[key], options, value => { rule[key] = value; }, `${id}-${key}`);
    let tokens;
    if (rule.source === 'sensor') tokens = [t('cardInsp.when'), pick('key', t('inspector.sensor'), [...catalog.filter(s => s.quantity !== 'text').map(s => [s.key, sensorLabel(t, s)]), ...(!catalog.some(s => s.key === rule.key) && rule.key ? [[rule.key, rule.key]] : [])]), pick('compare', t('profileRules.ui.comparison'), ['above', 'below'].map(value => [value, t(`profileRules.${value}`)])), input('value', t('profileRules.ui.threshold'), rule.value, { type: 'number', step: 'any', required: true }), t('profileRules.ui.for'), input('forSeconds', t('profileRules.ui.hold'), rule.forSeconds ?? 0, { type: 'number', min: 0, max: 3600 }), t('unit.seconds', { value: '' }).trim()];
    else if (rule.source === 'time') tokens = [t('profileRules.ui.from'), input('from', t('profileRules.ui.start'), rule.from, { type: 'time', required: true }), t('profileRules.ui.to'), input('to', t('profileRules.ui.end'), rule.to, { type: 'time', required: true })];
    else if (rule.source === 'idle') tokens = [t('profileRules.ui.idleAfter'), input('minutes', t('profileRules.ui.minutes'), rule.minutes, { type: 'number', min: 1, max: 1440, required: true })];
    else if (rule.source === 'locked') tokens = [t('profileRules.locked')];
    else {
      const key = rule.source === 'mediaPlaying' ? 'profiles.summary.mediaIn' : rule.source === 'foreground' ? 'profiles.summary.foreground' : 'profiles.summary.process';
      const [before, after] = t(key, { app: '\0' }).split('\0');
      tokens = [before, input('app', t('profiles.app'), rule.app, { required: rule.source !== 'mediaPlaying', maxlength: 256, placeholder: rule.source === 'mediaPlaying' ? t('profiles.anyPlayer') : t('profiles.app') }), after];
    }
    const more = iconButton(t('profiles.removeRule'), ICONS.more, () => openMenuAt(more, { label: t('profiles.event'), items: [{ label: t('profiles.removeRule'), run: () => { profile.rules.splice(index, 1); changed(true); } }] }));
    return el('article', { class: 'pf-rule', dataset: { rule: String(index), kind: rule.source } }, [
      el('div', { class: 'pf-rule-head notice-head' }, [select(t('profiles.event'), rule.source, PROFILE_RULE_SOURCES.map(source => [source, label(source)]), source => {
        profile.rules[index] = { ...newProfileRule(source, catalog), priority: rule.priority, returnSeconds: rule.returnSeconds };
      }, `${id}-source`), el('span', { class: 'pf-state', dataset: { profileRule: String(index) } }), more]),
      sentence(tokens),
      rule.source === 'time' && el('div', { class: 'chip-group', role: 'group', 'aria-label': t('profileRules.ui.days') }, Array.from({ length: 7 }, (_, day) => chipButton(t(`profileRules.day${day}`), (rule.days ?? []).includes(day), () => { rule.days = (rule.days ?? []).includes(day) ? rule.days.filter(d => d !== day) : [...(rule.days ?? []), day]; changed(true); }))),
      numberField(t('trigger.priority'), rule.priority, value => { rule.priority = Math.max(0, Math.min(100, value)); changed(); }, { min: 0, max: 100 }),
    ]);
  }
  function detail() {
    const profile = selectedProfile();
    const title = profile?.name ?? t('profiles.currentThemes');
    const more = profile && iconButton(t('insp.more'), ICONS.more, () => profileMenu(profile, more));
    return el('section', { class: 'pf-detail', 'aria-label': title }, [
      el('div', { class: 'pf-object-head' }, [thumb(profile && Object.values(profile.themes)[0]),
        profile ? el('input', { id: 'pf-name', class: 'pf-name', type: 'text', 'aria-label': t('profiles.nameLabel'), value: profile.name, required: true, maxlength: 160, oninput: event => {
          profile.name = event.target.value; const name = dialog.querySelector(`[data-item="${profile.id}"] strong`); if (name) name.textContent = profile.name; changed();
        } }) : el('h3', { text: title }),
        profile && iconButton(t('profileRules.ui.try'), ICONS.play, () => void tryProfile(profile)), more,
        profile && el('span', { class: 'pf-state', dataset: { profileActive: profile.id } }),
      ]),
      section('themes', t('profileRules.ui.screenThemes'), 'screen', body => body.append(...assignments(profile))),
      profile && section('rules', t('profiles.activatesWhen'), 'data', body => body.append(
        ...profile.rules.map((rule, index) => ruleEditor(profile, rule, index)),
        iconChoice(t('profileRules.ui.combine'), profile.match ?? 'any', [['any', t('profileRules.ui.any')], ['all', t('profileRules.ui.all')]], value => { if (value === 'all') profile.match = value; else delete profile.match; changed(); }, { wide: true }),
        el('div', { class: 'pf-add-types', role: 'group', 'aria-label': t('profiles.addRule') }, ADD_RULE_SOURCES.map(source => el('button', { type: 'button', text: label(source), disabled: profile.rules.length >= 16, onclick: () => addRule(profile, source) }))),
      )),
      section('behavior', t('insp.section.behavior'), 'behav', body => body.append(...[
        select(t('profiles.transition'), config.transition.effect, PROFILE_EFFECTS.map(effect => [effect, t(effect === 'none' ? 'fade.none' : `profiles.effect.${effect}`)]), value => { config.transition.effect = value; }, 'pf-effect'),
        config.transition.effect !== 'none' && numberField(t('profiles.durationMs'), config.transition.durationMs, value => { config.transition.durationMs = value; changed(); }, { min: 150, max: 3000, step: 50, unit: 'ms' }),
        directed(config.transition.effect) && iconChoice(t('common.direction'), config.transition.direction, ['up', 'down', 'left', 'right'].map(direction => [direction, t(`card.direction.${direction}`), direction === 'left' ? 'arrowLeft' : direction === 'right' ? 'arrowRight' : direction]), value => { config.transition.direction = value; changed(); }),
        profile && numberField(t('profiles.returnDelay'), profile.rules[0]?.returnSeconds ?? 3, value => { profile.rules.forEach(rule => { rule.returnSeconds = value; }); changed(); }, { min: 0, max: 300, unit: 's' }),
        !profile && select(t('profiles.fallback'), config.defaultProfile ?? '', choices(), value => { config.defaultProfile = value || null; }, 'pf-fallback'),
        iconButton(t('profiles.preview'), ICONS.play, () => { void bridge.previewTransition(config.transition, triedThemes(config, profile?.id ?? BASE, overview.displays)).catch(error => notify(errorText(t, error))); }),
        el('p', { class: 'hint', text: t('profiles.editingPause') }),
      ].filter(Boolean))),
    ]);
  }
  function render() {
    if (!dialog) return;
    const active = document.activeElement, focused = dialog.contains(active) ? active.id : null, scroll = dialog.querySelector('.pf-detail')?.scrollTop ?? 0;
    dialog.replaceChildren(el('form', { onsubmit: event => { event.preventDefault(); void saver.flush().catch(() => {}); } }, [
      el('header', { class: 'dialog-head pf-head' }, [el('h2', { id: 'profiles-title', text: t('profiles.title') }), el('span', { class: 'pf-save-status', role: 'status' }), iconButton(t('dialog.close'), ICONS.close, () => void close())]),
      el('div', { class: 'pf-body' }, [list(), detail()]),
    ]));
    const detailNode = dialog.querySelector('.pf-detail'); if (detailNode) detailNode.scrollTop = scroll;
    if (focused) dialog.querySelector(`#${CSS.escape(focused)}`)?.focus({ preventScroll: true });
    paintStatus();
  }
  return { refresh: refreshStatus, async open() {
    if (dialog) return;
    try {
      const [saved, entries, hardware, sensors] = await Promise.all([bridge.profiles(), bridge.listThemes(), bridge.listDevices(), bridge.catalog()]);
      overview = saved; config = structuredClone(saved.config); themes = entries; devices = hardware.screens; catalog = sensors;
      config.transition ??= { effect: 'none', direction: 'left', durationMs: 1500 }; selected = BASE; saveState = 'saved';
      saver = createProfilesSave({ save: next => bridge.saveProfiles(next).then(refreshStatus), state: (state, error) => { saveState = state; paintStatus(); if (error) notify(errorText(t, error)); } });
      const opener = document.activeElement; dialog = el('dialog', { class: 'confirm-dialog profiles-dialog', 'aria-labelledby': 'profiles-title' });
      dialog.addEventListener('cancel', event => { event.preventDefault(); void close(); });
      dialog.addEventListener('close', () => { clearTimeout(timer); saver.dispose(); document.removeEventListener('visibilitychange', visibility); dialog.remove(); dialog = null; if (opener?.isConnected) opener.focus(); }, { once: true });
      document.body.append(dialog); render(); dialog.showModal(); poll(); document.addEventListener('visibilitychange', visibility);
    } catch (error) { notify(errorText(t, error)); }
  } };
}
