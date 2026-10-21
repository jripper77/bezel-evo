import { describeSections } from './sections.js';
// Existing weather controls, isolated from inspector navigation.
import { el } from '../dom.js';
import { numberField, iconChoice, propertySection as section, selectField, toggleField as switchRow, textField } from '../fields.js';
import { textStyleFields } from './text.js';
import { drawWeatherIcon } from '../../demo-render.js';

export function panels(ctx) {
  const { t, update, root, searchCities } = ctx;
  function fields(e) {
    const c = e.kind.content, s = e.kind.style;
    const iconPreview = style => { const canvas = el('canvas', { width: 40, height: 28, 'aria-hidden': 'true' }); drawWeatherIcon(canvas.getContext('2d'), style, getComputedStyle(root).color, 6, 0, 28, 3); return canvas; };
    const nodes = [], weatherAppearance = [];
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
      nodes.push(iconChoice(t('insp.weather.units'), c.fahrenheit ? 'f' : 'c', [['c', '\u00b0C'], ['f', '\u00b0F']], value => update(e.id, { kind: { content: { fahrenheit: value === 'f' } } }), { wide: true }));
      weatherAppearance.push(switchRow(t('weather.icon'), c.showIcon !== false, (showIcon) => update(e.id, { kind: { content: { showIcon } } })));
      if (c.showIcon !== false) {
        weatherAppearance.push(ctx.optionTiles(t('weather.iconStyle'), [['outline', t('icons.outline')], ['filled', t('icons.filled')], ['colored', t('weather.colored')], ['dimensional', t('weather.dimensional')]].map(([value, text]) => ({ text, preview: iconPreview(value), pressed: (c.iconStyle ?? 'outline') === value, onClick: () => update(e.id, { kind: { content: { iconStyle: value } } }) }))));
        weatherAppearance.push(numberField(t('weather.iconGap'), Math.round((c.iconGap ?? s.size * 0.3) * 100) / 100, (iconGap) => update(e.id, { kind: { content: { iconGap: Math.max(0, Math.min(256, iconGap)) } } }), { min: 0, max: 256, step: 'any' }));
        weatherAppearance.push(numberField(t('insp.weather.iconSize'), c.iconSize, size => update(e.id, { kind: { content: { iconSize: size == null ? null : Math.max(1, Math.min(512, size)) } } }), { min: 1, max: 512, nullable: true, unit: 'px', placeholder: t('insp.automatic') }));
      }
      nodes.push(el('p', { class: 'hint', text: t('weather.source') }));
    }
    return { nodes, weatherAppearance };
  }
  const e = ctx.element, c = e.kind.content, { nodes, weatherAppearance } = fields(e);
  const set = content => update(e.id, { kind: { content } });
  return { data: [section('content', t('inspector.content'), nodes)], look: [section('appearance', t('inspector.appearance'), weatherAppearance), section('font', t('inspector.font'), textStyleFields(ctx, e, { parts: false })), section('color', t('inspector.color'), c.showIcon !== false && ['outline', 'filled'].includes(c.iconStyle ?? 'outline') ? [ctx.colorField(t('insp.weather.iconColor'), c.iconColor ?? e.kind.style.paint, iconColor => set({ iconColor })), c.iconColor != null && el('button', { type: 'button', class: 'text-button', text: t('insp.weather.inheritColor'), onclick: () => set({ iconColor: null }) })] : [])], behavior: c.showIcon === false ? [] : [section('weather-motion', t('insp.weather.motion'), [
    iconChoice(t('insp.weather.motion'), c.iconMotion ?? 'still', ['still', 'gentle', 'lively'].map(value => [value, t(`insp.weather.motion.${value}`)]), iconMotion => set({ iconMotion: iconMotion === 'still' ? null : iconMotion }), { wide: true }),
    c.iconMotion && c.iconMotion !== 'still' && iconChoice(t('insp.playback.speed'), String(c.iconSpeed ?? 1), [...new Set([0.5, 1, 2, c.iconSpeed ?? 1])].sort((a,b) => a-b).map(value => [String(value), `${value}×`]), value => set({ iconSpeed: Number(value) === 1 ? null : Number(value) }), { wide: true }),
    el('p', { class: 'hint', text: t('insp.weather.motionHint') }),
  ])] };
}

export function sections(ctx) {
  return describeSections(ctx, ctx.completePanels(panels(ctx)));
}
