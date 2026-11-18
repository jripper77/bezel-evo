/** Recent session choices, including an explicitly saved session that went offline. */
export function agentSessionOptions(content, sessions, t) {
  const options = [['', t('agent.automatic')]];
  for (const session of sessions ?? []) {
    if (session.agent === content.agent && projectMatches(content.project, session.projectPath) && !options.some(([id]) => id === session.id)) {
      options.push([session.id, `${session.project || t('agent.session')} · ${session.id.slice(0, 8)}`]);
    }
  }
  if (content.session && !options.some(([id]) => id === content.session)) {
    options.push([content.session, `${t('agent.savedSession')} · ${content.session.slice(0, 8)}`]);
  }
  return options;
}

const normalize = path => String(path ?? '').replaceAll('\\', '/').replace(/\/+$/, '').toLowerCase();
export function projectMatches(project, directory) {
  const filter = normalize(project), path = normalize(directory);
  return !filter || path === filter || path.startsWith(`${filter}/`);
}

/** Full paths distinguish equally named workspaces on different drives. */
export function agentProjectOptions(content, sessions, t) {
  const projects = new Map();
  for (const session of sessions ?? []) {
    if (session.agent === content.agent && session.projectPath) projects.set(normalize(session.projectPath), session.projectPath);
  }
  if (content.project) projects.set(normalize(content.project), content.project);
  for (const path of content.excludedProjects ?? []) projects.set(normalize(path), path);
  return [['', t('agent.allProjects')], ...[...projects.values()].sort((a, b) => a.localeCompare(b)).map(path => [path, path])];
}

export const AGENT_PARTS = Object.freeze(['showMessage', 'showActivity', 'showStatus', 'showFiveHour', 'showWeekly', 'showReset']);

/** Inclusion uses the same Windows path rules as the native collector. */
export function agentProjectIncluded(content, path) {
  return !(content.excludedProjects ?? []).some(excluded => projectMatches(excluded, path));
}
export function toggleAgentProject(content, path) {
  return agentProjectIncluded(content, path)
    ? [...(content.excludedProjects ?? []), path]
    : (content.excludedProjects ?? []).filter(excluded => !projectMatches(excluded, path));
}

/** Color settings, in inspector order. Unset keys keep the built-in palette of the native renderer. */
export const AGENT_STATE_COLORS = Object.freeze(['workingColor', 'attentionColor', 'completedColor', 'unknownColor']);
export const AGENT_QUOTA_COLORS = Object.freeze(['quotaColor', 'quotaTrackColor', 'warningColor']);
export const AGENT_COLOR_KEYS = Object.freeze([...AGENT_STATE_COLORS, ...AGENT_QUOTA_COLORS]);

/** Quota figure the native renderer prints; unset follows the layout (one line: remaining, two lines: both). */
export function quotaValueOf(content) {
  return content?.quotaValue ?? (content?.compactQuotas ? 'remaining' : 'both');
}
const ATTENTION = '#f45d60ff';

/** Color the native renderer uses for `key`: the saved value, otherwise its default. */
export function agentColor(content, key, textPaint = '#ffffffff') {
  if (content?.[key]) return content[key];
  switch (key) {
    case 'workingColor': return '#30bf8bff';
    case 'completedColor': return agentColor(content, 'attentionColor');
    case 'unknownColor': return '#8c8c8cff';
    case 'quotaColor': return content?.pixelStyle ? '#ff9248ff' : typeof textPaint === 'string' ? textPaint : '#ffffffff';
    case 'quotaTrackColor': return content?.pixelStyle ? '#222220ff' : '#64646464';
    default: return ATTENTION;
  }
}

/** Patch that clears every custom color. */
export function agentColorReset() {
  return Object.fromEntries(AGENT_COLOR_KEYS.map(key => [key, null]));
}

/** Parts of an agent widget, in inspector order. */
export function agentParts(content) {
  return content?.projectList ? ['title', 'rows', 'fiveHour', 'weekly'] : ['content', 'fiveHour', 'weekly'];
}

/** The detail view a part opens: both quota windows share one. */
export function agentPartView(part) {
  return part === 'fiveHour' || part === 'weekly' ? 'quotas' : part;
}

const PART_FLAG = Object.freeze({ title: 'showHeader', fiveHour: 'showFiveHour', weekly: 'showWeekly' });

