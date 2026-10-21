// The primitives every area builds its panels from (D-14: one builder per
// visual object; the look is base.css's). Field builders return a labelled
// element and call `onChange(value)` with a parsed value; nothing here knows
// the theme. Texts come translated from the caller.
import { el, icon } from './dom.js';
import { ICONS } from './icons.js';
import { joinWords, joinsPrevious } from '../sentence.js';
import { CORNERS, cornerDrawing, cornerMode, radiiOf, withCorner, withCornerMode, withRadius } from '../corner-radii.js';
import { alphaPercent, colorOf, hexOfHsva, hsvaOf, normalizeHex, readHexInput, rgbText, stepHsva, withAlphaPercent } from '../color-math.js';
import { FONT_FILTERS, faceForFamily, faceOptions, faceValue, findFamily, fontSections, fontStatus, nearestFace, parseFaceValue } from '../font-catalog.js';
import { openPopover } from './popover.js';

let counter = 0;
const fieldId = () => `f${(counter += 1)}`;

/** A named icon action, with the same native button and tooltip in every panel. */
export function iconButton(label, paths, onclick, { size = 18, class: extra = '', ...attrs } = {}) {
  return el('button', { type: 'button', class: `icon-button ${extra}`.trim(), 'aria-label': label, title: label, onclick, ...attrs }, [icon(paths, size)]);
}

// What each number input of these builders does on change, kept apart from
// the input so a form drawn again can hand it to the input still focused.
const changeOf = new WeakMap();

/** A number input whose `change` calls its handler with the parsed value (`step` 1 rounds to integers). */
function numberInput(attrs, step, onChange, nullable = false) {
  const input = el('input', { ...attrs, type: 'number', step });
  changeOf.set(input, raw => {
    if (nullable && raw === '') { onChange(null); return; }
    const v = Number(raw);
    if (Number.isFinite(v)) onChange(step === 1 ? Math.round(v) : v);
  });
  input.addEventListener('change', () => changeOf.get(input)(input.value));
  return input;
}

/** What the input of a form drawn again takes from its counterpart in the new form. */
const ADOPTED = Object.freeze(['id', 'min', 'max', 'step', 'aria-label', 'disabled', 'placeholder']);

/**
 * Keeps `current`, a number input of these builders that has focus, in the
 * place of `fresh`, its counterpart in a form drawn again (both in the
 * document): `current` takes the handler, attributes and value of `fresh`
 * (unless it shows the same number), so stepping it with the arrow keys or
 * its spin buttons keeps focus, caret and key repeat. Moved with
 * `moveBefore` (focus kept); elsewhere moved and focused again.
 * @returns {boolean} false when either input is not one of these builders'
 */
export function adoptControl(current, fresh) {
  if (current === fresh || !changeOf.has(current) || !changeOf.has(fresh) || !fresh.parentNode) return false;
  changeOf.set(current, changeOf.get(fresh));
  for (const name of ADOPTED) {
    const v = fresh.getAttribute(name);
    if (v === null) current.removeAttribute(name);
    else current.setAttribute(name, v);
  }
  if (Number(current.value) !== Number(fresh.value) || current.value === '') current.value = fresh.value;
  const parent = fresh.parentNode;
  try {
    parent.moveBefore(current, fresh);
  } catch {
    parent.insertBefore(current, fresh);
    current.focus({ preventScroll: true });
  }
  fresh.remove();
  return true;
}

/** A labelled number input. `step` 1 rounds to integers. */
export function numberField(label, value, onChange, { min, max, step = 1, unit, iconName, prefix, binding = null, nullable = false } = {}) {
  if (prefix !== undefined) {
    const field = prefixedField(prefix, label, value, onChange, { min, max, step, unit, iconName, nullable });
    return binding ? el('div', { class: 'bound-field' }, [field, bindButton(binding.label, binding.connected, binding.onClick)]) : field;
  }
  const id = fieldId();
  const input = numberInput({ id, value: String(value ?? ''), min, max, 'aria-label': label }, step, onChange, nullable);
  const field = el('label', { class: 'field number-field', for: id }, [
    el('span', { class: 'field-label' }, [iconName ? icon(ICONS[iconName], 16) : null, label]),
    el('span', { class: 'number-control' }, [input, unit ? el('span', { class: 'fld-unit', 'aria-hidden': 'true', text: unit }) : null]),
  ]);
  return binding ? el('div', { class: 'bound-field' }, [field, bindButton(binding.label, binding.connected, binding.onClick)]) : field;
}

/** A labelled text input (committed on change). */
export function textField(label, value, onChange, { placeholder, list, multiline = false } = {}) {
  const id = fieldId();
  const input = el(multiline ? 'textarea' : 'input', { id, ...(multiline ? { text: value ?? '', rows: 3 } : { type: 'text', value: value ?? '' }), placeholder, list, spellcheck: false });
  input.addEventListener('change', () => onChange(input.value));
  return el('label', { class: 'field', for: id }, [el('span', { text: label }), input]);
}

/** A labelled select from `[value, label]` pairs or `{group: [[v,l]]}` groups. */
export function selectField(label, value, options, onChange) {
  const id = fieldId();
  const select = el('select', { id });
  const add = (parent, [v, l]) => parent.append(el('option', { value: v, text: l, selected: v === value }));
  if (Array.isArray(options)) options.forEach((o) => add(select, o));
  else {
    for (const [group, list] of Object.entries(options)) {
      const og = el('optgroup', { label: group });
      list.forEach((o) => add(og, o));
      select.append(og);
    }
  }
  select.value = value ?? '';
  select.addEventListener('change', () => onChange(select.value));
  return el('label', { class: 'field', for: id }, [el('span', { text: label }), select]);
}

/** A checkbox. */
export function checkField(label, checked, onChange) {
  const input = el('input', { type: 'checkbox', checked: Boolean(checked) });
  input.addEventListener('change', () => onChange(input.checked));
  return el('label', { class: 'check' }, [input, el('span', { text: label })]);
}

/**
 * A slider under its label and mono value (artboards' "Spessore 24 px").
 * `format` writes the value with its unit; `change` commits it.
 */
export function rangeField(label, value, onChange, { min = 0, max = 100, step = 1, format = (v) => String(v), compact = false } = {}) {
  const id = fieldId();
  const out = el('output', { for: id, text: format(value) });
  const input = el('input', { id, type: 'range', min, max, step, value: String(value) });
  input.addEventListener('input', () => { out.textContent = format(Number(input.value)); });
  input.addEventListener('change', () => onChange(Number(input.value)));
  return el('label', { class: `slider-field${compact ? ' slider-field-inline' : ''}`, for: id }, [el('span', { class: 'slider-head' }, [el('span', { text: label }), ' ', out]), input]);
}

/**
 * Segmented buttons for a small set of choices (`[value, label]`), under
 * their label; `{bare: true}` gives the group alone (a toolbar's).
 */
