import { canRotate, appliedRotation } from '../editor/rotation.js';
import { visualization, visualStyles, visualArt, matrixFrame } from '../visualization.js';
import { defaultSensorMotion, supportsSensorMotion } from '../sensor-motion.js';
import { sensorNativeUnit } from '../sensor-picker-model.js';
import { sensorPicker, closeSensorPicker, refreshSensorPicker } from './sensor-picker.js';
import { applyTextCase, TEXT_CASES } from '../text-case.js';
import { SPECTRUM_STYLES, followedPlayer, spectrumOf, spectrumPatch } from '../spectrum.js';
import { MAX_COLOR_THRESHOLDS, addColorThreshold, defaultValueColor, supportsValueColor } from '../value-color.js';
import { moveRule } from '../card-rules.js';
import { canFitContent } from '../editor/fit-content.js';
// The right inspector: the theme when nothing is selected, one element's
// properties, or align/distribute tools for several. Every edit is one
// store command (one undo step). An element's head (kind, name, layer,
// eye, lock, type and more actions), its preview rendered alone by the backend
// (D-2026-10-09-studio-redesign-fidelity-8), membership, position and size, tabs,
// then the sections of its tab (artboards Main, GraphData/Look,
// PlayerData/Look), all built from the primitives of ui/fields.js (D-14).
import { canGroup } from '../editor/cards.js';
import { CLOCK_PATTERNS, formatClock } from '../clock-format.js';
import { readIcon } from '../icon-catalog.js';
import { el, icon, svgEl as node } from './dom.js';
import { ICONS } from './icons.js';
import {
  chipButton, colorPicker, dashedHint, fieldGrid, fontPicker, kindIcon, numberField,
  adoptControl, cornerRadii, iconButton, prefixedField, rangeField, propertySection as section, segmented, selectField, sentence, switchRow, textField,
} from './fields.js';
import { joinColor, splitColor } from '../color-math.js';
import { sentenceParts } from '../sentence.js';
import { CORNERS, cornerValue, coverBox } from '../corner-radii.js';
import { boundKey, createWidget, widgetOf } from '../editor/widgets.js';
import { isHorizontal, isTurned, orientationOf } from '../editor/geometry.js';
import { videoStatus } from '../editor/background.js';
import { ROTATIONS, ZOOM_MAX, ZOOM_MIN, ZOOM_STEP, framingOf, framingPercents, reframed, resolvedRotation, withoutFraming } from '../editor/video-framing.js';
import { formatBytes } from './storage.js';
import { DURATION_UNITS, INTERVAL_UNITS, TIME_UNITS, appChips, arcDrawing, createTabMemory, durationParts, graphRuler, graphSentence, graphTiming, graphTimingPatch, graphWindowRange, layerFacts, quickDurations, refreshChoices, tabAfterKey, tabsOf } from '../inspector-tabs.js';
import { createPeakWatch, createPreviewScheduler, fitPreview, previewOffset, previewSide } from '../element-preview.js';
import { createPreviewAnimation } from '../preview-animation.js';
import { CARD_TABS, renderCardPanel } from './inspector-card.js';
import { createPickers } from './pickers.js';
import { createBackgroundPanel } from './inspector-background.js';
import { TRANSITION_RANGE, formatMs, transitionOf, transitionPatch } from '../value-transition.js';
import { PLAYER_PARTS, PROGRESS_DEFAULTS, PROGRESS_PATTERNS, PROGRESS_SEGMENTS, PROGRESS_THICKNESS, TIME_FORMATS, TIME_PLACEMENTS, partPatch, partsOf, progressOf } from '../player-parts.js';

