import { describeSections } from './sections.js';
import { el, icon } from '../dom.js';
import { ICONS } from '../icons.js';
import { chipButton, fieldGrid, fontPicker, iconButton, iconChoice, numberField, prefixedField, propertySection as section, selectField, textField, toggleField } from '../fields.js';
import {
  AGENT_QUOTA_COLORS, AGENT_STATE_COLORS, AGENT_VIEW_TEXT, agentColor, agentColorReset, agentHasPlaces, agentPartPlace, agentPartRects, agentPartToggle, agentPartView,
  agentPartVisible, agentParts, agentPlacePatch, agentProjectIncluded, agentProjectOptions, agentSessionOptions, quotaValueOf, toggleAgentProject,
} from '../../agent-widget.js';
import { agentPart, focusAgentPart } from '../../editor/agent-part.js';
import { partStylePatch } from '../../inspector-edit.js';
import { overflowField } from './text.js';

const clamp = (value, min, max) => Math.max(min, Math.min(max, Math.round(value)));

/** Pictures of the parts and of what a project row shows (24 px stroke paths). */
const ART = Object.freeze({
  title: ICONS.text,
  rows: ['M9 6h11', 'M9 12h11', 'M9 18h11', 'M4.5 6h.01', 'M4.5 12h.01', 'M4.5 18h.01'],
  quota: ['M3 12h18', 'M3 12h11'],
  content: ['M4 6h16', 'M4 12h16', 'M4 18h10'],
  dot: ['M12 8a4 4 0 1 1 0 8a4 4 0 1 1 0-8z'],
  state: ['M7 9a3 3 0 1 1 0 6a3 3 0 1 1 0-6z', 'M13 12h7'],
  tool: ['M14.7 6.3a4 4 0 0 0-5.4 5.4L4 17l3 3 5.3-5.3a4 4 0 0 0 5.4-5.4l-2.5 2.5-2.4-.6-.6-2.4z'],
  dividers: ['M4 8h16', 'M4 16h16'],
  path: ['M4 7h6l2 2h8v9H4z'],
  page: ['M4 5h16v14H4z', 'M14 12h3'],
  message: ['M4 6h16v10H8l-4 4z'],
  all: ['M9 6h11', 'M9 12h11', 'M9 18h11', 'M4.5 6h.01', 'M4.5 12h.01', 'M4.5 18h.01'],
  one: ['M12 4a8 8 0 1 1 0 16a8 8 0 1 1 0-16z', 'M12 9a3 3 0 1 1 0 6a3 3 0 1 1 0-6z'],
  right: ['M3 12h9', 'M15 12h6'],
  below: ['M3 8h14', 'M3 15h9'],
  oneLine: ['M3 10h10', 'M16 10h5', 'M3 15h18'],
  twoLines: ['M3 7h12', 'M3 12h8', 'M3 17h18'],
  continuous: ['M3 12h18'],
  segments: ['M3 12h3', 'M8 12h3', 'M13 12h3', 'M18 12h3'],
  remaining: ['M3 12h18', 'M3 12h13'],
  used: ['M3 12h18', 'M3 12h5'],
});
const PART_ART = Object.freeze({ title: ART.title, rows: ART.rows, fiveHour: ART.quota, weekly: ART.quota, content: ART.content });

/** Label of a part in the parts list, the header and the canvas tag. */
export function agentPartLabel(t, part) {
  return t({ title: 'insp.partTitle', rows: 'agent.rows', fiveHour: 'agent.showFiveHour', weekly: 'agent.showWeekly', content: 'inspector.content', quotas: 'agent.quotas' }[part]);
}

const TEXT_LABELS = Object.freeze({ agentTitle: 'insp.partTitle', agentName: 'agent.nameText', agentState: 'agent.showStateText', agentQuota: 'agent.quotas' });

/**
 * Details of a Codex or Claude Code panel. The whole widget shows its data,
 * its parts (each can be hidden or opened) and the text every part inherits;
 * an opened part shows only its own controls and its place.
 */