export function segmented(label, value, options, onChange, { bare = false } = {}) {
  const group = el('div', { class: 'segmented', role: 'group', 'aria-label': label });
  for (const [v, l] of options) {
    group.append(el('button', { type: 'button', text: l, 'aria-pressed': String(v === value), onclick: () => onChange(v) }));
  }
  return bare ? group : el('div', { class: 'field' }, [el('span', { text: label }), group]);
}

/** Native pressed choices. Each option is [value, translated label, icon name]. */
export function iconChoice(label, value, options, onChange, { wide = false, iconOnly = false, bare = false } = {}) {
  const group = el('div', { class: `segmented field-choice${wide ? ' is-wide' : ''}`, role: 'group', 'aria-label': label });
  for (const [v, text, name] of options) group.append(el('button', {
    type: 'button', title: text, 'aria-label': text, 'aria-pressed': String(v === value), onclick: () => onChange(v),
  }, [name ? icon(ICONS[name], 18) : null, iconOnly ? null : text]));
  return bare ? group : el('div', { class: 'field' }, [el('span', { text: label }), group]);
}

/** The same preview tile in every widget. */
export function previewChoice(label, tiles) {
  return el('div', { class: 'option-tiles', role: 'group', 'aria-label': label }, tiles.map(({ text, art, pressed, onClick }) =>
    el('button', { type: 'button', class: 'option-tile', 'aria-pressed': String(pressed), onclick: onClick }, [icon(art, 32), el('span', { text })])));
}

/** Inspector toggles use native buttons with a visible state and accessible name. */
export function toggleField(label, pressed, onChange, { iconName = 'eye' } = {}) {
  return el('div', { class: 'inspector-toggle' }, [el('span', { text: label }),
    iconName === 'bolt' ? bindButton(label, pressed, () => onChange(!pressed))
      : iconButton(label, ICONS[iconName], () => onChange(!pressed), { 'aria-pressed': String(Boolean(pressed)) }),
  ]);
}

/** Binding affordance: callers own the supported binding and its editor. */
export function bindButton(label, connected, onClick) {
  return iconButton(label, ICONS.bolt ?? ICONS.sliders, onClick, { class: 'bind-button', 'aria-pressed': String(Boolean(connected)) });
}

/** A compact header, whose preview comes from the renderer rather than an example. */
export function objectHeader({ preview, name, subtitle, live, tryButton, menuButton }) {
  return el('div', { class: 'object-header' }, [preview,
    el('div', { class: 'object-heading' }, [name, subtitle]),
    el('div', { class: 'object-tools' }, [tryButton, menuButton, live]),
  ]);
}

export function typeRow(label, value, options, onChange) {
  const row = iconChoice(label, value, options, onChange, { iconOnly: true, wide: true });
  row.classList.add('inspector-type');
  return row;
}

/** Clickable styled parts, followed by the active part's own editor. */
export function partsRow(label, parts, selected, onSelect, renderEditor) {
  return el('div', { class: 'parts-field' }, [iconChoice(label, selected, parts, onSelect), renderEditor?.(selected)]);
}

/** One asset choice and its shared detail controls; import stays with the caller. */
export function imageField(label, value, choices, onChange, { preview = null, details = [] } = {}) {
  return el('div', { class: 'image-field' }, [preview, selectField(label, value, choices, onChange), ...details]);
}

/** Width and height. Linking is UI state; the caller owns aspect-ratio updates. */
export function sizeField(texts, value, onChange, { linked = false, onLink = null, min = 4 } = {}) {
  return el('div', { class: 'size-field' }, [
    prefixedField(texts.widthPrefix, texts.width, value.width, width => onChange({ width: Math.max(min, width) }), { min, unit: 'px' }),
    onLink && iconButton(texts.link, ICONS.link ?? ICONS.lock, () => onLink(!linked), { 'aria-pressed': String(linked) }),
    prefixedField(texts.heightPrefix, texts.height, value.height, height => onChange({ height: Math.max(min, height) }), { min, unit: 'px' }),
  ]);
}

// Public name for the existing shared corner implementation.
export const cornersField = (...args) => cornerRadii(...args);

/** Existing fade model, including imported radial settings, without new theme fields. */
export function fadeField(texts, value, onChange) {
  const f = value;
  const percent = (label, v, apply, min = 0, max = 100) => numberField(label, Math.round(v * 100), n => apply(Math.max(min, Math.min(max, n)) / 100), { min, max, unit: '%' });
  return el('div', { class: 'fade-field' }, [
    iconChoice(texts.mode, !f ? 'none' : f.radial ? 'radial' : 'linear', ['none', 'linear', 'radial'].map(mode => [mode, texts[mode], `fade-${mode}`]), mode => onChange(mode === 'none' ? null : { angle: f?.angle ?? 0, start: f?.start ?? 1, end: f?.end ?? 0, radial: mode === 'radial' ? f?.radial ?? [.5, .5, .5] : null })),
    ...(f ? [
      ...(f.radial ? [
        percent(texts.centerX, f.radial[0], x => onChange({ radial: [x, f.radial[1], f.radial[2]] })),
        percent(texts.centerY, f.radial[1], y => onChange({ radial: [f.radial[0], y, f.radial[2]] })),
        percent(texts.extent, f.radial[2], r => onChange({ radial: [f.radial[0], f.radial[1], r] }), 1, 200),
      ] : [numberField(texts.angle, f.angle, angle => onChange({ angle: Math.max(-360, Math.min(360, angle)) }), { min: -360, max: 360, unit: '°' }),
        rangeField(texts.midpoint, Math.round((f.midpoint ?? .5) * 100), v => onChange({ midpoint: v / 100 }), { min: 5, max: 95, format: v => `${v}%` })]),
      rangeField(texts.start, Math.round(f.start * 100), v => onChange({ start: v / 100 }), { format: v => `${v}%` }),
      rangeField(texts.end, Math.round(f.end * 100), v => onChange({ end: v / 100 }), { format: v => `${v}%` }),
    ] : []),
  ]);
}

/**
 * The artboards' field: a number in a well with a short mono prefix inside
 * (`X 90`, `min 0`, `inizio −135°`); `label` names it for assistive tech.
 * @param {string} prefix shown, e.g. `X` (translated when it is a word)
 * @param {string} label the accessible name, e.g. "X (px)"
 * @param {{min?: number, max?: number, step?: number|'any', unit?: string}} [opts]
 */
export function prefixedField(prefix, label, value, onChange, { min, max, step = 1, unit, iconName, nullable = false } = {}) {
  const id = fieldId();
  const input = numberInput({ id, value: String(value ?? ''), min, max, 'aria-label': label }, step, onChange, nullable);
  return el('label', { class: 'fld', for: id }, [
    iconName ? icon(ICONS[iconName], 16) : null,
    el('span', { class: 'fld-prefix', 'aria-hidden': 'true', text: prefix }),
    input,
    unit ? el('span', { class: 'fld-unit', 'aria-hidden': 'true', text: unit }) : null,
  ]);
}

