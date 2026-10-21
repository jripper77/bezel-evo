// The demo's font catalogue, shaped like the bridge's `fontCatalog`
// (D-2026-10-16-studio-redesign-fidelity-28): Bezel's bundled families and
// the families a stock Windows 11 installs, with their real faces and
// categories as the renderer's font database reports them. Inter is the
// fallback, as on a machine without the theme's family.

const f = (weight, italic = false) => ({ weight, italic });
const regular = [f(400)];
const four = [f(400), f(400, true), f(700), f(700, true)];
const family = (name, faces, category, extra = {}) => ({ name, faces, monospace: category === 'mono', category, bundled: false, ...extra });

/** Families shipped with Bezel (`themes/fonts`). */
const BUNDLED = [
  family('Inter', [f(400), f(500), f(600), f(700)], 'sans', { bundled: true }),
  family('JetBrains Mono', [f(400), f(500), f(700)], 'mono', { bundled: true }),
  family('Doto', Array.from({ length: 9 }, (_, i) => f((i + 1) * 100)), 'display', { bundled: true }),
  family('Doto Rounded', Array.from({ length: 9 }, (_, i) => f((i + 1) * 100)), 'display', { bundled: true }),
  family('IBM Plex Mono', [100, 200, 300, 400, 500, 600, 700].flatMap((w) => [f(w), f(w, true)]), 'mono', { bundled: true }),
  family('Fraunces 144pt', [f(300, true), f(400, true), f(300), f(400), f(600)], 'serif', { bundled: true }),
  family('Instrument Sans', [f(400), f(500), f(600)], 'sans', { bundled: true }),
  family('Space Grotesk', [f(400), f(500), f(600)], 'sans', { bundled: true }),
  family('Bricolage Grotesque 96pt', [f(400), f(500), f(700), f(800)], 'sans', { bundled: true }),
];

/** A stock Windows 11's families. */
const SYSTEM = [
  family('Arial', [...four, f(900)], 'sans'),
  family('Bahnschrift', regular, 'sans'),
  family('Book Antiqua', four, 'serif'),
  family('Bookman Old Style', [f(300), f(300, true), f(600), f(600, true)], 'serif'),
  family('Calibri', [f(300), f(300, true), ...four], 'sans'),
  family('Cambria', four, 'serif'),
  family('Cambria Math', regular, 'serif'),
  family('Candara', four, 'sans'),
  family('Cascadia Code', regular, 'mono'),
  family('Cascadia Mono', regular, 'mono'),
  family('Century', regular, 'serif'),
  family('Century Gothic', four, 'sans'),
  family('Comic Sans MS', four, 'display'),
  family('Consolas', four, 'mono'),
  family('Constantia', four, 'serif'),
  family('Corbel', [f(300), f(300, true), ...four], 'sans'),
  family('Courier New', four, 'mono'),
  family('Ebrima', [f(400), f(700)], 'sans'),
  family('Franklin Gothic Medium', [f(400), f(400, true)], 'sans'),
  family('Gabriola', regular, 'display'),
  family('Gadugi', [f(400), f(700)], 'sans'),
  family('Garamond', [f(400), f(400, true), f(700)], 'serif'),
  family('Georgia', four, 'serif'),
  family('Impact', regular, 'sans'),
  family('Ink Free', regular, 'display'),
  family('Javanese Text', regular, 'sans'),
  family('Leelawadee UI', [f(350), f(400), f(700)], 'sans'),
  family('Lucida Console', regular, 'mono'),
  family('Lucida Sans Unicode', regular, 'sans'),
  family('Malgun Gothic', [f(300), f(400), f(700)], 'sans', { aliases: ['맑은 고딕'] }),
  family('Marlett', [f(500)], 'sans'),
  family('Microsoft Himalaya', regular, 'sans'),
  family('Microsoft JhengHei', [f(290), f(400), f(700)], 'sans', { aliases: ['微軟正黑體'] }),
  family('Microsoft JhengHei UI', [f(290), f(400), f(700)], 'sans'),
  family('Microsoft New Tai Lue', [f(400), f(700)], 'sans'),
  family('Microsoft PhagsPa', [f(400), f(700)], 'sans'),
  family('Microsoft Sans Serif', regular, 'sans'),
  family('Microsoft Tai Le', [f(400), f(700)], 'sans'),
  family('Microsoft YaHei', [f(290), f(400), f(700)], 'sans', { aliases: ['微软雅黑'] }),
  family('Microsoft YaHei UI', [f(290), f(400), f(700)], 'sans'),
  family('Microsoft Yi Baiti', regular, 'display'),
  family('MingLiU-ExtB', regular, 'serif'),
  family('Mongolian Baiti', regular, 'display'),
  family('MS Gothic', regular, 'mono', { aliases: ['ＭＳ ゴシック'] }),
  family('MS PGothic', regular, 'sans'),
  family('MS UI Gothic', regular, 'sans'),
  family('MV Boli', [f(400, true)], 'sans'),
  family('Myanmar Text', [f(400), f(700)], 'sans'),
  family('Nirmala UI', [f(350), f(400), f(700)], 'sans'),
  family('NSimSun', regular, 'mono'),
  family('Palatino Linotype', four, 'serif'),
  family('Segoe Fluent Icons', regular, 'display'),
  family('Segoe MDL2 Assets', regular, 'display'),
  family('Segoe Print', [f(400), f(700)], 'display'),
  family('Segoe Script', [f(400), f(700)], 'display'),
  family('Segoe UI', [f(300), f(300, true), f(350), f(350, true), ...four.slice(0, 2), f(600), f(600, true), ...four.slice(2), f(900), f(900, true)], 'sans'),
  family('Segoe UI Emoji', regular, 'sans'),
  family('Segoe UI Historic', regular, 'sans'),
  family('Segoe UI Symbol', regular, 'sans'),
  family('Segoe UI Variable', regular, 'sans'),
  family('SimSun', regular, 'serif', { aliases: ['宋体'] }),
  family('Sitka', [f(400), f(400, true)], 'serif'),
  family('Sylfaen', regular, 'serif'),
  family('Symbol', regular, 'display'),
  family('Tahoma', [f(400), f(700)], 'sans'),
  family('Times New Roman', four, 'serif'),
  family('Trebuchet MS', four, 'sans'),
  family('Verdana', four, 'sans'),
  family('Webdings', regular, 'display'),
  family('Wingdings', regular, 'display'),
  family('Yu Gothic', [f(300), f(400), f(500), f(700)], 'sans', { aliases: ['游ゴシック'] }),
  family('Yu Gothic UI', [f(300), f(350), f(400), f(600), f(700)], 'sans'),
];

/** The demo's `fontCatalog()` answer, families sorted by name as the renderer sorts them. */
export const DEMO_FONT_CATALOG = Object.freeze({
  families: [...BUNDLED, ...SYSTEM].sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase())),
  fallback: 'Inter',
});