export function sections(ctx, provider = null) {
  const part = agentPart(ctx.element.id);
  return part ? partSections(ctx, part) : widgetSections(ctx, provider);
}

/** Header of an opened part: back to the widget, the path to the part and its name. */
export function partHeader(ctx) {
  const { element: e, t } = ctx;
  const part = agentPart(e.id);
  return el('div', { class: 'agent-part-head', 'data-agent-part': part }, [
    iconButton(t('agent.backToWidget'), ICONS.chevronLeft, () => focusAgentPart(e.id, null), { class: 'agent-part-back' }),
    el('div', { class: 'agent-part-titles' }, [
      el('span', { class: 'agent-part-path' }, [
        el('button', { type: 'button', class: 'agent-part-crumb', text: e.name, onclick: () => focusAgentPart(e.id, null) }),
        icon(ICONS.chevronRight, 12),
        el('span', { class: 'agent-part-kind', text: t('agent.partKind') }),
      ]),
      el('strong', { text: agentPartLabel(t, part) }),
    ]),
    el('span', { class: 'agent-part-art', 'aria-hidden': 'true' }, [icon(PART_ART[part] ?? ART.content, 18)]),
  ]);
}

/** A project or part list summary: what the row draws, in words. */
function partSummary(t, c, part, style) {
  if (part === 'title') return [style.parts?.agentTitle?.font ?? style.font.family, `${Math.round((style.parts?.agentTitle?.sizePercent ?? 100) * style.size / 100)} px`].join(' · ');
  if (part === 'rows') return [c.showStateText !== false && t('agent.showStateText'), c.showAge && t('agent.showAge'), c.showRowActivity && t('agent.showRowActivity')].filter(Boolean).join(' · ') || t('agent.stateDot');
  if (part === 'content') return c.statusOnly ? t('agent.statusOnly') : t('agent.showMessage');
  return [t(`agent.quota${quotaValueOf(c) === 'used' ? 'Used' : quotaValueOf(c) === 'both' ? 'Both' : 'Remaining'}`), c.pixelStyle ? t('agent.barPixel') : t('insp.patternContinuous')].join(' · ');
}

