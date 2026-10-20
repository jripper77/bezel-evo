// Common existing inspector controls. Each builder has one owner.
import { visualization, visualStyles, visualArt, matrixFrame } from '../../visualization.js';
import { defaultSensorMotion, supportsSensorMotion } from '../../sensor-motion.js';
import { sensorNativeUnit } from '../../sensor-picker-model.js';
import { sensorPicker } from '../sensor-picker.js';
import { applyTextCase, TEXT_CASES } from '../../text-case.js';
import { MAX_COLOR_THRESHOLDS, addColorThreshold, defaultValueColor, supportsValueColor } from '../../value-color.js';
import { moveRule } from '../../card-rules.js';
import { formatClock } from '../../clock-format.js';
import { readIcon } from '../../icon-catalog.js';
import { el, icon } from '../dom.js';
import { ICONS } from '../icons.js';
import { chipButton, fieldGrid, fontPicker, numberField, cornerRadii, iconButton, prefixedField, rangeField, propertySection as section, segmented, selectField, switchRow, textField } from '../fields.js';
import { CORNERS, cornerValue, coverBox } from '../../corner-radii.js';
import { boundKey } from '../../editor/widgets.js';
import { appChips, durationParts, graphTiming } from '../../inspector-tabs.js';
import { TRANSITION_RANGE, formatMs, transitionOf, transitionPatch } from '../../value-transition.js';

export function createCommonFields({ t, store, update, root, sensors, host, pickers, colorField, video, barCornerModes, sourceState, redraw, shownValue, optionTiles }) {
  const button = (text, onclick) => el('button', { type: 'button', class: 'text-button', text, onclick });

  /** A graph's own cadence, otherwise the theme's refresh. */
  const timingOf = (e) => graphTiming(e.kind, store.getState().theme.refreshSeconds);

  /** Window observed by the live peak readout, milliseconds. */
  const graphWindowMs = (e) => timingOf(e).duration * 1000;

  /** The same prefixed angle controls for rings and needles. */
  const angleFields = (e) => fieldGrid([
    prefixedField(t('insp.arcStart'), t('inspector.startAngle'), e.kind.startAngle, (v) => update(e.id, { kind: { startAngle: v } }), { min: -360, max: 360, unit: '°' }),
    prefixedField(t('insp.arcSweep'), t('inspector.sweep'), e.kind.sweep, (v) => update(e.id, { kind: { sweep: v } }), { min: 1, max: 360, unit: '°' }),
  ]);

  /** The same track toggle and paint for bars and rings. */
  const trackFields = (e) => [
    switchRow(t('inspector.showTrack'), Boolean(e.kind.track), (on) => update(e.id, { kind: { track: on ? '#ffffff26' : null } })),
    e.kind.track && colorField(t('inspector.track'), typeof e.kind.track === 'string' ? e.kind.track : '#ffffff26', (v) => update(e.id, { kind: { track: v } })),
  ];

  const manualSource = (e) => textField(t('player.source'), e.kind.content.source ?? '', (source) => update(e.id, { kind: { content: { source: source.slice(0, 160) } } }));

  const appSignature = (activity) => JSON.stringify((activity?.media ?? []).map((m) => [m.source, m.playing]));

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

  /** The app chips of a player: Automatic, one per app with a media session (a dot when it plays), Other…. */
  function appChipGroup(e) {
    const c = e.kind.content;
    const set = (patch) => {
      sourceState.custom = false;
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
      chipButton(t('insp.otherApp'), sourceState.custom || custom !== null, () => {
        sourceState.custom = true;
        redraw();
        root.querySelector('.player-source input')?.focus();
      }, { class: 'chip other-chip' }),
    ]);
    return { group, chips, custom };
  }

  /** A choice drawn as a card with its radio (artboard PlayerData, "Quando non suona niente"). */
  function choiceCard(name, checked, title, note, onChoose, extra = null) {
    const input = el('input', { type: 'radio', name, checked });
    input.addEventListener('change', () => input.checked && onChoose());
    return el('div', { class: 'choice-card', dataset: { checked: String(checked) } }, [
      el('label', { class: 'choice-card-head' }, [input, el('span', { class: 'choice-card-text' }, [el('strong', { text: title }), el('span', { text: note })])]),
      checked ? extra : null,
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

  /** How a duration of `seconds` is written ("30 s", "5 min", "1 h"). */
  function durationText(seconds) {
    const { value, unit } = durationParts(seconds);
    if (unit === 'h') return t('unit.hours', { value });
    if (unit === 'min') return t('unit.minutes', { value });
    return t('unit.seconds', { value });
  }

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
  return { sourceCard, rangeFields, sourceSection, sampleText, fontFields, appChipGroup, choiceCard, cornersSection, coverCorners, barCorners, segmentsFields, transitionSection, fadeFields, durationText, valueColorSection, sensorMotionSection, visualizationFields, appSignature, manualSource, trackFields, angleFields, timingOf, graphWindowMs, button };
}
