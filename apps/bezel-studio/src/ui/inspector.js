import { resizedFrame } from '../inspector-model.js';
import { createSectionMemory } from '../inspector-sections.js';
import { describeSections } from './inspector/sections.js';
import { objectMenuItems } from './object-menu.js';
import { openMenuAt } from './menu.js';
import { createCommonFields } from './inspector/common.js';
import { sections as multiSections, header as multiHeader } from './inspector/multi.js';
import { panel as themePanel, header as themeHeader, orientationField as themeOrientation, sections as themeSections } from './inspector/theme.js';
import { sections as clockPanels } from './inspector/clock.js';
import { sections as weatherPanels } from './inspector/weather.js';
import { sections as valuePanels } from './inspector/value.js';
import { sections as ringPanels } from './inspector/ring.js';
import { sections as barPanels } from './inspector/bar.js';
import { sections as needlePanels } from './inspector/needle.js';
import { sections as graphPanels } from './inspector/graph.js';
import { sections as spectrumPanels } from './inspector/spectrum.js';
import { sections as playerPanels } from './inspector/player.js';
import { sections as shapePanels } from './inspector/shape.js';
import { sections as imagePanels } from './inspector/image.js';
import { canRotate, appliedRotation } from '../editor/rotation.js';
import { createInspectorTrial } from '../inspector-trial.js';

import { closeSensorPicker, refreshSensorPicker } from './sensor-picker.js';

import { canFitContent } from '../editor/fit-content.js';
// The inspector shell owns selection, shared header, type and disclosure state.
// Widget modules supply the existing controls through the sections(ctx) contract.
// Every edit is still one store command and one Undo step.

import { el, icon, svgEl as node } from './dom.js';
import { ICONS } from './icons.js';
import { accordionSection, sizeField, objectHeader, typeRow, previewChoice as optionTiles, colorPicker, fieldGrid, adoptControl, iconButton, bindButton, numberField, prefixedField, rangeField, propertySection as section, segmented, selectField } from './fields.js';
import { withChildren } from '../editor/cards.js';
import { splitColor } from '../color-math.js';

import { boundKey, createWidget, widgetOf } from '../editor/widgets.js';

import { videoStatus } from '../editor/background.js';
import { ROTATIONS, ZOOM_MAX, ZOOM_MIN, ZOOM_STEP, framingOf, framingPercents, reframed, resolvedRotation, withoutFraming } from '../editor/video-framing.js';
import { formatBytes } from './storage.js';
import { arcDrawing, layerFacts } from '../inspector-tabs.js';
import { createPeakWatch, createPreviewScheduler, fitPreview, previewOffset, previewSide } from '../element-preview.js';
import { createPreviewAnimation } from '../preview-animation.js';
import { sections as cardSections } from './inspector/card.js';
import { createPickers } from './pickers.js';
import { createBackgroundPanel } from './inspector-background.js';

const FORMS = Object.freeze({ card: cardSections, clock: clockPanels, weather: weatherPanels, value: valuePanels, ring: ringPanels, bar: barPanels, needle: needlePanels, graph: graphPanels, spectrum: spectrumPanels, player: playerPanels, shape: shapePanels, image: imagePanels, text: valuePanels });
const BOUND = ['value', 'bar', 'ring', 'needle', 'graph'];
/** The tallest the preview is drawn, CSS pixels. */
const PREVIEW_HEIGHT = 72;
/** The edges an object aligns to on the screen, and their icons. */
const ALIGN_EDGES = Object.freeze([['left', 'alignLeft'], ['centerX', 'alignCenterX'], ['right', 'alignRight'], ['top', 'alignTop'], ['centerY', 'alignCenterY'], ['bottom', 'alignBottom']]);
// The same reading order for every object's Look tab: structure/type,
// typography, paint, edges, then effects. A kind omits what it does not use.
const LOOK_ORDER = Object.freeze([
  'show', 'shape', 'visualization', 'style', 'spectrum-bars', 'arc', 'scale-marks', 'needle-shape', 'spacing', 'appearance', 'font',
  'color', 'picture', 'spectrum-paint', 'fill', 'progress', 'track', 'segments',
  'outline', 'corners', 'shadow', 'transparency', 'preview', 'visibility',
]);
/** The video-background actions and what they depend on, when none are given. */
const NO_VIDEO_ACTIONS = Object.freeze({
  context: () => ({ screen: null, live: false, liveVideo: null, tools: null, locale: 'en', auto: null, framing: false, motion: true }),
  openStorage: () => {},
  pickFile: async () => null,
  importFile: async () => null,
  restoreStored: () => {},
  sendToScreen: () => {},
  setFraming: () => {},
  openGuide: () => {},
});

/** What the inspector asks the app, when nothing is given. */
const NO_HOST = Object.freeze({
  fitContent: () => {},
  lastSample: () => null,
  renderElement: async () => null,
});

/** Buttons of one choice (`aria-pressed`), each with a stable id so focus stays on it. */
function choice(id, label, value, options, onChange) {
  const field = segmented(label, value, options, (v) => v !== value && onChange(v));
  const group = field.querySelector('.segmented');
  group.classList.add('framing-choice', id);
  Array.from(group.children).forEach((button, i) => { button.id = `${id}-${options[i][0]}`; });
  return field;
}

/**
 * Tiles with a drawing over their name (artboards GraphLook "Stile",
 * PlayerLook "Cosa mostrare"), each pressed or not.
 * @param {string} label the group's accessible name
 * @param {{text: string, art: string[], pressed: boolean, onClick: () => void}[]} tiles
 */