function widgetSections(ctx, provider) {
  const { element: e, t, update, host, bridge, fontFields } = ctx;
  const c = e.kind.content;
  const agent = provider ?? c.agent ?? 'codex';
  const set = patch => update(e.id, { kind: { content: patch } });
  const sessions = () => host.lastSample()?.activity?.agents;

  const project = selectField(t('agent.project'), c.project ?? '', agentProjectOptions(c, sessions(), t), value => set({ project: value, session: '' }));
  project.dataset.agentProjects = c.agent;
  const session = selectField(t('agent.session'), c.session ?? '', agentSessionOptions(c, sessions(), t), value => set({ session: value }));
  session.dataset.agentSessions = c.agent;
  // Projects as chips: the dot tells their state, a press hides or shows them.
  const inclusions = el('div', { class: 'chip-group agent-project-chips', role: 'group', 'aria-label': t('agent.shownProjects'), 'data-agent-inclusions': '' });
  inclusions.refresh = () => {
    if (inclusions.contains(document.activeElement)) return;
    const all = sessions() ?? [];
    const options = agentProjectOptions(c, all, t).filter(([path]) => path);
    const stateOf = path => all.filter(s => s.agent === c.agent && String(s.projectPath ?? '').replaceAll('\\', '/').toLowerCase() === path.replaceAll('\\', '/').toLowerCase())
      .sort((a, b) => (b.updatedAt ?? 0) - (a.updatedAt ?? 0))[0]?.state ?? 'unknown';
    const signature = JSON.stringify(options.map(([path]) => [path, stateOf(path), agentProjectIncluded(c, path)]));
    if (inclusions.dataset.signature === signature) return;
    inclusions.dataset.signature = signature;
    inclusions.replaceChildren(...(options.length ? options.map(([path]) => {
      const name = path.replaceAll('\\', '/').split('/').filter(Boolean).pop() ?? path;
      const chip = chipButton(name, agentProjectIncluded(c, path), () => set({ excludedProjects: toggleAgentProject(c, path) }), { title: path });
      chip.prepend(el('span', { class: 'pill-dot agent-state-dot', 'aria-hidden': 'true', dataset: { state: String(stateOf(path)).toLowerCase() } }));
      return chip;
    }) : [el('p', { class: 'hint', text: t('agent.noSession') })]));
  };
  inclusions.refresh();

  const hint = el('p', { class: 'hint', text: t(agent === 'codex' ? 'agent.codexHint' : 'agent.quotaHint') });
  const connect = el('button', { type: 'button', class: 'fld agent-refresh', onclick: async () => {
    connect.disabled = true;
    try {
      await bridge.connectClaudeAgent();
      hint.textContent = t('agent.connected');
    } catch { hint.textContent = t('agent.connectError'); }
    finally { connect.disabled = false; }
  } }, [icon(ICONS.refresh, 16), el('span', { text: t('agent.connectClaude') })]);

  const parts = agentParts(c).map(part => {
    const label = agentPartLabel(t, part);
    const toggle = agentPartToggle(c, part);
    const shown = agentPartVisible(c, part);
    return el('div', { class: `agent-part-row${shown ? '' : ' is-hidden'}`, 'data-agent-part-row': part }, [
      el('button', { type: 'button', class: 'agent-part-open', 'aria-label': t('agent.openPart', { part: label }), onclick: () => focusAgentPart(e.id, part) }, [
        icon(PART_ART[part], 16),
        el('span', { class: 'agent-part-name', text: label }),
        el('span', { class: 'agent-part-sum', text: agentPartPlace(c, part) ? `${partSummary(t, c, part, e.kind.style)} · ${t('agent.placedByHand')}` : partSummary(t, c, part, e.kind.style) }),
        icon(ICONS.chevronRight, 12),
      ]),
      toggle ? iconButton(t('agent.togglePart', { part: label }), shown ? ICONS.eye : ICONS.eyeOff, () => set(toggle), { class: 'agent-part-eye', 'aria-pressed': String(shown) }) : el('span', { class: 'agent-part-eye-gap' }),
    ]);
  });

  return describeSections(ctx, ctx.completePanels({
    data: [
      section('source', t('agent.source'), [
        !provider && selectField(t('agent.source'), c.agent ?? 'codex', [['codex', t('agent.codex')], ['claude', t('agent.claude')]], value => set({ agent: value, session: '', project: '' })),
        iconChoice(t('agent.view'), c.projectList ? 'projects' : 'session', [['projects', t('agent.projectList'), ART.all], ['session', t('agent.singleSession'), ART.one]], value => set({ projectList: value === 'projects' }), { wide: true }),
        !c.projectList && project,
        !c.projectList && session,
        c.projectList && fieldGrid([
          prefixedField(t('agent.recentPrefix'), t('agent.recentMinutes'), c.recentMinutes ?? 10, value => set({ recentMinutes: clamp(value, 0, 1440) }), { min: 0, max: 1440, unit: 'min' }),
          prefixedField(t('agent.pagePrefix'), t('agent.pageSeconds'), c.pageSeconds ?? 10, value => set({ pageSeconds: clamp(value, 1, 300) }), { min: 1, max: 300, unit: 's' }),
        ], { columns: 2 }),
        !c.projectList && el('p', { class: 'hint', text: t('agent.localHint') }),
      ]),
      c.projectList && section('agent-projects', t('agent.shownProjects'), [inclusions]),
      section('agent-usage', t('agent.quotas'), [agent === 'claude' && connect, hint]),
    ],
    look: [
      section('agent-parts', t('insp.section.parts'), [
        el('div', { class: 'agent-part-list' }, parts),
        agentHasPlaces(c) && el('button', { type: 'button', class: 'btn agent-auto-layout', text: t('agent.resetPlaces'), onclick: () => set({ layout: null }) }),
        el('p', { class: 'hint', text: t('agent.partsHint') }),
      ], { heading: false }),
      section('font', t('inspector.font'), fontFields(e, { compact: true })),
      section('agent-motion', t('agent.stateChanges'), [iconChoice(t('agent.blinkOnChange'), c.blinkOnChange === true ? 'on' : 'off', [['off', t('agent.highlightNone')], ['on', t('agent.highlightRow'), ART.rows]], value => set({ blinkOnChange: value === 'on' }), { wide: true })]),
      section('overflow', t('insp.textOverflow'), [overflowField(ctx, e, { bare: true })]),
    ],
  }));
}

