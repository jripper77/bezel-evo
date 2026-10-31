// Pixel dancers of the `dancer` spectrum style, mirroring
// crates/bezel-render/src/dancer.rs (a fixture of every pose keeps both equal
// to the design) and the pose choice of crates/bezel-core's spectrum domain:
// a 24×24 character built from its head and torso, with arms, legs and tail
// drawn along the paths of each pose, on whole pixels.

/** Cells on each side of the grid. */
const GRID = 24;
/** Column of the character's left edge, the row its feet stand on, a standing leg's length. */
const LEFT = 8, FLOOR = 22, LEG = 4;

/** Who can dance, in the order of the inspector. */
export const DANCERS = Object.freeze(['robot', 'headphones', 'cat']);
/** The poses, in the order of the backend's `DancePose`. */
export const DANCE_POSES = Object.freeze(['still', 'down', 'armsUp', 'stepLeft', 'stepRight', 'jump']);
/** One character per colour; `.` is transparent. */
export const DANCER_PALETTE = Object.freeze({ W: '#f2ede6', O: '#ff9248', G: '#30bf8b', M: '#8a857c', D: '#3c3a36', K: '#101010' });

const CHARACTERS = {
  robot: { head: ['...OO...', '....W...', 'WWWWWWWW', 'WKKKKKKW', 'WKGKKGKW', 'WKKKKKKW', 'WWWWWWWW'], torso: ['.MWWWWM.', '.WOOOOW.', '.WOGGOW.', '.WOOOOW.', '.MWWWWM.'], arm: 'M', hand: 'W', leg: 'M', foot: 'W', tail: null },
  headphones: { head: ['.OOOOOO.', 'O.WWWW.O', 'OWWWWWWO', 'OWKWWKWO', 'OWWWWWWO', '.WWKKWW.', '..WWWW..'], torso: ['.GGGGGG.', '.GGGGGG.', '.GGWWGG.', '.GGGGGG.', '.MMMMMM.'], arm: 'G', hand: 'W', leg: 'M', foot: 'O', tail: null },
  cat: { head: ['O......O', 'OO....OO', 'OOOOOOOO', 'OGOOOOGO', 'OOOWWOOO', 'OOWKKWOO', '.OOOOOO.'], torso: ['.OOOOOO.', '.OWWWWO.', '.OWWWWO.', '.OWWWWO.', '.OOOOOO.'], arm: 'O', hand: 'W', leg: 'O', foot: 'W', tail: 'O' },
};

const ARM = { down: [[0, 1], [0, 2], [0, 3]], out: [[-1, 0], [-2, 0], [-3, -1]], up: [[-1, 0], [-2, -1], [-2, -2], [-2, -3], [-2, -4]] };
const TAIL = { down: [[1, 0], [2, 1], [2, 2]], out: [[1, 0], [2, 0], [3, -1]], up: [[1, 0], [2, -1], [2, -2], [3, -3]] };
/** Each pose: body offset, left and right arms, left and right legs from the hips (null: the left one mirrored), tail. */
const SHAPES = {
  still: { dy: 0, arms: [ARM.down, ARM.down], legs: [[[0, 1], [0, 2], [0, 3], [0, 4]], null], tail: TAIL.down },
  down: { dy: 2, arms: [ARM.out, ARM.out], legs: [[[-1, 1], [-1, 2]], null], tail: TAIL.out },
  armsUp: { dy: 0, arms: [ARM.up, ARM.up], legs: [[[-1, 1], [-1, 2], [-2, 3], [-2, 4]], null], tail: TAIL.up },
  stepLeft: { dy: 1, arms: [ARM.up, ARM.down], legs: [[[-1, 1], [-1, 2], [-1, 3]], [[1, 0], [2, 0], [2, 1]]], tail: TAIL.out },
  stepRight: { dy: 1, arms: [ARM.down, ARM.up], legs: [[[-1, 0], [-2, 0], [-2, 1]], [[1, 1], [1, 2], [1, 3]]], tail: TAIL.up },
  jump: { dy: -4, arms: [ARM.up, ARM.up], legs: [[[0, 1], [1, 2], [0, 3]], null], tail: TAIL.up },
};