/** The drawing's frame for each mode, CSS px: four fields around it, or one beside it. */
const CORNER_FRAMES = Object.freeze({ each: { width: 88, height: 72 }, same: { width: 56, height: 40 } });
/** The least length of a corner's mark along each side, CSS px. */
const CORNER_MARK = 10;
/** The style property of each corner's radius, in CORNERS order. */
const CORNER_RADIUS = Object.freeze(['borderTopLeftRadius', 'borderTopRightRadius', 'borderBottomRightRadius', 'borderBottomLeftRadius']);

/**
 * Corner radii (D-2026-10-15-studio-redesign-fidelity-22): a drawing of the
 * object with its corners as they are, and the radius beside it, or, set
 * one by one, a field at each corner around it. The field being edited
 * lights its corner. `value` is `{radius, corners}` (src/corner-radii.js:
 * corners null, or [top-left, top-right, bottom-right, bottom-left]) and
 * `onChange(value)` gets the next one, never below 0. Texts come
 * translated: `label` names the group and the shared radius, `prefix` is
 * the shared field's mono prefix, `mode`/`same`/`each` the choice between
 * one radius and individual corners (left out when `individual` is false),
 * `corners` the four fields' names in CORNERS order.
 * @param {{label: string, prefix: string, mode?: string, same?: string, each?: string, corners?: Record<string, string>}} texts
 * @param {{radius: number, corners: number[]|null}} value
 * @param {(value: {radius: number, corners: number[]|null}) => void} onChange
 * @param {{individual?: boolean, box?: {width: number, height: number}|null, unit?: string}} [opts]
 *   `box`: the object's size, for its proportions in the drawing
 * @returns {{tool: HTMLElement|null, control: HTMLElement}} the mode choice (a section head's tool) and the control
 */
export function cornerRadii(texts, value, onChange, { individual = true, box = null, unit = 'px' } = {}) {
  const mode = individual ? cornerMode(value) : 'same';
  const tool = individual
    ? segmented(texts.mode, mode, [['same', texts.same], ['each', texts.each]], (next) => next !== mode && onChange(withCornerMode(value, next)), { bare: true })
    : null;
  const drawing = cornerDrawing(mode === 'same' ? { radius: value.radius, corners: null } : value, box, CORNER_FRAMES[mode]);
  // Each corner's mark traces that corner (its arc, or an L when square) and lights while its field is edited.
  const shape = el('span', { class: 'corner-shape', style: { width: `${drawing.width}px`, height: `${drawing.height}px`, borderRadius: drawing.radii.map((r) => `${r}px`).join(' ') } },
    CORNERS.map((name, i) => {
      const r = drawing.radii[i], side = `${Math.max(CORNER_MARK, r)}px`;
      return el('span', { class: 'corner-mark', dataset: { corner: name }, style: { width: side, height: side, [CORNER_RADIUS[i]]: `${r}px` } });
    }));
  const art = el('span', { class: 'corner-drawing', 'aria-hidden': 'true' }, [shape]);
  const field = (prefix, label, r, apply, corner) => {
    const node = prefixedField(prefix, label, r, apply, { min: 0, unit });
    if (corner) node.dataset.corner = corner;
    return node;
  };
  if (mode === 'same') {
    const shared = field(texts.prefix, texts.label, value.radius, (r) => onChange(withRadius(value, r)), null);
    return { tool, control: el('div', { class: 'corner-radii', role: 'group', 'aria-label': texts.label, dataset: { mode } }, [art, shared]) };
  }
  const radii = radiiOf(value);
  const corner = (i) => field('', texts.corners[CORNERS[i]], radii[i], (r) => onChange(withCorner(value, i, r)), CORNERS[i]);
  // Reading order: the top corners, the drawing, the bottom ones.
  return { tool, control: el('div', { class: 'corner-radii', role: 'group', 'aria-label': texts.label, dataset: { mode } }, [corner(0), corner(1), art, corner(3), corner(2)]) };
}

/**
 * A sentence the user completes (card triggers, a graph's window): `parts`
 * are words (strings) and tokens (nodes), in reading order. The input or
 * select of a token takes the token look; words that follow each other
 * read as one, and a word that starts with punctuation sticks to the
 * token before it.
 * @param {(string|Node)[]} parts
 * @param {Record<string, any>} [attrs]
 */
export function sentence(parts, attrs = {}) {
  return el('div', { ...attrs, class: `sentence ${attrs.class ?? ''}`.trim() }, joinWords(parts).map((part) => {
    if (typeof part === 'string') return el('span', { class: joinsPrevious(part) ? 'sentence-word is-joined' : 'sentence-word', text: part });
    const control = part.matches('input, select') ? part : part.querySelector('input, select');
    control?.classList.add('sentence-token');
    // A sentence supplies the label in its surrounding words. Keep only the
    // named input so its numeric well cannot be hidden with a field's label.
    if (part.matches('.number-field') && control) return control;
    return part;
  }));
}

/** Prefixed fields two by two (position and size, min and max). */
export const fieldGrid = (fields) => el('div', { class: 'fld-grid' }, fields);

/** A switch with its label on the left and the switch on the right (artboards' toggles). */
export function switchRow(label, checked, onChange) {
  const input = el('input', { type: 'checkbox', role: 'switch', checked: Boolean(checked) });
  input.addEventListener('change', () => onChange(input.checked));
  return el('label', { class: 'switch-row' }, [
    el('span', { text: label }),
    input,
    el('span', { class: 'track', 'aria-hidden': 'true' }, [el('span', { class: 'thumb' })]),
  ]);
}

/** A count or short mono readout beside a title ("13 elementi"). */
export const count = (text) => el('span', { class: 'count', text: String(text) });

/** A section's 11px caps title, with its count when given. */
export function sectionTitle(text, { count: n = null, id } = {}) {
  return el('div', { class: 'section-head' }, [el('h3', { class: 'section-title', id, text }), n === null ? null : count(n)]);
}

/** A property group shared by the object and Card inspector. */
const SECTION_ICONS = Object.freeze({
  membership: 'card', position: 'drag', source: 'database', content: 'count',
  transition: 'frequency', font: 'text', color: 'brush', appearance: 'brush',
  style: 'sliders', shape: 'shape', corners: 'radius', outline: 'strokeWidth',
  transparency: 'sunDim', visibility: 'eye', shadow: 'sunDim', track: 'bar',
  segments: 'grid', arc: 'ring', scale: 'height', window: 'history',
  'spectrum-source': 'spectrum', 'spectrum-bands': 'frequency',
  'spectrum-paint': 'brush', 'spectrum-bars': 'spectrum',
  apps: 'player', stopped: 'pause', time: 'clock', show: 'eye', spacing: 'spacing',
  progress: 'bar', fill: 'brush', preview: 'eye', faces: 'layers', objects: 'widgets',
  motion: 'play', triggers: 'sliders', actions: 'more', refresh: 'refresh',
});

