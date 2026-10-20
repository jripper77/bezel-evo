// The theme's background in the inspector
// (D-2026-10-16-studio-redesign-fidelity-25, artboards BgCurrent,
// BgChooser, BgStates): one card for the background in use (preview, kind,
// file, what the kind means, where it is) with only that kind's options
// under it, and "Change background…", a panel inside the inspector with the
// three sources (one open at a time) that sets nothing until confirmed,
// in one undo step. The logic is in editor/background.js; the primitives
// come from ui/fields.js (D-14).
import { el, icon, svgEl } from './dom.js';
import { ICONS } from './icons.js';
import { count, iconButton, listCard, panelHead, pill, propertySection as section, segmented, toggleField as switchRow } from './fields.js';
import { errorText } from '../messages.js';
import {
  DEFAULT_BACKGROUND_COLOR, backgroundKind, backgroundOf, backgroundName, backgroundOptions, backgroundSource, backgroundStatus, chooserSources, chosenBackground,
  confirmKey, fileNameOf, formatDuration, imageFits, localCopyOf, mediaItems, mediumOf, moves, playsVideos, sameBackground, screenVideos,
  startPicks, storedOnScreen, uploadMedium, videoFacts, videoWindowIds,
} from '../editor/background.js';

/** The label of each kind on the card, and the line that says what it means. */
const KIND_TEXT = Object.freeze({
  color: Object.freeze({ type: 'insp.bgSolid', hint: 'insp.bgSolidHint', group: 'insp.bgSolid' }),
  image: Object.freeze({ type: 'insp.bgComputer', hint: 'insp.bgPcHint', group: 'bg.image' }),
  video: Object.freeze({ type: 'insp.bgComputer', hint: 'insp.bgPcHint', group: 'bg.video' }),
  deviceVideo: Object.freeze({ type: 'insp.bgScreenVideo', hint: 'insp.bgScreenHint', group: 'insp.bgScreenVideo' }),
});
/** Each source in the chooser: its title, what it means and its icon. */
const SOURCE_TEXT = Object.freeze({
  color: Object.freeze({ name: 'insp.bgSolid', hint: 'insp.bgSolidHint', icon: null }),
  computer: Object.freeze({ name: 'insp.bgComputer', hint: 'insp.bgPcHint', icon: 'monitor' }),
  screen: Object.freeze({ name: 'insp.bgScreenVideo', hint: 'insp.bgScreenHint', icon: 'screen' }),
});
/** The status dot's state (base.css `.status-dot`) for each tone. */
const DOT = Object.freeze({ ok: 'ok', warn: 'partial', error: 'error' });

/** A screen's short name: its model's, else its key. */
const screenName = (screen) => (screen?.models?.length === 1 ? screen.models[0].name : null) ?? screen?.key ?? '';

