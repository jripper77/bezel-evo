// Existing weather controls, isolated from inspector navigation.
import { el, icon } from '../dom.js';
import { numberField, selectField, switchRow, textField } from '../fields.js';
import { panels as textPanels } from './value.js';

export function panels(ctx) {
  const { t, update, root, searchCities, button } = ctx;
  function fields(e) {
    const c = e.kind.content, s = e.kind.style;
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
      nodes.push(switchRow(t('inspector.fahrenheit'), c.fahrenheit, (fahrenheit) => update(e.id, { kind: { content: { fahrenheit } } })));
      weatherAppearance.push(switchRow(t('weather.icon'), c.showIcon !== false, (showIcon) => update(e.id, { kind: { content: { showIcon } } })));
      if (c.showIcon !== false) {
        weatherAppearance.push(selectField(t('weather.iconStyle'), c.iconStyle ?? 'outline', [['outline', t('icons.outline')], ['filled', t('icons.filled')], ['colored', t('weather.colored')], ['dimensional', t('weather.dimensional')]], (iconStyle) => update(e.id, { kind: { content: { iconStyle } } })));
        weatherAppearance.push(numberField(t('weather.iconGap'), c.iconGap ?? s.size * 0.3, (iconGap) => update(e.id, { kind: { content: { iconGap: Math.max(0, Math.min(256, iconGap)) } } }), { min: 0, max: 256, step: 'any' }));
        weatherAppearance.push(numberField(t('weather.iconSize'), c.iconSize ?? 0, (size) => update(e.id, { kind: { content: { iconSize: size <= 0 ? null : Math.min(512, size) } } }), { min: 0, max: 512 }));
      }
      nodes.push(el('p', { class: 'hint', text: t('weather.source') }));
    }
    return { nodes, weatherAppearance };
  }
  return textPanels({ ...ctx, extra: fields(ctx.element) });
}