export function propertySection(key, label, nodes, { count: n = null, tool = null } = {}) {
  const contents = nodes.filter(Boolean);
  if (!contents.length) return null;
  const head = sectionTitle(label, { count: n });
  head.querySelector('h3').prepend(icon(ICONS[SECTION_ICONS[key] ?? 'sliders'], 17));
  if (tool) head.append(tool);
  return el('section', { class: 'property-section', dataset: { section: key } }, [head, el('div', { class: 'property-section-body' }, contents)]);
}

/**
 * Controlled, independent inspector disclosure (D-01/D-04). The owner keeps
 * state with createSectionMemory and calls setExpanded on all peers after a
 * toggle. Bodies stay mounted: closing never discards a field or its edit.
 * Native buttons provide Enter/Space; arrows/Home/End visit sibling titles.
 */
export function accordionSection({ id, title, icon: name = 'sliders', summary = '', expanded = false, onToggle, render }, ctx) {
  const bodyId = fieldId();
  const titleId = fieldId();
  const readout = el('span', { class: 'accordion-summary', 'aria-hidden': 'true', text: summary });
  const toggle = el('button', {
    type: 'button', id: titleId, class: 'accordion-toggle', 'aria-controls': bodyId,
    'aria-expanded': String(expanded), 'aria-label': title,
    onclick: event => onToggle({ exclusive: event.altKey }),
  }, [icon(ICONS[name], 16), el('span', { class: 'accordion-title', text: title }), readout, icon(ICONS.chevronDown, 12)]);
  const body = el('div', { class: 'accordion-body', id: bodyId, role: 'region', 'aria-labelledby': titleId, hidden: !expanded });
  render(body, ctx);
  const section = el('section', { class: 'accordion-section', dataset: { inspectorSection: id } }, [el('h3', { class: 'accordion-heading' }, [toggle]), body]);
  toggle.addEventListener('keydown', event => {
    if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) return;
    const peers = [...section.parentElement.querySelectorAll(':scope > .accordion-section > .accordion-heading > .accordion-toggle')];
    const index = peers.indexOf(toggle);
    const next = event.key === 'Home' ? 0 : event.key === 'End' ? peers.length - 1
      : (index + (event.key === 'ArrowDown' ? 1 : -1) + peers.length) % peers.length;
    event.preventDefault();
    peers[next]?.focus();
  });
  const setExpanded = (open) => {
    toggle.setAttribute('aria-expanded', String(open));
    body.hidden = !open;
    readout.hidden = open;
  };
  setExpanded(expanded);
  return { element: section, setExpanded };
}

/**
 * A panel's head: its 15px title, a count (a node with an id that the panel
 * keeps up to date, or a text) and quiet tools.
 * @param {string} title
 * @param {{count?: Node|string|null, tools?: Node[], id?: string}} [opts]
 */
export function panelHead(title, { count: n = null, tools = [], id } = {}) {
  return el('div', { class: 'panel-head' }, [
    el('h2', { id, text: title }),
    n === null ? null : typeof n === 'string' ? count(n) : n,
    tools.length ? el('span', { class: 'panel-tools' }, tools) : null,
  ]);
}

/** A filter chip (aria-pressed). */
export function chipButton(text, pressed, onClick, attrs = {}) {
  return el('button', { type: 'button', class: 'chip', 'aria-pressed': String(Boolean(pressed)), text, onclick: onClick, ...attrs });
}

/**
 * A status pill: mono, with a dot when `dot`; `tone` is `ok`, `accent`,
 * `warn`, `danger`, `info` or none. `as: 'button'` makes it open something.
 */
export function pill(text, { tone = null, dot = false, as = 'span', attrs = {} } = {}) {
  const node = el(as, { class: 'pill', ...(as === 'button' ? { type: 'button' } : {}), dataset: tone ? { tone } : {}, ...attrs });
  node.append(...(dot ? [el('span', { class: 'pill-dot', 'aria-hidden': 'true' })] : []), text);
  return node;
}

/** A search box: the glass and the input in a well; `onInput(text)` on every keystroke. */
export function searchField(label, onInput, { placeholder = label, id, value = '' } = {}) {
  const input = el('input', { type: 'search', id, value, placeholder, 'aria-label': label, spellcheck: false });
  input.addEventListener('input', () => onInput(input.value));
  return el('label', { class: 'search-box' }, [icon(ICONS.search, 16), input]);
}

/** How to use a panel, at its foot: an icon and the text (nodes, e.g. with `kbd`). */
export function dashedHint(children, { iconName = 'dropHint' } = {}) {
  return el('div', { class: 'dashed-hint' }, [icon(ICONS[iconName], 18), el('p', {}, children)]);
}

/**
 * A choice shown as a card: icon, title over a mono key, a value and a chevron.
 * @param {{icon?: string, title: string, subtitle?: string, value?: string, label?: string, onClick?: () => void, pressed?: boolean}} card
 */
export function listCard({ icon: name = null, title, subtitle = null, value = null, label = null, onClick = null, pressed = null }) {
  const chevron = icon(ICONS.chevronDown, 14);
  chevron.classList.add('list-card-chevron');
  return el('button', { type: 'button', class: 'list-card', 'aria-label': label, 'aria-pressed': pressed === null ? null : String(pressed), onclick: onClick }, [
    name ? icon(ICONS[name], 18) : null,
    el('span', { class: 'list-card-text' }, [
      el('span', { class: 'list-card-title', text: title }),
      subtitle ? el('span', { class: 'list-card-sub', text: subtitle }) : null,
    ]),
    value === null ? null : el('span', { class: 'list-card-value', text: value }),
    chevron,
  ]);
}

/** An object's kind on an accent tile (inspector head, layer rows): ICONS[kind]. */
export function kindIcon(kind, { small = false } = {}) {
  return el('span', { class: `kind-icon${small ? ' small' : ''}`, 'aria-hidden': 'true' }, [icon(ICONS[kind] ?? ICONS.shape, small ? 16 : 18)]);
}

/**
 * A screen in miniature, in its panel's shape: dark, or showing `url`.
 * The parent sizes it; it keeps `width:height`.
 */
export function screenThumb({ width, height, url = null }) {
  const thumb = el('span', { class: 'screen-thumb', 'aria-hidden': 'true' });
  thumb.style.setProperty('--ratio', `${width} / ${height}`);
  if (url) thumb.style.backgroundImage = `url("${url}")`;
  return thumb;
}


// --------------------------------------------------------------- pickers --
// The font picker (D-2026-10-16-studio-redesign-fidelity-28) and the colour
// picker (-29): one of each for every place a font or a colour is chosen.
// Both open an anchored popover (ui/popover.js); a value they change is
// handed to `onChange`, so each change is the caller's one undo step.

/** A family name as a CSS font-family value; where the window lacks it, the UI's sans, never a stray serif. */
const cssFamily = (name) => `"${String(name).replace(/["\\]/g, '\\$&')}", var(--font-sans)`;

