import { appliedRotation, canRotate, centerOf, toLocal, rotatePoint, rotatedBounds, snapAngle, normalizeDegrees } from '../editor/rotation.js';
import { canvasTarget, groupTarget, selectionRoots, isShown, withChildren, owners } from '../editor/cards.js';
import { editedFill, frameHandles, scaledFrames, scaledText, propertyHandles, editPropertyHandle, resetPropertyHandle, textPartAt } from '../editor/handles.js';
import { canvasMeasure } from '../demo-render.js';
import { boundKey } from '../editor/widgets.js';
// The canvas: shows the rendered frame at a zoom level and handles direct
// manipulation on a DOM overlay (selection, eight resize handles, snapping
// guides, marquee). Everything is in canvas pixels until drawn. In framing
// mode the overlay frames the video background instead
// (D-2026-10-01-video-background-framing-6): drag pans it, the wheel zooms
// around the pointer, keys nudge, zoom, reset and leave; each drag or wheel
// burst is one undo step, and no element can be selected meanwhile.
import { el, icon } from './dom.js';
import { segmented, iconButton } from './fields.js';
import { NATURAL_REPEATS } from './inspector/art.js';
import { ICONS } from './icons.js';
import { boxFromPoints, hitTest, marqueeSelect, resize, unionBox } from '../editor/geometry.js';
import { alignmentDistances, snapEdge, snapMove } from '../editor/snap.js';
import { applyFramingKey, createBurst, framingKey, framingOf, panned, pictureBox, reframed, resolvedRotation, wheelPixels, wheelZoom, zoomedAt } from '../editor/video-framing.js';

const MIN_ZOOM = 0.05;
const MAX_ZOOM = 8;
const ZOOM_STEP = 1.25;
/** Pointer travel (screen px) before a press becomes a drag. */
const DRAG_THRESHOLD = 3;
/** The floating object bar's height, CSS px (shell.css .canvas-object-bar). */
const BAR_HEIGHT = 36;
/** Half its widest width (sensor, Try, Duplicate and the menu). */
const BAR_HALF_WIDTH = 120;

/**
 * @param {object} opts
 * @param {ReturnType<import('../editor/store.js').createStore>} opts.store
 * @param {HTMLElement} opts.scroll scroll container of the stage
 * @param {HTMLElement} opts.box sized box holding canvas + overlay
 * @param {HTMLCanvasElement} opts.canvas
 * @param {HTMLElement} opts.overlay
 * @param {(zoom:number) => void} [opts.onZoom]
 * @param {(label:string) => string} [opts.describe] accessible label of an element box
 * @param {() => void} [opts.onFrameRequest] a double click on the video background, off any element
 */