/** One text part's own font, weight, color and size; empty fields follow the widget. */
function textPartEditor(ctx, part, { size = true } = {}) {
  const { element: e, t, update } = ctx;
  const style = e.kind.style;
  const value = style.parts?.[part] ?? {};
  const patch = change => update(e.id, { kind: { style: partStylePatch(style, part, change) } });
  const { texts, source } = ctx.pickers.font(ctx.sampleText(e));
  const font = fontPicker({ ...texts, family: t('insp.partFont'), face: t('insp.partFace') },
    { family: value.font ?? style.font.family, weight: value.weight ?? style.font.weight, italic: style.font.italic, asset: value.font ? value.fontAsset ?? null : style.font.asset },
    chosen => patch({ font: chosen.family, fontAsset: chosen.asset ?? null, weight: chosen.weight }), source,
    { faceSample: t('insp.faceSample'), fixedItalic: Boolean(style.font.italic), hideLabels: true });
  return el('div', { class: 'part-editor', role: 'group', 'aria-label': t('insp.partEditor', { part: t(TEXT_LABELS[part]) }) }, [
    el('div', { class: 'part-editor-head' }, [el('strong', { text: t(TEXT_LABELS[part]) }), el('span', { class: 'hint', text: t('insp.partInheritanceHint') }), iconButton(t('insp.inheritPart'), ICONS.undo, () => patch(null))]),
    font,
    fieldGrid([
      ctx.colorField(t('insp.partColor'), value.color ?? style.paint, color => patch({ color }), { hideLabel: true }),
      size && prefixedField(t('insp.sizePrefix'), t('insp.partSize'), value.sizePercent, sizePercent => patch({ sizePercent: sizePercent === null ? null : Math.max(10, Math.min(400, sizePercent)) }), { min: 10, max: 400, unit: '%', nullable: true }),
    ].filter(Boolean), { tail: true }),
  ]);
}