/**
 * Where focus goes back when a popover closes: its button, or, when the
 * panel holding it was drawn again meanwhile, the button that took its
 * place (same `data-picker` key, same position among those).
 */
function refocusFor(anchor) {
  const key = anchor.dataset.picker;
  const index = [...document.querySelectorAll(`[data-picker="${CSS.escape(key)}"]`)].indexOf(anchor);
  return () => {
    const target = anchor.isConnected ? anchor : document.querySelectorAll(`[data-picker="${CSS.escape(key)}"]`)[index];
    target?.focus();
  };
}

/** Options of the system list drawn at a time; more as it scrolls. */
const FONT_CHUNK = 40;

/**
 * The font of a text: a button showing the family in its own letters, which
 * opens the picker (search, Tutti/Sans/Serif/Mono, the theme's families,
 * recent ones, Bezel's and the system's, each name in its font, a sample of
 * the object's text), and the face select with only the faces the family
 * has. A family the machine lacks shows a warning and the family drawn
 * instead. Keyboard: arrows move in the list, Enter picks, Escape closes;
 * focus goes back to the button.
 * @param {object} texts translated: `family`, `face`, `title`, `search`,
 *   `filter`, `filters` {all,sans,serif,mono}, `sections`
 *   {assets,theme,recent,bundled,user,system}, `faces(n)`, `missing`,
 *   `missingNote(family, fallback)`, `faceName(face)`,
 *   `faceMissing(face, nearest)`, `none`, `loading`
 * @param {{family: string, weight: number, italic: boolean, asset?: string|null}} value
 * @param {(font: {family: string, weight: number, italic: boolean, asset?: string|null}) => void} onChange
 * @param {{catalog: () => object|null, themeFamilies: () => string[], recent: () => string[], remember: (name: string) => void, sample: string, showFace?: (name: string, face: object) => void}} source
 */
export function fontPicker(texts, value, onChange, source, { faceSample = null, fixedItalic = null } = {}) {
  const catalog = source.catalog();
  const status = fontStatus(catalog, value);
  const current = { weight: value.weight, italic: Boolean(value.italic) };
  const shown = status.missing ? status.fallback : value.family;
  const id = fieldId();
  const nameNode = el('span', { class: 'font-button-name', id: `${id}-v`, text: value.family });
  if (shown) nameNode.style.fontFamily = cssFamily(source.faceFamily?.(shown, value.asset) ?? shown);
  if (status.family && (status.family.bundled || status.family.origin === 'user' || value.asset)) source.showFace?.(status.family.name, nearestFace(status.family.faces, current), value.asset);
  const note = status.missing
    ? el('p', { class: 'font-missing', id: `${id}-m` }, [icon(ICONS.warning, 14), texts.missingNote(value.family, status.fallback ?? '—')])
    : null;
  const button = el('button', {
    type: 'button', class: 'font-button', id, 'aria-haspopup': 'dialog', 'aria-expanded': 'false',
    'aria-label': texts.family, 'aria-describedby': [`${id}-v`, note ? `${id}-m` : null].filter(Boolean).join(' '),
    dataset: { picker: `font:${texts.family}` },
  }, [nameNode, status.missing ? pill(texts.missing, { tone: 'warn' }) : null, icon(ICONS.chevronDown, 14)]);
  button.addEventListener('click', async () => {
    if (button.getAttribute('aria-expanded') === 'true') return;
    button.disabled = true;
    try { await source.reload?.(); } finally { button.disabled = false; }
    openFontList(button, texts, value, onChange, source);
  });
  const faces = faceOptions(status.family, current).filter(option => fixedItalic === null || Boolean(option.italic) === fixedItalic).map((o) => [o.value, o.missing ? texts.faceMissing(o, o.nearest) : texts.faceName(o)]);
  const choose = (v) => {
    const next = parseFaceValue(v);
    if (next) onChange({ family: value.family, asset: value.asset ?? null, ...next });
  };
  const face = faceSample && faces.length <= 6
    ? iconChoice(texts.face, faceValue(current), faces, choose, { wide: true })
    : selectField(texts.face, faceValue(current), faces, choose);
  if (faceSample) for (const button of face.querySelectorAll('button')) {
    const index = [...face.querySelectorAll('button')].indexOf(button), spec = parseFaceValue(faces[index][0]);
    button.textContent = faceSample;
    button.style.fontWeight = String(spec.weight);
    button.style.fontStyle = spec.italic ? 'italic' : 'normal';
  }
  return el('div', { class: 'font-picker' }, [
    el('div', { class: 'field' }, [el('label', { for: id, text: texts.family }), button, note]),
    face,
  ]);
}