export function createCanvasView({ store, scroll, box, canvas, overlay, onZoom = () => {}, describe = (n) => n, onFrameRequest = () => {}, onFitContent = () => {}, rotationLabel = () => '', handleLabel = key => key, handleLocale = () => navigator.language, handleMotion = () => true, onHandleEdit = () => {}, textParts = () => null, onTextPart = () => {}, onBarSensor = () => {}, onBarTry = () => {}, onBarMenu = () => {}, onTrialHold = () => {}, onGesture = () => {} }) {
  const ctx = canvas.getContext('2d');
  const bezel = box.closest('.bezel') ?? box;
  const stageFrame = bezel.closest('.stage-frame') ?? bezel;
  const editingSpace = el('div', { class: 'editing-space' });
  bezel.before(editingSpace);
  editingSpace.append(bezel);
  let size = { width: canvas.width, height: canvas.height };
  let zoom = 1;
  let fitting = true;
  let wheelFrame = null;
  let wheelDelta = 0;
  let wheelClient = null;
  let guides = [];
  let marquee = null;
  let hoverId = null;
  let press = null;
  let handleReadout = null;
  let textEditor = null;
  let editingCard = null;
  let editingFill = null;
  // Where each bar's trial handle was let go: the next drag starts there.
  const trialFractions = new Map();
  const handleOptions = e => ({ motion: handleMotion(), fraction: trialFractions.get(e.id) ?? .5 });
  const handleFormats = new Map();
  const handleText = (value, unit) => {
    const locale = handleLocale();
    if (!handleFormats.has(locale)) handleFormats.set(locale, new Intl.NumberFormat(locale, { maximumFractionDigits: 2 }));
    return `${(Array.isArray(value) ? value : [value]).map(number => handleFormats.get(locale).format(number)).join(' · ')}${unit ? ` ${unit}` : ''}`;
  };
  const showHandleValue = (position, value, unit, field) => {
    handleReadout = { position, text: handleText(value, unit) };
    onHandleEdit({ field });
  };
  const clearHandleValue = () => {
    handleReadout = null; onHandleEdit(null);
    overlay.querySelector('.handle-readout')?.remove();
  };

  function applyProperty(element, id, position, options = {}) {
    const next = editPropertyHandle(element, id, position, { ...handleOptions(element), ...options });
    if (next && next.trial !== undefined) {
      // A trial value is drawn, never stored: the document does not change.
      trialFractions.set(element.id, next.trial);
      onTrialHold({ element: element.id, fraction: next.trial });
      handleReadout = { position, text: handleText(next.value, next.unit) };
      onHandleEdit(next);
      return;
    }
    if (!next?.patch && !next?.kind) return;
    if (next.kind) store.dispatch('setKind', { id: element.id, kind: next.kind });
    else store.dispatch('patchElements', { patches: [{ id: element.id, patch: next.patch }] });
    handleReadout = { position, text: handleText(next.value, next.unit) };
    onHandleEdit(next);
  }
  // Framing mode: what Auto is, how the surface is named and described, and
  // how it is left (Esc, Enter); `null` while editing elements.
  let framing = null;

  const theme = () => store.getState().theme;
  const selection = () => store.getState().selection;
  // A burst of wheel turns is one gesture: one undo step.
  const fillBurst = createBurst({ begin: () => store.beginGesture(), end: () => store.endGesture() });
  const wheelBurst = createBurst({ begin: () => store.beginGesture(), end: () => store.endGesture() });

  /** The video background's framing and what it is drawn against, or `null` without a video. */
  function framingView() {
    const bg = theme().background;
    if (bg.type !== 'video') return null;
    const current = framingOf(bg);
    const auto = framing?.auto() ?? null;
    return { framing: current, view: { source: auto?.size ?? null, rotation: resolvedRotation(current, auto), canvas: size } };
  }

  /** Frames the video background with `next` (one store command), unless nothing changes. */
  function applyFraming(next) {
    const framed = reframed(theme().background, next);
    if (framed) store.dispatch('setTheme', { patch: { background: framed } });
  }

  function layout() {
    // Exact, not rounded: the box and the overlay scale by the same zoom.
    box.style.width = `${size.width * zoom}px`;
    box.style.height = `${size.height * zoom}px`;
    drawOverlay();
  }

  /**
   * The room the canvas has: the stage less its padding (the floating bars'
   * room), the bezel's frame and the legend under it.
   */
  function room() {
    const style = getComputedStyle(scroll);
    const frame = bezel.parentElement?.closest('.stage-frame');
    const legend = frame?.querySelector('.stage-legend');
    const below = legend ? legend.offsetHeight + (parseFloat(getComputedStyle(frame).rowGap) || 0) : 0;
    return {
      width: scroll.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight) - (bezel.offsetWidth - box.offsetWidth),
      height: scroll.clientHeight - parseFloat(style.paddingTop) - parseFloat(style.paddingBottom) - (bezel.offsetHeight - box.offsetHeight) - below,
    };
  }

  function resetViewPosition() {
    if (wheelFrame !== null) cancelAnimationFrame(wheelFrame);
    wheelFrame = null;
    wheelDelta = 0;
    stageFrame.style.justifySelf = '';
    stageFrame.style.alignSelf = '';
    stageFrame.style.marginLeft = '';
    stageFrame.style.marginTop = '';
    stageFrame.style.marginRight = '';
    stageFrame.style.marginBottom = '';
    scroll.scrollLeft = 0;
    scroll.scrollTop = 0;
  }

  function fit() {
    resetViewPosition();
    const space = room();
    const w = Math.max(space.width, 50);
    const h = Math.max(space.height, 50);
    zoom = Math.min(Math.max(Math.min(w / size.width, h / size.height), MIN_ZOOM), MAX_ZOOM);
    fitting = true;
    layout();
    onZoom(zoom);
  }

  function setZoom(z) {
    resetViewPosition();
    zoom = Math.min(Math.max(z, MIN_ZOOM), MAX_ZOOM);
    fitting = false;
    layout();
    onZoom(zoom);
  }

  function setCanvasSize(next) {
    if (next.width === size.width && next.height === size.height) return;
    size = { ...next };
    canvas.width = size.width;
    canvas.height = size.height;
    if (fitting) fit();
    else layout();
  }

  /** Draws a rendered frame (`{width, height, rgba}`). */
  function drawFrame(frame) {
    if (!frame.width || !frame.height) return;
    const logical = { width: frame.logicalWidth || frame.width, height: frame.logicalHeight || frame.height };
    setCanvasSize(logical);
    const pixels = new ImageData(frame.rgba, frame.width, frame.height);
    if (frame.width === logical.width && frame.height === logical.height) ctx.putImageData(pixels, 0, 0);
    else {
      const small = document.createElement('canvas');
      small.width = frame.width; small.height = frame.height;
      small.getContext('2d').putImageData(pixels, 0, 0);
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.drawImage(small, 0, 0, canvas.width, canvas.height);
    }
    updateClockTips();
  }

  /** Only clock tip positions follow a new frame; keep native button focus alive. */
  function updateClockTips() {
    const selected = selection();
    if (selected.length !== 1) return;
    const element = theme().elements.find(e => e.id === selected[0]);
    if (element?.kind?.content?.type !== 'clock' || !element.kind.content.analog) return;
    for (const handle of propertyHandles(element, { motion: handleMotion() })) {
      const node = overlay.querySelector(`[data-property-handle="${handle.id}"]`);
      if (node) Object.assign(node.style, { left: px(handle.x), top: px(handle.y) });
    }
  }

  /** Canvas coordinates of a client point. */
  function point(clientX, clientY) {
    const r = box.getBoundingClientRect();
    return { x: (clientX - r.left) / zoom, y: (clientY - r.top) / zoom };
  }

  /** What keeps the wheel for itself over the stage: its floating bars and form controls. */
  const WHEEL_KEEPERS = ['.canvas-edit-tools', '.canvas-card-faces', '.canvas-object-bar', '.stage-tools', '.stage-zoom', 'input', 'select', 'textarea'].join();

  function containsClient(clientX, clientY) {
    const r = box.getBoundingClientRect();
    return clientX >= r.left && clientX <= r.right && clientY >= r.top && clientY <= r.bottom;
  }

  const px = (v) => `${v * zoom}px`;
  const boxStyle = (f) => ({ left: px(f.x), top: px(f.y), width: px(f.width), height: px(f.height) });

  /** Framing mode's overlay: the turned picture's edges and a rule-of-thirds grid. */
  function drawFramingOverlay() {
    const fv = framingView();
    const nodes = [el('div', { class: 'framing-grid' })];
    if (fv) nodes.unshift(el('div', { class: 'framing-picture', style: boxStyle(pictureBox(fv.view.source, fv.view.rotation, fv.framing, size)) }));
    overlay.replaceChildren(...nodes);
  }

  function editingRoom() {
    if (press) return; // Do not move the canvas origin beneath an active gesture.
    const boxes = theme().elements.filter(e => isShown(theme(), e)).map(e => rotatedBounds(e.frame, appliedRotation(e)))
      .filter(f => [f.x, f.y, f.width, f.height].every(Number.isFinite));
    const bounds = unionBox([{ x: 0, y: 0, ...size }, ...boxes]);
    const gutter = distance => distance > 0 ? distance * zoom + 8 : 0;
    editingSpace.style.paddingLeft = `${gutter(-bounds.x)}px`;
    editingSpace.style.paddingTop = `${gutter(-bounds.y)}px`;
    editingSpace.style.paddingRight = `${gutter(bounds.x + bounds.width - size.width)}px`;
    editingSpace.style.paddingBottom = `${gutter(bounds.y + bounds.height - size.height)}px`;
  }

  function drawOverlay() {
    editingRoom();
    if (framing) {
      drawFramingOverlay();
      return;
    }
    if (textEditor) {
      const current = theme().elements.find(e => e.id === textEditor.id);
      if (selection().length === 1 && selection()[0] === textEditor.id && current?.kind?.content?.type === 'static' && !owners(theme(), current).some(e => e.locked)) return;
      finishTextEdit(false);
    }
    const t = theme();
    const sel = new Set(selection());
    const nodes = [];
    for (const e of t.elements) {
      const f = rotatedBounds(e.frame, appliedRotation(e));
      if (isShown(t, e) && !sel.has(e.id) && (f.x < 0 || f.y < 0 || f.x + f.width > size.width || f.y + f.height > size.height)) {
        nodes.push(el('div', { class: 'off-canvas-box', style: boxStyle(f), role: 'img', 'aria-label': describe(e.name) }, [el('span', { class: 'off-canvas-label', text: e.name })]));
      }
    }
    const hovered = t.elements.find((e) => e.id === hoverId && !sel.has(e.id) && isShown(t, e));
    if (hovered) nodes.push(el('div', { class: 'hover-box', style: { ...boxStyle(hovered.frame), transform: `rotate(${appliedRotation(hovered)}deg)` } }));
    const chosen = t.elements.filter((e) => sel.has(e.id) && (!e.cardMember || isShown(t, e)));
    for (const e of chosen) {
      nodes.push(el('div', { class: `sel-box${e.locked ? ' locked' : ''}`, style: { ...boxStyle(e.frame), transform: `rotate(${appliedRotation(e)}deg)` }, role: 'img', 'aria-label': describe(e.name) }));
    }
    const entered = t.elements.find(e => e.id === editingCard && e.card);
    if (editingCard !== null && (!entered || !isShown(t, entered) || !chosen.length || chosen.some(e => !owners(t, e).some(p => p.id === editingCard)))) editingCard = null;
    const card = editingCard !== null ? entered : chosen.length === 1 && chosen[0].card ? chosen[0] : null;
    if (card) {
      if (editingCard !== null) nodes.push(el('div', { class: 'hover-box card-edit-boundary', style: boxStyle(card.frame) }));
      const locked = owners(t, card).some(e => e.locked);
      const choices = segmented(handleLabel('card.tab.faces'), card.card.activeFace, card.card.faces.map((name, face) => [face, name]), face => {
        if (face !== card.card.activeFace) store.dispatch('cardFace', { id: card.id, face });
        overlay.querySelector(`[data-canvas-face="${face}"]`)?.focus();
      }, { bare: true });
      [...choices.querySelectorAll('button')].forEach((button, face) => { button.dataset.canvasFace = String(face); button.disabled = locked; });
      nodes.push(el('div', { class: 'canvas-card-faces', style: { left: px(card.frame.x), top: `calc(${px(card.frame.y + card.frame.height)} + 14px)` } }, [
        choices,
        iconButton(handleLabel('card.addFace'), ICONS.plus, () => {
          const face = card.card.faces.length;
          store.dispatch('addCardFace', { id: card.id, name: handleLabel('card.faceName', { number: face + 1 }) });
          overlay.querySelector(`[data-canvas-face="${face}"]`)?.focus();
        }, { disabled: locked || card.card.faces.length >= 16 }),
        editingCard !== null && iconButton(handleLabel('shell.exitCard'), ICONS.arrowLeft, leaveCard),
      ]));
    }
    const fill = chosen.length === 1 && chosen[0].id === editingFill && isShown(t, chosen[0]) && NATURAL_REPEATS.includes(chosen[0].kind.image?.repeat) && !owners(t, chosen[0]).some(e => e.locked) ? chosen[0] : null;
    if (editingFill !== null && !fill) { editingFill = null; fillBurst.flush(); }
    if (fill) nodes.push(el('div', { class: 'canvas-edit-tools', style: { left: px(fill.frame.x), top: `calc(${px(fill.frame.y + fill.frame.height)} + 14px)` } }, [
      el('span', { text: handleLabel('shell.editShapeImage'), title: handleLabel('shell.imageEditHint') }),
      iconButton(handleLabel('shell.finishImage'), ICONS.close, leaveFill),
    ]));
    const roots = selectionRoots(t, chosen.map(e => e.id));
    const editable = chosen.length > 0 && chosen.every(e => !owners(t, e).some(owner => owner.locked));
    const common = editable && !fill ? frameHandles(roots, zoom) : [];
    const commonFrame = unionBox(roots.map(e => e.frame));
    const angle = roots.length === 1 ? appliedRotation(roots[0]) : 0;
    if (chosen.length > 1) nodes.push(el('div', { class: 'sel-box selection-union', style: boxStyle(commonFrame) }));
    for (const handle of common.filter(handle => handle.symbol === 'size')) {
      const h = handle.id;
      nodes.push(el('button', { type: 'button', class: 'handle', 'aria-label': handleLabel(`shell.handle.resize.${h}`), dataset: { h }, style: { left: px(handle.x), top: px(handle.y), transform: `rotate(${angle}deg)` },
        onkeydown: evt => {
          if (!['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].includes(evt.key)) return;
          evt.preventDefault(); evt.stopPropagation();
          const step = evt.shiftKey ? 10 : 1;
          const dx = evt.key === 'ArrowLeft' ? -step : evt.key === 'ArrowRight' ? step : 0;
          const dy = evt.key === 'ArrowUp' ? -step : evt.key === 'ArrowDown' ? step : 0;
          const frame = resize(commonFrame, h, dx, dy, roots.length > 1, angle);
          const text = chosen.length === 1 && h === 'se' && evt.ctrlKey && scaledText(chosen[0], frame);
          if (text) {
            store.dispatch('patchElements', { patches: [{ id: chosen[0].id, patch: text }] });
            showHandleValue(handle, text.kind.style.size, 'px', 'inspector.size');
          } else applyFrames(roots, commonFrame, frame, angle);
          if (!text) showHandleValue(handle, [frame.width, frame.height], 'px', h.includes('e') || h.includes('w') ? 'inspector.width' : 'inspector.height');
          drawOverlay(); overlay.querySelector(`[data-h="${h}"]`)?.focus();
        },
        onblur: () => { if (!press) clearHandleValue(); },
      }));
    }
    if (chosen.length === 1 && editable && !fill) {
      const e = chosen[0];
      for (const handle of propertyHandles(e, handleOptions(e))) {
        const title = handleLabel(handle.field, handle.fieldParams);
        nodes.push(el('button', { type: 'button', class: `property-handle ${handle.symbol}`, dataset: { propertyHandle: handle.id },
          title, 'aria-label': title, style: { left: px(handle.x), top: px(handle.y) },
          onkeydown: evt => {
            if (!['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].includes(evt.key)) return;
            evt.preventDefault(); evt.stopPropagation();
            const current = propertyHandles(e, handleOptions(e)).find(h => h.id === handle.id);
            // A discrete handle (the image anchor) moves one position per key.
            const step = current.keyStep ?? { x: evt.shiftKey ? 10 : 1, y: evt.shiftKey ? 10 : 1 };
            applyProperty(e, handle.id, { x: current.x + (evt.key === 'ArrowLeft' ? -step.x : evt.key === 'ArrowRight' ? step.x : 0), y: current.y + (evt.key === 'ArrowUp' ? -step.y : evt.key === 'ArrowDown' ? step.y : 0) }, { shift: evt.shiftKey, alt: evt.altKey });
            drawOverlay(); overlay.querySelector(`[data-property-handle="${handle.id}"]`)?.focus();
          },
          onblur: () => { if (!press) { clearHandleValue(); if (handle.id === 'trial-value') onTrialHold(null); } },
        }));
      }
    }
    const rotationHandle = common.find(handle => handle.symbol === 'rotation');
    if (rotationHandle && chosen.every(e => canRotate(t, e))) {
      nodes.push(el('button', { type: 'button', class: 'rotate-handle', dataset: { rotate: 'selection' }, title: rotationLabel(), 'aria-label': rotationLabel(),
        style: { left: px(rotationHandle.x), top: px(rotationHandle.y), transform: `rotate(${angle}deg)` },
        onkeydown: evt => {
          if (!['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', 'Home'].includes(evt.key)) return;
          evt.preventDefault(); evt.stopPropagation();
          const delta = ['ArrowLeft', 'ArrowDown'].includes(evt.key) ? -1 : 1;
          const rotation = evt.key === 'Home' ? 0 : appliedRotation(chosen[0]) + delta * (evt.shiftKey ? 15 : 1);
          store.dispatch('rotateSelection', { ids: chosen.map(e => e.id), rotation });
          showHandleValue(rotationHandle, normalizeDegrees(rotation), '\u00b0', 'insp.rotation'); drawOverlay();
          overlay.querySelector('[data-rotate]')?.focus();
        },
        onblur: () => { if (!press) clearHandleValue(); },
      }));
    }
    if (chosen.length && !fill && !press?.started) nodes.push(objectBar(chosen, roots));
    for (const g of guides) {
      nodes.push(el('div', { class: `guide ${g.axis}`, style: g.axis === 'x' ? { left: px(g.position) } : { top: px(g.position) } }));
    }
    if (guides.length && commonFrame) for (const gap of alignmentDistances(commonFrame, othersThan(chosen.map(e => e.id)), guides)) {
      const text = handleText(gap.distance, 'px');
      nodes.push(el('span', { class: `guide-distance ${gap.axis}`, role: 'status', 'aria-label': handleLabel('shell.guideDistance', { value: text }),
        style: gap.axis === 'x' ? { left: px(gap.position), top: px((gap.from + gap.to) / 2) } : { top: px(gap.position), left: px((gap.from + gap.to) / 2) }, text }));
    }
    if (marquee) nodes.push(el('div', { class: 'marquee', style: boxStyle(marquee) }));
    if (handleReadout) nodes.push(el('span', { class: 'handle-readout', role: 'status', style: { left: `calc(${px(handleReadout.position.x)} + 14px)`, top: `calc(${px(handleReadout.position.y)} - 28px)` }, text: handleReadout.text }));
    overlay.replaceChildren(...nodes);
  }

  /**
   * The bar over the selection (D-2026-10-20-studio-ux-v2-10): its linked
   * sensor, Try, Duplicate and the object menu. Every action is an existing
   * command; Try plays the inspector's detached trial and saves nothing.
   */
  function objectBar(chosen, roots) {
    const top = unionBox(roots.map(e => rotatedBounds(e.frame, appliedRotation(e))));
    const single = chosen.length === 1 ? chosen[0] : null, key = single && boundKey(single);
    const refocus = name => overlay.querySelector(`[data-object-bar="${name}"]`)?.focus();
    const tag = (node, name) => { node.dataset.objectBar = name; return node; };
    // Above the selection unless the stage's toolbar or its top edge is in the way: then below it
    // (and below a Card's face tabs).
    const origin = box.getBoundingClientRect(), tools = scroll.parentElement?.querySelector('.stage-tools')?.getBoundingClientRect();
    const rise = origin.top + top.y * zoom - 36 - BAR_HEIGHT, middle = origin.left + (top.x + top.width / 2) * zoom;
    const blocked = tools && tools.height > 0 && rise < tools.bottom + 4 && middle + BAR_HALF_WIDTH > tools.left && middle - BAR_HALF_WIDTH < tools.right;
    const below = rise < scroll.getBoundingClientRect().top || blocked;
    const faces = single?.card || editingCard !== null ? 44 : 0;
    return el('div', { class: `canvas-object-bar${below ? ' below' : ''}`, role: 'toolbar', 'aria-label': handleLabel('menu.title'),
      style: { left: px(top.x + top.width / 2), top: below ? `calc(${px(top.y + top.height)} + ${14 + faces}px)` : `calc(${px(top.y)} - 36px)` } }, [
      key && tag(el('button', { type: 'button', class: 'canvas-object-sensor', title: handleLabel('shell.objectSensor', { key }), 'aria-label': handleLabel('shell.objectSensor', { key }),
        onclick: () => onBarSensor(single) }, [icon(ICONS.bolt, 14), el('span', { text: key })]), 'sensor'),
      single && tag(iconButton(handleLabel('insp.tryObject'), ICONS.play, () => onBarTry(single), { class: 'quiet', disabled: !handleMotion() }), 'try'),
      tag(iconButton(handleLabel('common.duplicate'), ICONS.copy, () => { store.dispatch('duplicate', { ids: chosen.map(e => e.id) }); refocus('duplicate'); }, { class: 'quiet' }), 'duplicate'),
      tag(iconButton(handleLabel('menu.title'), ICONS.more, evt => onBarMenu(evt.currentTarget, () => refocus('menu')), { class: 'quiet', 'aria-haspopup': 'menu', 'aria-expanded': 'false' }), 'menu'),
    ]);
  }

  // ------------------------------------------------------ interactions ----

  function othersThan(ids) {
    const set = new Set(withChildren(theme(), ids));
    return theme().elements.filter((e) => !set.has(e.id) && isShown(theme(), e)).map((e) => e.frame);
  }

  /** A press in framing mode: the drag that follows pans the video. */
  function framePress(evt) {
    const fv = framingView();
    if (!fv) return;
    overlay.focus({ preventScroll: true, focusVisible: false });
    const picture = pictureBox(fv.view.source, fv.view.rotation, fv.framing, size);
    press = { mode: 'frame', start: fv.framing, picture, client: [evt.clientX, evt.clientY], started: false };
    overlay.classList.add('grabbing');
  }

  /** The picture follows the pointer from where the drag began. */
  function onFrame(evt) {
    const dx = (evt.clientX - press.client[0]) / zoom;
    const dy = (evt.clientY - press.client[1]) / zoom;
    applyFraming({ position: panned(press.start, press.picture, size, dx, dy) });
  }

  function onPointerDown(evt) {
    if (evt.button !== 0 || press || evt.target.closest?.('.canvas-card-faces, .canvas-edit-tools, .canvas-object-bar')) return;
    if (textEditor?.node.contains(evt.target)) return;
    if (textEditor) finishTextEdit(true);
    const viewport = scroll.getBoundingClientRect();
    if (evt.clientX >= viewport.left + scroll.clientWidth || evt.clientY >= viewport.top + scroll.clientHeight) return;
    if (framing && !containsClient(evt.clientX, evt.clientY)) return;
    evt.preventDefault();
    // A press takes the keys to the stage without the keyboard's focus ring
    // (D-2026-10-15-studio-redesign-fidelity-22): only the keyboard shows it.
    if (!framing) { scroll.tabIndex = 0; scroll.focus({ preventScroll: true, focusVisible: false }); }
    scroll.setPointerCapture?.(evt.pointerId);
    if (framing) {
      framePress(evt);
      return;
    }
    const start = point(evt.clientX, evt.clientY);
    const handle = evt.target?.dataset?.h;
    const state = store.getState();
    if (editingFill !== null) {
      const element = state.theme.elements.find(e => e.id === editingFill);
      const local = toLocal(element.frame, appliedRotation(element), start);
      const f = element.frame;
      if (local.x >= f.x && local.y >= f.y && local.x <= f.x + f.width && local.y <= f.y + f.height) {
        fillBurst.flush();
        press = { mode: 'fill', element: structuredClone(element), start: local, client: [evt.clientX, evt.clientY], started: false };
        return;
      }
      leaveFill();
    }
    const property = evt.target.closest?.('[data-property-handle]')?.dataset.propertyHandle;
    if (property && state.selection.length === 1) {
      const element = state.theme.elements.find(e => e.id === state.selection[0]);
      press = { mode: 'property', element: structuredClone(element), handle: property, client: [evt.clientX, evt.clientY], started: false };
      return;
    }
    const chosen = state.theme.elements.filter(e => state.selection.includes(e.id));
    const editable = chosen.length > 0 && chosen.every(e => !owners(state.theme, e).some(owner => owner.locked));
    const roots = selectionRoots(state.theme, state.selection);
    if (evt.target?.dataset?.rotate && editable && chosen.every(e => canRotate(state.theme, e))) {
      const center = centerOf(unionBox(roots.map(e => e.frame)));
      press = { mode: 'rotate', ids: chosen.map(e => e.id), center, angle: appliedRotation(chosen[0]),
        pointerAngle: Math.atan2(start.y - center.y, start.x - center.x) * 180 / Math.PI,
        client: [evt.clientX, evt.clientY], started: false };
      return;
    }
    if (handle && editable) {
      press = { mode: 'resize', elements: structuredClone(roots), rotation: roots.length === 1 ? appliedRotation(roots[0]) : 0,
        scaleText: chosen.length === 1 && handle === 'se' && evt.ctrlKey && !!scaledText(chosen[0], chosen[0].frame),
        handle, frame: unionBox(roots.map(e => e.frame)), start, client: [evt.clientX, evt.clientY], started: false };
      return;
    }
    let id = hitTest(state.theme.elements.filter(e => isShown(state.theme, e)), start.x, start.y);
    if (id !== null && !evt.ctrlKey && !evt.metaKey && !state.selection.includes(id)) id = canvasTarget(state.theme, id, editingCard);
    if (id === null) {
      if (!evt.shiftKey && !evt.ctrlKey && !evt.metaKey) store.select([]);
      press = { mode: 'marquee', start, base: evt.shiftKey || evt.ctrlKey || evt.metaKey ? [...state.selection] : [], client: [evt.clientX, evt.clientY], started: false };
      return;
    }
    let sel = state.selection;
    if (evt.shiftKey || evt.ctrlKey || evt.metaKey) {
      sel = sel.includes(id) ? sel.filter((x) => x !== id) : [...sel, id];
      store.select(sel);
    } else if (!sel.includes(id)) {
      sel = [id];
      store.select(sel);
    }
    // A plain click on the one selected value or clock may pick the part under it.
    const part = !evt.shiftKey && !evt.ctrlKey && !evt.metaKey && state.selection.length === 1 && state.selection[0] === id ? id : null;
    const movable = selectionRoots(state.theme, sel).filter((e) => sel.includes(e.id) && !e.locked && (!e.cardMember || isShown(state.theme, e)));
    press = { mode: 'move', ids: movable.map((e) => e.id), frames: movable.map((e) => ({ ...e.frame })), start, applied: { x: 0, y: 0 }, part, client: [evt.clientX, evt.clientY], started: false };
  }

  function beginIfMoved(evt) {
    if (press.started) return true;
    const moved = Math.hypot(evt.clientX - press.client[0], evt.clientY - press.client[1]) >= DRAG_THRESHOLD;
    if (!moved) return false;
    press.started = true;
    if (press.mode !== 'marquee') store.beginGesture();
    return true;
  }

  function onMove(evt, p) {
    if (press.ids.length === 0) return;
    const dx = p.x - press.start.x;
    const dy = p.y - press.start.y;
    const union = unionBox(press.frames);
    const proposed = { ...union, x: union.x + dx, y: union.y + dy };
    const snapped = evt.altKey ? { box: proposed, guides: [] } : snapMove(proposed, size, othersThan(press.ids));
    guides = snapped.guides;
    const target = { x: Math.round(snapped.box.x - union.x), y: Math.round(snapped.box.y - union.y) };
    const change = { x: target.x - press.applied.x, y: target.y - press.applied.y };
    if (change.x !== 0 || change.y !== 0) {
      store.dispatch('move', { ids: press.ids, dx: change.x, dy: change.y });
      press.applied = target;
    }
  }

  function applyFrames(elements, from, to, rotation) {
    if (elements.length === 1) store.dispatch('setFrame', { id: elements[0].id, frame: to, precise: !!rotation });
    else store.dispatch('patchElements', { patches: scaledFrames(elements, from, to) });
  }

  function onResize(evt, p) {
    let next = resize(press.frame, press.handle, p.x - press.start.x, p.y - press.start.y, evt.shiftKey || press.elements.length > 1, press.rotation);
    guides = [];
    if (press.scaleText) {
      const patch = scaledText(press.elements[0], next);
      store.dispatch('patchElements', { patches: [{ id: press.elements[0].id, patch }] });
      showHandleValue(p, patch.kind.style.size, 'px', 'inspector.size');
      return;
    }
    if (!evt.altKey && !press.rotation && !(press.elements.length > 1 && press.handle.length === 2)) {
      const others = othersThan(press.elements.map(e => e.id));
      const h = press.handle;
      const snapX = (edge) => snapEdge(edge, 'x', size, others);
      const snapY = (edge) => snapEdge(edge, 'y', size, others);
      if (h.includes('e')) {
        const s = snapX(next.x + next.width);
        next = { ...next, width: s.value - next.x };
        if (s.guide !== null) guides.push({ axis: 'x', position: s.guide });
      } else if (h.includes('w')) {
        const s = snapX(next.x);
        next = { ...next, x: s.value, width: next.x + next.width - s.value };
        if (s.guide !== null) guides.push({ axis: 'x', position: s.guide });
      }
      if (h.includes('s')) {
        const s = snapY(next.y + next.height);
        next = { ...next, height: s.value - next.y };
        if (s.guide !== null) guides.push({ axis: 'y', position: s.guide });
      } else if (h.includes('n')) {
        const s = snapY(next.y);
        next = { ...next, y: s.value, height: next.y + next.height - s.value };
        if (s.guide !== null) guides.push({ axis: 'y', position: s.guide });
      }
    }
    applyFrames(press.elements, press.frame, next, press.rotation);
    showHandleValue(p, [next.width, next.height], 'px', press.handle.includes('e') || press.handle.includes('w') ? 'inspector.width' : 'inspector.height');
  }

  function onPointerMove(evt) {
    const p = point(evt.clientX, evt.clientY);
    if (!press) {
      const id = framing ? null : canvasTarget(theme(), hitTest(theme().elements.filter(e => isShown(theme(), e)), p.x, p.y), editingCard);
      if (id !== hoverId) {
        hoverId = id;
        drawOverlay();
      }
      return;
    }
    if (!beginIfMoved(evt)) return;
    if (press.mode === 'frame') onFrame(evt);
    else if (press.mode === 'move') onMove(evt, p);
    else if (press.mode === 'resize') onResize(evt, p);
    else if (press.mode === 'fill') {
      const local = toLocal(press.element.frame, appliedRotation(press.element), p);
      const next = editedFill(press.element.kind.image, { dx: local.x - press.start.x, dy: local.y - press.start.y });
      applyFill(press.element, next);
      showHandleValue(p, [next.offsetX, next.offsetY], 'px', 'shape.imageOffsetX');
    }
    else if (press.mode === 'property') applyProperty(press.element, press.handle, p, { shift: evt.shiftKey, alt: evt.altKey });
    else if (press.mode === 'rotate') {
      const pointerAngle = Math.atan2(p.y - press.center.y, p.x - press.center.x) * 180 / Math.PI;
      const angle = normalizeDegrees(press.angle + pointerAngle - press.pointerAngle);
      store.dispatch('rotateSelection', { ids: press.ids, rotation: evt.shiftKey ? snapAngle(angle, 15) : angle });
      showHandleValue(p, evt.shiftKey ? snapAngle(angle, 15) : angle, '°', 'insp.rotation');
    }
    else {
      marquee = boxFromPoints(press.start.x, press.start.y, p.x, p.y);
      const hits = marqueeSelect(theme().elements.filter(e => isShown(theme(), e)), marquee);
      store.select([...new Set([...press.base, ...hits.map(id => canvasTarget(theme(), id, editingCard))])]);
    }
    // The statusbar follows a move, a resize or a turn (D-2026-10-20-studio-ux-v2-11).
    if (['move', 'resize', 'rotate'].includes(press.mode)) onGesture({ mode: press.mode, guides });
    drawOverlay();
  }

  function onPointerUp() {
    // Keep the handle DOM alive across two stationary clicks so the browser
    // can deliver dblclick to the same frame (capture also ends here).
    const redraw = press?.started || guides.length > 0 || marquee !== null;
    const picked = press?.mode === 'move' && !press.started && press.part != null ? press : null;
    if (press?.mode === 'property' && press.handle === 'trial-value') onTrialHold(null);
    if (press?.started && press.mode !== 'marquee') store.endGesture();
    if (press?.started && ['move', 'resize', 'rotate'].includes(press.mode)) onGesture(null);
    press = null;
    guides = [];
    marquee = null;
    handleReadout = null;
    onHandleEdit(null);
    overlay.classList.remove('grabbing');
    if (redraw) drawOverlay();
    if (picked) pickTextPart(picked.part, picked.start);
  }

  /** The wheel in framing mode zooms the video around the pointer (Ctrl+wheel still zooms the view). */
  function onFramingWheel(evt) {
    if (!framing || evt.ctrlKey) return;
    const fv = framingView();
    if (!fv) return;
    evt.preventDefault();
    wheelBurst.touch();
    applyFraming(zoomedAt(fv.framing, fv.view, wheelZoom(fv.framing.zoom, wheelPixels(evt)), point(evt.clientX, evt.clientY)));
  }

  /** Accumulate fractional trackpad travel and mouse notches once per paint.
   * One 100px notch is the same step as the zoom buttons. No percent rounding:
   * even tiny pinch deltas contribute to the next zoom.
   */
  function onViewWheel(evt) {
    // The whole stage zooms (the dotted area too); its floating bars and form controls keep the wheel.
    if (evt.defaultPrevented || evt.target.closest?.(WHEEL_KEEPERS)) return;
    const delta = wheelPixels(evt);
    if (!delta) return;
    evt.preventDefault();
    wheelDelta += delta;
    wheelClient = { x: evt.clientX, y: evt.clientY };
    if (wheelFrame !== null) return;
    wheelFrame = requestAnimationFrame(() => {
      const client = wheelClient;
      const anchor = point(client.x, client.y);
      const next = zoom * ZOOM_STEP ** (-wheelDelta / 100);
      setZoom(next);
      const rect = box.getBoundingClientRect();
      placeBy(client.x - anchor.x * zoom - rect.left, client.y - anchor.y * zoom - rect.top);
    });
  }

  /** Moves the canvas by (`dx`, `dy`) CSS pixels inside the stage (the wheel's anchor, the middle-button pan). */
  function placeBy(dx, dy) {
    // Where the canvas is now, before any change of alignment.
    const before = box.getBoundingClientRect();
    // Start alignment plus a positive margin/scroll pair can place the
    // origin on either side of the viewport, including when the canvas is
    // smaller than it. Scroll alone would clamp and lose the cursor anchor.
    stageFrame.style.justifySelf = 'start';
    stageFrame.style.alignSelf = 'start';
    const after = box.getBoundingClientRect();
    // The offset the margin/scroll pair gives now, plus the move asked for
    // (relative to where the canvas was seen, whatever the alignment did).
    const x = (parseFloat(stageFrame.style.marginLeft) || 0) - scroll.scrollLeft + (before.left - after.left) + dx;
    const y = (parseFloat(stageFrame.style.marginTop) || 0) - scroll.scrollTop + (before.top - after.top) + dy;
    const left = Math.max(0, Math.ceil(-x));
    const top = Math.max(0, Math.ceil(-y));
    // Scroll offsets are integral in WebView2/Chromium. Keep the fractional
    // remainder in the margin so rounding cannot accumulate across turns.
    stageFrame.style.marginLeft = `${x + left}px`;
    stageFrame.style.marginTop = `${y + top}px`;
    // When shrinking near the viewport edge, retain enough trailing room
    // for the requested scroll offset instead of letting it clamp to zero.
    const viewport = scroll.getBoundingClientRect();
    const frame = stageFrame.getBoundingClientRect();
    const padding = getComputedStyle(scroll);
    stageFrame.style.marginRight = `${Math.max(0, left + scroll.clientWidth - (frame.right - viewport.left) - parseFloat(padding.paddingRight))}px`;
    stageFrame.style.marginBottom = `${Math.max(0, top + scroll.clientHeight - (frame.bottom - viewport.top) - parseFloat(padding.paddingBottom))}px`;
    scroll.scrollLeft = left;
    scroll.scrollTop = top;
  }

  /** The middle button drags the view (pan), from anywhere on the stage. */
  let pan = null;
  function onPanDown(evt) {
    if (evt.button !== 1) return;
    evt.preventDefault();
    pan = { id: evt.pointerId, x: evt.clientX, y: evt.clientY };
    scroll.setPointerCapture(evt.pointerId);
    scroll.classList.add('panning');
  }
  function onPanMove(evt) {
    if (!pan || evt.pointerId !== pan.id) return;
    placeBy(evt.clientX - pan.x, evt.clientY - pan.y);
    pan.x = evt.clientX;
    pan.y = evt.clientY;
  }
  function onPanUp(evt) {
    if (!pan || evt.pointerId !== pan.id) return;
    pan = null;
    scroll.classList.remove('panning');
  }

  /** Framing mode's keys: arrows, + and -, 0, Esc and Enter; the rest (undo, save) goes on to the editor. */
  function onFramingKey(evt) {
    if (!framing) return;
    const action = framingKey(evt);
    if (!action) return;
    evt.preventDefault();
    evt.stopPropagation();
    if (action.type === 'leave') {
      framing.onLeave();
      return;
    }
    const fv = framingView();
    if (fv) applyFraming(applyFramingKey(fv.framing, action, fv.view));
  }

  const partMeasure = canvasMeasure(new OffscreenCanvas(1, 1).getContext('2d'));
  function pickTextPart(id, at) {
    const element = theme().elements.find(e => e.id === id);
    const parts = element && textParts(element);
    const part = parts && textPartAt(element, parts.pieces, at, partMeasure);
    if (part) onTextPart(element, parts.scope, part);
  }

  function applyFill(element, image) {
    store.dispatch('patchElements', { patches: [{ id: element.id, patch: { kind: { image } } }] });
  }

  function leaveFill() {
    editingFill = null; fillBurst.flush(); clearHandleValue(); drawOverlay();
    scroll.focus({ preventScroll: true });
  }

  function onFillWheel(evt) {
    if (editingFill === null || evt.ctrlKey || evt.target.closest?.(WHEEL_KEEPERS)) return;
    evt.preventDefault();
    const element = theme().elements.find(e => e.id === editingFill);
    fillBurst.touch();
    const image = editedFill(element.kind.image, { scale: Math.exp(-wheelPixels(evt) / 500) });
    applyFill(element, image);
    showHandleValue(point(evt.clientX, evt.clientY), image.scale * 100, '%', 'shape.imageScale'); drawOverlay();
  }

  function leaveCard() {
    const id = editingCard;
    editingCard = null;
    if (id !== null && theme().elements.some(e => e.id === id)) store.select([id]);
    drawOverlay(); scroll.focus({ preventScroll: true });
  }

  function finishTextEdit(commit) {
    if (!textEditor) return;
    const { id, node, original } = textEditor;
    textEditor = null;
    node.remove();
    if (commit && node.value !== original) store.dispatch('patchElements', { patches: [{ id, patch: { kind: { content: { text: node.value } } } }] });
    drawOverlay();
  }

  function editText(element) {
    const original = element.kind.content.text;
    const node = el('textarea', { class: 'canvas-text-editor', text: original, 'aria-label': handleLabel('common.text'),
      title: handleLabel('shell.textEditHint'), spellcheck: false,
      style: { ...boxStyle(element.frame), transform: `rotate(${appliedRotation(element)}deg)`, fontSize: px(element.kind.style.size) },
      onkeydown: evt => {
        if (evt.key === 'Escape' || evt.key === 'Enter' && (evt.ctrlKey || evt.metaKey)) {
          evt.preventDefault(); evt.stopPropagation(); finishTextEdit(evt.key === 'Enter'); scroll.focus({ preventScroll: true });
        } else if (evt.key.toLowerCase() === 's' && (evt.ctrlKey || evt.metaKey)) finishTextEdit(true);
      },
      onblur: () => finishTextEdit(true),
    });
    textEditor = { id: element.id, node, original };
    overlay.replaceChildren(node); node.focus({ preventScroll: true }); node.select();
  }

  /** A double click on the video background, off any element, asks for framing mode. */
  function onDoubleClick(evt) {
    if (textEditor?.node.contains(evt.target) || evt.target.closest?.('.canvas-card-faces, .canvas-edit-tools, .canvas-object-bar')) return;
    if (framing) return;
    const state = store.getState();
    const property = evt.target.closest?.('[data-property-handle]')?.dataset.propertyHandle;
    if (property) {
      const element = state.theme.elements.find(e => e.id === state.selection[0]);
      const patch = resetPropertyHandle(element, property);
      if (patch) store.dispatch('patchElements', { patches: [{ id: element.id, patch }] });
      return;
    }
    if (state.selection.length === 1) {
      const e = theme().elements.find(e => e.id === state.selection[0]);
      const p = point(evt.clientX, evt.clientY), f = e?.frame;
      const tolerance = 6 / zoom;
      if (f && (evt.target.closest('.handle') ||
        p.x >= f.x - tolerance && p.x <= f.x + f.width + tolerance &&
        p.y >= f.y - tolerance && p.y <= f.y + f.height + tolerance &&
        Math.min(Math.abs(p.x - f.x), Math.abs(p.x - f.x - f.width),
          Math.abs(p.y - f.y), Math.abs(p.y - f.y - f.height)) <= tolerance)) {
        onFitContent(e.id);
        return;
      }
    }
    const p = point(evt.clientX, evt.clientY);
    const hit = hitTest(state.theme.elements.filter(e => isShown(state.theme, e)), p.x, p.y);
    const element = state.theme.elements.find(e => e.id === hit);
    const target = state.theme.elements.find(e => e.id === canvasTarget(state.theme, hit, editingCard));
    if (target?.card && !owners(state.theme, target).some(e => e.locked)) {
      editingCard = target.id;
      const member = hit !== target.id && owners(state.theme, element).some(e => e.id === target.id);
      store.select([member ? groupTarget(state.theme, hit) : target.id]); drawOverlay(); return;
    }
    if (element?.kind.type === 'shape' && element.kind.image?.asset && NATURAL_REPEATS.includes(element.kind.image.repeat) && !owners(state.theme, element).some(e => e.locked)) {
      store.select([element.id]); editingFill = element.id; drawOverlay(); return;
    }
    if (element?.kind.type === 'text' && element.kind.content.type === 'static' && !owners(state.theme, element).some(e => e.locked)) {
      store.select([element.id]); editText(element); return;
    }
    if (theme().background.type !== 'video' || !containsClient(evt.clientX, evt.clientY)) return;
    if (hitTest(theme().elements.filter(e => isShown(theme(), e)), p.x, p.y) === null) onFrameRequest();
  }

  /**
   * Framing mode on (`mode`) or off (`null`). On, the overlay is a focusable
   * surface named `mode.label` and described by the element `mode.describedBy`
   * (the keys); `mode.auto()` is what Auto is now (`video_auto`), and
   * `mode.onLeave()` is asked for by Esc or Enter.
   * @param {{auto: () => {rotation: number, size: {width: number, height: number}|null}|null, label: string, describedBy: string, onLeave: () => void}|null} mode
   */
  function setFraming(mode) {
    finishTextEdit(true);
    editingFill = null; fillBurst.flush();
    if (wheelFrame !== null) cancelAnimationFrame(wheelFrame);
    wheelFrame = null;
    wheelDelta = 0;
    wheelBurst.flush();
    if (press?.started && press.mode !== 'marquee') store.endGesture();
    framing = mode;
    press = null;
    hoverId = null;
    guides = [];
    marquee = null;
    overlay.classList.toggle('framing', Boolean(mode));
    overlay.classList.remove('grabbing');
    if (mode) {
      overlay.tabIndex = 0;
      overlay.setAttribute('role', 'application');
      overlay.setAttribute('aria-label', mode.label);
      overlay.setAttribute('aria-describedby', mode.describedBy);
    } else {
      for (const name of ['tabindex', 'role', 'aria-label', 'aria-describedby']) overlay.removeAttribute(name);
    }
    drawOverlay();
  }

  scroll.addEventListener('pointerdown', onPanDown);
  scroll.addEventListener('pointermove', onPanMove);
  scroll.addEventListener('pointerup', onPanUp);
  scroll.addEventListener('pointercancel', onPanUp);
  scroll.addEventListener('auxclick', (evt) => { if (evt.button === 1) evt.preventDefault(); });
  scroll.addEventListener('pointerdown', onPointerDown);
  scroll.addEventListener('pointermove', onPointerMove);
  scroll.addEventListener('pointerup', onPointerUp);
  scroll.addEventListener('pointercancel', onPointerUp);
  scroll.addEventListener('lostpointercapture', onPointerUp);
  scroll.addEventListener('pointerleave', () => {
    if (!press && hoverId !== null) {
      hoverId = null;
      drawOverlay();
    }
  });
  scroll.addEventListener('wheel', onFillWheel, { passive: false });
  overlay.addEventListener('wheel', onFramingWheel, { passive: false });
  overlay.addEventListener('keydown', onFramingKey);
  document.addEventListener('pointerdown', () => fillBurst.flush(), { capture: true });
  document.addEventListener('visibilitychange', () => { if (document.hidden) fillBurst.flush(); });
  scroll.addEventListener('keydown', evt => {
    if (evt.ctrlKey || evt.metaKey) fillBurst.flush();
    if (editingFill !== null && !evt.ctrlKey && !evt.metaKey) {
      if (['Enter', 'Escape'].includes(evt.key)) { evt.preventDefault(); evt.stopPropagation(); leaveFill(); return; }
      if (['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', '+', '=', '-'].includes(evt.key)) {
        evt.preventDefault(); evt.stopPropagation(); fillBurst.flush();
        const element = theme().elements.find(e => e.id === editingFill), step = evt.shiftKey ? 10 : 1;
        const image = editedFill(element.kind.image, { dx: evt.key === 'ArrowLeft' ? -step : evt.key === 'ArrowRight' ? step : 0,
          dy: evt.key === 'ArrowUp' ? -step : evt.key === 'ArrowDown' ? step : 0, scale: ['+', '='].includes(evt.key) ? 1.1 : evt.key === '-' ? 1 / 1.1 : 1 });
        applyFill(element, image);
        showHandleValue(centerOf(element.frame), evt.key.startsWith('Arrow') ? [image.offsetX, image.offsetY] : image.scale * 100, evt.key.startsWith('Arrow') ? 'px' : '%', evt.key.startsWith('Arrow') ? 'shape.imageOffsetX' : 'shape.imageScale');
        drawOverlay(); return;
      }
    }
    if (evt.key === 'Escape' && editingCard !== null && !textEditor) { evt.preventDefault(); evt.stopPropagation(); leaveCard(); }
  });
  scroll.addEventListener('dblclick', onDoubleClick);
  scroll.addEventListener('wheel', onViewWheel, { passive: false });
  new ResizeObserver(() => { if (fitting) fit(); }).observe(scroll);

  return {
    fit,
    setZoom,
    stepZoom: (steps) => setZoom(zoom * ZOOM_STEP ** steps),
    zoom: () => zoom,
    setCanvasSize,
    drawFrame,
    clearFrame: () => ctx.clearRect(0, 0, canvas.width, canvas.height),
    drawOverlay,
    point,
    containsClient,
    setFraming,
    framing: () => framing !== null,
    /** Focuses the framing surface (framing mode only). */
    focusFraming: () => framing && overlay.focus({ preventScroll: true }),
  };
}