/** The ring's arc in small (artboard Main, "Arco"): the track, the arc and where it starts. */
function arcPreview(kind) {
  const arc = arcDrawing(kind.startAngle, kind.sweep, kind.clockwise !== false);
  const svg = node('svg', { class: 'arc-preview', viewBox: '0 0 64 64', width: 64, height: 64, 'aria-hidden': 'true' });
  svg.append(
    node('circle', { class: 'arc-track', cx: 32, cy: 32, r: 24 }),
    node('path', { class: 'arc-fill', d: arc.d }),
    node('circle', { class: 'arc-handle', cx: arc.start.x, cy: arc.start.y, r: 4 }),
  );
  return svg;
}

/**
 * @param {object} deps
 * @param {HTMLElement} deps.root
 * @param {() => number} deps.minRefresh the fastest refresh a theme may ask for, seconds (from the backend)
 * @param {ReturnType<import('./pickers.js').createPickers>|null} [deps.pickers] the font and colour pickers' data
 *   (D-2026-10-16-studio-redesign-fidelity-28, -29), shared with the library
 * @param {{context: () => {screen: object|null, live: boolean, liveVideo: object|null, tools: object|null, locale: string, auto: {rotation: number, size: object|null}|null, framing: boolean, motion: boolean}, openStorage: () => void, pickFile: () => Promise<object|null>, importFile: (source: string) => Promise<object|null>, restoreStored: (path: string, medium: string) => void, sendToScreen: (medium: string) => void, setFraming: (on: boolean) => void, openGuide: () => void}} [deps.video]
 *   the screen and ffmpeg the video background is told against, what Auto
 *   turns it (`video_auto`), whether the canvas frames it now and whether
 *   the preview may move, and its actions: show the storage tab (where the
 *   missing video is sent), pick a picture, GIF or video and add it to the
 *   chooser as a draft, and import it only on confirmation, send a stored
 *   video's local copy again or another video to the screen (both in the
 *   storage tab, with its confirmation), frame it on the canvas, and open
 *   the ffmpeg guide
 * @param {object|null} [deps.bridge] the backend, for what the inspector reads itself
 * @param {{lastSample: () => object|null, renderElement: (id: string, opts?: {max?: number, motion?: boolean}) => Promise<object|null>}} [deps.host]
 *   the last sample read (sensor values, apps, media) and an element of the
 *   edited theme rendered alone
 */