/** The picker's popover: search, filters, the sections, the sample. */
function openFontList(button, texts, value, onChange, source) {
  let catalog = source.catalog();
  const listId = fieldId();
  const current = { weight: value.weight, italic: Boolean(value.italic) };
  let query = '', filter = 'all', options = [], active = -1, rows = [], drawn = 0;
  // The shared search box, its input a combobox over the list.
  const searchBox = searchField(texts.search, () => {});
  const search = searchBox.querySelector('input');
  for (const [k, v] of Object.entries({ role: 'combobox', 'aria-controls': listId, 'aria-expanded': 'true', 'aria-autocomplete': 'list', autocomplete: 'off' })) search.setAttribute(k, v);
  const chips = el('div', { class: 'chip-group', role: 'group', 'aria-label': texts.filter });
  const list = el('div', { class: 'font-list', id: listId, role: 'listbox', 'aria-label': texts.title });
  const sentinel = el('div', { class: 'font-more', 'aria-hidden': 'true' });
  const sample = el('p', { class: 'font-sample', text: source.sample });
  const feedback = el('p', { class: 'empty-note', role: 'status' });
  const include = el('button', { type: 'button', class: 'text-button', text: texts.include });
  const reload = el('button', { type: 'button', class: 'text-button', text: texts.reload });
  const folder = el('button', { type: 'button', class: 'text-button', text: texts.openFolder });
  const remove = value.asset ? el('button', { type: 'button', class: 'text-button', text: texts.remove }) : null;
  const actions = el('div', { class: 'dialog-actions' }, [folder, reload, include, remove]);
  const panel = el('div', { class: 'font-popover' }, [searchBox, chips, list, sample, actions, feedback]);
  const report = (error) => { feedback.textContent = error ? texts.error : ''; };
  folder.addEventListener('click', async () => { try { await source.openFolder?.(); report(null); } catch (e) { report(e); } });
  reload.addEventListener('click', async () => {
    reload.disabled = true;
    try { await source.reload?.(); catalog = source.catalog(); draw(); report(null); } catch (e) { report(e); }
    finally { reload.disabled = false; }
  });
  include.addEventListener('click', async () => {
    const entry = options[active]?.entry;
    if (!entry || entry.bundled || entry.asset || entry.missing) return;
    const face = faceForFamily(entry, current);
    include.disabled = true;
    close('done');
    try {
      const asset = await source.include?.(entry.name, face);
      if (asset) { source.remember(entry.name); onChange({ family: entry.name, ...face, asset }); }
    } catch (e) { report(e); button.after(feedback); }
    finally { refocus(); }
  });
  remove?.addEventListener('click', () => { close('done'); refocus(); if (source.remove) source.remove(value.asset); else onChange({ ...value, asset: null }); });

  const preview = (entry) => {
    const family = entry?.missing ? catalog?.fallback : entry?.name ?? value.family;
    if (family) sample.style.fontFamily = cssFamily(source.faceFamily?.(family, entry?.asset) ?? family);
    include.disabled = !entry || entry.bundled || entry.asset || entry.missing;
    include.hidden = include.disabled;
    const real = entry && !entry.missing ? entry : findFamily(catalog, value.family);
    const face = real ? nearestFace(real.faces, current) ?? current : current;
    sample.style.fontWeight = String(face.weight);
    sample.style.fontStyle = face.italic ? 'italic' : 'normal';
  };
  const setActive = (i) => {
    options[active]?.classList.remove('is-active');
    active = Math.max(0, Math.min(options.length - 1, i));
    const node = options[active];
    if (!node) return search.removeAttribute('aria-activedescendant');
    node.classList.add('is-active');
    search.setAttribute('aria-activedescendant', node.id);
    node.scrollIntoView?.({ block: 'nearest' });
    preview(node.entry);
  };
  const pick = (entry) => {
    close('done');
    refocus();
    const family = entry.missing ? null : entry;
    source.remember(entry.name);
    onChange({ family: family?.name ?? entry.name, ...(family ? faceForFamily(family, current) : current), asset: entry.asset ?? null });
  };
  const option = (entry) => {
    const i = options.length;
    const name = el('span', { class: 'font-option-name', text: entry.name });
    const family = entry.missing ? catalog?.fallback : entry.name;
    if (family) name.style.fontFamily = cssFamily(source.faceFamily?.(family, entry.asset) ?? family);
    if (!entry.missing && (entry.bundled || entry.origin === 'user' || entry.asset)) source.showFace?.(entry.name, nearestFace(entry.faces, current), entry.asset);
    const node = el('div', {
      class: 'font-option', role: 'option', id: `${listId}-${i}`,
      'aria-selected': String(entry.name.toLowerCase() === value.family.toLowerCase() && (entry.asset ?? null) === (value.asset ?? null)),
    }, [name, entry.missing ? pill(texts.missing, { tone: 'warn' }) : el('span', { class: 'font-option-meta', text: texts.faces(entry.faces.length) })]);
    node.entry = entry;
    node.addEventListener('pointerdown', (evt) => evt.preventDefault());
    node.addEventListener('click', () => pick(entry));
    node.addEventListener('pointerenter', () => setActive(options.indexOf(node)));
    options.push(node);
    return node;
  };
  // Draws the next rows (section heads and options), at most FONT_CHUNK options.
  let group = null;
  const drawMore = () => {
    let added = 0;
    while (drawn < rows.length && added < FONT_CHUNK) {
      const row = rows[drawn++];
      if (row.head) {
        const headId = `${listId}-h${row.head.key}`;
        group = el('div', { class: 'font-section', role: 'group', 'aria-labelledby': headId }, [
          el('div', { class: 'section-head font-section-head', id: headId }, [
            el('span', { class: 'section-title', text: texts.sections[row.head.key] }), count(row.head.count),
          ]),
        ]);
        list.insertBefore(group, sentinel);
      } else {
        group.append(option(row.entry));
        added += 1;
      }
    }
    sentinel.hidden = drawn >= rows.length;
  };
  const draw = () => {
    const sections = fontSections(catalog, { query, filter, theme: source.themeFamilies(), recent: source.recent() });
    rows = sections.flatMap((section) => [{ head: section }, ...section.entries.map((entry) => ({ entry }))]);
    options = [];
    active = -1;
    drawn = 0;
    list.replaceChildren(sentinel);
    if (!catalog) list.prepend(el('p', { class: 'empty-note', text: texts.loading }));
    else if (!rows.length) list.prepend(el('p', { class: 'empty-note', role: 'status', text: texts.none }));
    drawMore();
    const chosen = options.findIndex((o) => o.getAttribute('aria-selected') === 'true');
    setActive(query && options.length ? 0 : Math.max(0, chosen));
  };
  const drawFilters = () => chips.replaceChildren(...FONT_FILTERS.map((f) => chipButton(texts.filters[f], f === filter, () => {
    filter = f;
    drawFilters();
    draw();
  })));
  const more = typeof IntersectionObserver === 'function'
    ? new IntersectionObserver((seen) => { if (seen.some((e) => e.isIntersecting)) drawMore(); }, { root: list })
    : null;
  more?.observe(sentinel);
  search.addEventListener('input', () => {
    query = search.value;
    draw();
  });
  search.addEventListener('keydown', (evt) => {
    const step = { ArrowDown: 1, ArrowUp: -1, PageDown: 8, PageUp: -8 }[evt.key];
    if (step) {
      evt.preventDefault();
      if (active + step >= options.length && drawn < rows.length) drawMore();
      setActive(active + step);
    } else if (evt.key === 'Enter') {
      evt.preventDefault();
      if (options[active]) pick(options[active].entry);
    }
  });
  const refocus = refocusFor(button);
  const close = openPopover({ node: panel, anchor: button, label: texts.title, refocus, onClose: () => more?.disconnect() });
  drawFilters();
  draw();
  search.focus();
}

/** The colour of a swatch chip, for its CSS (`--swatch`). */
function swatchChip(hex, extra = '') {
  const chip = el('span', { class: `swatch ${extra}`.trim(), 'aria-hidden': 'true' });
  chip.style.setProperty('--swatch', normalizeHex(hex) ?? '#ffffffff');
  return chip;
}

/**
 * A colour: a button with its swatch, hex and opacity, which opens the
 * picker: a saturation/value area, hue and opacity bars, hex and %, an
 * eyedropper where the system has one, and the theme's swatches, the
 * colours the theme uses and the recent ones. Every change is handed to
 * `onChange` (`#rrggbbaa`); a drag in the area or on a bar is one gesture
 * (`gesture.begin()` … `gesture.end()`), a key press one change.
 * @param {string} label the colour's name (button and dialog)
 * @param {string} hex `#rrggbbaa`
 * @param {(hex: string) => void} onChange
 * @param {{texts: object, source: object, alpha?: boolean, gesture?: {begin: () => void, end: () => void}|null}} opts
 *   `texts` translated (see colorTexts in ui/pickers.js); `source`:
 *   `swatches()`, `used()`, `recent()`, `remember(hex)` and `palette`
 *   (`add(hex)`, `rename(i, name)`, `move(i, to)`, `remove(i)`) or null
 *   where nothing is saved; `alpha` false for an opaque colour; `live`
 *   false hands a drag's colour over only when it ends (a change that is costly)
 */
