// The Filters button shared by the library panels and gallery.
import { el, icon } from './dom.js';
import { ICONS } from './icons.js';
export function filtersToggle(search, row) {
  const button = el('button', { type: 'button', class: 'icon-button filters-toggle', 'aria-controls': row.id, 'aria-expanded': 'false', onclick: () => show(row.hidden) }, [icon(ICONS.filter, 18)]);
  search.classList.add('search-with-filters'); search.append(button);
  function show(open) { row.hidden = !open; button.setAttribute('aria-expanded', String(open)); }
  show(false);
  return { button, show };
}