export function createInspector({ root, store, t, sensors, minRefresh, pickers: shared = null, editIcon = () => {}, searchCities = async () => [], video = NO_VIDEO_ACTIONS, bridge = null, host = NO_HOST }) {
  const update = (id, patch) => store.dispatch('update', { id, patch });
  // Spectrum bars' corner mode chosen in this session, by element id (true = each corner).
  const barCornerModes = new Map();
  const sizeLinks = new Map();
  const rotationBindings = new Map();
  const sourceState = { custom: false };
  // The app's pickers (ui/pickers.js); an inspector on its own makes its own, without fonts until asked.
  const pickers = shared ?? createPickers({ store, t, bridge: bridge ?? { fontCatalog: async () => null, fontFace: async () => null } });
  // A framing slider being dragged: its moves are one gesture (one undo
  // step), and the form is not drawn again under the pointer meanwhile.
  let sliding = false;
  // A drag in a colour picker is one gesture, one undo step, as a framing slider's.
  const gesture = {
    begin() {
      if (sliding) return;
      sliding = true;
      store.beginGesture();
    },
    end: () => endSlide(),
  };
  /** A colour of the theme: the shared picker (D-2026-10-16-studio-redesign-fidelity-29). */
  const colorField = (label, hex, onChange, opts = {}) => colorPicker(label, hex, onChange, { ...pickers.color(), gesture, ...opts });

  /** The last reading of `key`, from the app's last sample (null: none yet). */
  const readingOf = (key) => host.lastSample()?.readings?.[key] ?? null;
  const shownValue = (key) => readingOf(key)?.display ?? '…';

  // What the form was last drawn with: a video background is drawn again
  // when the screen, live mode, its video or ffmpeg change.
  let shown = { assets: [], signature: '' };
  let themePreview = null;
  const common = createCommonFields({ t, store, update, root, sensors, host, pickers, colorField, video, barCornerModes, sourceState, shownValue, optionTiles, redraw: () => render(shown.assets) });
  const { sourceCard, appChipGroup, durationText, valueColorSection, sensorMotionSection, visualizationFields, appSignature, manualSource, timingOf, button } = common;

  // --------------------------------------------------------------- theme --

  // ------------------------------------------------------------ framing --
  /** Applies a framing patch to the video background: one store command, none when it changes nothing. */
  function frame(patch) {
    const framed = reframed(store.getState().theme.background, patch);
    if (framed) store.dispatch('setTheme', { patch: { background: framed } });
  }

  function endSlide() {
    if (!sliding) return;
    sliding = false;
    store.endGesture();
  }

  /**
   * A framing slider with its readout: dragging it frames the preview live
   * and is one undo step; each arrow key on it is one too.
   */
  function framingRange({ id, label, value, min, max, step, format, onInput }) {
    const out = el('output', { for: id, text: format(value) });
    const input = el('input', { id, type: 'range', min, max, step, value: String(value) });
    input.addEventListener('input', () => {
      const v = Number(input.value);
      out.textContent = format(v);
      if (!sliding) {
        sliding = true;
        store.beginGesture();
      }
      onInput(v);
    });
    for (const done of ['change', 'pointerup', 'keyup', 'blur']) input.addEventListener(done, endSlide);
    return el('label', { class: 'field', for: id }, [el('span', {}, [label, ' ', out]), input]);
  }

  /** The pad color: the shared colour picker, opaque. */
  const padField = (color) => colorField(t('framing.padColor'), color, (hex) => frame({ padColor: splitColor(hex).rgb }), { alpha: false });

  /** The zoom: a slider and a number, 100–400 %, step 5 %. */
  function zoomField(zoom) {
    const percent = Math.round(zoom * 100);
    const number = el('input', { type: 'number', id: 'framing-zoom-number', min: ZOOM_MIN * 100, max: ZOOM_MAX * 100, step: ZOOM_STEP * 100, value: String(percent), 'aria-label': t('framing.zoomNumber') });
    number.addEventListener('change', () => {
      const v = Number(number.value);
      if (Number.isFinite(v)) frame({ zoom: v / 100 });
    });
    const range = framingRange({
      id: 'framing-zoom', label: t('framing.zoom'), value: percent, min: ZOOM_MIN * 100, max: ZOOM_MAX * 100, step: ZOOM_STEP * 100,
      format: (v) => t('framing.percent', { value: v }),
      onInput: (v) => {
        number.value = String(v);
        frame({ zoom: v / 100 });
      },
    });
    return el('div', { class: 'framing-zoom' }, [range, number]);
  }

  /**
   * The Framing group of a video background (D-2026-10-01-video-background-framing-6):
   * "Frame on canvas", Rotation (Auto names the angle it detected), Fill or
   * Fit, Zoom, Position X and Y with Center, the pad color (Fit only) and
   * Reset; why the preview shows the poster, when it does.
   */
  function framingGroup(bg, context) {
    const f = framingOf(bg);
    const percents = framingPercents(f);
    const auto = context.auto;
    const autoLabel = auto ? t('framing.auto', { degrees: auto.rotation }) : t('framing.autoUnknown');
    const rotation = resolvedRotation(f, auto);
    const nodes = [
      el('button', {
        type: 'button', id: 'framing-canvas', class: 'text-button framing-toggle', 'aria-pressed': String(Boolean(context.framing)), onclick: () => video.setFraming(!context.framing),
      }, [icon(ICONS.frame, 16), t('framing.onCanvas')]),
      el('p', { class: 'hint', text: t('framing.onCanvasHint') }),
      choice('framing-rotation', t('framing.rotation'), f.rotation === null ? 'auto' : String(f.rotation), [
        ['auto', autoLabel],
        ...ROTATIONS.map((r) => [String(r), t('framing.degrees', { degrees: r })]),
      ], (v) => frame({ rotation: v === 'auto' ? null : Number(v) })),
      f.rotation === null && rotation !== 0 ? el('p', { class: 'hint', text: t('framing.autoTurned', { degrees: rotation }) }) : null,
      choice('framing-fit', t('framing.fit'), f.fit, [['cover', t('framing.cover')], ['contain', t('framing.contain')]], (v) => frame({ fit: v })),
      el('p', { class: 'hint', text: t('framing.fitHint') }),
      zoomField(f.zoom),
      framingRange({ id: 'framing-x', label: t('framing.positionX'), value: percents.x, min: 0, max: 100, step: 1, format: (v) => t('framing.percent', { value: v }), onInput: (v) => frame({ position: { x: v / 100 } }) }),
      framingRange({ id: 'framing-y', label: t('framing.positionY'), value: percents.y, min: 0, max: 100, step: 1, format: (v) => t('framing.percent', { value: v }), onInput: (v) => frame({ position: { y: v / 100 } }) }),
      f.fit === 'contain' ? padField(f.padColor) : null,
      el('div', { class: 'button-row' }, [
        el('button', { type: 'button', id: 'framing-center', class: 'text-button', text: t('framing.center'), disabled: f.position.x === 0.5 && f.position.y === 0.5, onclick: () => frame({ position: { x: 0.5, y: 0.5 } }) }),
        el('button', { type: 'button', id: 'framing-reset', class: 'text-button', text: t('framing.reset'), disabled: !bg.framing, onclick: () => store.dispatch('setTheme', { patch: { background: withoutFraming(bg) } }) }),
      ]),
    ];
    if (context.tools && !context.tools.ready) {
      nodes.push(
        el('p', { class: 'hint framing-note', text: t('framing.noFfmpeg') }),
        el('div', { class: 'button-row' }, [el('button', { type: 'button', id: 'framing-guide', class: 'text-button', text: t('framing.guide'), onclick: () => video.openGuide() })]),
      );
    } else if (context.motion === false) {
      nodes.push(el('p', { class: 'hint framing-note', text: t('framing.reducedMotion') }));
    }
    return el('div', { class: 'framing', role: 'group', 'aria-labelledby': 'framing-title' }, [
      el('h3', { id: 'framing-title', text: t('framing.title') }),
      ...nodes.filter(Boolean),
    ]);
  }

  /**
   * The options of a video from the computer (the card shows its poster,
   * name and where it is): what the connected screen does with it (the
   * storage tab sends a missing one), its framing, and what ffmpeg is for
   * when there is no poster.
   */
  function videoOptions(bg) {
    const context = video.context();
    const status = videoStatus(context);
    const nodes = [el('p', { class: 'hint video-status', dataset: { status }, text: t(`inspector.video.${status}`) })];
    if (status === 'missing') nodes.push(el('div', { class: 'button-row' }, [button(t('inspector.video.openStorage'), () => video.openStorage())]));
    if (!bg.poster && context.tools && !context.tools.ready) {
      nodes.push(el('p', { class: 'hint', text: t('inspector.video.noPoster') }));
      nodes.push(el('ul', { class: 'hints' }, (context.tools.installHints ?? []).map((h) => el('li', {}, [el('code', { text: h })]))));
    } else if (!bg.poster) {
      nodes.push(el('p', { class: 'hint', text: t('inspector.video.noPosterReady') }));
    }
    nodes.push(framingGroup(bg, context));
    return nodes;
  }

  // ---------------------------------------------------------- background --
  // One card for the background in use, its kind's options, and the chooser
  // that changes it (D-2026-10-16-studio-redesign-fidelity-25).
  const background = createBackgroundPanel({
    t, store, bridge, video, colorField, videoOptions, bytes: formatBytes, redraw: () => render(shown.assets),
  });

  // ---------------------------------------------------------- per kind ----
  // Each kind gives the sections of its Data tab and of its Look tab.

  function showAs(e, theme) {
    const current = widgetOf(e);
    if (e.isGroup || !BOUND.includes(current)) return null;
    const key = boundKey(e);
    const quantity = sensors.catalog().find((s) => s.key === key)?.quantity;
    return typeRow(t('insp.type'), current, BOUND.map((w) => [w, t(`widget.${w}`), ({value:'type-valore',bar:'type-barra',ring:'type-anello',needle:'type-lancetta',graph:'type-grafico'})[w]]), (w) => {
      const made = createWidget(w, theme.canvas, { key, quantity });
      if (made.kind.binding && e.kind.binding) made.kind.binding = { ...made.kind.binding, min: e.kind.binding.min, max: e.kind.binding.max };
      store.dispatch('setKind', { id: e.id, kind: made.kind });
    });
  }

  function membershipForm(e, theme) {
    if (!e.isGroup) {
      const descendants = new Set(withChildren(theme, [e.id]));
      const parents = theme.elements.filter(parent => (parent.isGroup || parent.card) && !descendants.has(parent.id)
        && (!e.card || !parent.card && !parent.cardMember));
      const current = e.groupParent ?? e.cardMember?.parent;
      const card = parents.find(parent => parent.id === current && parent.card);
      return [selectField(t('inspector.membership'), current == null ? '' : String(current), [['', t('layers.rootDrop')], ...parents.map(parent => [String(parent.id), parent.name])], value => {
        const parent = parents.find(parent => String(parent.id) === value);
        store.dispatch('placeLayer', { ids: [e.id], ...(parent ? { target: parent.id, inside: true } : { root: true }) });
      }), card && selectField(t('card.face'), e.cardMember.face == null ? '' : String(e.cardMember.face), [['', t('cardInsp.base')], ...card.card.faces.map((name, index) => [String(index), name])], value => store.dispatch('attachCard', { ids: [e.id], parent: card.id, face: value === '' ? null : Number(value) }))];
    }
    const facts = layerFacts(theme, e);
    const summary = el('p', { class: 'hint membership-location', text: facts.group
      ? t('insp.inGroup', { group: facts.group }) : t('layers.rootDrop') });
    if (e.card) return [summary];
    const cards = theme.elements.filter(p => p.card);
    if (!cards.length) return [summary];
    const parent = cards.find(p => p.id === e.cardMember?.parent);
    return [
      e.groupParent != null && summary,
      selectField(t('widget.card'), parent ? String(parent.id) : '', [['', t('card.none')], ...cards.map(p => [String(p.id), p.name])], value => {
        const target = cards.find(p => String(p.id) === value);
        store.dispatch('attachCard', { ids: [e.id], parent: target?.id, face: target?.card.activeFace ?? null });
      }),
      parent && selectField(t('card.face'), e.cardMember.face === null ? '' : String(e.cardMember.face), [['', t('cardInsp.base')], ...parent.card.faces.map((name, i) => [String(i), name])], value => store.dispatch('attachCard', { ids: [e.id], parent: parent.id, face: value === '' ? null : Number(value) })),
    ].filter(Boolean);
  }

  const sectionMemory = createSectionMemory();
  let shownElement = null;
  function sectioned(kind, descriptors, element) {
    const controls = descriptors.map(descriptor => accordionSection({
      ...descriptor, summary: descriptor.summary(element), expanded: sectionMemory.isOpen(kind, descriptor.id),
      onToggle: options => {
        sectionMemory.toggle(kind, descriptor.id, options);
        controls.forEach((control, i) => control.setExpanded(sectionMemory.isOpen(kind, descriptors[i].id)));
      },
    }, { element }));
    return controls.map(control => control.element);
  }

  // ------------------------------------------------------------ preview --
  // The object rendered alone by the backend (`renderElement`, D-8): asked
  // once per change or new frame, one render at a time, the last wins.
  const previewBox = (figure) => {
    const stage = figure.querySelector('.inspector-preview-stage');
    return { width: stage.clientWidth > 16 ? stage.clientWidth : PREVIEW_HEIGHT, height: PREVIEW_HEIGHT };
  };
  const trial = createInspectorTrial();
  window.matchMedia?.('(prefers-reduced-motion: reduce)').addEventListener('change', () => {
    trial.stop();
    render(shown.assets);
  });
  const previews = createPreviewScheduler({
    render: async (id) => {
      const figure = root.querySelector('.inspector-preview');
      const ratio = window.devicePixelRatio || 1;
      const max = previewSide(figure ? previewBox(figure) : { width: PREVIEW_HEIGHT, height: PREVIEW_HEIGHT }, ratio);
      const started = performance.now();
      const motion = video.context().motion !== false;
      const element = store.getState().theme.elements.find(e => e.id === id);
      const trialMs = trial.at(element, motion);
      const frame = await host.renderElement(id, { max, motion, ...(trialMs === null ? {} : { trialMs }) });
      return { frame, elapsed: performance.now() - started };
    },
    show: ({ frame, elapsed }, id) => {
      drawPreview(frame, id);
      root.querySelector('.inspector-try')?.setAttribute('aria-pressed', String(trial.at(store.getState().theme.elements.find(e => e.id === id), video.context().motion !== false) !== null));
      previewAnimation.shown({ nextMs: frame?.nextMs, elapsed });
    },
    fail: (_, id) => { trial.stop(); previewAnimation.stop(); root.querySelector('.inspector-try')?.setAttribute('aria-pressed', 'false'); drawPreview(null, id); },
  });
  const previewAnimation = createPreviewAnimation({
    request: requestPreview,
    perSecond: 30,
    enabled: () => !document.hidden && video.context().motion !== false && Boolean(root.querySelector('.inspector-preview')),
  });
  document.addEventListener('visibilitychange', () => {
    if (document.hidden) { trial.stop(); previewAnimation.stop(); previews.cancel(); }
    else requestPreview();
  });
  let previewWidth = root.clientWidth;
  new ResizeObserver(() => {
    if (root.clientWidth !== previewWidth) {
      previewWidth = root.clientWidth;
      requestPreview();
    }
  }).observe(root);

  /** Draws a rendered frame of element `id` in the preview (or says there is nothing to show). */
  function drawPreview(frame, id) {
    const figure = root.querySelector('.inspector-preview');
    if (!figure || figure.dataset.elementId !== String(id)) return;
    const target = figure.querySelector('canvas');
    const ratio = window.devicePixelRatio || 1;
    const size = fitPreview(frame, previewBox(figure), ratio);
    target.hidden = !size;
    figure.querySelector('.inspector-preview-off').hidden = Boolean(size);
    if (!size) return;
    const pixels = frame.pixels ?? frame.rgba;
    target.width = frame.width;
    target.height = frame.height;
    target.style.width = `${size.width}px`;
    target.style.height = `${size.height}px`;
    const offset = previewOffset(frame);
    target.style.transform = `translate(${offset.x * size.width / frame.width}px, ${offset.y * size.height / frame.height}px)`;
    target.getContext('2d').putImageData(new ImageData(pixels instanceof Uint8ClampedArray ? pixels : new Uint8ClampedArray(pixels), frame.width, frame.height), 0, 0);
  }

  /** The preview: "Preview · live", the value its sensor reads now, the object alone, and a graph's time axis. */
  function preview(e) {
    return el('figure', { class: 'inspector-preview', dataset: { elementId: String(e.id) } }, [
      el('div', { class: 'inspector-preview-stage' }, [
        el('canvas', { width: 1, height: 1, role: 'img', 'aria-label': t('inspector.previewOf', { name: e.name }) }),
        el('p', { class: 'hint inspector-preview-off', hidden: true, text: t('insp.previewNone') }),
      ]),
    ]);
  }

  /**
   * The same object's preview drawn again keeps the picture shown until the
   * next one arrives, so the panel does not shrink and jump meanwhile.
   */
  function keepPreviewPicture(next) {
    const before = root.querySelector('.inspector-preview');
    const after = next.querySelector('.inspector-preview');
    if (!before || !after || before.dataset.elementId !== after.dataset.elementId) return;
    const stage = before.querySelector('.inspector-preview-stage');
    const freshStage = after.querySelector('.inspector-preview-stage');
    stage.querySelector('canvas').setAttribute('aria-label', freshStage.querySelector('canvas').getAttribute('aria-label'));
    freshStage.replaceWith(stage);
  }

  /** Asks for the selected object's preview again. */
  function requestPreview() {
    previewAnimation.stop();
    if (document.hidden) { previews.cancel(); return; }
    const figure = root.querySelector('.inspector-preview');
    if (figure) previews.request(Number(figure.dataset.elementId));
    else previews.cancel();
  }

  // ---------------------------------------------------------- live data --
  const peaks = createPeakWatch();
  let seenSample = null;
  let seenAt = 0;

  /** The values read now: on the source card and preview, a graph's peak, a player's apps. */
  function refreshLive() {
    const sample = host.lastSample();
    const live = root.querySelector('.inspector-live');
    if (live) {
      const fixed = shownElement?.kind?.content?.type === 'static';
      live.textContent = t(fixed ? 'insp.fixedState' : sample ? 'insp.live' : 'insp.waiting');
      live.dataset.live = String(!fixed && Boolean(sample));
    }
    if (!sample) return;
    if (sample !== seenSample) {
      seenSample = sample;
      seenAt = Date.now();
    }
    for (const node of root.querySelectorAll('[data-live-key]')) node.textContent = sample.readings?.[node.dataset.liveKey]?.display ?? '…';
    refreshSensorPicker();
    const peak = root.querySelector('[data-peak-key]');
    if (peak) {
      const key = peak.dataset.peakKey;
      peaks.observe(key, sample.readings?.[key], seenAt);
      peak.textContent = peaks.peak(key, Number(peak.dataset.windowMs), seenAt)?.display ?? '—';
    }
    const chips = root.querySelector('.app-chips');
    if (chips && shownElement && chips.dataset.signature !== appSignature(sample.activity)) {
      const next = appChipGroup(shownElement);
      const active = chips.contains(document.activeElement) ? document.activeElement.textContent : null;
      chips.replaceWith(next.group);
      if (active !== null) Array.from(next.group.children).find((button) => button.textContent === active)?.focus();
      root.querySelector('.player-apps-hint').textContent = t(next.chips.length ? 'insp.appsHint' : 'insp.appsNone');
      const source = root.querySelector('.player-source');
      const manual = sourceState.custom || next.custom !== null;
      if (manual && !source.children.length) {
        source.append(manualSource(shownElement));
      } else if (!manual && !source.contains(document.activeElement)) source.replaceChildren();
    }
  }

  // --------------------------------------------------------------- head --
  /** The object's kind, its name (edited in place), what it is and where, the eye, the lock and more actions. */
  function header(e, widget, theme) {
    const facts = layerFacts(theme, e);
    const where = facts.group ? t('insp.inGroup', { group: facts.group }) : t('insp.layer', { level: facts.level, total: facts.total });
    const name = el('input', { type: 'text', class: 'inspector-name', value: e.name, 'aria-label': t('common.name'), spellcheck: false });
    name.addEventListener('change', () => {
      const v = name.value.trim();
      if (v) update(e.id, { name: v });
      else name.value = e.name;
    });
    const key = e.isGroup || e.card ? null : boundKey(e);
    const subtitle = el('span', { class: 'inspector-sub' }, [
      e.card ? t('cardInsp.facePosition', { number: e.card.activeFace + 1, total: e.card.faces.length }) : key ? el('span', { text: sensors.catalog().find(sensor => sensor.key === key)?.label ?? key }) : t(`widget.${widget}`),
      key ? el('span', { class: 'inspector-preview-value', dataset: { liveKey: key }, text: shownValue(key) }) : ` \u00b7 ${e.card ? e.card.faces[e.card.activeFace] : where}`,
    ]);
    const head = objectHeader({ preview: preview(e), name, subtitle,
      live: el('span', { class: 'inspector-live', text: t('insp.live') }),
      tryButton: iconButton(t('insp.tryObject'), ICONS.play, () => { trial.start(e); requestPreview(); }, { class: 'quiet inspector-try', disabled: video.context().motion === false, 'aria-pressed': String(trial.at(e, video.context().motion !== false) !== null) }),
      menuButton: iconButton(t('insp.more'), ICONS.more, event => openMenuAt(event.currentTarget, {
        label: t('menu.title'), items: objectMenuItems({ store, t, fitContent: id => host.fitContent?.(id) }),
        onClose: reason => { if (reason === 'item') root.querySelector('.object-menu-button')?.focus(); },
      }), { class: 'quiet object-menu-button', 'aria-haspopup': 'menu', 'aria-expanded': 'false' }),
    });
    head.title = `${t(`widget.${widget}`)} · ${where}`;
    head.dataset.layer = String(facts.level);
    head.dataset.total = String(facts.total);
    return head;
  }

  /** An align button: the selection to an edge (a lone object: of the screen). */
  const alignTool = (ids, edge, paths) => el('button', { type: 'button', class: 'icon-button', title: t(`align.${edge}`), 'aria-label': t(`align.${edge}`), onclick: () => store.dispatch('align', { ids, edge }) }, [icon(paths, 16)]);

  function panelsOf(e, theme, assets, specific, layout) {
    // Conditional colour belongs beside the paint it controls. Opacity has a
    // stable group of its own instead of ending up among corners or thresholds.
    if (e.kind.type === 'graph') {
      const seconds = timingOf(e).duration;
      specific.data?.find(group => group?.dataset.section === 'window')?.append(el('div', { class: 'inspector-preview-axis', 'aria-hidden': 'true' }, [el('span', { text: t('insp.ago', { duration: durationText(seconds) }) }), el('span', { text: t('insp.now') })]));
    }
    const opacity = numberField(t('inspector.opacity'), Math.round((e.opacity ?? 1) * 100), v => update(e.id, { opacity: Math.max(0, Math.min(100, v)) / 100 }), { min: 0, max: 100, unit: '%' });
    const linked = !e.card && !e.isGroup;
    const motion = sensorMotionSection(e);
    if (linked && motion && (rotationBindings.get(e.id) ?? Boolean(e.sensorMotion))) {
      motion.dataset.section = 'rotation-binding';
      layout.push(motion);
    }
    const look = [visualizationFields(e), ...[...(specific.look ?? []), ...(specific.behavior ?? [])].filter(group => {
      if (['matrix','pixel'].includes(e.visualization?.style) && group?.dataset.section === 'segments') return false;
      return !(e.visualization?.style === 'matrix' && group?.dataset.section === 'corners');
    }), !linked && motion].filter(Boolean);
    if (e.kind.type === 'graph' && e.visualization?.style === 'mirrored') {
      const v = e.visualization;
      const set = patch => update(e.id, { visualization: { ...v, ...patch } });
      const b = v.secondary;
      (specific.data ??= []).splice(specific.data.findIndex(group => group?.dataset.section === 'source') + 1, 0, section('secondary-source', t('visual.secondary'), [
        sourceCard(e.id, b?.key ?? '', key => ({ visualization: { ...v, secondary: { key, min: b?.min ?? 0, max: b?.max ?? 100 } } }), { numeric: true, label: t('visual.secondary'), scope: 'secondary-source' }),
        b && !e.kind.autoscale && fieldGrid([
          prefixedField(t('insp.minPrefix'), t('visual.secondaryMin'), b.min, min => set({ secondary: { ...b, min: Math.min(min,b.max-0.001) } }), { step:'any' }),
          prefixedField(t('insp.maxPrefix'), t('visual.secondaryMax'), b.max, max => set({ secondary: { ...b, max: Math.max(max,b.min+0.001) } }), { step:'any' }),
        ]),
        el('p', { class:'hint', text:t('visual.mirroredHint') }),
      ]));
      look.find(group => group.dataset.section === 'color')?.querySelector('.property-section-body').append(colorField(t('visual.secondaryColor'), v.secondaryColor, secondaryColor => set({ secondaryColor })));
    }
    const fontGroup = look.find(group => group.dataset.section === 'font');
    const textPaint = fontGroup?.querySelector(':scope > .property-section-body > .color-field');
    if (textPaint) look.push(section('color', t('inspector.color'), [textPaint]));
    const conditional = valueColorSection(e, assets);
    if (conditional) {
      const paint = look.filter(group => ['color', 'spectrum-paint'].includes(group.dataset.section)).flatMap(group => [...group.querySelectorAll('.color-field')])[0] ?? look.flatMap(group => [...group.querySelectorAll('.color-field')])[0];
      const group = paint?.closest('.property-section');
      if (['color', 'spectrum-paint'].includes(group?.dataset.section)) group.querySelector('.property-section-body').append(conditional);
      else if (paint) paint.after(conditional);
      else look.push(section('color', t('inspector.color'), [conditional]));
    }
    const transparency = look.find(group => group.dataset.section === 'transparency');
    if (transparency) transparency.querySelector('.property-section-body').append(opacity);
    else look.push(section('visibility', t('insp.visibility'), [opacity]));
    const rank = group => { const index = LOOK_ORDER.indexOf(group.dataset.section); return index < 0 ? LOOK_ORDER.length : index; };
    look.sort((a, b) => rank(a) - rank(b));
    return {
      layout, data: specific.data ?? [],
      look,
    };
  }

  function elementForm(e, theme, assets) {
    const f = e.frame, widget = widgetOf(e);
    const kind = e.card ? 'card' : e.isGroup ? 'group' : widget;
    if (shownElement?.id !== e.id) { sourceState.custom = false; trial.stop(); }
    shownElement = e;
    const setFrame = patch => store.dispatch('setFrame', { id: e.id, frame: { ...f, ...patch } });
    const position = section('position', t('inspector.position'), [
      el('div', { class: 'fld-grid geometry-fields' }, [
        prefixedField(t('insp.xPrefix'), t('inspector.x'), f.x, v => setFrame({ x: v }), { unit: 'px' }),
        prefixedField(t('insp.yPrefix'), t('inspector.y'), f.y, v => setFrame({ y: v }), { unit: 'px' }),
      ]),
      sizeField({ widthPrefix: t('insp.widthPrefix'), heightPrefix: t('insp.heightPrefix'), width: t('inspector.width'), height: t('inspector.height'), link: t('insp.linkSize') }, f,
        patch => setFrame(resizedFrame(f, patch, sizeLinks.get(e.id) ?? ['ring', 'image'].includes(widget))), {
          linked: sizeLinks.get(e.id) ?? ['ring', 'image'].includes(widget),
          onLink: linked => { sizeLinks.set(e.id, linked); render(shown.assets); },
        }),
      canRotate(theme, e) && el('div', { class: 'geometry-rotation bound-field' }, [
        prefixedField('', t('insp.rotation'), appliedRotation(e), rotation => store.dispatch('rotate', { id: e.id, rotation }), { step: 'any', unit: '°', iconName: 'rotate' }),
        !e.card && !e.isGroup && bindButton(t('insp.sensorMotion'), Boolean(e.sensorMotion), () => { rotationBindings.set(e.id, !(rotationBindings.get(e.id) ?? Boolean(e.sensorMotion))); render(shown.assets); }),
      ]),
      el('div', { class: 'align-row', role: 'group', 'aria-label': t('insp.alignOn') }, ALIGN_EDGES.map(([edge, name]) => alignTool([e.id], edge, ICONS[name]))),
    ], { tool: iconButton(t('insp.fitContent'), ICONS.frame, () => host.fitContent?.(e.id), { disabled: !canFitContent(theme, e.id), class: 'quiet' }) });
    const actions = section('actions', !e.isGroup ? t('insp.order') : t('inspector.actions'), [el('div', { class: 'actions' }, [
      e.isGroup && el('button', { type: 'button', class: 'text-button', text: t('group.ungroup'), onclick: () => store.dispatch('ungroup', { ids: [e.id] }) }),
      el('button', { type: 'button', class: 'text-button', onclick: () => store.dispatch('reorder', { id: e.id, index: theme.elements.length }) }, [t('inspector.front')]),
      iconButton(t('insp.oneForward'), ICONS.up, () => store.dispatch('reorder', { id: e.id, index: theme.elements.findIndex(item => item.id === e.id) + 1 })),
      iconButton(t('insp.oneBackward'), ICONS.down, () => store.dispatch('reorder', { id: e.id, index: theme.elements.findIndex(item => item.id === e.id) - 1 })),
      el('button', { type: 'button', class: 'text-button', onclick: () => store.dispatch('reorder', { id: e.id, index: 0 }) }, [t('inspector.back')]),
      el('button', { type: 'button', class: 'text-button', onclick: () => store.dispatch('duplicate', { ids: [e.id] }) }, [icon(ICONS.copy, 16), t('common.duplicate')]),
      el('button', { type: 'button', class: 'text-button danger', onclick: () => store.dispatch('remove', { ids: [e.id] }) }, [icon(ICONS.trash, 16), t('common.delete')]),
    ])]);
    const rotationField = position.querySelector('.geometry-rotation');
    if (rotationField) position.querySelector('.geometry-fields').append(rotationField);
    const layout = [position, section('membership', t('inspector.membership'), membershipForm(e, theme)), actions];
    const context = { element: e, assets, ...common, t, store, update, root, sensors, host, pickers, colorField, video, bridge, section, editIcon, searchCities, optionTiles, arcPreview, redraw: () => render(shown.assets), customSource: sourceState.custom,
      completePanels: specific => panelsOf(e, theme, assets, specific, layout) };
    const descriptors = e.isGroup ? describeSections(context, context.completePanels({})) : FORMS[e.card ? 'card' : widget]?.(context) ?? [];
    return [header(e, widget, theme), showAs(e, theme), ...sectioned(kind, descriptors, e)];
  }

  /** Renders for the current store state. */
  function render(assets = []) {
    closeSensorPicker();
    // A framing slider under the pointer stays; the form follows at its end.
    if (sliding && store.isGesturing()) return;
    sliding = false;
    const { theme, selection } = store.getState();
    const chosen = selection.map(id => theme.elements.find(e => e.id === id)).filter(Boolean);
    root.classList.toggle('inspector-linked', chosen.length !== 1 || !chosen[0].isGroup);
    root.classList.toggle('inspector-theme', chosen.length === 0);
    let nodes;
    if (chosen.length === 0) {
      trial.stop();
      shownElement = null;
      const context = { theme, assets, t, store, background, minRefresh, video, host };
      nodes = background.isOpen() ? themePanel(context) : [themeHeader(context), themeOrientation(context), ...sectioned('theme', themeSections(context), null), el('p', { class: 'hint theme-selection-hint', text: t('insp.theme.selectHint') })];
    } else {
      if (chosen.length > 1) trial.stop();
      background.dismiss();
      if (chosen.length > 1) shownElement = null;
      nodes = chosen.length === 1 ? elementForm(chosen[0], theme, assets) : [multiHeader({ ids: chosen.map(e => e.id), t, store, host, video }), ...sectioned('multi', multiSections({ ids: chosen.map(e => e.id), t, store, alignTool, ALIGN_EDGES, colorField, sensors, host }), null)];
    }
    // Keep focus where the user was: on the same tab, on the field of the
    // same label, or of the same stable id (the framing's). A number field
    // stepped with its arrows or spin buttons stays the same input (focus,
    // caret, key repeat), the preview keeps its picture until the next one
    // and the panel its scroll (D-2026-10-15-studio-redesign-fidelity-22).
    const active = document.activeElement;
    const inside = root.contains(active);
    const scope = inside ? active.closest('[data-focus-scope]')?.dataset.focusScope : null;
    const tab = inside && active.getAttribute('role') === 'tab' ? active.dataset.tab : null;
    const id = inside && !tab && active.id?.startsWith('framing-') ? active.id : null;
    const label = inside && !tab && !id ? active.getAttribute('aria-label') : null;
    const nativeLabel = inside && !tab && !id && !label ? active.labels?.[0]?.textContent : null;
    const buttonText = inside && active.tagName === 'BUTTON' ? active.textContent : null;
    const controlSelector = inside ? `${active.tagName.toLowerCase()}${active.tagName === 'INPUT' ? `[type="${active.type}"]` : ''}` : null;
    const counterpart = (within) => {
      const focusRoot = scope ? within.querySelector(`[data-focus-scope="${CSS.escape(scope)}"]`) ?? within : within;
      if (tab) return within.querySelector(`[role="tab"][data-tab="${CSS.escape(tab)}"]`);
      if (id) return within.querySelector(`#${CSS.escape(id)}`);
      if (label) return focusRoot.querySelector(`${controlSelector}[aria-label="${CSS.escape(label)}"]`);
      if (nativeLabel) return Array.from(within.querySelectorAll(controlSelector)).find((node) => node.labels?.[0]?.textContent === nativeLabel) ?? null;
      if (buttonText !== null) return Array.from(within.querySelectorAll('button')).find((node) => node.textContent === buttonText) ?? null;
      return null;
    };
    const scrollTop = root.scrollTop;
    const next = el('div', {}, nodes.filter(Boolean));
    keepPreviewPicture(next);
    const fresh = inside ? counterpart(next) : null;
    const old = Array.from(root.childNodes);
    root.append(...next.childNodes);
    const kept = fresh !== null && adoptControl(active, fresh);
    old.forEach((child) => child.remove());
    if (!kept && inside) counterpart(root)?.focus({ preventScroll: true });
    root.scrollTop = scrollTop;
    requestPreview();
    themePreview = root.querySelector('.theme-preview');
    if (themePreview) host.drawThemePreview?.(themePreview);
    refreshLive();
    shown = { assets, signature: videoSignature() };
  }

  function videoSignature() {
    const { theme, selection } = store.getState();
    if (selection.length) return '';
    const c = video.context();
    if (background.isOpen()) return JSON.stringify([c.screen, c.live, c.liveVideo?.state]);
    if (!['video', 'deviceVideo'].includes(theme.background.type)) return '';
    if (theme.background.type === 'deviceVideo') return JSON.stringify([c.live, c.screen?.key, c.liveVideo?.state, c.liveVideo?.path]);
    return JSON.stringify([videoStatus(c), c.tools?.ready ?? null, c.tools?.installHints?.length ?? 0, c.auto?.rotation ?? null, Boolean(c.framing), c.motion !== false]);
  }

  /** The screen, live mode, the live video or ffmpeg changed. */
  function contextChanged() {
    const signature = videoSignature();
    if (signature !== shown.signature) render(shown.assets);
    else requestPreview();
  }

  /** A new frame is on the canvas: the preview is rendered again and the live values follow. */
  function frameShown() {
    if (themePreview) host.drawThemePreview?.(themePreview);
    // Canvas animations need not render a static selected object again.
    // Isolated animations follow their own nextMs; new samples still apply.
    if (host.lastSample() !== seenSample) requestPreview();
    refreshLive();
  }

  return { render, contextChanged, frameShown,
    highlightField(key, params) {
      root.querySelectorAll('.canvas-field-active').forEach(node => node.classList.remove('canvas-field-active'));
      if (!key) return;
      const label = t(key, params);
      const field = [...root.querySelectorAll('input')].find(input => input.getAttribute('aria-label') === label || input.labels?.[0]?.textContent.trim() === label);
      if (!field) return;
      const section = field.closest('.accordion-section');
      const toggle = section?.querySelector(':scope > .accordion-heading > button');
      if (toggle?.getAttribute('aria-expanded') === 'false') toggle.click();
      const highlighted = field.closest('.fld, .number-control, .field') ?? field;
      highlighted.classList.add('canvas-field-active');
      const visible = root.getBoundingClientRect(), bounds = highlighted.getBoundingClientRect();
      if (bounds.top < visible.top || bounds.bottom > visible.bottom) highlighted.scrollIntoView({ block: 'nearest', inline: 'nearest' });
    },
    storageChanged(screen, overview) {
      if (background.updateListing(screen, overview) && !store.getState().selection.length) render(shown.assets);
    },
  };
}