export function colorPicker(label, hex, onChange, { texts, source, alpha = true, gesture = null, live = true }) {
  const value = normalizeHex(hex) ?? '#ffffffff';
  const id = fieldId();
  const readout = el('span', { class: 'color-button-value', id: `${id}-v` }, [
    el('span', { class: 'mono', text: rgbText(value) }),
    alpha ? el('span', { class: 'mono color-button-alpha', text: `${alphaPercent(value)}%` }) : null,
  ]);
  const button = el('button', {
    type: 'button', class: 'color-button', id, 'aria-haspopup': 'dialog', 'aria-expanded': 'false',
    'aria-label': label, 'aria-describedby': `${id}-v`, dataset: { picker: `color:${label}`, color: value },
  }, [swatchChip(value), readout]);
  button.addEventListener('click', () => {
    if (button.getAttribute('aria-expanded') === 'true') return;
    openColorPanel(button, label, value, onChange, { texts, source, alpha, gesture, live });
  });
  return el('div', { class: 'field color-field' }, [el('label', { for: id, text: label }), button]);
}

/** A slider of the colour panel (area or bar) with its keyboard. */
function colorSlider(cls, label, keys) {
  const thumb = el('span', { class: 'cp-thumb', 'aria-hidden': 'true' });
  const node = el('div', { class: cls, role: 'slider', tabindex: '0', 'aria-label': label }, [thumb]);
  node.addEventListener('keydown', (evt) => {
    const move = keys[evt.key];
    if (!move) return;
    evt.preventDefault();
    move(evt.shiftKey ? 10 : 1);
  });
  return { node, thumb };
}