/** Whether a part is drawn, and the patch that turns it on or off (null: always drawn). */
export function agentPartVisible(content, part) {
  return PART_FLAG[part] ? content?.[PART_FLAG[part]] !== false : true;
}
export function agentPartToggle(content, part) {
  return PART_FLAG[part] ? { [PART_FLAG[part]]: !agentPartVisible(content, part) } : null;
}

/** Text parts edited from each detail view. */
export const AGENT_VIEW_TEXT = Object.freeze({ title: ['agentTitle'], rows: ['agentName', 'agentState'], quotas: ['agentQuota'], content: [] });

const LAYOUT_KEY = Object.freeze({ title: 'title', rows: 'rows', fiveHour: 'fiveHour', weekly: 'weekly', content: 'content' });

/** A part's own place as shares of its widget, or null when automatic. */
export function agentPartPlace(content, part) {
  const r = content?.layout?.[LAYOUT_KEY[part]];
  return r && [r.x, r.y, r.width, r.height].every(Number.isFinite) && r.width > 0 && r.height > 0 ? r : null;
}

/** `box` kept inside `frame`, at least `min` theme pixels on each side. */
export function clampPartBox(frame, box, min = 8) {
  const width = Math.max(min, Math.min(box.width, frame.width)), height = Math.max(min, Math.min(box.height, frame.height));
  const x = Math.max(frame.x, Math.min(box.x, frame.x + frame.width - width));
  const y = Math.max(frame.y, Math.min(box.y, frame.y + frame.height - height));
  return { x, y, width, height };
}

/** The patch that places `part` at `box` (theme coordinates) inside `frame`, or back in its automatic place (null). */
export function agentPlacePatch(frame, part, box) {
  if (!box) return { layout: { [LAYOUT_KEY[part]]: null } };
  const b = clampPartBox(frame, box);
  const share = value => Math.round(value * 10000) / 10000;
  const x = share((b.x - frame.x) / frame.width), y = share((b.y - frame.y) / frame.height);
  return { layout: { [LAYOUT_KEY[part]]: { x, y, width: Math.min(1 - x, share(b.width / frame.width)), height: Math.min(1 - y, share(b.height / frame.height)) } } };
}

/** Whether any part has its own place. */
export function agentHasPlaces(content) {
  return Object.keys(LAYOUT_KEY).some(part => agentPartPlace(content, part));
}

/**
 * Where each part sits, in theme coordinates. Mirrors the native layout:
 * a part placed by hand keeps its own place; the others stack as before
 * (usage windows at the bottom, header and project rows above them).
 */
export function agentPartRects(element) {
  const f = element.frame, c = element.kind.content, s = element.kind.style ?? {};
  const size = s.size ?? 16;
  const placed = part => {
    const r = agentPartPlace(c, part);
    return r && { x: f.x + r.x * f.width, y: f.y + r.y * f.height, width: r.width * f.width, height: r.height * f.height };
  };
  const builtIn = c.pixelStyle ? 5 : 4;
  const extra = Math.max(0, (c.barHeight ?? builtIn) - builtIn);
  const quotaRow = size * (c.quotaSizePercent ?? 100) / 100 * (c.showReset !== false && !c.compactQuotas ? 2.8 : 1.6) + 8 + extra;
  const shown = ['fiveHour', 'weekly'].filter(part => agentPartVisible(c, part));
  const automatic = shown.filter(part => !placed(part));
  const total = quotaRow * automatic.length;
  const rects = {};
  let y = f.y + Math.max(0, f.height - total);
  for (const part of shown) {
    rects[part] = placed(part) ?? { x: f.x, y, width: f.width, height: quotaRow };
    if (!placed(part)) y += quotaRow;
  }
  const body = { x: f.x, y: f.y, width: f.width, height: Math.max(0, f.height - total - (automatic.length ? 4 : 0)) };
  if (!c.projectList) { rects.content = placed('content') ?? body; return rects; }
  const percent = s.parts?.agentTitle?.sizePercent;
  const ownTitle = placed('title');
  const header = c.showHeader !== false && !ownTitle ? Math.min(body.height, (percent != null ? size * percent / 100 : size) * 1.65) : 0;
  if (c.showHeader !== false) rects.title = ownTitle ?? { ...body, height: header };
  rects.rows = placed('rows') ?? { ...body, y: body.y + header, height: Math.max(0, body.height - header) };
  return rects;
}

/** The part under a theme point, or null. */
export function agentPartAt(element, x, y) {
  const rects = agentPartRects(element);
  return agentParts(element.kind.content).find(part => {
    const r = rects[part];
    return r && x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.height;
  }) ?? null;
}