/** `#rrggbbaa` as a CSS colour (the theme's own colour, drawn as it is). */
const cssColor = (hex) => (/^#[0-9a-f]{8}$/i.test(hex) ? `#${hex.slice(1)}` : hex);

/**
 * @param {object} deps
 * @param {(key: string, params?: object) => string} deps.t
 * @param {object} deps.store
 * @param {object|null} deps.bridge the backend: what the screen stores (`managerOverview`, `managerThumbnail`)
 * @param {object} deps.video the video actions and their context (see ui/inspector.js)
 * @param {(label: string, hex: string, onChange: (hex: string) => void) => HTMLElement} deps.colorField
 * @param {(bg: object, assets: object[]) => Node[]} deps.videoOptions a video's own options (status, framing)
 * @param {(n: number, locale: string) => string} deps.bytes
 * @param {() => void} deps.redraw draws the inspector again
 */
export function createBackgroundPanel({ t, store, bridge, video, colorField, videoOptions, bytes, redraw }) {
  /** The chooser while open: the open source and each source's pick. */
  let chooser = null;
  /** The last solid colour seen, to start from when coming back to a colour. */
  let lastColor = null;
  /** What the connected screen stores (`managerOverview`), for the screen and context it was read in. */
  let listing = { key: null, overview: null, loading: false, error: null };
  /** Thumbnails of stored files, by `screen|path`: a data URL or null. */
  const thumbs = new Map();

  const setBackground = (background) => store.dispatch('setTheme', { patch: { background } });

  // ------------------------------------------------------------- listing --
  /** The listing's identity: the screen, and live mode and its video (a change may mean a file came or went). */
  function listingKey(context) {
    return context.screen ? JSON.stringify([context.screen.key, context.live, context.liveVideo?.state ?? null]) : null;
  }

  /** Reads what the screen stores, once per screen and context (again when `force`). */
  function ensureListing(context, { force = false } = {}) {
    const key = listingKey(context);
    if (!key || !bridge?.managerOverview || !playsVideos(context.screen)) return;
    if (!force && listing.key === key) return;
    const request = { key, overview: force && listing.key === key ? listing.overview : null, loading: true, error: null };
    listing = request;
    const screenKey = context.screen.key;
    bridge.managerOverview(screenKey).then((overview) => {
      if (listing !== request) return;
      listing = { key, overview, loading: false, error: null };
      redraw();
    }, (error) => {
      if (listing !== request) return;
      listing = { key, overview: null, loading: false, error };
      redraw();
    });
  }

  /** The thumbnail of a stored file (from its local copy), once read. */
  function thumbOf(screenKey, path) {
    const id = `${screenKey}|${path}`;
    if (thumbs.has(id)) return thumbs.get(id);
    thumbs.set(id, null);
    bridge?.managerThumbnail?.(screenKey, path).then((url) => {
      if (!url) return;
      thumbs.set(id, url);
      redraw();
    }, () => {});
    return null;
  }

  /** What the listing says about `path`: the file, whether it is stored, its local copy. */
  function storedFacts(context, path) {
    const current = listing.key === listingKey(context) ? listing.overview : null;
    return {
      file: screenVideos(current).find((f) => f.path === path) ?? null,
      stored: storedOnScreen(current, path),
      copy: localCopyOf(current, path),
    };
  }

  // ---------------------------------------------------------------- card --
  /** The preview on the card: the poster or picture, a screen video's thumbnail, or the colour. */
  function preview(bg, assets, context, facts) {
    const kind = backgroundKind(bg);
    const node = el('span', { class: 'thumb bg-preview', dataset: { kind }, 'aria-hidden': 'true' });
    let url = null;
    if (kind === 'image') url = assets.find((a) => a.ref === bg.asset)?.dataUrl ?? null;
    if (kind === 'video') url = bg.poster ? assets.find((a) => a.ref === bg.poster)?.dataUrl ?? null : null;
    if (kind === 'deviceVideo' && context.screen && facts?.file) url = thumbOf(context.screen.key, bg.path);
    if (kind === 'color') node.style.backgroundColor = cssColor(bg.color);
    else if (url) node.style.backgroundImage = `url(${url})`;
    else node.append(icon(kind === 'image' ? ICONS.image : ICONS.film, 22));
    const caption = previewCaption(bg, assets, context, facts);
    if (caption) node.append(el('span', { class: 'bg-preview-facts', text: caption }));
    return node;
  }

  /** The mono caption on the preview: a video's play time (and size), as known. */
  function previewCaption(bg, assets, context, facts) {
    if (bg.type === 'video') {
      const asset = assets.find((a) => a.ref === bg.asset) ?? { ref: bg.asset };
      return videoFacts(asset, (n) => bytes(n, context.locale)).join(' · ') || null;
    }
    if (bg.type === 'deviceVideo') return formatDuration(facts?.file?.entry?.durationMs) ?? null;
    return null;
  }

  /** The status line: a dot and where the video is. */
  function statusLine(bg, context, facts) {
    const status = backgroundStatus(bg, { ...context, stored: facts?.stored ?? null });
    if (!status) return null;
    return el('p', { class: 'bg-status', dataset: { tone: status.tone } }, [
      el('span', { class: 'status-dot', 'aria-hidden': 'true', dataset: DOT[status.tone] ? { state: DOT[status.tone] } : {} }),
      el('span', { text: t(status.key, { screen: screenName(context.screen) }) }),
    ]);
  }

  /** The background in use, as one card. */
  function card(bg, assets, context) {
    const kind = backgroundKind(bg);
    const text = KIND_TEXT[kind];
    const facts = kind === 'deviceVideo' ? storedFacts(context, bg.path) : null;
    const missing = kind === 'deviceVideo' && playsVideos(context.screen) && backgroundStatus(bg, { ...context, stored: facts?.stored }).key === 'insp.bgNotOnScreen';
    return el('div', { class: 'bg-current', role: 'group', 'aria-label': t(text.group), dataset: { kind } }, [
      preview(bg, assets, context, facts),
      el('div', { class: 'bg-current-body' }, [
        el('span', { class: 'badge bg-type', dataset: { source: backgroundSource(bg) } }, [icon(ICONS[SOURCE_TEXT[backgroundSource(bg)].icon ?? 'background'], 13), t(text.type)]),
        // A colour is shown once: its own field, where it is also changed.
        kind === 'color'
          ? colorField(t('inspector.color'), bg.color, (color) => setBackground({ type: 'color', color }))
          : el('strong', { class: 'bg-name', text: backgroundName(bg) }),
        el('p', { class: 'hint', text: t(text.hint) }),
        statusLine(bg, context, facts),
        missing && facts?.copy ? el('button', {
          type: 'button', id: 'bg-send-stored', class: 'primary-button', text: t('insp.bgSendFile', { name: backgroundName(bg) }),
          onclick: () => video.restoreStored(bg.path, mediumOf(bg.path)),
        }) : null,
        el('button', { type: 'button', id: 'bg-change', class: 'text-button bg-change', text: t('insp.bgChange'), onclick: () => open(bg) }),
      ]),
    ]);
  }

  // ------------------------------------------------------------- options --
  /** A picture's fit: Fill or Fit, and the theme's own when it is another. */
  function fitOptions(bg) {
    const fit = bg.fit ?? 'fill';
    const label = (f) => (f === 'cover' || f === 'contain' ? t(`framing.${f}`) : t(`fit.${f}`));
    return [segmented(t('inspector.fit'), fit, imageFits(fit).map((f) => [f, label(f)]), (v) => v !== fit && setBackground({ ...bg, fit: v }))];
  }

  /** The video windows of a screen video: what they are, the colour outside them, how many, and selecting them. */
  function windowsBox(bg, theme) {
    const ids = videoWindowIds(theme);
    const countId = 'bg-windows-count';
    const drawing = svgEl('svg', { class: 'bg-windows-art', viewBox: '0 0 120 36', width: 120, height: 36, 'aria-hidden': 'true' });
    const screen = svgEl('rect', { class: 'bg-windows-screen', x: 0.5, y: 0.5, width: 119, height: 35, rx: 3 });
    screen.style.fill = cssColor(bg.color ?? DEFAULT_BACKGROUND_COLOR);
    drawing.append(screen, svgEl('rect', { class: 'bg-windows-window', x: 8, y: 8, width: 34, height: 20, rx: 3 }), svgEl('rect', { class: 'bg-windows-window', x: 78, y: 8, width: 34, height: 20, rx: 3 }));
    const many = ids.length === 1 ? t('insp.bgWindowsOne') : t('insp.bgWindowsMany', { count: ids.length });
    return el('div', { class: 'bg-windows', role: 'group', 'aria-label': t('deviceVideo.outsideColor') }, [
      el('div', { class: 'bg-windows-explain' }, [drawing, el('p', { class: 'hint', text: t('insp.bgWindowsHint') })]),
      ids.length ? colorField(t('deviceVideo.outsideColor'), bg.color ?? DEFAULT_BACKGROUND_COLOR, (color) => setBackground({ ...bg, color })) : null,
      el('div', { class: 'bg-windows-count' }, [
        el('span', { id: countId, text: ids.length ? many : t('insp.bgWindowsNone') }),
        ids.length ? el('button', { type: 'button', class: 'text-button quiet bg-select', id: 'bg-select-windows', 'aria-describedby': countId, text: t('insp.bgSelect'), onclick: () => store.select(ids) }) : null,
      ]),
    ]);
  }

  /** A video stored on the screen: loop, the video windows, where to manage it. */
  function screenOptions(bg) {
    const theme = store.getState().theme;
    return [
      switchRow(t('deviceVideo.loop'), bg.looping !== false, (looping) => setBackground({ ...bg, looping })),
      windowsBox(bg, theme),
      el('p', { class: 'hint bg-note' }, [icon(ICONS.info, 14), el('span', { text: t('deviceVideo.preview') })]),
      el('button', { type: 'button', id: 'bg-open-storage', class: 'text-button bg-link', onclick: () => video.openStorage() }, [icon(ICONS.sdCard, 16), t('insp.bgOpenStorage')]),
    ];
  }

  /** The options of the kind in use, and only those. */
  function options(bg, assets) {
    const [first] = backgroundOptions(bg);
    if (first === 'color') return [];
    if (first === 'fit') return fitOptions(bg);
    if (first === 'videoStatus') return videoOptions(bg, assets);
    return screenOptions(bg);
  }

  /** The Background section: the card, then the options of its kind. */
  function backgroundSection(bg, assets) {
    if (backgroundKind(bg) === 'color') lastColor = bg.color;
    const context = video.context();
    if (bg.type === 'deviceVideo') ensureListing(context);
    return section('background', t('common.background'), [card(bg, assets, context), ...options(bg, assets)]);
  }

  // ------------------------------------------------------------- chooser --
  /** Opens the chooser on the source in use, its file picked. */
  function open(bg) {
    chooser = { source: backgroundSource(bg), picks: startPicks(bg, lastColor), screenKey: video.context().screen?.key ?? null, picking: false, committing: false, file: null };
    if (chooser.source === 'screen') ensureListing(video.context(), { force: true });
    redraw();
    document.getElementById('bg-chooser-back')?.focus();
  }

  /** Closes the chooser without a change; the focus goes back to "Change background…". */
  function close({ refocus = true } = {}) {
    if (!chooser) return;
    chooser = null;
    redraw();
    if (refocus) document.getElementById('bg-change')?.focus();
  }

  /** Sets the chosen background (one undo step) and closes. */
  async function confirm() {
    const pending = chooser;
    let next = pendingBackground();
    if (!next || pending.committing) return;
    if (pending.source === 'computer' && pending.file?.ref === pending.picks.computer) {
      pending.committing = true;
      redraw();
      const added = await video.importFile(pending.file.source);
      if (chooser !== pending) return;
      pending.committing = false;
      if (!added) { redraw(); return; }
      next = backgroundOf(added);
    }
    chooser = null;
    if (!sameBackground(next, store.getState().theme.background)) setBackground(next);
    else redraw();
    document.getElementById('bg-change')?.focus();
  }

  function toggle(source, context) {
    chooser.source = chooser.source === source ? null : source;
    if (chooser.source === 'screen') ensureListing(context, { force: true });
    redraw();
  }

  /** Picks a file in the native dialog (pictures, GIFs and videos): it remains a draft, outside the theme's media, until confirmed. */
  async function pickFile() {
    const pending = chooser;
    if (pending.picking) return;
    pending.picking = true;
    redraw();
    const added = await video.pickFile();
    pending.picking = false;
    if (chooser !== pending) return;
    if (!added) { redraw(); return; }
    chooser.file = added;
    chooser.picks.computer = added.ref;
    chooser.source = 'computer';
    redraw();
  }

  /**
   * A pick changed: only the foot (old → new, the confirm button) and the
   * colour tile follow, so the control in use keeps the focus.
   */
  function pickChanged() {
    const shown = document.querySelector('.bg-chooser-foot');
    if (!shown || !chooser.shown) {
      redraw();
      return;
    }
    shown.replaceWith(foot(chooser.shown.bg));
    const tile = document.querySelector('.bg-source-icon[data-source="color"]');
    if (tile) tile.style.backgroundColor = cssColor(chooser.picks.color);
  }

  /** One radio row of a list in the chooser. */
  function radioRow(name, value, checked, { title, meta, thumb, current }) {
    const input = el('input', { type: 'radio', name, value, checked, disabled: chooser.committing });
    input.addEventListener('change', () => {
      chooser.picks[name === 'bg-pick-computer' ? 'computer' : 'screen'] = value;
      pickChanged();
    });
    return el('label', { class: 'bg-option' }, [
      input,
      thumb,
      el('span', { class: 'bg-option-text' }, [el('span', { class: 'bg-option-title', text: title }), meta ? el('span', { class: 'bg-option-meta', text: meta }) : null]),
      current ? el('span', { class: 'bg-option-current', text: t('insp.bgCurrent') }) : null,
    ]);
  }

  /** The colour source: a colour and its opacity. */
  const colorBody = () => [colorField(t('inspector.color'), chooser.picks.color, (color) => {
    chooser.picks.color = color;
    pickChanged();
  })];

  /** The computer source: the file picker, and the pictures and videos already in the theme. */
  function computerBody(bg, assets, context) {
    const items = mediaItems(assets);
    const kindText = (a) => (a.kind === 'video' ? t('common.video') : moves(a) ? t('media.animatedGif') : t('common.image'));
    const thumbOfAsset = (a) => {
      const url = (moves(a) && a.poster ? assets.find((p) => p.ref === a.poster)?.dataUrl : a.dataUrl) ?? null;
      const node = el('span', { class: 'bg-option-thumb', 'aria-hidden': 'true' });
      if (url) node.style.backgroundImage = `url(${url})`;
      else node.append(icon(moves(a) ? ICONS.film : ICONS.image, 14));
      return node;
    };
    return [
      el('button', { type: 'button', id: 'bg-pick-file', class: 'text-button bg-pick', disabled: chooser.picking || chooser.committing, onclick: () => void pickFile() }, [icon(ICONS.upload, 16), t('insp.bgPickFile')]),
      chooser.file ? radioRow('bg-pick-computer', chooser.file.ref, chooser.picks.computer === chooser.file.ref, {
        title: fileNameOf(chooser.file.ref), meta: kindText(chooser.file), thumb: thumbOfAsset(chooser.file), current: false,
      }) : null,
      items.length ? el('fieldset', { class: 'bg-options' }, [
        el('legend', { class: 'bg-options-head' }, [t('media.thisTheme'), count(items.length)]),
        ...items.map((a) => radioRow('bg-pick-computer', a.ref, chooser.picks.computer === a.ref, {
          title: fileNameOf(a.ref),
          meta: [kindText(a), ...videoFacts(a, (n) => bytes(n, context.locale))].join(' · '),
          thumb: thumbOfAsset(a),
          current: backgroundSource(bg) === 'computer' && bg.asset === a.ref,
        })),
      ]) : el('p', { class: 'hint', text: t('media.empty') }),
    ];
  }

  /** The screen source: the videos it stores, and sending one more. */
  function screenBody(bg, context) {
    const nodes = [];
    if (listing.loading && !listing.overview) nodes.push(el('p', { class: 'loading-note', text: t('storage.loading') }));
    else if (listing.error) nodes.push(el('p', { class: 'hint', role: 'alert', text: t('storage.loadError', { message: errorText(t, listing.error) }) }));
    else {
      const videos = screenVideos(listing.overview);
      nodes.push(videos.length ? el('fieldset', { class: 'bg-options' }, [
        el('legend', { class: 'bg-options-head' }, [t('insp.bgScreenList', { screen: screenName(context.screen) }), count(videos.length)]),
        ...videos.map((f) => radioRow('bg-pick-screen', f.path, chooser.picks.screen === f.path, {
          title: f.name,
          meta: [t(`storage.medium.${f.medium}`), formatDuration(f.entry?.durationMs), typeof f.size === 'number' ? bytes(f.size, context.locale) : null].filter(Boolean).join(' · '),
          thumb: el('span', { class: 'bg-option-thumb', 'aria-hidden': 'true' }, [icon(ICONS.film, 14)]),
          current: bg.type === 'deviceVideo' && bg.path === f.path,
        })),
      ]) : el('p', { class: 'hint', text: t('insp.bgNoVideos') }));
    }
    nodes.push(el('button', {
      type: 'button', id: 'bg-send-video', class: 'text-button bg-send', disabled: !listing.overview,
      onclick: () => {
        const medium = uploadMedium(listing.overview);
        close({ refocus: false });
        video.sendToScreen(medium);
      },
    }, [icon(ICONS.upload, 14), t('insp.bgSendVideo')]));
    return nodes;
  }

  /** One source of the chooser: its head (expands it) and, when open, its body. */
  function sourceCard({ source, disabled }, bg, assets, context) {
    const text = SOURCE_TEXT[source];
    const isOpen = chooser.source === source && !disabled;
    const inUse = backgroundSource(bg) === source;
    const bodyId = `bg-source-${source}-body`;
    const reasonId = `bg-source-${source}-reason`;
    const tile = el('span', { class: 'bg-source-icon', dataset: { source }, 'aria-hidden': 'true' }, text.icon ? [icon(ICONS[text.icon], 18)] : []);
    if (source === 'color') tile.style.backgroundColor = cssColor(chooser.picks.color);
    const head = listCard({ title: t(text.name), subtitle: t(text.hint), onClick: () => toggle(source, context) });
    head.classList.add('bg-source-head');
    head.id = `bg-source-${source}`;
    head.disabled = Boolean(disabled) || chooser.committing;
    head.setAttribute('aria-expanded', String(isOpen));
    if (isOpen) head.setAttribute('aria-controls', bodyId);
    if (disabled) head.setAttribute('aria-describedby', reasonId);
    head.prepend(tile);
    head.querySelector('.list-card-text').classList.add('bg-source-text');
    const title = head.querySelector('.list-card-title');
    title.classList.add('bg-source-title');
    if (inUse) title.append(pill(t('insp.bgInUse'), { tone: 'info', attrs: { class: 'pill bg-in-use' } }));
    head.querySelector('.list-card-sub').classList.add('bg-source-hint');
    head.lastElementChild.replaceWith(icon(isOpen ? ICONS.chevronDown : ICONS.chevronRight, 16));
    let body = null;
    if (isOpen) {
      const content = source === 'color' ? colorBody() : source === 'computer' ? computerBody(bg, assets, context) : screenBody(bg, context);
      body = el('div', { class: 'bg-source-body', id: bodyId }, content);
    }
    return el('div', { class: 'bg-source', dataset: { source, open: String(isOpen), inUse: String(inUse) } }, [
      head,
      disabled ? el('p', { class: 'hint bg-source-reason', id: reasonId, text: t(disabled) }) : null,
      body,
    ]);
  }

  /** A pending screen choice belongs to this screen and its latest successful listing. */
  function pendingBackground() {
    if (!chooser?.shown || !chooser.source) return null;
    const { bg, assets } = chooser.shown;
    const context = video.context();
    if (chooser.source === 'screen') {
      if (!playsVideos(context.screen) || chooser.screenKey !== context.screen.key || listing.key !== listingKey(context) || listing.loading || listing.error) return null;
      if (!screenVideos(listing.overview).some((file) => file.path === chooser.picks.screen)) return null;
    }
    return chosenBackground(bg, chooser.source, chooser.picks[chooser.source], chooser.file ? [...assets, chooser.file] : assets);
  }

  /** The footer compares the old and new backgrounds and confirms the draft. */
  function foot(bg) {
    const next = pendingBackground();
    const changed = Boolean(next) && !sameBackground(next, bg);
    const line = changed
      ? el('p', { class: 'bg-change-line' }, [
        el('s', { text: backgroundName(bg) }), icon(ICONS.arrowRight, 14),
        el('span', { class: 'visually-hidden', text: ` ${t('insp.bgReplacedBy')} ` }), el('strong', { text: backgroundName(next) }),
      ])
      : el('p', { class: 'hint bg-change-line', text: t('insp.bgNoChange') });
    return el('div', { class: 'bg-chooser-foot' }, [
      line,
      el('div', { class: 'bg-chooser-actions' }, [
        el('button', { type: 'button', id: 'bg-cancel', class: 'text-button', text: t('dialog.cancel'), onclick: () => close() }),
        el('button', { type: 'button', id: 'bg-confirm', class: 'primary-button', disabled: !changed || chooser.committing, text: t(confirmKey(next, chooser.source)), onclick: () => void confirm() }),
      ]),
    ]);
  }

  /** The shared panel head, with Back before the title. */
  function chooserHead() {
    const head = panelHead(t('insp.bgChooserTitle'), { id: 'bg-chooser-title' });
    head.classList.add('bg-chooser-head');
    head.prepend(iconButton(t('insp.back'), ICONS.chevronLeft, () => close(), { class: 'quiet', id: 'bg-chooser-back' }));
    return head;
  }

  /** The chooser, in place of the theme's form. */
  function chooserPanel(bg, assets) {
    const context = video.context();
    const key = context.screen?.key ?? null;
    if (chooser.screenKey !== key) {
      chooser.screenKey = key;
      chooser.picks.screen = null;
    }
    if (chooser.source === 'screen') ensureListing(context);
    chooser.shown = { bg, assets };
    const panel = el('div', { class: 'bg-chooser', role: 'region', 'aria-labelledby': 'bg-chooser-title' }, [
      chooserHead(),
      el('p', { class: 'hint bg-chooser-intro', text: t('insp.bgChooserIntro') }),
      el('div', { class: 'bg-sources' }, chooserSources(context.screen).map((s) => sourceCard(s, bg, assets, context))),
      foot(bg),
    ]);
    panel.addEventListener('keydown', (evt) => {
      if (evt.key !== 'Escape') return;
      evt.preventDefault();
      evt.stopPropagation();
      close();
    });
    return [panel];
  }

  return {
    /** Reuse the Storage panel's fresh bridge answer after an upload, restore or delete. */
    updateListing(screen, overview) {
      const context = video.context();
      if (context.screen?.key !== screen) return false;
      listing = { key: listingKey(context), overview, loading: false, error: null };
      thumbs.clear();
      return Boolean(chooser) || store.getState().theme.background.type === 'deviceVideo';
    },
    section: backgroundSection,
    chooser: chooserPanel,
    /** Whether the chooser is open (it takes the theme form's place). */
    isOpen: () => chooser !== null,
    /** Something else is shown in the inspector: the chooser closes, nothing changed. */
    dismiss: () => { chooser = null; },
  };
}