const BOUND = ['value', 'bar', 'ring', 'needle', 'graph'];
/** The tallest the preview is drawn, CSS pixels. */
const PREVIEW_HEIGHT = 120;
/** The edges an object aligns to on the screen, and their icons. */
const ALIGN_EDGES = Object.freeze([['left', 'alignLeft'], ['centerX', 'alignCenterX'], ['right', 'alignRight'], ['top', 'alignTop'], ['centerY', 'alignCenterY'], ['bottom', 'alignBottom']]);
// The same reading order for every object's Look tab: structure/type,
// typography, paint, edges, then effects. A kind omits what it does not use.
const LOOK_ORDER = Object.freeze([
  'show', 'shape', 'visualization', 'style', 'spectrum-bars', 'arc', 'spacing', 'appearance', 'font',
  'color', 'picture', 'spectrum-paint', 'fill', 'progress', 'track', 'segments',
  'outline', 'corners', 'shadow', 'transparency', 'preview', 'visibility',
]);
// Illustrations of the option tiles (24-unit paths drawn like every icon):
// the graph's styles, what the player shows and how its bar is cut.
const ART = Object.freeze({
  line: ['M2 17l5-4 4 2 5-8 6 4'],
  area: ['M2 17l5-4 4 2 5-8 6 4v8H2z'],
  bars: ['M4 21v-5', 'M9 21V9', 'M14 21v-7', 'M19 21V5'],
  cover: ['M3 6h10v12H3z', 'M16 9h5', 'M16 14h3'],
  progress: ['M3 11h18', 'M3 11h8', 'M3 17h3', 'M18 17h3'],
  source: ['M3 8h18v8H3z', 'M7 12h8'],
  continuous: ['M3 12h18'],
  segments: ['M3 12h4', 'M10 12h4', 'M17 12h4'],
  dots: ['M4 12h.01', 'M9 12h.01', 'M14 12h.01', 'M19 12h.01'],
  // How a shape's picture covers it (artboard-style tiles over the frame).
  repeatCover: ['M3 4h18v16H3z', 'M3 17l6-7 4 4 3-3 5 5'],
  repeatContain: ['M3 4h18v16H3z', 'M7 8h10v8H7z'],
  repeatStretch: ['M3 4h18v16H3z', 'M7 8l10 8', 'M7 8h3', 'M7 8v3', 'M17 16h-3', 'M17 16v-3'],
  repeatCenter: ['M3 4h18v16H3z', 'M10 10h4v4h-4z'],
  repeatTile: ['M3 4h18v16H3z', 'M6 7h4v4H6z', 'M14 7h4v4h-4z', 'M6 13h4v4H6z', 'M14 13h4v4h-4z'],
  repeatTileX: ['M3 4h18v16H3z', 'M5 10h4v4H5z', 'M10 10h4v4h-4z', 'M15 10h4v4h-4z'],
  repeatTileY: ['M3 4h18v16H3z', 'M10 5h4v4h-4z', 'M10 10h4v4h-4z', 'M10 15h4v4h-4z'],
});
/** How a shape's picture covers it, in the order the tiles show. */
const IMAGE_REPEATS = Object.freeze(['cover', 'contain', 'stretch', 'center', 'tile', 'tileX', 'tileY']);
/** The name of each repeat; stretch and center reuse the image fit's words. */
const REPEAT_KEYS = Object.freeze({ cover: 'shape.repeat.cover', contain: 'shape.repeat.contain', stretch: 'fit.fill', center: 'framing.center', tile: 'shape.repeat.tile', tileX: 'shape.repeat.tileX', tileY: 'shape.repeat.tileY' });
/** Repeats that keep the picture's own size: scale and offset apply. */
const NATURAL_REPEATS = Object.freeze(['center', 'tile', 'tileX', 'tileY']);
/** The player's parts: their name and drawing ("What to show"). */
const PART_TILES = Object.freeze({
  cover: ['insp.cover', ART.cover],
  title: ['insp.partTitle', ICONS.text],
  artist: ['insp.partArtist', ICONS.person],
  time: ['insp.partTime', ICONS.clock],
  progress: ['insp.progress', ART.progress],
  source: ['insp.appName', ART.source],
});
/** Names of the player's time formats, of where its times sit and of its bar's patterns. */
const TIME_FORMAT_KEYS = Object.freeze({
  elapsedTotal: 'insp.timeElapsedTotal', elapsedRemaining: 'insp.timeElapsedRemaining', elapsed: 'insp.timeElapsed', remaining: 'insp.timeRemaining', total: 'insp.timeTotal',
});
/** In the order of TIME_PLACEMENTS: in the text, at the sides, above. */
const TIME_PLACEMENT_KEYS = Object.freeze(['insp.timesText', 'insp.timesSides', 'insp.timesAbove']);
const PATTERN_KEYS = Object.freeze({ continuous: 'insp.patternContinuous', segments: 'insp.patternSegments', dots: 'insp.patternDots' });

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
function optionTiles(label, tiles) {
  return el('div', { class: 'option-tiles', role: 'group', 'aria-label': label }, tiles.map(({ text, art, pressed, onClick }) =>
    el('button', { type: 'button', class: 'option-tile', 'aria-pressed': String(pressed), onclick: onClick }, [icon(art, 32), el('span', { text })])));
}

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

  /**
   * The sensor an object reads, as a card (artboards' "Sorgente"): its
   * icon, name, mono key, value now and chevron. Every context opens the
   * same searchable picker, with the appropriate numeric/text scope.
   */
  function sourceCard(id, key, patchFor, { numeric = false, label = t('inspector.sensor'), scope = 'source' } = {}) {
    return sensorPicker({ t, key, label, numeric, scope, catalog: () => sensors.catalog(), readings: () => host.lastSample()?.readings,
      onChange: key => update(id, patchFor(key)) });
  }

  /** Min and max of a binding, mono-prefixed side by side. */
  function rangeFields(id, binding, { muted = false } = {}) {
    const grid = fieldGrid([
      prefixedField(t('insp.minPrefix'), t('inspector.min'), binding.min, (v) => update(id, { kind: { binding: { min: v } } }), { step: 'any' }),
      prefixedField(t('insp.maxPrefix'), t('inspector.max'), binding.max, (v) => update(id, { kind: { binding: { max: v } } }), { step: 'any' }),
    ]);
    if (muted) {
      grid.classList.add('is-muted');
      for (const input of grid.querySelectorAll('input')) input.disabled = true;
    }
    return grid;
  }

  // What the form was last drawn with: a video background is drawn again
  // when the screen, live mode, its video or ffmpeg change.
  let shown = { assets: [], signature: '' };

  // --------------------------------------------------------------- theme --
  const button = (text, onclick) => el('button', { type: 'button', class: 'text-button', text, onclick });

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

  /** Vertical | Horizontal, keeping the 180° turn (like the stage's buttons). */
  function orientationField(theme) {
    const axis = isHorizontal(theme.orientation) ? 'horizontal' : 'vertical';
    const field = segmented(t('inspector.orientation'), axis, [['vertical', t('axis.vertical')], ['horizontal', t('axis.horizontal')]], (next) => {
      if (next !== axis) store.dispatch('setOrientation', { orientation: orientationOf(next, isTurned(theme.orientation)) });
    }, { bare: true });
    field.classList.add('theme-orientation');
    const [vertical, horizontal] = field.children;
    vertical.prepend(icon(ICONS.vertical, 14));
    horizontal.prepend(icon(ICONS.horizontal, 14));
    return field;
  }

  /** How often the theme reads its sensors: the quick values, and the theme's own when it is another. */
  function refreshSection(theme) {
    const set = (refreshSeconds) => store.dispatch('setTheme', { patch: { refreshSeconds } });
    return section('refresh', t('inspector.refresh'), [
      el('div', { class: 'chip-group', role: 'group', 'aria-label': t('inspector.refresh') }, refreshChoices(theme.refreshSeconds, minRefresh())
        .map(({ seconds, pressed }) => chipButton(t('unit.seconds', { value: seconds }), pressed, () => !pressed && set(seconds)))),
    ]);
  }

  function themeForm(theme, assets) {
    if (background.isOpen()) return background.chooser(theme.background, assets);
    return [
      el('h2', { text: t('inspector.theme') }),
      textField(t('common.name'), theme.name, (v) => store.dispatch('setTheme', { patch: { name: v } })),
      orientationField(theme),
      el('p', { class: 'hint theme-canvas', text: t('inspector.canvas', { width: theme.canvas.width, height: theme.canvas.height }) }),
      background.section(theme.background, assets),
      refreshSection(theme),
    ];
  }

  // ---------------------------------------------------------- per kind ----
  // Each kind gives the sections of its Data tab and of its Look tab.

  /** The source section of an object bound to a sensor: card, Show as, and min/max when it has them. */
  function sourceSection(e, key, patchFor, { range = true } = {}) {
    return section('source', t('insp.source'), [
      sourceCard(e.id, key, patchFor, { numeric: e.kind.type !== 'text' }),
      range && e.kind.binding ? rangeFields(e.id, e.kind.binding) : null,
    ]);
  }

  /**
   * The words the font picker's sample shows: the object's own text (its
   * words, the sensor's value with prefix and suffix, the playing title,
   * the time), else its name.
   */
  function sampleText(e) {
    const c = e.kind.content ?? {};
    if (c.type === 'static' && c.text) return c.text;
    if (c.type === 'sensor') return `${c.prefix ?? ''}${shownValue(c.key)}${c.suffix ?? ''}`;
    if (c.type === 'clock') return formatClock(c.pattern ?? '%H:%M', new Date(), c.language ?? null);
    if (c.type === 'player') return host.lastSample()?.activity?.media?.[0]?.title ?? e.name;
    return e.name;
  }

  /** The font of a text: family and face, size (unless shown elsewhere), color, alignment and spacing. */
  function fontFields(e, { size = true } = {}) {
    const s = e.kind.style;
    const { texts, source } = pickers.font(applyTextCase(sampleText(e), s.casing));
    // A family picked instead of the theme's own font file drops that file.
    const setFont = (font) => update(e.id, { kind: { style: { font } } });
    return [
      fontPicker(texts, s.font, setFont, source),
      selectField(t('textCase.label'), s.casing ?? 'normal', TEXT_CASES.map((value) => [value, t(`textCase.${value}`)]), (casing) => update(e.id, { kind: { style: { casing } } })),
      size && numberField(t('inspector.size'), s.size, (v) => update(e.id, { kind: { style: { size: Math.max(4, v) } } }), { min: 4, max: 800, unit: 'px', iconName: 'textSize' }),
      colorField(t('inspector.color'), typeof s.paint === 'string' ? s.paint : '#ffffffff', (v) => update(e.id, { kind: { style: { paint: v } } })),
      segmented(t('inspector.align'), s.align, [['left', t('common.left')], ['center', t('inspector.alignCenter')], ['right', t('common.right')]], (v) => update(e.id, { kind: { style: { align: v } } })),
      segmented(t('axis.vertical'), s.valign, [['top', t('inspector.valignTop')], ['middle', t('inspector.valignMiddle')], ['bottom', t('inspector.valignBottom')]], (v) => update(e.id, { kind: { style: { valign: v } } })),
      numberField(t('inspector.letterSpacing'), s.letterSpacing ?? 0, (v) => update(e.id, { kind: { style: { letterSpacing: v } } }), { step: 'any', unit: 'px', iconName: 'spacing' }),
    ];
  }

  function textForm(e) {
    const k = e.kind;
    const s = k.style;
    const c = k.content;
    if (c.type === 'player') return playerForm(e);
    if (c.type === 'spectrum') return spectrumForm(e);
    const nodes = [], weatherAppearance = [];
    if (c.type === 'static') nodes.push(textField(t('common.text'), c.text, (v) => update(e.id, { kind: { content: { text: v } } })));
    if (c.type === 'clock') {
      nodes.push(selectField(t('clock.language'), c.language ?? '', [['', t('clock.system')], ['it', t('language.it')], ['en', t('language.en')], ['pt-BR', t('language.pt-BR')]], (language) => update(e.id, { kind: { content: { language: language || null } } })));
      nodes.push(selectField(t('clock.format'), CLOCK_PATTERNS.includes(c.pattern) ? c.pattern : 'custom', [...CLOCK_PATTERNS.map((pattern) => [pattern, formatClock(pattern, new Date(), c.language, c.casing)]), ['custom', t('clock.custom')]], (pattern) => {
        if (pattern === 'custom') root.querySelector('[list="clock-patterns"]')?.focus();
        else update(e.id, { kind: { content: { pattern } } });
      }));
      nodes.push(selectField(t('clock.casing'), e.kind.style.casing ?? c.casing ?? 'normal', TEXT_CASES.map((value) => [value, t(`textCase.${value}`)]), (casing) => update(e.id, { kind: { content: { casing: 'normal' }, style: { casing } } })));
      nodes.push(textField(t('inspector.pattern'), c.pattern, (v) => update(e.id, { kind: { content: { pattern: v } } }), { list: 'clock-patterns' }));
      nodes.push(el('datalist', { id: 'clock-patterns' }, CLOCK_PATTERNS.map((p) => el('option', { value: p }))));
      nodes.push(el('p', { id: 'clock-preview', class: 'hint', text: t('clock.preview', { value: applyTextCase(formatClock(c.pattern, new Date(), c.language, c.casing), e.kind.style.casing) }) }));
      nodes.push(el('p', { class: 'hint', text: t('inspector.patternHelp') }));
    }
    if (c.type === 'weather') {
      let query = c.city;
      const results = el('div', { id: 'weather-results', role: 'status', 'aria-live': 'polite' });
      const search = el('button', { type: 'button', class: 'text-button', text: t('weather.search'), onclick: async () => {
        search.disabled = true;
        results.textContent = t('weather.loading');
        try {
          const cities = await searchCities(query);
          if (!root.contains(results)) return;
          results.replaceChildren();
          if (!cities.length) results.textContent = t('weather.empty');
          for (const city of cities) results.append(el('button', { type: 'button', class: 'text-button', text: [city.name, city.region, city.country].filter(Boolean).join(', '), onclick: () => update(e.id, { kind: { content: { city: city.name, latitude: city.latitude, longitude: city.longitude } } }) }));
        } catch { if (root.contains(results)) results.textContent = t('weather.error'); }
        finally { search.disabled = false; }
      } });
      nodes.push(el('p', { class: 'hint', text: t('weather.location', { city: c.city }) }));
      nodes.push(textField(t('weather.city'), query, (value) => { query = value; }));
      nodes.push(search, results);
      nodes.push(selectField(t('weather.language'), c.language ?? '', [['', t('clock.system')], ['it', t('language.it')], ['en', t('language.en')], ['pt-BR', t('language.pt-BR')]], (language) => update(e.id, { kind: { content: { language: language || null } } })));
      nodes.push(switchRow(t('inspector.fahrenheit'), c.fahrenheit, (fahrenheit) => update(e.id, { kind: { content: { fahrenheit } } })));
      weatherAppearance.push(switchRow(t('weather.icon'), c.showIcon !== false, (showIcon) => update(e.id, { kind: { content: { showIcon } } })));
      if (c.showIcon !== false) {
        weatherAppearance.push(selectField(t('weather.iconStyle'), c.iconStyle ?? 'outline', [['outline', t('icons.outline')], ['filled', t('icons.filled')], ['colored', t('weather.colored')], ['dimensional', t('weather.dimensional')]], (iconStyle) => update(e.id, { kind: { content: { iconStyle } } })));
        weatherAppearance.push(numberField(t('weather.iconGap'), c.iconGap ?? s.size * 0.3, (iconGap) => update(e.id, { kind: { content: { iconGap: Math.max(0, Math.min(256, iconGap)) } } }), { min: 0, max: 256, step: 'any' }));
        weatherAppearance.push(numberField(t('weather.iconSize'), c.iconSize ?? 0, (size) => update(e.id, { kind: { content: { iconSize: size <= 0 ? null : Math.min(512, size) } } }), { min: 0, max: 512 }));
      }
      nodes.push(el('p', { class: 'hint', text: t('weather.source') }));
    }
    const data = [];
    if (c.type === 'sensor') {
      data.push(sourceSection(e, c.key, (key) => ({ kind: { content: { key } } })));
      nodes.push(el('div', { class: 'field-row' }, [
        textField(t('inspector.prefix'), c.prefix, (v) => update(e.id, { kind: { content: { prefix: v } } })),
        textField(t('inspector.suffix'), c.suffix, (v) => update(e.id, { kind: { content: { suffix: v } } })),
      ]));
      nodes.push(selectField(t('inspector.decimals'), c.format.decimals === undefined || c.format.decimals === null ? 'auto' : String(c.format.decimals),
        [['auto', t('inspector.decimalsAuto')], ['0', '0'], ['1', '1'], ['2', '2']],
        (v) => update(e.id, { kind: { content: { format: { decimals: v === 'auto' ? null : Number(v) } } } })));
      nodes.push(switchRow(t('inspector.showUnit'), c.format.showUnit !== false, (v) => update(e.id, { kind: { content: { format: { showUnit: v } } } })));
      if (sensors.catalog().find(sensor => sensor.key === c.key)?.quantity === 'celsius') {
        nodes.push(switchRow(t('inspector.fahrenheit'), c.format.fahrenheit, (v) => update(e.id, { kind: { content: { format: { fahrenheit: v } } } })));
      }
    }
    data.push(section('content', t('inspector.content'), nodes));
    if (c.type === 'sensor') data.push(transitionSection(e));
    const look = [
      section('appearance', t('inspector.appearance'), weatherAppearance),
      section('font', t('inspector.font'), fontFields(e)),
    ];
    return { data, look };
  }

  /** The app chips of a player: Automatic, one per app with a media session (a dot when it plays), Other…. */
  function appChipGroup(e) {
    const c = e.kind.content;
    const set = (patch) => {
      customSource = false;
      update(e.id, { kind: { content: patch } });
    };
    const activity = host.lastSample()?.activity ?? null;
    const { chips, custom } = appChips(activity, c.source ?? '');
    const group = el('div', { class: 'chip-group app-chips', role: 'group', 'aria-label': t('insp.apps'), dataset: { signature: appSignature(activity) } }, [
      chipButton(t('insp.automatic'), !(c.source ?? '').trim(), () => set({ source: '' })),
      ...chips.map((chip) => {
        const button = chipButton(chip.name, chip.pressed, () => set({ source: chip.name }));
        button.prepend(el('span', { class: 'pill-dot', 'aria-hidden': 'true', dataset: { playing: String(chip.playing) } }));
        if (chip.playing) button.setAttribute('aria-label', `${chip.name}, ${t('insp.playing')}`);
        return button;
      }),
      chipButton(t('insp.otherApp'), customSource || custom !== null, () => {
        customSource = true;
        render(shown.assets);
        root.querySelector('.player-source input')?.focus();
      }, { class: 'chip other-chip' }),
    ]);
    return { group, chips, custom };
  }

  const appSignature = (activity) => JSON.stringify((activity?.media ?? []).map((m) => [m.source, m.playing]));

  const manualSource = (e) => textField(t('player.source'), e.kind.content.source ?? '', (source) => update(e.id, { kind: { content: { source: source.slice(0, 160) } } }));

  /** A choice drawn as a card with its radio (artboard PlayerData, "Quando non suona niente"). */
  function choiceCard(name, checked, title, note, onChoose, extra = null) {
    const input = el('input', { type: 'radio', name, checked });
    input.addEventListener('change', () => input.checked && onChoose());
    return el('div', { class: 'choice-card', dataset: { checked: String(checked) } }, [
      el('label', { class: 'choice-card-head' }, [input, el('span', { class: 'choice-card-text' }, [el('strong', { text: title }), el('span', { text: note })])]),
      checked ? extra : null,
    ]);
  }

  /** The media player (artboards PlayerData and PlayerLook). */
  function spectrumForm(e) {
    const c = spectrumOf(e.kind.content), theme = store.getState().theme;
    const set = patch => update(e.id, { kind: { content: spectrumPatch(c, patch) } });
    const players = theme.elements.filter(item => item.kind.type === 'text' && item.kind.content.type === 'player');
    const selected = followedPlayer(theme, c);
    const options = [['', t('spectrum.automatic')], ...players.map(item => [String(item.id), item.name])];
    if (c.player != null && !players.some(p => p.id === c.player)) options.push([String(c.player), t('spectrum.playerMissing')]);
    const paint = e.kind.style.paint, gradient = typeof paint === 'object';
    const start = gradient ? paint.stops?.[0]?.[1] ?? '#38bdf8ff' : paint;
    const end = gradient ? paint.stops?.at(-1)?.[1] ?? '#f97316ff' : '#f97316ff';
    const setPaint = (mode, a = start, b = end) => update(e.id, { kind: { style: { paint: mode === 'gradient' ? { angle: 90, stops: [[0,a],[1,b]] } : a } } });
    const STYLE_LABELS = { mirrored: 'spectrum.mirrored', bars: 'graph.bars', blocks: 'spectrum.blocks', line: 'graph.line', radial: 'fade.radial' };
    // Fields that do not apply to the chosen style are hidden, never disabled.
    const line = c.style === 'line';
    const baseLabel = { mirrored: 'spectrum.centre', radial: 'spectrum.ring' }[c.style] ?? 'spectrum.baseline';
    return {
      data: [section('spectrum-source', t('spectrum.source'), [
        // D-AS-13: the followed Player's app, or everything the PC plays.
        segmented(t('spectrum.source'), c.audio, [['player', t('spectrum.audioPlayer')], ['system', t('spectrum.audioSystem')]], audio => set({ audio }), { bare: true }),
        ...(c.audio === 'system' ? [el('p', { class: 'hint', text: t('spectrum.systemHint') })] : [
          selectField(t('spectrum.follow'), c.player == null ? '' : String(c.player), options, v => set({ player: v ? Number(v) : null })),
          !selected && el('p', { class: 'hint', role: 'status', text: t('spectrum.playerMissing') }),
          el('p', { class: 'hint', text: t('spectrum.browser') }),
        ]),
        segmented(t('common.preview'), c.demo ? 'demo' : 'live', [['live',t('spectrum.live')],['demo',t('spectrum.demo')]], v => set({ demo: v === 'demo' })),
        c.demo && el('p', { class: 'hint', role: 'status', text: t('spectrum.demoHint') }),
        el('p', { class: 'hint', text: t('spectrum.enableHint') }),
      ]), section('spectrum-bands', t('spectrum.bands'), [
        numberField(t('spectrum.bands'), c.bands, bands => set({ bands }), { min: 8, max: 64, step: 1, iconName: 'count' }),
        numberField(t('spectrum.low'), c.low, low => set({ low }), { min: 20, max: 23999, step: 1, unit: 'Hz', iconName: 'frequency' }),
        numberField(t('spectrum.high'), c.high, high => set({ high }), { min: c.low + 1, max: 24000, step: 1, unit: 'Hz', iconName: 'frequency' }),
        numberField(t('spectrum.gain'), c.gain, gain => set({ gain }), { min: 0.1, max: 4, step: 0.1, unit: '×', iconName: 'gain' }),
        el('p', { class: 'hint', text: t('spectrum.gainHint') }),
      ])],
      look: [section('spectrum-paint', t('inspector.color'), [
        selectField(t('inspector.fill'), gradient ? 'gradient' : 'solid', [['solid', t('ring.solid')],['gradient',t('spectrum.gradient')]], mode => setPaint(mode)),
        colorField(t('inspector.color'), start, v => setPaint(gradient ? 'gradient' : 'solid', v)),
        gradient && colorField(t('ring.endColor'), end, v => setPaint('gradient', start, v)),
      ]), section('spectrum-bars', t('graph.bars'), [
        selectField(t('spectrum.style'), c.style, SPECTRUM_STYLES.map(v => [v, t(STYLE_LABELS[v])]), style => set({ style })),
        !line && numberField(t('spectrum.thickness'), c.thickness, thickness => set({ thickness }), { min: 1, max: 256, unit: 'px', iconName: 'width' }),
        !line && numberField(t('spectrum.gap'), c.gap, gap => set({ gap }), { min: 0, max: 128, unit: 'px', iconName: 'spacing' }),
        c.style === 'blocks' && numberField(t('spectrum.block'), c.block, block => set({ block }), { min: 1, max: 64, unit: 'px', iconName: 'height' }),
        c.style === 'blocks' && el('p', { class: 'hint', text: t('spectrum.blockColors') }),
        line && numberField(t('spectrum.stroke'), c.stroke, stroke => set({ stroke }), { min: 1, max: 32, unit: 'px', iconName: 'strokeWidth' }),
        line && switchRow(t('spectrum.fill'), c.fill, fill => set({ fill })),
        c.style === 'radial' && numberField(t('spectrum.inner'), c.inner, inner => set({ inner }), { min: 0, max: 90 }),
        c.style !== 'blocks' && switchRow(t(baseLabel), c.centre, centre => set({ centre })),
        !line && switchRow(t('spectrum.peaks'), c.peaks, peaks => set({ peaks })),
        segmented(t('card.tab.motion'), c.motion, ['smooth','balanced','snappy'].map(v => [v,t(`spectrum.${v}`)]), motion => set({ motion })),
      ]), !line && barCorners(e.id, c, set, e.frame)],
    };
  }

  function playerForm(e) {
    const c = e.kind.content;
    const s = e.kind.style;
    const set = (patch) => update(e.id, { kind: { content: patch } });
    const parts = partsOf(c);
    const { group, chips, custom } = appChipGroup(e);
    const data = [
      section('apps', t('insp.apps'), [
        group,
        el('div', { class: 'player-source' }, customSource || custom !== null ? [manualSource(e)] : []),
        el('p', { class: 'hint player-apps-hint', text: t(chips.length ? 'insp.appsHint' : 'insp.appsNone') }),
      ]),
      section('stopped', t('insp.stopped'), [el('div', { class: 'choice-cards', role: 'radiogroup', 'aria-label': t('insp.stopped') }, [
        choiceCard(`player-stopped-${e.id}`, Boolean(c.hideWhenStopped), t('insp.stoppedHide'), t('insp.stoppedHideNote'), () => set({ hideWhenStopped: true })),
        choiceCard(`player-stopped-${e.id}`, !c.hideWhenStopped, t('insp.stoppedText'), t('insp.stoppedTextNote'), () => set({ hideWhenStopped: false }),
          el('div', { class: 'choice-card-extra' }, [textField(t('player.empty'), c.emptyText ?? '', (emptyText) => set({ emptyText: emptyText.slice(0, 512) }))])),
      ])]),
      parts.time ? section('time', t('insp.partTime'), [
        selectField(t('insp.timeFormat'), c.timeFormat ?? 'elapsedTotal', TIME_FORMATS.map((f) => [f, t(TIME_FORMAT_KEYS[f])]), (timeFormat) => set({ timeFormat })),
      ]) : null,
      dashedHint([t('insp.playerHint')], { iconName: 'info' }),
    ];
    const tiles = PLAYER_PARTS.map((part) => ({ text: t(PART_TILES[part][0]), art: PART_TILES[part][1], pressed: parts[part], onClick: () => set(partPatch(c, part)) }));
    const look = [
      section('show', t('insp.show'), [optionTiles(t('insp.show'), tiles)]),
      parts.cover ? coverCorners(e) : null,
      parts.progress ? progressSection(e, parts) : null,
      section('spacing', t('insp.spacing'), [
        fieldGrid([
          prefixedField(t('insp.gap'), t('player.coverGap'), c.coverGap ?? s.size * 0.3, (v) => set({ coverGap: Math.max(0, v) }), { min: 0, step: 'any', unit: 'px', iconName: 'spacing' }),
          prefixedField(t('insp.textSize'), t('inspector.size'), s.size, (v) => update(e.id, { kind: { style: { size: Math.max(4, v) } } }), { min: 4, max: 800, unit: 'px', iconName: 'textSize' }),
        ]),
        el('p', { class: 'hint', text: t('insp.gapHint') }),
      ]),
      section('font', t('inspector.font'), fontFields(e, { size: false })),
    ];
    return { data: data.filter(Boolean), look };
  }

  /**
   * The player's progress bar (D-2026-10-16-studio-redesign-fidelity-26):
   * colours, thickness, ends, knob, pattern and, with the times on, where
   * they sit. Absent fields draw the bar as before it had options.
   */
  function progressSection(e, parts) {
    const c = e.kind.content;
    const style = progressOf(c);
    const set = (patch) => update(e.id, { kind: { content: { progressStyle: patch } } });
    const textColor = typeof e.kind.style.paint === 'string' ? e.kind.style.paint : PROGRESS_DEFAULTS.track;
    const pattern = (value) => ({ text: t(PATTERN_KEYS[value]), art: ART[value], pressed: style.pattern === value, onClick: () => style.pattern !== value && set({ pattern: value }) });
    return section('progress', t('insp.progressBar'), [
      colorField(t('inspector.fill'), style.fill ?? textColor, (fill) => set({ fill })),
      colorField(t('inspector.track'), style.track, (track) => set({ track })),
      rangeField(t('insp.thickness'), style.thickness, (thickness) => set({ thickness }), { ...PROGRESS_THICKNESS, format: (v) => t('insp.px', { value: v }) }),
      switchRow(t('inspector.roundCaps'), style.ends !== 'square', (on) => set({ ends: on ? 'round' : 'square' })),
      switchRow(t('insp.progressKnob'), style.knob, (knob) => set({ knob })),
      optionTiles(t('inspector.style'), PROGRESS_PATTERNS.map(pattern)),
      style.pattern === 'segments' && numberField(t('inspector.count'), style.segments,
        (v) => set({ segments: Math.min(PROGRESS_SEGMENTS.max, Math.max(PROGRESS_SEGMENTS.min, Math.round(v))) }), PROGRESS_SEGMENTS),
      parts.time && segmented(t('insp.timesPlace'), style.times, TIME_PLACEMENTS.map((p, i) => [p, t(TIME_PLACEMENT_KEYS[i])]), (times) => times !== style.times && set({ times })),
    ]);
  }

  /**
   * The corners of an object (cornerRadii, D-2026-10-15-studio-redesign-fidelity-22):
   * `value` and `onChange` in src/corner-radii.js's shape, `label` names
   * the shared radius, `box` gives the drawing the object's proportions.
   */
  function cornersSection(title, label, value, onChange, { individual = true, box = null } = {}) {
    const texts = {
      label, prefix: t('insp.radius'), mode: t('insp.cornersMode'), same: t('insp.cornersSame'), each: t('insp.cornersEach'),
      corners: Object.fromEntries(CORNERS.map((name) => [name, t(`inspector.${name}`)])),
    };
    const { tool, control } = cornerRadii(texts, value, onChange, { individual, box });
    return section('corners', title, [control], { tool });
  }

  /** The cover's corners, drawn on the cover's own box. */
  function coverCorners(e) {
    const c = e.kind.content;
    return cornersSection(t('insp.corners'), t('player.coverRadius'), cornerValue(c.coverRadius ?? 0, c.coverCorners),
      ({ radius, corners }) => update(e.id, { kind: { content: { coverRadius: radius, coverCorners: corners } } }),
      { box: coverBox(e.frame, c.coverGap ?? e.kind.style.size * 0.3) });
  }

  /**
   * The corners of every spectrum bar, drawn on one bar. The spectrum always
   * stores four radii, so the chosen mode is remembered per element; without
   * a choice, equal corners show as one radius.
   */
  function barCorners(id, c, set, frame) {
    const each = barCornerModes.get(id) ?? c.corners.some((r) => r !== c.corners[0]);
    return cornersSection(t('insp.cornersTitle'), t('inspector.radius'), cornerValue(c.corners[0], each ? c.corners : null),
      ({ radius, corners }) => {
        barCornerModes.set(id, Boolean(corners));
        set({ corners: corners ?? Array(4).fill(radius) });
      },
      { box: { width: c.thickness, height: Math.max(c.thickness, frame.height / 2) } });
  }

  /** The track of a bar or ring: on or off, and its color. */
  const trackFields = (e) => [
    switchRow(t('inspector.showTrack'), Boolean(e.kind.track), (on) => update(e.id, { kind: { track: on ? '#ffffff26' : null } })),
    e.kind.track && colorField(t('inspector.track'), typeof e.kind.track === 'string' ? e.kind.track : '#ffffff26', (v) => update(e.id, { kind: { track: v } })),
  ];

  function segmentsFields(e) {
    const seg = e.kind.segments;
    return [
      switchRow(t('inspector.segments'), Boolean(seg), (on) => update(e.id, { kind: { segments: on ? { count: 10, gap: e.kind.type === 'ring' ? 4 : 2 } : null } })),
      seg && el('div', { class: 'field-row' }, [
        numberField(t('inspector.count'), seg.count, (v) => update(e.id, { kind: { segments: { count: Math.max(1, v) } } }), { min: 1, max: 200, iconName: 'count' }),
        numberField(t('inspector.gap'), seg.gap, (v) => update(e.id, { kind: { segments: { gap: Math.max(0, v) } } }), { min: 0, step: 'any', unit: e.kind.type === 'ring' ? '°' : 'px', iconName: 'spacing' }),
      ]),
    ];
  }

  /** How the shown value moves to a new reading (D-2026-10-15-studio-redesign-fidelity-18): none or smooth, and how long. */
  function transitionSection(e) {
    const { mode, durationMs } = transitionOf(e);
    const set = (change) => update(e.id, { valueTransition: transitionPatch(e, change) });
    return section('transition', t('insp.transition'), [
      segmented(t('insp.transition'), mode, [['none', t('fade.none')], ['smooth', t('insp.transitionSmooth')], ['inertia', t('insp.transitionInertia')]], (next) => set({ mode: next }), { bare: true }),
      mode !== 'none' && rangeField(t(mode === 'inertia' ? 'insp.transitionSettling' : 'card.duration'), durationMs, (next) => set({ durationMs: next }), { ...TRANSITION_RANGE, format: formatMs, compact: true }),
      el('p', { class: 'hint', text: t(mode === 'inertia' ? 'insp.transitionInertiaHint' : 'insp.transitionHint') }),
    ]);
  }

  function barForm(e) {
    const k = e.kind;
    const data = [sourceSection(e, k.binding.key, (key) => ({ kind: { binding: { key } } })), transitionSection(e)];
    const look = [section('style', t('inspector.style'), [
      selectField(t('common.direction'), k.direction, ['leftToRight', 'rightToLeft', 'bottomToTop', 'topToBottom'].map((d) => [d, t(`dir.${d}`)]), (v) => update(e.id, { kind: { direction: v } })),
    ]), section('color', t('inspector.color'), [
      colorField(t('inspector.fill'), typeof k.fill === 'string' ? k.fill : '#38bdf8ff', (v) => update(e.id, { kind: { fill: v } })),
    ]), section('track', t('inspector.track'), trackFields(e)),
    section('segments', t('inspector.segments'), segmentsFields(e)),
    cornersSection(t('insp.cornersTitle'), t('inspector.radius'), cornerValue(k.radius ?? 0), ({ radius }) => update(e.id, { kind: { radius } }),
      { individual: false, box: e.frame })];
    return { data, look };
  }

  function ringFillFields(e) {
    const p = e.kind.fill;
    const arc = p?.scale === 'arc';
    const linear = typeof p === 'object' && Boolean(p?.stops);
    const start = arc ? p.start : typeof p === 'string' ? p : p?.stops?.[0]?.[1] ?? '#38bdf8ff';
    const end = arc ? p.end : p?.stops?.at(-1)?.[1] ?? '#f97316ff';
    const modes = [['solid', t('ring.solid')], ['arc', t('ring.arc')]];
    if (linear) modes.push(['linear', t('ring.importedLinear')]);
    const setArc = (patch) => update(e.id, { kind: { fill: { scale: 'arc', start, end, transition: p?.transition ?? 50, ...patch } } });
    return [
      selectField(t('ring.fillMode'), arc ? 'arc' : linear ? 'linear' : 'solid', modes, (v) => {
        if (v === 'arc') setArc({});
        else if (v === 'solid') update(e.id, { kind: { fill: start } });
      }),
      ...(arc ? [
        colorField(t('ring.startColor'), start, (v) => setArc({ start: v })),
        colorField(t('ring.endColor'), end, (v) => setArc({ end: v })),
        rangeField(t('ring.transition'), p.transition ?? 50, (v) => setArc({ transition: v }), { min: 0, max: 100, step: 1, format: (v) => `${v}%` }),
        el('p', { class: 'hint', text: t('ring.transitionHint') }),
      ] : [!linear && colorField(t('inspector.fill'), start, (v) => update(e.id, { kind: { fill: v } }))]),
    ];
  }

  /** Start and sweep of a ring or needle, mono-prefixed, in degrees. */
  const angleFields = (e) => fieldGrid([
    prefixedField(t('insp.arcStart'), t('inspector.startAngle'), e.kind.startAngle, (v) => update(e.id, { kind: { startAngle: v } }), { min: -360, max: 360, unit: '°' }),
    prefixedField(t('insp.arcSweep'), t('inspector.sweep'), e.kind.sweep, (v) => update(e.id, { kind: { sweep: v } }), { min: 1, max: 360, unit: '°' }),
  ]);

  function ringForm(e) {
    const k = e.kind;
    const data = [
      sourceSection(e, k.binding.key, (key) => ({ kind: { binding: { key } } })),
      section('arc', t('insp.arc'), [
        el('div', { class: 'arc-row' }, [arcPreview(k), angleFields(e)]),
        rangeField(t('insp.thickness'), k.thickness, (v) => update(e.id, { kind: { thickness: Math.max(1, v) } }), { min: 1, max: Math.max(64, k.thickness), format: (v) => t('insp.px', { value: v }) }),
        switchRow(t('inspector.clockwise'), k.clockwise !== false, (v) => update(e.id, { kind: { clockwise: v } })),
        switchRow(t('inspector.roundCaps'), Boolean(k.roundCaps), (v) => update(e.id, { kind: { roundCaps: v } })),
      ]),
      transitionSection(e),
    ];
    const look = [
      section('color', t('inspector.color'), ringFillFields(e)),
      section('track', t('inspector.track'), trackFields(e)),
      section('segments', t('inspector.segments'), segmentsFields(e)),
      section('preview', t('common.preview'), [
        switchRow(t('ring.testFull'), Boolean(k.testFull), (v) => update(e.id, { kind: { testFull: v } })),
      ]),
    ];
    return { data, look };
  }

  function needleForm(e) {
    const k = e.kind;
    const data = [sourceSection(e, k.binding.key, (key) => ({ kind: { binding: { key } } })), transitionSection(e)];
    const look = [section('arc', t('insp.arc'), [
      angleFields(e),
      fieldGrid([
        prefixedField(t('insp.xPrefix'), t('inspector.pivotX'), Math.round((k.pivot?.[0] ?? 0.5) * 100), (v) => update(e.id, { kind: { pivot: [v / 100, k.pivot?.[1] ?? 0.5] } }), { min: 0, max: 100, unit: '%' }),
        prefixedField(t('insp.yPrefix'), t('inspector.pivotY'), Math.round((k.pivot?.[1] ?? 0.5) * 100), (v) => update(e.id, { kind: { pivot: [k.pivot?.[0] ?? 0.5, v / 100] } }), { min: 0, max: 100, unit: '%' }),
      ]),
      rangeField(t('inspector.lineWidth'), k.width, (v) => update(e.id, { kind: { width: Math.max(1, v) } }), { min: 1, max: Math.max(32, k.width), format: (v) => t('insp.px', { value: v }) }),
    ]), section('color', t('inspector.color'), [
      colorField(t('inspector.color'), k.color, (v) => update(e.id, { kind: { color: v } })),
    ])];
    return { data, look };
  }

  /** How a duration of `seconds` is written ("30 s", "5 min", "1 h"). */
  function durationText(seconds) {
    const { value, unit } = durationParts(seconds);
    if (unit === 'h') return t('unit.hours', { value });
    if (unit === 'min') return t('unit.minutes', { value });
    return t('unit.seconds', { value });
  }

  /** A graph's timing: its own interval, else the theme refresh (D-2026-10-15-studio-redesign-fidelity-16). */
  const timingOf = (e) => graphTiming(e.kind, store.getState().theme.refreshSeconds);

  /** The window a graph looks back over, ms. */
  const graphWindowMs = (e) => timingOf(e).duration * 1000;

  /**
   * The tokens of a time in the window sentence: its number and the select
   * of its unit (words agreeing with the number), `onChange(seconds)` when
   * either changes. Built from numberField and selectField; the sentence
   * hides their labels, so each control carries its name.
   */
  function timeTokens(shown, units, label, unitLabel, onChange) {
    const number = numberField(label, shown.value, (v) => onChange(v * TIME_UNITS[shown.unit]), { min: 0, step: 'any' });
    const unit = selectField(unitLabel, shown.unit, units.map((u) => [u, t(`insp.unitWord.${u}.${shown.plural}`)]), (u) => onChange(shown.value * TIME_UNITS[u]));
    number.querySelector('input').setAttribute('aria-label', label);
    unit.querySelector('select').setAttribute('aria-label', unitLabel);
    unit.querySelector('select').classList.add('sentence-unit');
    return { number, unit };
  }

  /**
   * The graph's window as one sentence (D-2026-10-15-studio-redesign-fidelity-17):
   * "Show the last [1] [hour], with one point every [30] [seconds]", its
   * word order from the locale's template.
   */
  function windowSentence(timing, setTiming) {
    const shown = graphSentence(timing);
    const duration = timeTokens(shown.duration, DURATION_UNITS, t('insp.durationLabel'), t('insp.durationUnit'), (d) => setTiming(d, timing.interval));
    const interval = timeTokens(shown.interval, INTERVAL_UNITS, t('insp.intervalLabel'), t('insp.intervalUnit'), (i) => setTiming(timing.duration, i));
    const slots = { last: t(shown.last), duration: duration.number, durationUnit: duration.unit, interval: interval.number, intervalUnit: interval.unit };
    return sentence(sentenceParts(t('insp.windowSentence')).map((part) => part.word ?? slots[part.slot]), { class: 'graph-window' });
  }

  /** The time ruler under the sentence: from −duration to now, a mark per point (thinned out past a few dozen), how many. */
  function timeRuler(timing) {
    const line = node('svg', { class: 'graph-ruler-line', viewBox: '0 0 100 10', preserveAspectRatio: 'none', 'aria-hidden': 'true' });
    line.append(
      node('line', { class: 'graph-ruler-base', x1: 0, y1: 9.5, x2: 100, y2: 9.5 }),
      ...graphRuler(timing.points).ticks.map((x) => node('line', { class: 'graph-ruler-tick', x1: x * 100, y1: 3, x2: x * 100, y2: 9.5 })),
    );
    return el('div', { class: 'graph-ruler' }, [line, el('div', { class: 'graph-ruler-labels' }, [
      el('span', { 'aria-hidden': 'true', text: t('insp.ago', { duration: durationText(timing.duration) }) }),
      el('span', { class: 'graph-points', 'aria-live': 'polite', text: t('insp.points', { points: timing.points }) }),
      el('span', { 'aria-hidden': 'true', text: t('insp.now') }),
    ])]);
  }

  /** The graph's window: quick durations, the sentence of its duration and point interval, the ruler of what they make. */
  function graphWindow(e) {
    const timing = timingOf(e);
    const setTiming = (duration, interval) => {
      const { history, intervalSeconds } = graphTimingPatch(duration, interval);
      update(e.id, { kind: { history, intervalSeconds } });
    };
    const range = graphWindowRange(timing.interval);
    return section('window', t('insp.window'), [
      el('div', { class: 'chip-group', role: 'group', 'aria-label': t('insp.quickDurations') },
        quickDurations(timing.interval, timing.points).map((d) => chipButton(durationText(d.seconds), d.pressed, () => setTiming(d.seconds, timing.interval), { disabled: d.disabled }))),
      windowSentence(timing, setTiming),
      timeRuler(timing),
      el('p', { class: 'hint graph-range', text: t('insp.windowRange', { min: durationText(range.min), max: durationText(range.max) }) }),
      timing.own ? null : el('p', { class: 'hint', text: t('insp.followsRefresh', { seconds: timing.interval }) }),
    ]);
  }

  function graphForm(e) {
    const k = e.kind;
    // Data (artboard GraphData): what it plots, its vertical scale and how far back it looks.
    const data = [
      sourceSection(e, k.binding.key, (key) => ({ kind: { binding: { key } } }), { range: false }),
      section('scale', t('insp.scale'), [
        segmented(t('insp.scale'), k.autoscale ? 'auto' : 'fixed', [['auto', t('insp.automatic')], ['fixed', t('insp.fixed')]], (v) => update(e.id, { kind: { autoscale: v === 'auto' } }), { bare: true }),
        rangeFields(e.id, k.binding, { muted: Boolean(k.autoscale) }),
        k.autoscale ? el('p', { class: 'hint' }, [t('insp.autoscaleHint'), ' ', t('insp.peak'), ' ', el('span', { class: 'mono graph-peak', dataset: { peakKey: k.binding.key, windowMs: String(graphWindowMs(e)) }, text: '—' })]) : null,
      ]),
      graphWindow(e),
      dashedHint([t('insp.historyHint')], { iconName: 'info' }),
    ];
    // Look (artboard GraphLook): the style, the color, the fill of an area and the line.
    const style = (value, text) => ({ text, art: ART[value], pressed: k.style === value, onClick: () => k.style !== value && update(e.id, { kind: { style: value }, ...(e.visualization?.style === 'rounded' && value !== 'bars' ? { visualization:null } : {}) }) });
    const { rgb } = splitColor(k.color);
    const fillAlpha = typeof k.fill === 'string' ? splitColor(k.fill).alpha : 25;
    const look = [
      section('style', t('inspector.style'), [
        optionTiles(t('inspector.style'), [style('line', t('graph.line')), style('area', t('graph.area')), style('bars', t('graph.bars'))]),
        rangeField(t('inspector.lineWidth'), k.lineWidth, (v) => update(e.id, { kind: { lineWidth: Math.max(1, v) } }), { min: 1, max: Math.max(12, k.lineWidth), step: 0.5, format: (v) => t('insp.px', { value: v }) }),
      ]),
      section('color', t('inspector.color'), [colorField(t('inspector.color'), k.color, (v) => update(e.id, { kind: { color: v } }))]),
      k.style === 'area' ? section('fill', t('inspector.fill'), [
        rangeField(t('insp.fillShare'), fillAlpha, (v) => update(e.id, { kind: { fill: joinColor(rgb, v) } }), { format: (v) => `${v}%` }),
      ], { count: t('insp.areaOnly') }) : null,
    ];
    return { data, look };
  }

  function imageForm(e, assets) {
    const k = e.kind;
    const paint = readIcon(assets.find((a) => a.ref === k.asset)?.iconSvg);
    const change = (patch) => editIcon(e.id, k.asset, { ...paint, ...patch });
    const options = assets.filter((a) => a.kind === 'image').map((a) => [a.ref, a.ref.split('/').pop()]);
    if (k.asset && !options.some(([r]) => r === k.asset)) options.push([k.asset, k.asset]);
    const data = [section('content', t('inspector.content'), [
      options.length
        ? selectField(t('common.image'), k.asset, [['', t('inspector.noImage')], ...options], (v) => update(e.id, { kind: { asset: v } }))
        : el('p', { class: 'hint', text: t('inspector.addMediaFirst') }),
    ])];
    const look = [section('appearance', t('inspector.appearance'), [
      selectField(t('inspector.fit'), k.fit, ['fill', 'contain', 'cover', 'none'].map((f) => [f, t(`fit.${f}`)]), (v) => update(e.id, { kind: { fit: v } })),
    ]), section('color', t('inspector.color'), [
      paint && colorField(t('inspector.color'), paint.color, (color) => change({ color }), { live: false }),
      paint?.style === 'outline' && numberField(t('inspector.strokeWidth'), paint.stroke, (stroke) => change({ stroke: Math.min(4, Math.max(0.5, stroke)) }), { min: 0.5, max: 4, step: 0.5, unit: 'px', iconName: 'strokeWidth' }),
    ]), section('shadow', t('icons.shadow'), [
      paint && numberField(t('icons.shadow'), paint.shadow, (shadow) => change({ shadow: Math.min(4, Math.max(0, shadow)) }), { min: 0, max: 4, step: 0.5, unit: 'px' }),
      paint && paint.shadow > 0 && colorField(t('icons.shadowColor'), paint.shadowColor, (shadowColor) => change({ shadowColor }), { live: false }),
    ]), section('transparency', t('fade.mode'), fadeFields(e))];
    return { data, look };
  }

  function fadeFields(e) {
    const f = e.kind.fade;
    const set = patch => update(e.id, { kind: { fade: patch } });
    return [
      selectField(t('fade.mode'), !f ? 'none' : f.radial ? 'radial' : 'linear',
        ['none','linear','radial'].map(mode => [mode, t(`fade.${mode}`)]), mode => set(mode === 'none' ? null : {
          angle: f?.angle ?? 0, start: f?.start ?? 1, end: f?.end ?? 0,
          radial: mode === 'radial' ? f?.radial ?? [0.5,0.5,0.5] : null,
        })),
      ...(f ? [
        ...(f.radial ? [
          numberField(t('fade.centerX'), Math.round(f.radial[0]*100), x => set({ radial: [Math.max(0,Math.min(100,x))/100,f.radial[1],f.radial[2]] }), {min:0,max:100}),
          numberField(t('fade.centerY'), Math.round(f.radial[1]*100), y => set({ radial: [f.radial[0],Math.max(0,Math.min(100,y))/100,f.radial[2]] }), {min:0,max:100}),
          numberField(t('fade.extent'), Math.round(f.radial[2]*100), r => set({ radial: [f.radial[0],f.radial[1],Math.max(1,Math.min(200,r))/100] }), {min:1,max:200}),
        ] : [
          numberField(t('shape.fadeAngle'), f.angle, angle => set({angle:Math.max(-360,Math.min(360,angle))}), {min:-360,max:360}),
          rangeField(t('fade.midpoint'), Math.round((f.midpoint ?? .5)*100), v => set({midpoint:Math.max(5,Math.min(95,v))/100}), {min:5,max:95,format:v=>`${v}%`}),
        ]),
        rangeField(t('shape.fadeStart'), Math.round(f.start*100), v => set({start:v/100}), {format:v=>`${v}%`}),
        rangeField(t('shape.fadeEnd'), Math.round(f.end*100), v => set({end:v/100}), {format:v=>`${v}%`}),
      ] : []),
    ];
  }

  // Picture inside the shape: chosen from the theme's images, how it covers
  // the shape, and for the natural-size repeats its scale and offset.
  function shapePicture(e, assets) {
    const k = e.kind;
    const image = k.image ?? null;
    const set = (patch) => update(e.id, { kind: { image: patch } });
    const options = assets.filter((a) => a.kind === 'image').map((a) => [a.ref, a.ref.split('/').pop()]);
    if (image?.asset && !options.some(([r]) => r === image.asset)) options.push([image.asset, image.asset]);
    const pick = options.length
      ? selectField(t('common.image'), image?.asset ?? '', [['', t('inspector.noImage')], ...options], (asset) => {
        if (!asset) set(null);
        else if (image) set({ asset });
        else set({ asset, repeat: 'cover', scale: 1, offsetX: 0, offsetY: 0, opacity: 1 });
      })
      : el('p', { class: 'hint', text: t('inspector.addMediaFirst') });
    if (!image) return section('picture', t('common.image'), [pick]);
    const repeat = IMAGE_REPEATS.includes(image.repeat) ? image.repeat : 'cover';
    const art = (value) => ART[`repeat${value[0].toUpperCase()}${value.slice(1)}`];
    const tiles = IMAGE_REPEATS.map((value) => ({ text: t(REPEAT_KEYS[value]), art: art(value), pressed: repeat === value, onClick: () => repeat !== value && set({ repeat: value }) }));
    const natural = NATURAL_REPEATS.includes(repeat);
    return section('picture', t('common.image'), [
      pick,
      optionTiles(t('shape.repeat'), tiles),
      natural && numberField(t('shape.imageScale'), Math.round((image.scale ?? 1) * 100), (v) => set({ scale: Math.max(5, Math.min(1000, v)) / 100 }), { min: 5, max: 1000, unit: '%' }),
      natural && numberField(t('shape.imageOffsetX'), image.offsetX ?? 0, (v) => set({ offsetX: v }), { unit: 'px' }),
      natural && numberField(t('shape.imageOffsetY'), image.offsetY ?? 0, (v) => set({ offsetY: v }), { unit: 'px' }),
      rangeField(t('shape.imageOpacity'), Math.round((image.opacity ?? 1) * 100), (v) => set({ opacity: Math.max(0, Math.min(100, v)) / 100 }), { min: 0, max: 100, format: (v) => `${v}%` }),
      el('p', { class: 'hint', text: t('shape.imageHint') }),
    ]);
  }

  function shapeForm(e, assets = []) {
    const k = e.kind;
    return { look: [section('shape', t('common.shape'), [
      segmented(t('common.shape'), k.shape, [['rect', t('shape.rect')], ['ellipse', t('shape.ellipse')]], (v) => update(e.id, { kind: { shape: v } })),
      switchRow(t('shape.videoWindow'), Boolean(k.videoWindow), (videoWindow) => update(e.id, { kind: { videoWindow } })),
    ]), section('color', t('inspector.color'), [
      k.videoWindow ? el('div', {}, [el('p', { class: 'hint', text: t('shape.videoWindowHint') }), button(t('deviceVideo.choose'), () => video.openStorage())]) : colorField(t('inspector.fill'), typeof k.fill === 'string' ? k.fill : '#1e293bff', (v) => update(e.id, { kind: { fill: v } })),
    ]), k.videoWindow ? null : shapePicture(e, assets),
    section('transparency', t('fade.mode'), fadeFields(e)),
    section('outline', t('insp.outline'), [
      numberField(t('inspector.strokeWidth'), k.strokeWidth ?? 0, (v) => update(e.id, { kind: { strokeWidth: Math.max(0, v), stroke: v > 0 ? (k.stroke ?? '#ffffffff') : null } }), { min: 0, unit: 'px', iconName: 'strokeWidth' }),
      (k.strokeWidth ?? 0) > 0 && colorField(t('inspector.stroke'), k.stroke ?? '#ffffffff', (v) => update(e.id, { kind: { stroke: v } })),
    ]), k.shape === 'rect' ? cornersSection(t('insp.cornersTitle'), t('inspector.radius'), cornerValue(k.radius ?? 0, k.corners),
      ({ radius, corners }) => update(e.id, { kind: { radius, corners } }), { box: e.frame }) : null] };
  }

  function showAs(e, theme) {
    const current = widgetOf(e);
    if (e.isGroup || !BOUND.includes(current)) return null;
    const key = boundKey(e);
    const quantity = sensors.catalog().find((s) => s.key === key)?.quantity;
    return selectField(t('inspector.showAs'), current, BOUND.map((w) => [w, t(`widget.${w}`)]), (w) => {
      const made = createWidget(w, theme.canvas, { key, quantity });
      if (made.kind.binding && e.kind.binding) made.kind.binding = { ...made.kind.binding, min: e.kind.binding.min, max: e.kind.binding.max };
      store.dispatch('setKind', { id: e.id, kind: made.kind });
    });
  }

  function membershipForm(e, theme) {
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

  /** The Faces, Motion and Triggers tabs of a card (ui/inspector-card.js). */
  const cardPanels = (e) => Object.fromEntries(CARD_TABS.map((tab) => [tab, renderCardPanel(tab, { element: e, t, store, update, section, bridge, host })]));


  // The tab open for each kind of object: UI-only, it survives edits,
  // re-renders and Undo.
  const tabMemory = createTabMemory();
  // UI-only too: whether "More actions" is open, and whether the player's
  // source is typed by hand ("Other…").
  let actionsOpen = false;
  let customSource = false;
  let shownElement = null;

  const tabName = (kind, tab) => t(kind === 'card' && tab !== 'look' ? `card.tab.${tab}` : `inspector.tab.${tab}`);

  /**
   * The panels of an object as tabs (ARIA tabs: ←/→, Home, End), only those
   * with something in them; one alone is shown without a tab list.
   */
  function tabbed(kind, panels) {
    const offered = tabsOf(kind).filter(tab => panels[tab]?.some(Boolean));
    if (offered.length < 2) return offered.flatMap(tab => panels[tab]);
    const open = tabMemory.current(kind, offered);
    const list = el('div', { class: 'inspector-tabs subtabs', role: 'tablist', 'aria-label': t(kind === 'card' ? 'card.tabs' : 'inspector.tabs') });
    const views = offered.map(tab => el('div', {
      class: 'inspector-panel', role: 'tabpanel', id: `inspector-panel-${tab}`, 'aria-labelledby': `inspector-tab-${tab}`, hidden: tab !== open, dataset: { tab },
    }, panels[tab].filter(Boolean)));
    const show = (tab, focus) => {
      tabMemory.choose(kind, tab);
      for (const button of list.children) {
        const on = button.dataset.tab === tab;
        button.setAttribute('aria-selected', String(on));
        button.tabIndex = on ? 0 : -1;
        if (on && focus) button.focus();
      }
      for (const view of views) view.hidden = view.dataset.tab !== tab;
    };
    for (const tab of offered) {
      list.append(el('button', {
        type: 'button', role: 'tab', id: `inspector-tab-${tab}`, 'aria-controls': `inspector-panel-${tab}`, 'aria-selected': String(tab === open),
        tabindex: tab === open ? '0' : '-1', dataset: { tab }, title: tabName(kind, tab), onclick: () => show(tab, false),
      }, [icon(ICONS[tab === 'data' ? 'database' : tab === 'look' ? 'brush' : tab === 'faces' ? 'layers' : tab === 'motion' ? 'play' : 'sliders'], 16), tabName(kind, tab)]));
    }
    list.addEventListener('keydown', evt => {
      const next = tabAfterKey(offered, evt.target.dataset?.tab, evt.key);
      if (!next) return;
      evt.preventDefault();
      show(next, true);
    });
    return [list, ...views];
  }

  // ------------------------------------------------------------ preview --
  // The object rendered alone by the backend (`renderElement`, D-8): asked
  // once per change or new frame, one render at a time, the last wins.
  const previewBox = (figure) => {
    const stage = figure.querySelector('.inspector-preview-stage');
    return { width: stage.clientWidth > 16 ? stage.clientWidth : 240, height: PREVIEW_HEIGHT };
  };
  const previews = createPreviewScheduler({
    render: async (id) => {
      const figure = root.querySelector('.inspector-preview');
      const ratio = window.devicePixelRatio || 1;
      const max = previewSide(figure ? previewBox(figure) : { width: 240, height: PREVIEW_HEIGHT }, ratio);
      const started = performance.now();
      const frame = await host.renderElement(id, { max, motion: video.context().motion !== false });
      return { frame, elapsed: performance.now() - started };
    },
    show: ({ frame, elapsed }, id) => {
      drawPreview(frame, id);
      previewAnimation.shown({ nextMs: frame?.nextMs, elapsed });
    },
    fail: (_, id) => { previewAnimation.stop(); drawPreview(null, id); },
  });
  const previewAnimation = createPreviewAnimation({
    request: requestPreview,
    enabled: () => !document.hidden && video.context().motion !== false && Boolean(root.querySelector('.inspector-preview')),
  });
  document.addEventListener('visibilitychange', () => {
    if (document.hidden) { previewAnimation.stop(); previews.cancel(); }
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
    const key = e.isGroup || e.card ? null : boundKey(e);
    const seconds = e.kind?.type === 'graph' ? timingOf(e).duration : null;
    return el('figure', { class: 'inspector-preview', dataset: { elementId: String(e.id) } }, [
      el('figcaption', { class: 'inspector-preview-head' }, [
        el('span', { class: 'inspector-preview-label', text: t('inspector.previewLive') }),
        key ? el('span', { class: 'inspector-preview-value', dataset: { liveKey: key }, text: shownValue(key) }) : null,
      ]),
      el('div', { class: 'inspector-preview-stage' }, [
        el('canvas', { width: 1, height: 1, role: 'img', 'aria-label': t('inspector.previewOf', { name: e.name }) }),
        el('p', { class: 'hint inspector-preview-off', hidden: true, text: t('insp.previewNone') }),
      ]),
      seconds === null ? null : el('div', { class: 'inspector-preview-axis', 'aria-hidden': 'true' }, [el('span', { text: t('insp.ago', { duration: durationText(seconds) }) }), el('span', { text: t('insp.now') })]),
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
      const manual = customSource || next.custom !== null;
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
    const hidden = e.visible === false;
    const type = showAs(e, theme);
    type?.classList.add('inspector-type');
    const tool = (label, paths, onclick, attrs = {}) => iconButton(label, paths, onclick, { class: 'quiet', ...attrs });
    return el('div', { class: 'inspector-head' }, [
      kindIcon(e.isGroup ? 'groupFrame' : widget),
      el('div', { class: 'inspector-title' }, [name, el('div', { class: 'inspector-sub' }, [el('h2', { text: t(`widget.${widget}`) }), ` · ${where}`])]),
      tool(t(hidden ? 'common.show' : 'common.hide'), hidden ? ICONS.eyeOff : ICONS.eye, () => update(e.id, { visible: hidden })),
      tool(t(e.locked ? 'common.unlock' : 'common.lock'), e.locked ? ICONS.lock : ICONS.unlock, () => update(e.id, { locked: !e.locked }), { 'aria-pressed': String(Boolean(e.locked)) }),
      tool(t('insp.more'), ICONS.more, (evt) => {
        actionsOpen = !actionsOpen;
        evt.currentTarget.setAttribute('aria-expanded', String(actionsOpen));
        root.querySelector('#inspector-actions').hidden = !actionsOpen;
      }, { 'aria-expanded': String(actionsOpen), 'aria-controls': 'inspector-actions' }),
      type,
    ]);
  }

  /** An align button: the selection to an edge (a lone object: of the screen). */
  const alignTool = (ids, edge, paths) => el('button', { type: 'button', class: 'icon-button', title: t(`align.${edge}`), 'aria-label': t(`align.${edge}`), onclick: () => store.dispatch('align', { ids, edge }) }, [icon(paths, 16)]);

  /** Conditional paint controls live beside the paint they affect. */
  function valueColorSection(e, assets) {
    if (!supportsValueColor(e)) return null;
    const rule = e.colorByValue;
    const set = colorByValue => update(e.id, { colorByValue });
    const rows = rule?.thresholds ?? [];
    const change = (i, patch) => set({ ...rule, thresholds: rows.map((row, n) => n === i ? { ...row, ...patch } : row) });
    const focusRow = (index, action) => queueMicrotask(() => {
      const row = root.querySelector(`[data-color-row="${index}"]`);
      // The same move again, else the row's other move or removal (not its colour button).
      const target = row?.querySelector(`[data-color-action="${action}"]:not(:disabled)`) ?? row?.querySelector('[data-color-action]:not(:disabled)') ?? row?.querySelector('button:not(:disabled)');
      target?.focus();
    });
    const reorder = (i, to, action) => { set({ ...rule, thresholds: moveRule(rows, i, to) }); focusRow(to, action); };
    return el('div', { class: 'value-color', dataset: { section: 'value-color' }, role: 'group', 'aria-label': t('insp.colorByValue') }, [
      switchRow(t('insp.colorByValue'), Boolean(rule), enabled => set(enabled ? defaultValueColor(e,
        sensors.catalog()[0]?.key ?? '', readIcon(assets.find(a => a.ref === e.kind.asset)?.iconSvg)?.color) : null)),
      ...(rule ? [
        e.kind.content?.type === 'spectrum' ? el('p', { class: 'hint', text: t('spectrum.levelScale') }) : boundKey(e) ? sourceCard(e.id, boundKey(e), key => ({ kind: e.kind.type === 'text' ? { content: { key } } : { binding: { key } } }), { numeric: true, scope: 'color' })
          : sourceCard(e.id, rule.sensor ?? '', sensor => ({ colorByValue: { sensor } }), { numeric: true, scope: 'color' }),
        colorField(t('insp.baseColor'), rule.base, base => set({ ...rule, base })),
        el('p', { class: 'hint', text: t('insp.colorThresholdHint') }),
        ...rows.map((row, i) => el('div', { class: 'trigger-rule', role: 'group', 'aria-label': t('insp.colorThresholdRow', { value: i + 1 }), dataset: { colorRow: String(i), focusScope: `color-${i}` } }, [
          prefixedField(t('insp.colorFrom'), t('insp.colorThresholdValue'), row.value, value => change(i, { value }), { step: 'any' }),
          colorField(t('inspector.color'), row.color, color => change(i, { color })),
          el('div', { class: 'button-row' }, [
            iconButton(t('insp.colorMoveUp'), ICONS.up, () => reorder(i, i - 1, 'up'), { disabled: i === 0, dataset: { colorAction: 'up' } }),
            iconButton(t('insp.colorMoveDown'), ICONS.down, () => reorder(i, i + 1, 'down'), { disabled: i === rows.length - 1, dataset: { colorAction: 'down' } }),
            iconButton(t('insp.colorRemove'), ICONS.trash, () => set({ ...rule, thresholds: rows.filter((_, n) => n !== i) })),
          ]),
        ])),
        chipButton(t('insp.colorAdd'), false, () => set(addColorThreshold(rule)), { disabled: rows.length >= MAX_COLOR_THRESHOLDS }),
      ] : []),
    ]);
  }

  function sensorMotionSection(e) {
    if (!supportsSensorMotion(e)) return null;
    const motion = e.sensorMotion;
    const set = sensorMotion => update(e.id, { sensorMotion });
    return section('sensor-motion', t('insp.sensorMotion'), [
      switchRow(t('insp.sensorMotionEnabled'), Boolean(motion), enabled => set(enabled ? defaultSensorMotion(sensors.catalog(), e) : null)),
      ...(motion ? [
        sourceCard(e.id, motion.sensor, sensor => ({ sensorMotion: { ...motion, sensor } }), { numeric: true, label: t('insp.motionSource'), scope: 'motion' }),
        numberField(t('insp.motionReference'), motion.reference, value => set({ ...motion, reference: Math.max(.001, Math.min(1e12, value)) }), { min: .001, max: 1e12, step: 'any', unit: sensorNativeUnit(sensors.catalog().find(s => s.key === motion.sensor)?.quantity) }),
        numberField(t('insp.motionPeriod'), 1 / motion.turnsPerSecond, value => set({ ...motion, turnsPerSecond: 1 / Math.max(.1, Math.min(100, value)) }), { min: .1, max: 100, step: 'any', unit: t('insp.motionSeconds') }),
        segmented(t('insp.motionResponse'), String(motion.inertiaMs), [
          ['0', t('insp.motionImmediate')], ['600', t('insp.motionGradual')], ['1500', t('insp.motionSoft')],
          ...([0, 600, 1500].includes(motion.inertiaMs) ? [] : [[String(motion.inertiaMs), t('insp.motionCustom')]]),
        ], value => set({ ...motion, inertiaMs: Number(value) })),
        el('p', { class: 'hint', text: t('insp.motionMapping', { value: motion.reference, unit: sensorNativeUnit(sensors.catalog().find(s => s.key === motion.sensor)?.quantity), seconds: new Intl.NumberFormat(document.documentElement.lang || 'en', { maximumFractionDigits: 2 }).format(1 / motion.turnsPerSecond) }) }),
        switchRow(t('insp.motionReverse'), motion.reverse, reverse => set({ ...motion, reverse })),
        el('p', { class: 'hint', text: t('insp.motionHint') }),
      ] : []),
    ]);
  }

  function panelsOf(e, theme, assets) {
    const kindForms = { text: textForm, bar: barForm, ring: ringForm, needle: needleForm, graph: graphForm, shape: shapeForm };
    const specific = e.isGroup ? {} : e.kind.type === 'image' ? imageForm(e, assets) : e.kind.type === 'shape' ? shapeForm(e, assets) : kindForms[e.kind.type]?.(e) ?? {};
    // Conditional colour belongs beside the paint it controls. Opacity has a
    // stable group of its own instead of ending up among corners or thresholds.
    const opacity = rangeField(t('inspector.opacity'), Math.round((e.opacity ?? 1) * 100), v => update(e.id, { opacity: v / 100 }), { format: v => `${v}%` });
    const look = [visualizationFields(e), ...(specific.look ?? []).filter(group => {
      if (['matrix','pixel'].includes(e.visualization?.style) && group?.dataset.section === 'segments') return false;
      return !(e.visualization?.style === 'matrix' && group?.dataset.section === 'corners');
    }), sensorMotionSection(e)].filter(Boolean);
    if (e.kind.type === 'graph' && e.visualization?.style === 'mirrored') {
      const v = e.visualization;
      const set = patch => update(e.id, { visualization: { ...v, ...patch } });
      const b = v.secondary;
      (specific.data ??= []).splice(1, 0, section('secondary-source', t('visual.secondary'), [
        sourceCard(e.id, b?.key ?? '', key => ({ visualization: { ...v, secondary: { key, min: b?.min ?? 0, max: b?.max ?? 100 } } }), { numeric: true, label: t('visual.secondary'), scope: 'secondary-source' }),
        b && !e.kind.autoscale && fieldGrid([
          prefixedField(t('insp.minPrefix'), t('visual.secondaryMin'), b.min, min => set({ secondary: { ...b, min: Math.min(min,b.max-0.001) } }), { step:'any' }),
          prefixedField(t('insp.maxPrefix'), t('visual.secondaryMax'), b.max, max => set({ secondary: { ...b, max: Math.max(max,b.min+0.001) } }), { step:'any' }),
        ]),
        colorField(t('visual.secondaryColor'), v.secondaryColor, secondaryColor => set({ secondaryColor })),
        el('p', { class:'hint', text:t('visual.mirroredHint') }),
      ]));
    }
    const conditional = valueColorSection(e, assets);
    if (conditional) {
      const paint = look.flatMap(group => [...group.querySelectorAll('.color-field')])[0];
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
    if (e.card) return { ...cardPanels(e), look };
    return {
      data: specific.data ?? [],
      look,
    };
  }

  function visualizationFields(e) {
    const styles = visualStyles(e.kind.type);
    if (!styles.length || e.isGroup) return null;
    const v = e.visualization;
    const set = patch => update(e.id, { visualization: { ...v, ...patch } });
    const n = (key, value, min, max, label = `visual.${key}`) => numberField(t(label), value, x => set({ [key]: Math.min(max,Math.max(min,Math.round(x))) }), { min,max });
    const presets = optionTiles(t('visual.preset'), styles.map(style => ({
      text:t(`visual.${style}`), art:visualArt(e.kind.type,style), pressed:style === (v?.style ?? 'standard'),
      onClick:()=>{ if(style !== (v?.style ?? 'standard')) update(e.id, { visualization:visualization(style,v ?? {}), ...(style === 'rounded' ? { kind:{style:'bars'} } : {}) }); },
    })));
    presets.classList.add('visualization-presets');
    const fitMatrix = () => {
      // Commit a pending number edit before reading the current grid. Retain
      // pointer focus until click so that the blur redraw cannot remove it.
      if (root.contains(document.activeElement)) document.activeElement.blur();
      const { theme } = store.getState();
      const current = theme.elements.find(item => item.id === e.id);
      if (current?.visualization?.style !== 'matrix') return;
      store.dispatch('setFrame',{id:current.id,frame:matrixFrame(current.frame,current.visualization,theme.canvas)});
      root.querySelector('[data-action="fit-matrix"]')?.focus({preventScroll:true});
    };
    return section('visualization', t('visual.title'), [
      presets,
      v?.style === 'matrix' && fieldGrid([n('columns',v.columns,1,64),n('rows',v.rows,1,32)]),
      v?.style === 'matrix' && el('button',{type:'button',class:'text-button',text:t('visual.fitMatrix'),dataset:{action:'fit-matrix'},onpointerdown:event=>{if(event.button===0)event.preventDefault();},onclick:fitMatrix}),
      ['matrix','rounded','mirrored'].includes(v?.style) && numberField(t('visual.gap'),v.gap,gap=>set({gap:Math.max(0,Math.min(32,gap))}),{min:0,max:32,unit:'px'}),
      ['matrix','pixel'].includes(v?.style) && numberField(t('visual.radius'),v.radius,radius=>set({radius:Math.max(0,Math.min(32,radius))}),{min:0,max:32,unit:'px'}),
      ['pixel','dial'].includes(v?.style) && n('ticks',v.ticks,2,120,v.style === 'pixel' ? 'visual.cells' : 'visual.ticks'),
      v?.style === 'dial' && switchRow(t('visual.marker'),v.marker,marker=>set({marker})),
    ]);
  }

  function elementForm(e, theme, assets) {
    const f = e.frame, widget = widgetOf(e);
    const kind = e.card ? 'card' : e.isGroup ? 'group' : widget;
    if (shownElement?.id !== e.id) customSource = false;
    shownElement = e;
    const setFrame = patch => store.dispatch('setFrame', { id: e.id, frame: { ...f, ...patch } });
    const position = section('position', t('inspector.position'), [
      el('div', { class: 'fld-grid geometry-fields' }, [
        prefixedField(t('insp.xPrefix'), t('inspector.x'), f.x, v => setFrame({ x: v }), { unit: 'px' }),
        prefixedField(t('insp.yPrefix'), t('inspector.y'), f.y, v => setFrame({ y: v }), { unit: 'px' }),
        prefixedField(t('insp.widthLabel'), t('inspector.width'), f.width, v => setFrame({ width: Math.max(4, v) }), { min: 4, unit: 'px' }),
        prefixedField(t('insp.heightLabel'), t('inspector.height'), f.height, v => setFrame({ height: Math.max(4, v) }), { min: 4, unit: 'px' }),
      ]),
      canRotate(theme, e) && el('div', { class: 'geometry-rotation' }, [prefixedField(t('insp.rotation'), t('insp.rotation'), appliedRotation(e), rotation => store.dispatch('rotate', { id: e.id, rotation }), { step: 'any', unit: '°' })]),
      el('div', { class: 'align-row', role: 'group', 'aria-label': t('insp.alignOn') }, ALIGN_EDGES.map(([edge, name]) => alignTool([e.id], edge, ICONS[name]))),
    ], { tool: iconButton(t('insp.fitContent'), ICONS.frame, () => host.fitContent?.(e.id), { disabled: !canFitContent(theme, e.id), class: 'quiet' }) });
    const actions = section('actions', t('inspector.actions'), [el('div', { class: 'actions' }, [
      e.isGroup && el('button', { type: 'button', class: 'text-button', text: t('group.ungroup'), onclick: () => store.dispatch('ungroup', { ids: [e.id] }) }),
      el('button', { type: 'button', class: 'text-button', onclick: () => store.dispatch('reorder', { id: e.id, index: theme.elements.length }) }, [t('inspector.front')]),
      el('button', { type: 'button', class: 'text-button', onclick: () => store.dispatch('reorder', { id: e.id, index: 0 }) }, [t('inspector.back')]),
      el('button', { type: 'button', class: 'text-button', onclick: () => store.dispatch('duplicate', { ids: [e.id] }) }, [icon(ICONS.copy, 16), t('common.duplicate')]),
      el('button', { type: 'button', class: 'text-button danger', onclick: () => store.dispatch('remove', { ids: [e.id] }) }, [icon(ICONS.trash, 16), t('common.delete')]),
    ])]);
    actions.id = 'inspector-actions';
    actions.hidden = !actionsOpen;
    // Identify and preview the object, establish its context, then edit its
    // geometry. These areas stay visible independently of the active tab.
    return [
      header(e, widget, theme),
      actions,
      preview(e),
      section('membership', t('inspector.membership'), membershipForm(e, theme)),
      position,
      ...tabbed(kind, panelsOf(e, theme, assets)),
    ];
  }

  function multiForm(ids) {
    shownElement = null;
    return [
      el('h2', { text: t('inspector.multi', { count: ids.length }) }),
      section('position', t('inspector.alignTools'), [
        el('div', { class: 'button-row' }, ALIGN_EDGES.map(([edge, name]) => alignTool(ids, edge, ICONS[name]))),
        el('div', { class: 'button-row' }, [
          el('button', { type: 'button', class: 'text-button', disabled: ids.length < 3, onclick: () => store.dispatch('distribute', { ids, axis: 'x' }) }, [icon(ICONS.distributeX, 16), t('align.distributeX')]),
          el('button', { type: 'button', class: 'text-button', disabled: ids.length < 3, onclick: () => store.dispatch('distribute', { ids, axis: 'y' }) }, [icon(ICONS.distributeY, 16), t('align.distributeY')]),
        ]),
      ]),
      section('membership', t('inspector.membership'), [
        el('button', { type: 'button', class: 'text-button', text: t('group.create'), disabled: !canGroup(store.getState().theme, ids), title: t('group.scope'), onclick: () => store.dispatch('groupSelection', { ids }) }),
        ...(store.getState().theme.elements.some(e => ids.includes(e.id) && e.isGroup) ? [el('button', { type: 'button', class: 'text-button', text: t('group.ungroup'), onclick: () => store.dispatch('ungroup', { ids }) })] : []),
        el('button', { type: 'button', class: 'text-button', text: t('card.group'), onclick: () => store.dispatch('cardFromSelection', { ids }) }),
      ]),
      section('actions', t('inspector.actions'), [
        el('div', { class: 'actions' }, [
          el('button', { type: 'button', class: 'text-button', onclick: () => store.dispatch('duplicate', { ids }) }, [icon(ICONS.copy, 16), t('common.duplicate')]),
          el('button', { type: 'button', class: 'text-button danger', onclick: () => store.dispatch('remove', { ids }) }, [icon(ICONS.trash, 16), t('common.delete')]),
        ]),
      ]),
    ];
  }

  /** Renders for the current store state. */
  function render(assets = []) {
    closeSensorPicker();
    // A framing slider under the pointer stays; the form follows at its end.
    if (sliding && store.isGesturing()) return;
    sliding = false;
    const { theme, selection } = store.getState();
    const chosen = selection.map(id => theme.elements.find(e => e.id === id)).filter(Boolean);
    let nodes;
    if (chosen.length === 0) {
      shownElement = null;
      nodes = themeForm(theme, assets);
    } else {
      background.dismiss();
      nodes = chosen.length === 1 ? elementForm(chosen[0], theme, assets) : multiForm(chosen.map((e) => e.id));
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
    // Canvas animations need not render a static selected object again.
    // Isolated animations follow their own nextMs; new samples still apply.
    if (host.lastSample() !== seenSample) requestPreview();
    refreshLive();
  }

  return { render, contextChanged, frameShown,
    storageChanged(screen, overview) {
      if (background.updateListing(screen, overview) && !store.getState().selection.length) render(shown.assets);
    },
  };
}
