// The icon catalogue: every symbol of the design canvas (the 17 artboards of
// "Bezel Evo — proposta UI/UX") and of the app, once each. 24×24, stroked
// with the current colour (`icon()` in dom.js gives every one the `ic`
// class: fill none, round caps, base.css's stroke width). A value is the
// list of path data of one symbol; a dot is a zero-length segment (`h.01`).
// A module never redefines one of these: a symbol it lacks goes here.

const rect = (x, y, w, h, r = 0) => (r
  ? `M${x + r} ${y}h${w - 2 * r}a${r} ${r} 0 0 1 ${r} ${r}v${h - 2 * r}a${r} ${r} 0 0 1-${r} ${r}h-${w - 2 * r}a${r} ${r} 0 0 1-${r}-${r}v-${h - 2 * r}a${r} ${r} 0 0 1 ${r}-${r}z`
  : `M${x} ${y}h${w}v${h}h-${w}z`);
const circle = (cx, cy, r) => `M${cx - r} ${cy}a${r} ${r} 0 1 0 ${2 * r} 0a${r} ${r} 0 1 0-${2 * r} 0`;

export const ICONS = Object.freeze({
  textLeft: ['M4 6h16M4 12h10M4 18h13'],
  textCenter: ['M4 6h16M7 12h10M5.5 18h13'],
  textRight: ['M4 6h16M10 12h10M7 18h13'],
  textTop: ['M4 4h16M8 8h8M8 12h8'],
  textMiddle: ['M8 9h8M8 15h8M4 12h2M18 12h2'],
  textBottom: ['M4 20h16M8 12h8M8 16h8'],
  shadow: ['M6 6h10v10H6zM9 19h10V9'],
  'fixed-digits': ['M4 6h4v12H4zM10 6h4v12h-4zM16 6h4v12h-4z'],
  "bolt": ["M13 2 4 14h7l-1 8 9-12h-7z"],
  "filter": ["M4 6h16M7 12h10M10 18h4"],
  "star": ["M12 17.75l-6.17 3.25 1.18-6.88-5-4.86 6.9-1L12 2l3.09 6.26 6.9 1-5 4.86 1.18 6.88z"],
  "test-values": ["M9 3h6M10 3v6l-5 9a2 2 0 0 0 1.7 3h10.6a2 2 0 0 0 1.7-3l-5-9V3"],
  "section-layout": ["M4 9V4h5M15 4h5v5M20 15v5h-5M9 20H4v-5"],
  "data": ["M3 12h4l3-7 4 14 3-7h4"],
  "drop": ["M12 3s6 6.5 6 11a6 6 0 0 1-12 0c0-4.5 6-11 6-11z"],
  "behav": ["M3 17c4 0 4-10 9-10s5 10 9 10"],
  "type-valore": ["M4 8h3v8M4 16h6M13 9a2 2 0 1 1 4 0c0 2-4 3-4 7h4"],
  "type-barra": ["M6 9h12a3 3 0 0 1 0 6H6a3 3 0 0 1 0-6zM6 12h7"],
  "type-anello": ["M12 4a8 8 0 1 0 8 8"],
  "type-lancetta": ["M4 17a8 8 0 0 1 16 0M12 17l4-5"],
  "type-grafico": ["m3 17 5-6 4 3 4-7 5 5"],
  "fade-none": ["M4 6h16v12H4z"],
  "fade-linear": ["M4 6h16v12H4zM8 6v12M12 6v12M16 6v12"],
  "fade-radial": ["M12 4a8 8 0 1 1 0 16a8 8 0 1 1 0-16M12 8a4 4 0 1 1 0 8a4 4 0 1 1 0-8"],
  database: ['M4 6a8 3 0 1 0 16 0 8 3 0 1 0-16 0', 'M4 6v12a8 3 0 0 0 16 0V6', 'M4 12a8 3 0 0 0 16 0'],
  brush: ['m14 4 6 6', 'm5 15 12-12 4 4-12 12', 'M5 15a4 4 0 0 0-2 6c3 0 6 0 6-4'],
  radius: ['M4 20V10a6 6 0 0 1 6-6h10', 'M4 4h.01', 'M12 12h8v8h-8z'],
  rotate: ['M20 10a8 8 0 1 0-1 7', 'M20 4v6h-6'],
  width: ['M3 6v12', 'M21 6v12', 'M3 12h18', 'm7 8-4 4 4 4', 'm17 8 4 4-4 4'],
  height: ['M6 3h12', 'M6 21h12', 'M12 3v18', 'm8 7 4-4 4 4', 'm8 17 4 4 4-4'],
  textSize: ['M3 7V4h12v3', 'M9 4v16', 'M6 20h6', 'M16 12h6', 'M19 12v8'],
  spacing: ['M4 5v14', 'M20 5v14', 'M7 12h10', 'm10 9-3 3 3 3', 'm14 9 3 3-3 3'],
  strokeWidth: ['M4 5h16', 'M4 11h16', 'M4 18h16', 'M4 16v4', 'M20 16v4'],
  frequency: ['M3 12h3l3-8 6 16 3-8h3'],
  gain: ['M4 18h16', 'M7 14V6', 'm4 9 3-3 3 3', 'M17 14V3', 'm14 6 3-3 3 3'],
  count: ['M9 3 7 21', 'M17 3 15 21', 'M4 9h17', 'M3 15h17'],
  // ------------------------------------------------------- widget kinds --
  text: ['M5 6V4h14v2', 'M12 4v16', 'M9 20h6'],
  value: ['M4 18V6', 'M8 18V10', 'M12 18V4', 'M16 18v-6', 'M20 18V8'],
  clock: [circle(12, 12, 8), 'M12 8v4l3 2'],
  weather: ['M7 18a4 4 0 0 1-.6-8A6 6 0 0 1 18 9a4.5 4.5 0 0 1-.5 9H7z'],
  ring: ['M12 4a8 8 0 1 0 8 8'],
  bar: [rect(3, 9, 18, 6, 3), 'M6 12h7'],
  needle: ['M4 17a8 8 0 0 1 16 0', 'M12 17l4-5'],
  graph: ['m3 17 5-6 4 3 4-7 5 5'],
  image: [rect(3, 5, 18, 14, 2), 'm21 16-5-5-8 8'],
  spectrum: ['M3 9v6', 'M7 5v14', 'M11 2v20', 'M15 6v12', 'M19 8v8', 'M1 12h22'],
  player: ['M9 18V6l10-2v12', circle(6.5, 18, 2.5), circle(16.5, 16, 2.5)],
  shape: [rect(4, 4, 16, 16, 3)],
  card: [rect(5, 3, 14, 18, 2), 'M9 8h6', 'M9 12h6'],
  group: [rect(3, 3, 8, 8, 1.5), rect(13, 13, 8, 8, 1.5), 'M11 7h4v6'],
  // A card's shared base and one of its faces (Layers).
  base: ['m12 4 8 4-8 4-8-4 8-4z', 'M4 12l8 4 8-4'],
  face: ['M4 6h16', 'M4 6v12h16V6', 'M8 10h8'],

  // -------------------------------------------------------------- panels --
  widgets: [rect(4, 4, 7, 7, 1.5), rect(13, 4, 7, 7, 1.5), rect(4, 13, 7, 7, 1.5), 'M16.5 13v7', 'M13 16.5h7'],
  sensors: ['M3 12h4l3-7 4 14 3-7h4'],
  layers: ['m12 3 9 5-9 5-9-5 9-5z', 'm3 13 9 5 9-5'],
  themes: [rect(7, 3, 10, 18, 2), 'M3 7v10', 'M21 7v10'],
  media: [rect(3, 5, 18, 14, 2), circle(9, 10, 1.5), 'm21 16-5-5-8 8'],
  screen: [rect(7, 2, 10, 20, 2), 'M11 18h2'],
  // The profile active on the screen (board MainNew).
  profile: ['M12 3l7 4v5c0 4.5-3 8-7 9-4-1-7-4.5-7-9V7z'],
  // The theme's name can be edited (board MainNew).
  pencil: ['M4 20h4L18.5 9.5a2.1 2.1 0 0 0-4-4L4 16v4z', 'm13.5 6.5 4 4'],
  // The Bezel chip: how fast it draws (board StatusBarNew).
  performance: ['M4 18a8 8 0 1 1 16 0', 'M12 18l4-6'],
  vertical: [rect(7.5, 3, 9, 18, 2)],
  horizontal: [rect(3, 7.5, 18, 9, 2)],
  // New theme, vertical or horizontal (Themes).
  newVertical: [rect(7.5, 3, 9, 18, 2), 'M12 9v6', 'M9 12h6'],
  newHorizontal: [rect(3, 7.5, 18, 9, 2), 'M12 9v6', 'M9 12h6'],

  // ------------------------------------------------------------- actions --
  undo: ['M9 14 4 9l5-5', 'M4 9h10a6 6 0 0 1 0 12h-3'],
  redo: ['m15 14 5-5-5-5', 'M20 9H10a6 6 0 0 0 0 12h3'],
  refresh: ['M20 12a8 8 0 1 1-2.34-5.66', 'M20 4v5h-5'],
  grid: ['M4 9h16', 'M4 15h16', 'M9 4v16', 'M15 4v16'],
  minus: ['M5 12h14'],
  plus: ['M12 5v14', 'M5 12h14'],
  close: ['M6 6l12 12', 'M18 6 6 18'],
  copy: [rect(8, 8, 12, 12, 2), 'M16 8V4H4v12h4'],
  paste: ['M9 5H5v16h14V5h-4', rect(9, 3, 6, 4, 1)],
  trash: ['M4 7h16', 'M10 11v6', 'M14 11v6', 'M6 7l1 13h10l1-13', 'M9 7V4h6v3'],
  eyedropper: ['M11 7l6 6', 'M4 16 15.7 4.3a1 1 0 0 1 1.4 0l2.6 2.6a1 1 0 0 1 0 1.4L8 20H4z'],
  more: ['M5 12h.01', 'M12 12h.01', 'M19 12h.01'],
  drag: ['M9 6h.01', 'M15 6h.01', 'M9 12h.01', 'M15 12h.01', 'M9 18h.01', 'M15 18h.01'],
  search: [circle(11, 11, 7), 'm20 20-3.5-3.5'],
  sliders: ['M4 6h9', 'M17 6h3', 'M15 4v4', 'M4 12h3', 'M11 12h9', 'M9 10v4', 'M4 18h11', 'M19 18h1', 'M17 16v4'],
  upload: ['M12 15V3', 'M7 8l5-5 5 5', 'M5 21h14'],
  download: ['M12 4v12', 'M7 11l5 5 5-5', 'M5 20h14'],
  dropHint: ['M9 4v12', 'M5 12l4 4 4-4', 'M15 20h5'],
  play: ['M7 5v14l11-7z'],
  pause: [rect(6, 5, 4, 14, 1), rect(14, 5, 4, 14, 1)],
  stop: [rect(6, 6, 12, 12)],
  power: ['M12 3v9', 'M6.3 6.3a8 8 0 1 0 11.4 0'],
  handBack: ['M15 3h6v6', 'M10 14 21 3', 'M21 14v7H3V3h7'],
  rename: ['M4 20h4L19 9l-4-4L4 16z', 'M13 7l4 4'],
  link: ['M10 14a4 4 0 0 0 6 0l3-3a4 4 0 0 0-6-6l-1 1', 'M14 10a4 4 0 0 0-6 0l-3 3a4 4 0 0 0 6 6l1-1'],
  sweep: ['M14 4l-5 9', 'M5 13h8l1 7H4z', 'M8 17v3', 'M11 17v3'],
  box: ['M4 7h16v13H4z', 'M3 4h18v3H3z', 'M10 11h4'],
  // Media: use as the background, put on the screen; the inspector frames a video on the canvas.
  background: [rect(3, 5, 18, 14), 'M3 15l5-5 4 4 3-3 6 6'],
  place: [rect(4, 4, 16, 16), 'M12 8v8', 'M8 12h8'],
  frame: ['M4 9V4h5', 'M15 4h5v5', 'M20 15v5h-5', 'M9 20H4v-5', 'M12 8v8', 'M8 12h8'],
  // The inspector's tile for a plain group: two boxes in a dashed frame.
  groupFrame: ['M4 4h4', 'M16 4h4v4', 'M20 16v4h-4', 'M8 20H4v-4', rect(8, 8, 5, 5), rect(11, 11, 5, 5)],
  live: ['M5 12a7 7 0 0 1 14 0', 'M8.5 12a3.5 3.5 0 0 1 7 0', 'M12 12h.01'],
  spinner: ['M21 12a9 9 0 1 1-6.2-8.6'],

  // ---------------------------------------------------------- navigation --
  chevronDown: ['m6 9 6 6 6-6'],
  chevronRight: ['m9 6 6 6-6 6'],
  chevronLeft: ['m15 18-6-6 6-6'],
  expandAll: ['m7 9 5-5 5 5', 'M7 15l5 5 5-5'],
  collapseAll: ['m7 4 5 5 5-5', 'M7 20l5-5 5 5'],
  arrowLeft: ['M19 12H5', 'M11 6l-6 6 6 6'],
  arrowRight: ['M5 12h14', 'M13 6l6 6-6 6'],
  up: ['M12 19V5', 'M6 11l6-6 6 6'],
  down: ['M12 5v14', 'M6 13l6 6 6-6'],

  // ----------------------------------------------------------- alignment --
  alignLeft: ['M4 3v18', 'M8 7h10', 'M8 15h6'],
  alignCenterX: ['M12 3v18', 'M6 7h12', 'M8 15h8'],
  alignRight: ['M20 3v18', 'M6 7h10', 'M10 15h6'],
  alignTop: ['M3 4h18', 'M7 8v10', 'M15 8v6'],
  alignCenterY: ['M3 12h18', 'M7 6v12', 'M15 8v8'],
  alignBottom: ['M3 20h18', 'M7 6v10', 'M15 10v6'],
  distributeX: ['M4 4v16', 'M20 4v16', rect(10, 8, 4, 8)],
  distributeY: ['M4 4h16', 'M4 20h16', rect(8, 10, 8, 4)],

  // --------------------------------------------------------------- state --
  eye: ['M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z', circle(12, 12, 3)],
  eyeOff: ['M3 3l18 18', 'M10.6 5.1A10 10 0 0 1 12 5c6.5 0 10 7 10 7a17 17 0 0 1-3 3.8', 'M6.6 6.6C3.6 8.4 2 12 2 12s3.5 7 10 7a9.7 9.7 0 0 0 5.4-1.6'],
  lock: [rect(5, 11, 14, 10, 2), 'M8 11V7a4 4 0 0 1 8 0v4'],
  unlock: [rect(5, 11, 14, 10, 2), 'M8 11V7a4 4 0 0 1 7.5-2'],
  warning: ['M12 9v4', 'M12 17h.01', 'M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z'],
  info: [circle(12, 12, 9), 'M12 8h.01', 'M11 12h1v4h1'],
  history: [circle(12, 12, 9), 'M12 7v5l3 2'],
  // A person (the player's artist).
  person: [circle(12, 8, 3.5), 'M5 20a7 7 0 0 1 14 0'],

  // ------------------------------------------------- hardware and media --
  chip: [rect(6, 6, 12, 12, 2), 'M9 2v4', 'M15 2v4', 'M9 18v4', 'M15 18v4', 'M2 9h4', 'M2 15h4', 'M18 9h4', 'M18 15h4'],
  sdCard: ['M8 3h9l3 3v15H4V7z', 'M9 7v3', 'M12 7v3', 'M15 7v3'],
  film: [rect(4, 5, 16, 14), 'M8 5v14', 'M16 5v14', 'M4 9h4', 'M4 15h4', 'M16 9h4', 'M16 15h4'],
  video: [rect(3, 6, 13, 12, 2), 'm16 10 5-3v10l-5-3'],
  sunDim: [circle(12, 12, 3), 'M12 5v1', 'M12 18v1', 'M5 12h1', 'M18 12h1'],
  monitor: [rect(4, 5, 16, 11), 'M9 20h6', 'M12 16v4'],
  sunBright: [circle(12, 12, 4), 'M12 2v2', 'M12 20v2', 'M2 12h2', 'M20 12h2', 'M4.9 4.9l1.4 1.4', 'M17.7 17.7l1.4 1.4', 'M4.9 19.1l1.4-1.4', 'M17.7 6.3l1.4-1.4'],
});