function partSections(ctx, part) {
  const { element: e, t, update, colorField, optionTiles } = ctx;
  const view = agentPartView(part);
  const c = e.kind.content;
  const set = patch => update(e.id, { kind: { content: patch } });
  const toggle = (key, fallback = true, label = `agent.${key}`) => toggleField(t(label), (c[key] ?? fallback) === true, () => set({ [key]: !(c[key] ?? fallback) }));
  const tile = (key, art, fallback = true, label = `agent.${key}`) => ({ text: t(label), art, pressed: (c[key] ?? fallback) === true, onClick: () => set({ [key]: !(c[key] ?? fallback) }) });
  // A color as in the boards: its swatch, its name and its value.
  const SHORT = { quotaColor: 'widget.bar', quotaTrackColor: 'agent.swatchTrack', warningColor: 'agent.swatchWarning' };
  const swatch = key => {
    const value = agentColor(c, key, e.kind.style?.paint);
    const follows = key === 'completedColor' && !c.completedColor;
    return el('div', { class: 'agent-swatch', title: `${t(`agent.${key}`)} · ${follows ? t('agent.sameAsAttention') : String(value).toUpperCase()}` }, [
      colorField(t(`agent.${key}`), value, chosen => set({ [key]: chosen }), { swatchOnly: true }),
      el('span', { class: 'agent-swatch-name', text: t(SHORT[key] ?? `agent.${key}`) }),
    ]);
  };
  const swatches = keys => el('div', { class: `agent-swatches cols-${keys.length > 3 ? 2 : 3}` }, keys.map(swatch));
  const reset = keys => keys.some(key => c[key]) && el('button', { type: 'button', class: 'btn', 'data-agent-reset-colors': '', text: t('agent.resetColors'),
    onclick: () => set(Object.fromEntries(Object.entries(agentColorReset()).filter(([key]) => keys.includes(key)))) });
  const texts = AGENT_VIEW_TEXT[view].map(name => section(name === 'agentState' ? 'agent-state-font' : 'font', t(TEXT_LABELS[name]), [textPartEditor(ctx, name, { size: name !== 'agentQuota' })]));

  // Where the part sits: automatic, or its own box inside the widget.
  const rect = agentPartRects(e)[part];
  const placed = agentPartPlace(c, part);
  const local = rect ? { x: Math.round(rect.x - e.frame.x), y: Math.round(rect.y - e.frame.y), width: Math.round(rect.width), height: Math.round(rect.height) } : null;
  const place = change => set(agentPlacePatch(e.frame, part, { x: e.frame.x + (change.x ?? local.x), y: e.frame.y + (change.y ?? local.y), width: change.width ?? local.width, height: change.height ?? local.height }));
  const placeSection = local && section('agent-part-place', t('agent.partPlace'), [
    fieldGrid([
      prefixedField('X', t('agent.partX'), local.x, x => place({ x }), { unit: 'px' }),
      prefixedField('Y', t('agent.partY'), local.y, y => place({ y }), { unit: 'px' }),
      prefixedField(t('insp.widthPrefix'), t('inspector.width'), local.width, width => place({ width: Math.max(8, width) }), { min: 8, unit: 'px' }),
      prefixedField(t('insp.heightPrefix'), t('inspector.height'), local.height, height => place({ height: Math.max(8, height) }), { min: 8, unit: 'px' }),
    ], { columns: 2 }),
    placed ? el('button', { type: 'button', class: 'btn', 'data-agent-auto-place': '', text: t('agent.autoPlace'), onclick: () => set(agentPlacePatch(e.frame, part, null)) })
      : el('p', { class: 'hint', text: t('agent.placeHint') }),
  ]);

  const panels = { data: [], look: [placeSection].filter(Boolean) };
  if (view === 'title') {
    panels.look.push(section('agent-header', t('agent.panel'), [
      textField(t('insp.partTitle'), c.title ?? '', value => set({ title: value.trim().slice(0, 64) }), { placeholder: t(`agent.${c.agent ?? 'codex'}`) }),
      toggle('showPage'),
    ]), ...texts);
  } else if (view === 'rows') {
    const rowTiles = optionTiles(t('agent.inEachRow'), [
      tile('showDot', ART.dot, true, 'agent.tileDot'), tile('showStateText', ART.state, true, 'agent.tileState'), tile('showAge', ICONS.clock, false, 'agent.tileAge'),
      tile('showRowActivity', ART.tool, false, 'agent.tileTool'), tile('showDividers', ART.dividers, true, 'agent.tileDividers'), tile('showPath', ART.path, false, 'agent.tilePath'),
    ]);
    rowTiles.classList.add('agent-row-tiles');
    panels.look.push(section('agent-rows', t('agent.rows'), [
      el('div', { class: 'field' }, [el('span', { class: 'field-label', text: t('agent.inEachRow') }), rowTiles]),
      c.showStateText !== false && iconChoice(t('agent.statePosition'), c.stateBelow ? 'below' : 'right', [['right', t('agent.stateRight'), ART.right], ['below', t('agent.stateBelow'), ART.below]], value => set({ stateBelow: value === 'below' }), { wide: true }),
      fieldGrid([
        prefixedField(t('agent.rowPrefix'), t('agent.rowHeight'), c.rowHeight ?? null, value => set({ rowHeight: value === null ? null : clamp(value, 8, 400) }), { min: 8, max: 400, unit: 'px', nullable: true, placeholder: t('insp.automatic') }),
      ], { columns: 2 }),
    ]), ...texts, section('agent-state-colors', t('agent.stateColors'), [swatches(AGENT_STATE_COLORS), reset(AGENT_STATE_COLORS)]));
  } else if (view === 'quotas') {
    panels.data.push(section('agent-quota-data', t('agent.quotas'), [
      iconChoice(t('agent.quotaValue'), quotaValueOf(c), [['remaining', t('agent.quotaRemaining'), ART.remaining], ['used', t('agent.quotaUsed'), ART.used], ['both', t('agent.quotaBoth')]], value => set({ quotaValue: value }), { wide: true }),
      toggle('showReset'),
      el('p', { class: 'hint', text: t('agent.quotasShared') }),
    ]));
    panels.look.push(section('agent-quotas', t('agent.quotas'), [
      fieldGrid([
        iconChoice(t('agent.quotaLayout'), c.compactQuotas ? 'one' : 'two', [['one', t('agent.oneLine'), ART.oneLine], ['two', t('agent.twoLines'), ART.twoLines]], value => set({ compactQuotas: value === 'one', quotaValue: quotaValueOf(c) }), { wide: true, iconOnly: true }),
        iconChoice(t('widget.bar'), c.pixelStyle ? 'pixel' : 'continuous', [['continuous', t('insp.patternContinuous'), ART.continuous], ['pixel', t('agent.barPixel'), ART.segments]], value => set({ pixelStyle: value === 'pixel' }), { wide: true, iconOnly: true }),
      ], { columns: 2 }),
      fieldGrid([
        prefixedField(t('agent.thickPrefix'), t('agent.barHeight'), c.barHeight ?? null, value => set({ barHeight: value === null ? null : clamp(value, 1, 24) }), { min: 1, max: 24, unit: 'px', nullable: true, placeholder: c.pixelStyle ? '5' : '4' }),
        c.pixelStyle && prefixedField(t('agent.gapPrefix'), t('agent.segmentGap'), c.segmentGap ?? null, value => set({ segmentGap: value === null ? null : clamp(value, 0, 12) }), { min: 0, max: 12, unit: 'px', nullable: true, placeholder: '3' }),
        prefixedField(t('agent.textPrefix'), t('agent.quotaSizePercent'), c.quotaSizePercent ?? 100, value => set({ quotaSizePercent: clamp(value, 50, 100) }), { min: 50, max: 100, unit: '%' }),
      ].filter(Boolean), { columns: 3 }),
    ]), ...texts, section('agent-quota-colors', t('agent.quotas'), [
      swatches(AGENT_QUOTA_COLORS),
      prefixedField(t('agent.warnPrefix'), t('agent.warningPercent'), c.warningPercent ?? 20, value => set({ warningPercent: clamp(value, 0, 50) }), { min: 0, max: 50, unit: '%' }),
      el('p', { class: 'hint', text: t('agent.warningHint') }),
      reset(AGENT_QUOTA_COLORS),
    ]));
  } else {
    const detailed = !c.statusOnly;
    const contentTiles = optionTiles(t('insp.show'), [tile('showStatus', ART.state), tile('showMessage', ART.message), tile('showActivity', ART.tool)]);
    panels.look.push(section('agent-header', t('inspector.content'), [
      textField(t('insp.partTitle'), c.title ?? '', value => set({ title: value.trim().slice(0, 64) }), { placeholder: t(`agent.${c.agent ?? 'codex'}`) }),
      toggle('statusOnly', false),
      detailed && contentTiles,
      detailed && c.showMessage !== false && numberField(t('agent.messageLines'), c.messageLines ?? 3, value => set({ messageLines: clamp(value, 1, 12) }), { min: 1, max: 12 }),
    ]), section('agent-state-colors', t('agent.stateColors'), [swatches(AGENT_STATE_COLORS), reset(AGENT_STATE_COLORS)]));
  }
  return describeSections(ctx, panels);
}