/** The grid of `dancer` in `pose`: 24 rows of 24 characters, `.` transparent. */
export function dancerGrid(dancer, pose) {
  const c = CHARACTERS[dancer] ?? CHARACTERS.robot, p = SHAPES[pose] ?? SHAPES.still;
  const cells = Array.from({ length: GRID }, () => Array(GRID).fill('.'));
  const put = (x, y, ch) => { if (x >= 0 && y >= 0 && x < GRID && y < GRID && ch !== '.') cells[y][x] = ch; };
  const sprite = (rows, top) => rows.forEach((row, y) => [...row].forEach((ch, x) => put(LEFT + x, top + y, ch)));
  const top = FLOOR - LEG - c.torso.length - c.head.length + 1 + p.dy;
  const torsoTop = top + c.head.length, hip = torsoTop + c.torso.length - 1;
  const shadow = p.dy >= 0 ? 6 : 4;
  for (let x = 12 - shadow / 2; x < 12 + shadow / 2; x++) put(x, GRID - 1, 'D');
  const right = p.legs[1] ?? p.legs[0].map(([dx, dy]) => [-dx, dy]);
  for (const [hx, path, side] of [[LEFT + 2, p.legs[0], -1], [LEFT + 5, right, 1]]) {
    for (const [dx, dy] of path) put(hx + dx, hip + dy, c.leg);
    const [fx, fy] = path.at(-1);
    put(hx + fx, hip + fy, c.foot); put(hx + fx + side, hip + fy, c.foot);
  }
  if (c.tail) for (const [dx, dy] of p.tail) put(LEFT + 7 + dx, hip - 1 + dy, c.tail);
  sprite(c.torso, torsoTop);
  for (const [shoulder, flip, path] of [[LEFT, 1, p.arms[0]], [LEFT + 7, -1, p.arms[1]]]) {
    for (const [dx, dy] of path) put(shoulder + dx * flip, torsoTop + dy, c.arm);
    const [hx, hy] = path.at(-1);
    put(shoulder + hx * flip, torsoTop + hy, c.hand);
  }
  sprite(c.head, top);
  return cells.map(row => row.join(''));
}

/** The moves taken in turn, one a beat; how long a move is held; how strong a beat jumps; the loudest band of silence. */
export const MOVES = Object.freeze(['armsUp', 'stepLeft', 'armsUp', 'stepRight']);
export const BEAT_HELD = 0.28, JUMP_STRENGTH = 0.8, SILENCE = 0.04, DEMO_TEMPO = 120;

/** The pose for `frame` ({ playing, levels, beat }): like the backend's `dance_pose`. */
export function dancePose({ playing = true, levels = [], beat = null }) {
  if (!playing || !levels.some(v => v >= SILENCE)) return 'still';
  if (beat && beat.since < BEAT_HELD) return beat.strength >= JUMP_STRENGTH ? 'jump' : MOVES[beat.count % MOVES.length];
  return 'down';
}

/** The demo's beat at `seconds`, like the backend's `demo_beat`: steady, every eighth one strong. */
export function demoBeat(seconds) {
  const beats = Math.max(0, seconds) * DEMO_TEMPO / 60, count = Math.floor(beats);
  return { count, since: (beats - count) * 60 / DEMO_TEMPO, strength: count % 8 === 7 ? 1 : 0.6 };
}

/** Draws `dancer` in `pose` in `area`, like the backend: whole pixels, centred across, standing on the bottom. */
export function drawDancer(ctx, area, dancer, pose) {
  const side = Math.min(area.width, area.height);
  if (!(side > 0)) return;
  const cell = side >= GRID ? Math.floor(side / GRID) : side / GRID;
  const snap = (v) => (cell >= 1 ? Math.floor(v + 0.5) : v);
  const ox = snap(area.x + (area.width - cell * GRID) / 2), oy = snap(area.y + area.height - cell * GRID);
  dancerGrid(dancer, pose).forEach((row, y) => [...row].forEach((ch, x) => {
    if (!DANCER_PALETTE[ch]) return;
    ctx.fillStyle = DANCER_PALETTE[ch];
    ctx.fillRect(ox + x * cell, oy + y * cell, cell, cell);
  }));
}
