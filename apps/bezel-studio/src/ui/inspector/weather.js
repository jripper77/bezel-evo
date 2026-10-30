import { describeSections } from './sections.js';
// Existing weather controls, isolated from inspector navigation.
import { el } from '../dom.js';
import { ICONS } from '../icons.js';
import { fieldAction, fieldGrid, iconButton, listCard, searchField, numberField, iconChoice, propertySection as section, selectField, textField, toggleField as switchRow } from '../fields.js';
import { textStyleFields } from './text.js';
import { drawWeatherIcon } from '../../demo-render.js';
import { WEATHER_PARTS } from '../../text-layout.js';
import { layoutPatch, moveLine, weatherLayout } from '../../weather-format.js';

export function panels(ctx) {
  const { t, update, root, searchCities } = ctx;
  function fields(e) {
    const c = e.kind.content, s = e.kind.style;
    const iconPreview = style => { const canvas = el('canvas', { width: 40, height: 28, 'aria-hidden': 'true' }); drawWeatherIcon(canvas.getContext('2d'), style, getComputedStyle(root).color, 6, 0, 28, 3); return canvas; };
    const nodes = [], weatherAppearance = [];
    if (c.type === 'weather') {
      let query = c.city;
      const results = el('div', { id: 'weather-results', role: 'status', 'aria-live': 'polite' });
      const search = iconButton(t('weather.search'), ICONS.arrowRight, async () => {
        search.disabled = true;
        results.textContent = t('weather.loading');
        try {
          const cities = await searchCities(query);
          if (!root.contains(results)) return;
          results.replaceChildren();
          if (!cities.length) results.textContent = t('weather.empty');
          for (const city of cities) results.append(listCard({ icon: 'map-pin', title: city.name, subtitle: [city.region, city.country].filter(Boolean).join(', '), label: [city.name, city.region, city.country].filter(Boolean).join(', '), pressed: city.name === c.city && city.latitude === c.latitude && city.longitude === c.longitude, onClick: () => update(e.id, { kind: { content: { city: city.name, latitude: city.latitude, longitude: city.longitude } } }) }));
        } catch { if (root.contains(results)) results.textContent = t('weather.error'); }
        finally { search.disabled = false; }
      });
      const cityField = searchField(t('weather.city'), value => { query = value; }, { value: query });
      cityField.querySelector('input').addEventListener('keydown', event => { if (event.key === 'Enter') { event.preventDefault(); if (!search.disabled) search.click(); } });
      nodes.push(fieldAction(el('div', { class: 'field' }, [el('span', { class: 'field-label', text: t('weather.location', { city: c.city }) }), cityField]), search), results);
      nodes.push(textField(t('insp.weather.label'), c.label ?? '', label => update(e.id, { kind: { content: { label: label.trim() || null } } }), { placeholder: c.city }));
      const language = selectField(t('weather.language'), c.language ?? '', [['', t('clock.system')], ['it', t('language.it')], ['en', t('language.en')], ['pt-BR', t('language.pt-BR')]], (language) => update(e.id, { kind: { content: { language: language || null } } }));
      nodes.push(fieldGrid([iconChoice(t('insp.weather.units'), c.fahrenheit ? 'f' : 'c', [['c', '\u00b0C'], ['f', '\u00b0F']], value => update(e.id, { kind: { content: { fahrenheit: value === 'f' } } }), { wide: true }), language]));
      weatherAppearance.push(switchRow(t('weather.icon'), c.showIcon !== false, (showIcon) => update(e.id, { kind: { content: { showIcon } } }), { iconName: 'weather', leading: true }));
      if (c.showIcon !== false) {
        const styles = [['outline', t('icons.outline')], ['filled', t('icons.filled')], ['colored', t('weather.colored')], ['dimensional', t('weather.dimensional')], ['pixel', t('weather.pixel')]];
        const tiles = styles.map(([value, text]) => ({ text, preview: iconPreview(value), pressed: (c.iconStyle ?? 'outline') === value, onClick: () => update(e.id, { kind: { content: { iconStyle: value } } }) }));
        weatherAppearance.push(section('weather-style', t('weather.iconStyle'), [ctx.optionTiles(t('weather.iconStyle'), tiles)], { caption: true }));
        weatherAppearance.push(fieldGrid([
          numberField(t('insp.weather.iconSize'), c.iconSize, size => update(e.id, { kind: { content: { iconSize: size == null ? null : Math.max(1, Math.min(512, size)) } } }), { min: 1, max: 512, nullable: true, unit: 'px', placeholder: t('insp.automatic'), compact: true, iconName: 'shape-rect' }),
          numberField(t('weather.iconGap'), Math.round((c.iconGap ?? s.size * 0.3) * 100) / 100, iconGap => update(e.id, { kind: { content: { iconGap: Math.max(0, Math.min(256, iconGap)) } } }), { min: 0, max: 256, step: 'any', unit: 'px', compact: true, iconName: 'spacing' }),
        ]));
      }
      nodes.push(el('p', { class: 'hint', text: t('weather.source') }));
    }
    return { nodes, weatherAppearance };
  }
  const e = ctx.element, c = e.kind.content, { nodes, weatherAppearance } = fields(e);
  const set = content => update(e.id, { kind: { content } });
  const font = section('font', t('inspector.font'), textStyleFields(ctx, e, { parts: true, scope: 'weather', partIds: WEATHER_PARTS, partsLabel: t('insp.weather.parts'), partLabel: part => t(`insp.weatherPart.${part}`) }));
  const lines = weatherLayout(c), setLines = next => set({ layout: layoutPatch(next) });
  const lineRows = el('ol', { class: 'swatch-rows weather-lines' }, lines.map((line, i) => {
    const name = t(`insp.weatherPart.${line.field}`), change = patch => setLines(lines.map((other, j) => (j === i ? { ...other, ...patch } : other)));
    return el('li', { class: `swatch-row weather-line${line.hidden ? ' hidden-line' : ''}`, dataset: { field: line.field } }, [
      iconButton(t(line.hidden ? 'insp.weather.show' : 'insp.weather.hide', { part: name }), line.hidden ? ICONS.eyeOff : ICONS.eye, () => change({ hidden: !line.hidden }), { size: 16, 'aria-pressed': String(!line.hidden) }),
      el('span', { class: 'weather-line-name', text: name }),
      iconButton(t('insp.weather.inline', { part: name }), ICONS.link, () => change({ inline: !line.inline }), { size: 16, 'aria-pressed': String(Boolean(line.inline)), disabled: i === 0 }),
      iconButton(t('insp.weather.moveUp', { part: name }), ICONS.up, () => setLines(moveLine(lines, i, i - 1)), { size: 16, disabled: i === 0 }),
      iconButton(t('insp.weather.moveDown', { part: name }), ICONS.down, () => setLines(moveLine(lines, i, i + 1)), { size: 16, disabled: i === lines.length - 1 }),
    ]);
  }));
  const textPaint = font.querySelector(':scope > .property-section-body > .color-field');
  const colors = [textPaint];
  if (c.showIcon !== false && ['outline', 'filled'].includes(c.iconStyle ?? 'outline')) colors.push(ctx.colorField(t('insp.weather.iconColor'), c.iconColor ?? e.kind.style.paint, iconColor => set({ iconColor })), c.iconColor != null && el('button', { type: 'button', class: 'text-button', text: t('insp.weather.inheritColor'), onclick: () => set({ iconColor: null }) }));
  return { data: [section('content', t('inspector.content'), nodes, { heading: false }), section('weather-lines', t('insp.weather.lines'), [lineRows, el('p', { class: 'hint', text: t('insp.weather.linesHint') })])], look: [section('appearance', t('inspector.appearance'), weatherAppearance, { heading: false }), font, section('color', t('inspector.color'), colors)], behavior: c.showIcon === false ? [] : [section('weather-motion', t('insp.weather.motion'), [
    iconChoice(t('insp.weather.motion'), c.iconMotion ?? 'still', ['still', 'gentle', 'lively'].map(value => [value, t(`insp.weather.motion.${value}`), `weather-${value}`]), iconMotion => set({ iconMotion: iconMotion === 'still' ? null : iconMotion }), { wide: true }),
    c.iconMotion && c.iconMotion !== 'still' && iconChoice(t('insp.playback.speed'), String(c.iconSpeed ?? 1), [...new Set([0.5, 1, 2, c.iconSpeed ?? 1])].sort((a,b) => a-b).map(value => [String(value), `${value}×`]), value => set({ iconSpeed: Number(value) === 1 ? null : Number(value) }), { wide: true }),
    el('p', { class: 'hint', text: t('insp.weather.motionHint') }),
  ], { heading: false })] };
}

export function sections(ctx) {
  return describeSections(ctx, ctx.completePanels(panels(ctx)));
}