/** The colour panel of `colorPicker`. */
function openColorPanel(button, label, start, onChange, { texts, source, alpha, gesture, live }) {
  let color = start;
  let hsva = hsvaOf(color);
  let editing = false;
  let dragging = false;
  let frame = 0;

  // `keep`: the panel's own hue/saturation/value made `next` (exact, not re-read from the hex).
  const commit = (next, { live = false, keep = false } = {}) => {
    // The same colour again (Enter, then the field's change) is no change.
    if (next === color && !live) {
      if (keep) show();
      return;
    }
    color = next;
    if (!keep) hsva = hsvaOf(color, hsva.h);
    show();
    if (!live) {
      onChange(color);
      source.remember(color);
      drawSwatches();
    } else if (live && !frame) {
      frame = requestAnimationFrame(() => {
        frame = 0;
        onChange(color);
      });
    }
  };
  const setHsva = (next, { live = false } = {}) => {
    hsva = next;
    commit(hexOfHsva(hsva), { live, keep: true });
  };

  const area = colorSlider('cp-area', texts.area, {
    ArrowLeft: (n) => setHsva(stepHsva(hsva, 's', -n)), ArrowRight: (n) => setHsva(stepHsva(hsva, 's', n)),
    ArrowDown: (n) => setHsva(stepHsva(hsva, 'v', -n)), ArrowUp: (n) => setHsva(stepHsva(hsva, 'v', n)),
  });
  const hue = colorSlider('cp-hue', texts.hue, {
    ArrowLeft: (n) => setHsva(stepHsva(hsva, 'h', -n)), ArrowDown: (n) => setHsva(stepHsva(hsva, 'h', -n)),
    ArrowRight: (n) => setHsva(stepHsva(hsva, 'h', n)), ArrowUp: (n) => setHsva(stepHsva(hsva, 'h', n)),
  });
  hue.node.setAttribute('aria-valuemin', '0');
  hue.node.setAttribute('aria-valuemax', '360');
  const opacity = alpha ? colorSlider('cp-alpha', texts.opacity, {
    ArrowLeft: (n) => setHsva(stepHsva(hsva, 'a', -n)), ArrowDown: (n) => setHsva(stepHsva(hsva, 'a', -n)),
    ArrowRight: (n) => setHsva(stepHsva(hsva, 'a', n)), ArrowUp: (n) => setHsva(stepHsva(hsva, 'a', n)),
  }) : null;
  opacity?.node.setAttribute('aria-valuemin', '0');
  opacity?.node.setAttribute('aria-valuemax', '100');

  // A drag on the area or a bar: the colour follows the pointer, one gesture.
  const drag = (node, at) => {
    node.addEventListener('pointerdown', (evt) => {
      if (evt.button !== 0) return;
      evt.preventDefault();
      node.focus();
      node.setPointerCapture?.(evt.pointerId);
      dragging = true;
      gesture?.begin();
      const move = (e) => {
        const box = node.getBoundingClientRect();
        const x = Math.min(1, Math.max(0, (e.clientX - box.left) / box.width));
        const y = Math.min(1, Math.max(0, (e.clientY - box.top) / box.height));
        setHsva(at(x, y), { live: true });
      };
      const up = () => {
        node.removeEventListener('pointermove', move);
        node.removeEventListener('pointerup', up);
        node.removeEventListener('pointercancel', up);
        if (frame) cancelAnimationFrame(frame);
        frame = 0;
        dragging = false;
        onChange(color);
        gesture?.end();
        source.remember(color);
        drawSwatches();
      };
      node.addEventListener('pointermove', move);
      node.addEventListener('pointerup', up);
      node.addEventListener('pointercancel', up);
      move(evt);
    });
  };
  drag(area.node, (x, y) => ({ ...hsva, s: x, v: 1 - y }));
  drag(hue.node, (x) => ({ ...hsva, h: x * 360 }));
  if (opacity) drag(opacity.node, (x) => ({ ...hsva, a: x }));

  const hexInput = el('input', { type: 'text', class: 'cp-hex mono', 'aria-label': texts.hex, spellcheck: false, autocomplete: 'off', maxlength: 9 });
  const takeHex = () => {
    const next = readHexInput(hexInput.value, colorOf(color).a);
    if (next && !alpha) commit(withAlphaPercent(next, 100));
    else if (next) commit(next);
    else hexInput.value = rgbText(color);
  };
  hexInput.addEventListener('change', takeHex);
  hexInput.addEventListener('keydown', (evt) => { if (evt.key === 'Enter') { evt.preventDefault(); takeHex(); } });
  const percent = alpha ? el('input', { type: 'number', class: 'cp-percent', min: 0, max: 100, step: 1, 'aria-label': texts.opacityPercent }) : null;
  percent?.addEventListener('change', () => {
    const v = Number(percent.value);
    if (Number.isFinite(v)) commit(withAlphaPercent(color, v));
  });
  const pickScreenColor = source.pickScreenColor ?? (typeof window.EyeDropper === 'function'
    ? async () => (await new window.EyeDropper().open())?.sRGBHex
    : null);
  const pickStatus = el('p', { class: 'hint cp-pick-status', role: 'status', hidden: true });
  const eyedropper = pickScreenColor
    ? iconButton(texts.eyedropper, ICONS.eyedropper, async () => {
      eyedropper.disabled = true;
      pickStatus.hidden = false;
      pickStatus.textContent = texts.eyedropperHint;
      try {
        const picked = await pickScreenColor();
        const next = readHexInput(picked, alpha ? colorOf(color).a : 255);
        // A native pick may complete after this popover was dismissed.
        if (next && panel.isConnected) commit(next);
        pickStatus.hidden = true;
      } catch (error) {
        // Escape is a cancellation; unsupported/refused picks remain visible.
        pickStatus.hidden = error?.name === 'AbortError';
        pickStatus.textContent = texts.eyedropperError;
      } finally {
        eyedropper.disabled = false;
      }
    }, { class: 'cp-eyedropper' })
    : null;
  const preview = swatchChip(color, 'cp-preview');
  const inputs = el('div', { class: 'cp-inputs' }, [preview, hexInput, percent ? el('span', { class: 'fld cp-percent-field' }, [percent, el('span', { class: 'fld-unit', 'aria-hidden': 'true', text: '%' })]) : null, eyedropper]);

  const sections = el('div', { class: 'cp-sections' });
  // The control of the swatch sections that last had focus (`data-focus`).
  let lastFocus = null;
  sections.addEventListener('focusin', (evt) => { lastFocus = evt.target.closest('[data-focus]')?.dataset.focus ?? lastFocus; });
  const panel = el('div', { class: 'color-popover' }, [
    area.node, hue.node, opacity?.node ?? null, inputs, pickStatus, sections,
  ]);

  function show() {
    const pure = hexOfHsva({ h: hsva.h, s: 1, v: 1, a: 1 });
    panel.style.setProperty('--cp-hue', pure);
    panel.style.setProperty('--cp-color', color);
    panel.style.setProperty('--cp-opaque', withAlphaPercent(color, 100));
    area.thumb.style.left = `${hsva.s * 100}%`;
    area.thumb.style.top = `${(1 - hsva.v) * 100}%`;
    area.node.setAttribute('aria-valuetext', texts.areaValue(Math.round(hsva.s * 100), Math.round(hsva.v * 100)));
    hue.thumb.style.left = `${(hsva.h / 360) * 100}%`;
    hue.node.setAttribute('aria-valuenow', String(Math.round(hsva.h)));
    if (opacity) {
      opacity.thumb.style.left = `${hsva.a * 100}%`;
      opacity.node.setAttribute('aria-valuenow', String(Math.round(hsva.a * 100)));
      opacity.node.setAttribute('aria-valuetext', `${Math.round(hsva.a * 100)}%`);
    }
    preview.style.setProperty('--swatch', color);
    if (document.activeElement !== hexInput) hexInput.value = rgbText(color);
    if (percent && document.activeElement !== percent) percent.value = String(alphaPercent(color));
  }

  const swatchButton = (hexValue, name, focusKey) => el('button', {
    type: 'button', class: 'swatch-button', 'aria-label': name ? `${name} (${rgbText(hexValue)})` : texts.swatch(rgbText(hexValue), alphaPercent(hexValue)),
    title: name ?? rgbText(hexValue), 'aria-pressed': String(hexValue === color), onclick: () => commit(alpha ? hexValue : withAlphaPercent(hexValue, 100)), dataset: { focus: focusKey },
  }, [swatchChip(hexValue)]);
  const group = (key, title, items, tools = []) => {
    const head = sectionTitle(title, { count: items.length });
    head.append(...tools);
    return el('section', { class: 'cp-section', dataset: { section: key }, 'aria-label': title }, [head, items.length ? el('div', { class: 'swatch-grid' }, items) : null]);
  };
  function drawSwatches() {
    const palette = source.swatches();
    const parts = [];
    if (source.palette) {
      const save = el('button', { type: 'button', class: 'text-button quiet', dataset: { focus: 'save' }, onclick: () => { source.palette.add(color); drawSwatches(); } }, [icon(ICONS.plus, 14), texts.save]);
      const edit = palette.length ? el('button', { type: 'button', class: 'text-button quiet', dataset: { focus: 'edit' }, 'aria-pressed': String(editing), text: editing ? texts.done : texts.edit, onclick: () => { editing = !editing; drawSwatches(); } }) : null;
      if (!palette.length) editing = false;
      const items = editing ? [] : palette.map((s, i) => swatchButton(s.color, s.name ?? null, `theme:${i}`));
      const section = group('theme', texts.themeColors, items, [save, edit].filter(Boolean));
      if (editing) section.append(el('ul', { class: 'swatch-rows' }, palette.map((s, i) => swatchRow(s, i, palette.length))));
      else if (!palette.length) section.append(el('p', { class: 'hint', text: texts.themeEmpty }));
      parts.push(section);
    }
    const used = source.used();
    if (used.length) parts.push(group('used', texts.used, used.map((c, i) => swatchButton(c, null, `used:${i}`))));
    const recent = source.recent();
    if (recent.length) parts.push(group('recent', texts.recent, recent.map((c, i) => swatchButton(c, null, `recent:${i}`))));
    // Focus stays on the same control drawn again (or its section's first one), never lost to the page.
    const focused = sections.contains(document.activeElement) || document.activeElement === document.body ? lastFocus : null;
    sections.replaceChildren(...parts);
    if (focused) (sections.querySelector(`[data-focus="${CSS.escape(focused)}"]:not(:disabled)`) ?? sections.querySelector('[data-focus="edit"], [data-focus="save"]') ?? area.node).focus();
  }
  // A swatch being edited: its name, its place and its removal, each one change of the theme.
  function swatchRow(s, i, n) {
    const name = el('input', { type: 'text', class: 'swatch-name', value: s.name ?? '', placeholder: rgbText(s.color), 'aria-label': texts.swatchName(i + 1), maxlength: 40, spellcheck: false, dataset: { focus: `name:${i}` } });
    name.addEventListener('change', () => { source.palette.rename(i, name.value); drawSwatches(); });
    return el('li', { class: 'swatch-row' }, [
      swatchChip(s.color), name,
      iconButton(texts.moveUp(i + 1), ICONS.up, () => { lastFocus = `up:${i - 1}`; source.palette.move(i, i - 1); drawSwatches(); }, { size: 16, disabled: i === 0, 'data-focus': `up:${i}` }),
      iconButton(texts.moveDown(i + 1), ICONS.down, () => { lastFocus = `down:${i + 1}`; source.palette.move(i, i + 1); drawSwatches(); }, { size: 16, disabled: i === n - 1, 'data-focus': `down:${i}` }),
      iconButton(texts.remove(i + 1), ICONS.trash, () => { source.palette.remove(i); drawSwatches(); }, { size: 16, class: 'danger', 'data-focus': `remove:${i}` }),
    ]);
  }

  const refocus = refocusFor(button);
  openPopover({
    node: panel, anchor: button, label: texts.title(label), refocus, onClose: () => {
      if (dragging) gesture?.end();
    },
  });
  show();
  drawSwatches();
  area.node.focus();
}
