# Changelog

All notable changes to Bezel Evo are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/). The version is the one in the
`VERSION` file, set when a Windows release is built.

## [0.2.2] - 2026-10-24

0.2.1 was a local build; this entry and the one below cover it.

### Fixes
- Opening, importing or creating a theme clears the previous canvas image and
  waits for outgoing preview work before replacing the document.
- A video or display setup error no longer prevents the new theme's poster
  and objects from appearing in Studio. Runtime errors retain their cause.
- Changed documents request a complete preview even when their pixels match
  the previous frame.

## [0.2.1] - 2026-10-24

### Studio
- Importing `.turtheme` files includes fonts found in the vendor app's `fonts`
  folder. Studio uses installed families when available and reports missing
  fonts replaced with Inter.
- Imported GPU fan readings use RPM, and memory model text uses the memory
  sensor. Optional fan and memory text keeps the vendor's saved text when
  the PC has no matching sensor.
- Imported weather text uses Bezel's weather, initially for Rome, with a
  reminder to choose your city in the inspector.

### Fixes
- The media player's cover gap no longer shows a long decimal such as
  7.199999999999999 px in the inspector; it reads 7.2 px.

## [0.2.0] - 2026-10-22

0.1.64 was a local build; this entry and the one below cover it.

### Studio
- **A new inspector**: the controls of every object are in sections you open
  and close (Layout, Data, Shape, Text, Color, Behavior), each with a summary
  when closed; Studio remembers which are open for each kind of object.
  **▶ Try** in the preview plays an object with trial values without saving.
- **Handles for what an object draws**: a ring's start and end, a needle's
  arc and zones, a graph's scale, a value's parts and size, a clock's hands,
  an image's anchor, a player's cover fade, a Card's faces. Each shows its
  value while dragged, lights up its inspector field and takes the arrow keys.
- **A floating bar** above the selection: linked sensor, ▶ Try, Duplicate and
  the object menu.
- **Layer order** in Layout and the shared object menu, including moving a
  selection forward or backward in one undo step. Duplicate and Delete stay
  in the object menu.
- **Compact inspector controls**: consistent alignment, color and motion
  controls; distinct spectrum baseline and peak icons with nearby labels;
  one shared corner editor with a compact radius and individual-corner mode.
- **Trial values** in the canvas bar: the whole theme sweeps every sensor from
  empty to full, so you can see it at 0 % and at 100 %. Nothing is saved and
  the screen keeps showing the real sensors.
- **Top bar and status bar**: the save state sits under the theme's name; the
  status bar says what is selected or what a drag does, and has three chips
  for the sensors, Bezel (draw time and memory, with Copy diagnostics) and the
  screen (live state, frames, theme and active profile).
- **Panels**: widgets in four groups, sensors by category with full names and
  small graphs, an Align bar in Layers, and media that say where they are used.
- **Theme gallery** with real previews, the sensors each theme reads, where it
  is used, a 30-second try on the live screen and export to `.bezeltheme`.

### Widgets
- **Values and texts**: parts (prefix, number, unit, suffix, hours, minutes…)
  with their own font, colour and size, raised units, fixed-width digits,
  long text that shrinks, is cut or scrolls, Count and Roll transitions, an
  average over readings and a text shadow.
- **Analog clocks**: dial, ticks, numbers, three hands, shapes, date.
- **Needle gauges**: zones, ticks and numbers, needle shapes, pivot cap, a value
  in the centre and a peak marker.
- **Animated weather icons**: still, gentle or lively, each condition with its
  own movement.
- **Images and GIFs**: an anchor point for cropped pictures; GIFs play once or
  in a loop, at the speed you choose.
- **Graphs**: two series in different styles; bar thickness.
- **Four new themes** for 1920×480 screens: Pixel, Nebula, Paper and Now
  Playing, with the fonts Fraunces, Instrument Sans, Space Grotesk and
  Bricolage Grotesque now included.

### Profiles
- **New rules**: a sensor above or below a value for a while, times and days,
  idle, PC locked; combine a profile's rules with *any* or *all*.
- **A new Profiles dialog**: automatic or manual, the profile active now and
  why, rules as sentences, automatic saving and a 30-second try.

## [0.1.64] - 2026-10-20

### Widgets
- **A picture inside a shape**: a shape can show a PNG, JPEG, GIF or SVG over
  its fill and under its outline, cut to the shape. Choose how it covers the
  shape: fill (crop), fit inside, stretch, center, or tile in both directions,
  along a row or down a column; size and shift the copies, and fade the
  picture on its own. SVG stays sharp at any size and GIFs keep playing.

## [0.1.63] - 2026-10-20

0.1.62 was a local build; this entry and the ones below cover it.

### Fonts
- **Doto**, **Doto Rounded** and **IBM Plex Mono** are now built in, next to
  Inter and JetBrains Mono: every theme can use them on any PC with nothing to
  install. Doto is a dot-matrix face for clocks and big readings, in nine
  weights; IBM Plex Mono, Studio's own typeface, comes in seven weights with
  italics.

## [0.1.62] - 2026-10-20

0.1.61 was a local build; this entry and the one below cover it.

### Fixes
- The installer no longer reports "Another Bezel installation owns
  Bezel-Sensors" when updating over itself. Windows stores the sensor task's
  account in its short form, which the ownership check did not recognise; it
  now compares accounts by their security identifier.

## [0.1.61] - 2026-10-20

### Studio
- Remove unused imported files directly from **Media → This theme** with the
  trash button. Files still referenced by the theme are protected, and editor
  Undo retains the media it needs. Original source files are preserved.

## [0.1.60] - 2026-10-20

### Studio
- Visualization presets now have selectable schematic previews instead of a
  dropdown, with labels that stay readable in narrow inspectors.
- **Fit matrix cells** adjusts a block matrix's height for near-square cells,
  keeps its position and width, and supports Undo. Pending grid edits are
  committed before fitting.
- Clearer pixel-count and lower-series scale labels.

## [0.1.59] - 2026-10-20

0.1.32 to 0.1.58 were local builds; this entry and the ones below cover them.

### Studio
- The ▶ button in **Profiles and automation** now tries the transition towards
  the selected profile (or the base themes), saved or not, holds it for three
  seconds on the live displays and comes back the same way.
- New transitions default to 1500 ms; the help text recommends 1500 to 2500 ms,
  where the covers move smoothest.

### Widgets
- Native block matrices, square-pixel arcs, modern dials with scale ticks and
  endpoint markers, rounded history bars, and mirrored two-sensor graphs.
- Choose presets in Look → Visualization. Mirrored graphs have an independent
  lower source and color; automatic scaling shares zero and peak across both
  series. Missing readings remain gaps. Existing themes keep their appearance.

### Fixes
- Smoother covers: the lit edge of Curtain, Wipe and Blinds spills a little
  light onto the scene ahead of it (never over video the screen plays itself),
  and Mosaic tiles grow from their middle instead of appearing at once.
- Clicking a searchable sensor result keeps the popup open through the native
  focus change so the selected sensor is actually applied, just like keyboard selection.

## [0.1.58] - 2026-10-20

### Studio
- Rules and profile icons in **Profiles and automation** take the colour of
  what they watch, music or an app, and the controls inside each rule read as
  part of the sentence.

### Fixes
- The ▶ button now tries the transition on every live display, also those
  Light opened, and says when no display is live.
- No focus ring around the whole profile dialog after removing a rule.

## [0.1.57] - 2026-10-20

### Studio
- Redesign **Profiles and automation**: choose Automatic (with what shows when
  no rule holds) or Manual on two cards, pick the base themes or a profile
  from a list beside its detail, and edit each rule as a sentence. Deleting a
  profile asks first. "Selected themes" is now **Base themes**.
- Try the profile transition on the live displays with the ▶ button, without
  saving it.

### Fixes
- Profile covers move at up to 60 frames a second instead of 20, paced by
  what each display takes.

## [0.1.56] - 2026-10-19

### Studio
- Profile changes now use one of four covers: **Curtain**, **Wipe**, **Blinds**
  or **Mosaic**. A panel closes over the screen, the new theme is prepared
  underneath and the panel opens on its first frame, so loading never shortens
  the motion. Covers work on every display size and shape, also over a video
  the screen plays itself. Saved Fade, Slide and Flip settings become Mosaic,
  Wipe and Curtain.
- Show a solid background colour once in the theme inspector, as the field
  that changes it.

## [0.1.55] - 2026-10-19

### Fixes
- Remember the association between a display port and its wake controller.
  After Light restarts, a sleeping display can be found through the same wake
  controller even when its saved display port is absent or has changed.
  Recovery never falls back to another available monitor.

### Studio
- Name the base profile **Default** in the profile manager and tray, while
  keeping its individual display assignments visible.

## [0.1.54] - 2026-10-19

### Studio
- Use one searchable sensor picker for widget sources, colour rules and motion.
  Search names, keys, categories or probes; grouped results show live readings
  and availability, with keyboard navigation and numeric-only choices where needed.
- Configure sensor rotation with **Full speed at**, **One turn every** and
  **Start and stop** presets, including native sensor units.

### Widgets
- Add **Inertia** to value transitions: readings retain their velocity as the
  target changes, then settle within the selected time. Reduced motion still
  snaps to the reading; older builds fall back to the smooth transition.

## [0.1.53] - 2026-10-19

### Widgets
- Drive visual rotation from a numeric sensor: fan RPM, load or another reading.
  Set the reference reading, maximum visual turns per second, inertia and direction.
  Zero, negative or unavailable readings decelerate to a stop at the last angle.
- Apply motion to custom images, icons, shapes and other ordinary widgets,
  including card/group members. Each scene keeps its own motion state;
  output follows each display's pace without increasing sensor sampling.

### Studio
- Configure sensor-driven motion in Look. The isolated preview animates with
  room for every angle; reduced motion shows the original static orientation.
- Keep motion optional in theme files, preserving unchanged older themes.

## [0.1.52] - 2026-10-19

### Studio
- Filter Layers by object type alongside name or sensor search. Distinguish
  text, values, clocks and other widgets; keep matching objects' card/group
  ancestry visible and temporarily expand folded containers while filtering.
- Keep complete contents when a card or group itself matches the filters.
  Filtering does not change the theme, selection or saved fold state.

## [0.1.51] - 2026-10-19

### Fixes
- Wrap long sensor-source descriptions within the Sensors panel, including
  unbroken hardware paths, so they cannot force horizontal scrolling.

## [0.1.50] - 2026-10-19

### Studio
- Give the probe summary's scrollbar more breathing room beside labels and
  restart buttons, keeping its space reserved as the list changes.

## [0.1.49] - 2026-10-19

### Studio
- Use slim rounded scrollbars throughout Studio, with transparent tracks,
  no arrow buttons and clearer hover/drag feedback in light and dark themes.
  Preserve native scrollbars in Windows forced-colour mode.

## [0.1.48] - 2026-10-19

### Studio
- Group primary GPU aliases and unsupported metrics under their actual reader
  instead of showing individual GPU sensor keys as separate probes. Keep probe
  counts and Sensors filters consistent across multiple graphics adapters.
- Move restart onto each supported probe's row, with progress and duplicate-click
  protection. Currently only the managed LibreHardwareMonitor reader supports
  restart; remove the global footer and status-bar restart buttons.

## [0.1.47] - 2026-10-19

### Studio
- Summarize sensor health by probe instead of listing physical hardware and
  individual values. Show managed, available, unavailable and not-yet-checked
  sensor counts, with direct links to the corresponding Sensors view.
- Filter Sensors by probe and reading state alongside category and search.
  Keep checking the selected probe's scope so recovered readings leave the
  failed view automatically without disrupting unchanged rows or dragging.
- Count text readings and valid zeros as available; distinguish unread sensors
  from failed readings. Preserve Libre-only restart and keyboard navigation.

## [0.1.46] - 2026-10-19

### Studio
- Preview desktop colour picks with a magnified pixel grid, marked centre pixel,
  colour swatch and HEX value beside the cursor. Keep the preview on the pointed
  monitor, including negative monitor origins and high-DPI displays.
- Freeze the selection surface temporarily so the magnifier and selected colour
  match exactly and the preview cannot sample itself. Keep captures only in
  memory for the duration of the pick; Escape and right-click still cancel.

## [0.1.45] - 2026-10-19

0.1.44 was a local build; this entry and the one below cover it.

### Packaging
- Provide Windows x64 installer and portable packages with smooth native graph
  scrolling, independent display pacing and the rendering frame-budget fix.

## [0.1.44] - 2026-10-19

### Widgets
- Scroll line, area and bar histories smoothly between real sensor samples,
  targeting up to 50 frames per second when rendering and the display permit.
  Preserve sampling intervals, unavailable-reading gaps and saved theme formats.
- Pace graph animation independently for each display: skip intermediate motion
  while its device is busy, retain fresh samples and avoid accumulating frames.
  Respect reduced motion and keep card-face caches from freezing moving graphs.
- Bound graph motion after suspension to the normal sampling period. Scrolling
  introduces up to one sample interval of visual delay without predicting values.
- Include rendering time in the runtime's frame budget instead of adding it to
  the wait between frames.

## [0.1.43] - 2026-10-18

### Fixes
- Use a native Windows eyedropper in Studio to pick a colour from any monitor,
  independently of browser EyeDropper support. Keep the selected opacity and
  allow cancellation with Escape or right-click.
- Report eyedropper failures instead of silently ignoring them; prevent duplicate
  picks and ignore results arriving after their colour panel was dismissed.

## [0.1.42] - 2026-10-18

0.1.32 through 0.1.41 were local builds; these entries cover them too.

### Studio
- Format any text as uppercase, lowercase, first letter uppercase or each word
  capitalized, or preserve the original. Case formatting supports Unicode and
  applies to static text, sensors, clocks, weather and media without rewriting
  their source. Drawing and natural frame sizing use the same formatted text.
- Save case formatting with the theme and undo changes. Existing theme styles
  retain their original appearance; clock-specific formatting remains compatible.

### Themes
- Add six graphic-only animated flagship themes alongside the six modern
  originals: Synapse / Kinetic, Prism / Kinetic and Morph / Sound, each with
  Wide and Round layouts.
- Synapse combines animated orbital signals, segmented instruments and live
  audio; Prism pairs a changing geometric knot with poster typography and
  telemetry; Morph layers a live audio sculpture over generative geometry.
- Decorative motion is a bounded, local transparent GIF loop; sensor readings,
  graphs, gauges and audio spectra are native live widgets. No photographs or
  external downloads are required. Hide the Motion layer for a still layout.
- Redesign all six modern templates with distinct art direction: Pulse's
  calibrated instrument cockpit, Resonance's album-led listening room and vinyl
  spectrum, and Clarity's copper editorial clock and contrasting memory module.
- Add local vector artwork, matching round display layouts and clearer visual
  relationships between load, temperature, histories and audio activity.
- Music templates include a designed standby face and automatically crossfade
  to album artwork and track details when a media session starts playing.

## [0.1.41] - 2026-10-18

0.1.32 through 0.1.40 were local builds; these entries cover them too.

### Studio
- Profile changes reuse the Card effects: Fade, Slide and perspective Flip,
  alongside an instant switch. Set direction and duration once in Profiles and
  automation, for manual, automatic and return changes across all displays.
- Transitions follow each display's presentation speed, interrupt from the
  current image on rapid switches and preserve the editor's document and preview.
  Video played directly by a device switches immediately.

### Themes
- Add six editable templates to the bundled library: Pulse / Gaming, Resonance /
  Music and Clarity / Monitor, each in 1920×480 Wide and 480×480 Round versions.
- Demonstrate rotating Cards, smooth values, colour thresholds, gradient gauges,
  history graphs, palettes, independent player corners and progress styling,
  and live spectra from the computer's output audio.

## [0.1.40] - 2026-10-18

0.1.32 through 0.1.39 were local builds; these entries cover them too.

### Studio
- A profile now contains a theme assignment for each display. Choosing Gaming,
  manually or through an automatic rule, applies its different themes across
  both screens from one shared profile selection.
- Profile management shows the selected theme names for each display and seeds
  new profiles from those assignments. Theme pickers show only compatible themes
  for their display, while saved assignments for disconnected displays are kept.
- Return choices in Studio and both trays show the selected theme names instead
  of the ambiguous “Original theme” label.
- Existing single-display profiles migrate to the new assignment format without
  losing their saved theme or rules.

## [0.1.39] - 2026-10-18

0.1.32 through 0.1.38 were local builds; these entries cover them too.

### Studio
- Manage display profiles from Themes → Profiles and automation, or open the
  same editor from either tray menu. Profiles associate a saved theme with a
  display and optional application or media playback rules.
- Select a profile manually from the Studio or Light tray, or enable Automatic.
  Rules support foreground apps, running apps and media playing, with priorities,
  a configurable return delay and a fallback profile. Manual selection persists.
- Profile switching pauses while a display is being edited and resumes when
  Studio is hidden. Unsaved documents and the editor's preview remain separate
  from profile output; Light applies profiles without Studio running.
- System sensors have separate PC information, Audio and Power groups, full
  text values, readable keys and explanatory value tooltips. System status values
  use localized Yes/No labels and omit unnecessary sparklines.

### Sensors
- Read the Windows default output's volume, mute state and device name, recovering
  automatically when the output changes.
- Add Windows version, running process count, AC power, battery percentage,
  charging state and estimated remaining time. Missing hardware or estimates
  remain unavailable instead of being shown as zero.

## [0.1.38] - 2026-10-18

0.1.32 through 0.1.37 were local builds; these entries cover them too.

### Studio
- All widget property panels share the new inspector design: icon-led section
  headings, orange navigation states, cream primary text in dark mode, and
  lighter grouping with fewer nested borders.
- Data and Look are clearly separated from geometry by dedicated navigation
  spacing. Common properties remain outside the tabs, and geometry fields
  integrate their labels, values and units without changing numeric behaviour.
- Transitions use compact duration readouts; Fahrenheit is offered only for
  temperature sensors. Typography, conditional colours, player parts, card
  controls and all widget-specific settings remain available.
- Visible preview content is centred independently of the widget's saved
  text alignment; rendered pixels and screen output remain unchanged.

## [0.1.37] - 2026-10-18

### Studio
- Object inspectors show preview, membership and geometry before their tabs.
  Membership stays visible on every tab, including the top-level context.
- Convertible sensor widgets show their type selector beside their name;
  changing type no longer requires opening the more-actions menu.
- Numeric labels stay beside their compact controls. Tighter section spacing
  and a fit-to-content action in the geometry heading reduce vertical scrolling.

## [0.1.36] - 2026-10-18

0.1.32 through 0.1.35 were local builds; these entries cover them too.

### Studio
- Every object inspector starts with position and size, followed by the
  preview and property tabs, including graphs, spectrum, players and cards.
- Appearance groups follow one shared order: structure and typography,
  colours, edges and corners, then effects and transparency.

## [0.1.35] - 2026-10-18

0.1.32 through 0.1.34 were local builds; these entries cover them too.

### Studio
- Inspector number inputs now use compact controls, with explanatory icons
  and units for geometry, rotation, typography and spectrum settings.
- Conditional colours sit beside the paint they control. Colour, track,
  segments, outline, transparency and corners have clearer property groups;
  opacity stays with transparency or in its own visibility group.
- Shared corner controls keep the same drawing and editing behaviour across
  shapes, bars, spectrum bars and player covers, with compact shared radii.
- Colour and font pickers stay inside the window when their contents grow.

## [0.1.34] - 2026-10-18

0.1.32 and 0.1.33 were local builds; this entry and the two below cover them.

### Fixes
- Windows Setup waits for the sensor reader to release its files before an
  update or uninstall, including elevated readers whose process path Windows
  hides. Updates also check before running an older uninstaller, preventing
  write errors on sensor DLLs such as BlackSharp.Core.dll. Open Studio and
  Light still block the operation so you can save and close them yourself.

## [0.1.33] - 2026-10-18

0.1.32 was a local build; this entry and the one below cover it.

### Studio
- **Rotate objects**: a **Rotation** field in the inspector and a handle on
  the canvas turn any object around its centre; hold **Shift** to snap to
  15°. Text, graphs, the spectrum and pictures turn with it, on the screen and
  in every preview. Alignment and X/Y/W/H keep using the unrotated box. The
  device background, Cards, groups and their members do not rotate.

## [0.1.32] - 2026-10-18

### Widgets
- **Audio spectrum source**: each spectrum listens either to the app its
  Player follows, as before, or to **the whole PC**: everything playing on the
  output in use, with no Player needed. It is a choice, never a fallback.
- **Amplitude (×)**: a multiplier from 0.1× to 4× for the height of the bars,
  for loud sources such as Spotify that keep them near the top.
- **Transparency midpoint**: a linear transparency can move the point where it
  is halfway, from 5 % to 95 % across the object.

## [0.1.31] - 2026-10-17

0.1.30 was a local build; this entry and the one below cover it.

### Fixes
- An audio spectrum with nothing playing, or no sound yet, shows its bars at
  rest instead of "Audio capture disabled". That message now appears only when
  you turn capture off in **Preferences**.
- When a screen was off at start-up, Studio could take the screens away from
  Light and keep them until it was restarted, and the audio spectrum stopped
  moving. Studio now waits for a busy Light and takes over only if Light has
  really stopped.
- Debug logs keep hours of history instead of a few minutes: the editor's
  frame-by-frame traffic is summed every 10 seconds.

## [0.1.30] - 2026-10-17

### Widgets
- **Audio spectrum styles**: next to the mirrored bars, the spectrum can now
  draw **bars** rising from the bottom, **LED blocks** that take the colour of
  their own height like a level meter, a smooth **line** with an optional fill
  below it, and **radial** bars around an inner ring. Choose the style under
  **Look**; the inspector shows only the settings of that style.

### Fixes
- The audio spectrum moves evenly on the screen: the bars no longer stall for
  a moment about once a second, just before the sensors are read.
- A spectrum on every Card face keeps moving, and sensor readings no longer
  hold it back.

## [0.1.29] - 2026-10-17

0.1.27 and 0.1.28 were local builds; this entry covers them too.

### Widgets
- **Audio spectrum**: a new widget that draws the sound of the app a Player
  follows (Spotify, a browser, …) as mirrored bars, with a centre line, held
  peaks, three motion presets, solid or gradient colour, colour by value and
  rounded bar corners. It listens only to that app, never the microphone or
  the whole PC, analyses the sound in memory and records nothing. You can turn
  it off in **Preferences**, and the widget can show a **demo** for a quick
  look. It needs Windows 11 (build 20348 or newer).
- A spectrum on a Card face that opens when music plays works like any other.
  When playback stops the bars fall to zero.

### Fixes
- The **Render** chip in the status bar keeps its width when the time gains a
  digit, and its text is centred again.
- Light no longer crashes when the audio spectrum starts listening to an app.
- The audio spectrum hears apps installed from the Microsoft Store, such as
  Spotify, instead of reporting the source as unavailable.

## [0.1.26] - 2026-10-16

The first published release since 0.1.22 (0.1.23 to 0.1.25 were local builds),
so this entry covers everything since then.

### New interface
- **Studio is redesigned from the ground up**, in dark and light: an icon rail
  for Widgets, Sensors, Layers, Themes, Media and Screen; floating tool bars
  over a dot-grid canvas; library and properties panels you can resize; a new
  Pixel logo and tray icon. Studio now speaks Italian too, next to English and
  Portuguese.
- **The inspector shows what you change**: a live preview of the selected
  object, drawn on its own at the top of the panel, and its settings split
  into **Data** and **Look** tabs. Number fields keep their focus while you step
  them with the arrow keys.
- **One colour picker everywhere**: colour area, hue, opacity, hex code and an
  eyedropper, with **Theme colours** saved in the theme, the colours **used in
  this theme** and recent ones.
- **A real font picker**: search, Sans/Serif/Mono filters, each name written in
  its own font, a preview with the object's real text, and only the weights the
  font really has. A font the PC lacks is marked, with the font used instead.
- **Status at a glance**: a sensor chip that opens a popover with each piece of
  hardware, its last reading and **Restart Libre**; a memory chip with how much
  Bezel uses, per process; **Preferences → Logs** with **Open logs folder**.
- **Quick hands**: the mouse wheel zooms around the pointer (touchpad pinch
  too), the middle button pans, **Fit** shows the whole screen. The focus ring
  appears only when you use the keyboard.
- **Themes panel**: search, **New theme**, a larger theme gallery with
  **Duplicate** and **Open in the editor**, and when each theme was changed.
- **Screen panel**: the chosen screen only, with brightness, start with the
  computer, the shutdown choice and **Advanced**; a clearer Storage manager.

### Editing
- **Double-click an object** (or **Fit frame to content**) to fit its frame to
  what it draws: text in its font, images and icons at their proportions, the
  weather and media player blocks.
- **Colour by value**: texts, values, rings, bars, needles, graphs, shapes and
  SVG icons change colour at the thresholds you set (for example orange from 70,
  red from 90).
- **Smooth values**: rings, bars, needles and values glide to each new reading.
  New objects start smooth at 400 ms; older themes keep jumping until you turn
  it on. Frames are sent only while something moves, fewer to a slow screen.
- **One corners control** for shapes, Cards, images and the player's cover, with
  a drawing that stays readable at any size.
- **The background chooser**: one card shows the background in use; **Change
  background…** offers a solid colour, a picture or video from this computer,
  or a video already on the screen, and nothing changes until you confirm. The
  theme's orientation is two buttons and its refresh has quick values.

### Widgets
- **Cards**: the faces are a plain list with **Edit** and a menu (rename,
  duplicate, move, delete), a **Shared background** row, and **one transition**
  for the whole Card. Automatic rotation is now a **timed trigger** ("every
  30 s, the next face") next to the app and media triggers; themes using the
  old rotation are converted when opened.
- **Media player**: choose its parts (cover, title, artist, time, progress, app
  name) and style the progress bar: continuous, segments or dots, round or
  square ends, a knob, times in the text, at the sides or above.
- **Graphs** have their own time window, written as a sentence ("show the last
  5 minutes, with one point every 5 seconds") over a time ruler, independent of
  the theme's refresh.

### Light and screens
- **Light owns the screens.** One runtime in Light runs the themes, the clock,
  the sensors, the graph history, the transitions and the background videos;
  Studio's canvas shows the picture Light draws. Opening and closing Studio no
  longer redraws the screen: graphs keep their history, a video keeps playing.
  Closing without saving goes back to the saved theme without starting over.
- Studio and Light talk over a local channel that only your Windows user can
  open. If Light cannot run, Studio drives the screens itself.
- Each screen has its own worker and its own sampling, so a slow screen does not
  hold up a fast one; Studio shows the measured time to draw a frame.

### Fonts
- **Inter and JetBrains Mono are built into Bezel**: themes always find them,
  wherever they are saved.
- **My fonts**: drop `.ttf`, `.otf` or `.ttc` files in
  `%APPDATA%\Bezel Evo\fonts` to use them without installing them in Windows.
  **Open fonts folder** and **Reload fonts** are in the font picker.
- **Theme fonts**: **Include in theme** packs a font file inside the theme, after
  a licence reminder, so a shared theme looks the same elsewhere; **Remove font
  from theme** takes it out again.

### Fixes
- Themes with a refresh slower than 2 s are honoured instead of being capped,
  and changing the refresh realigns sampling.
- Sensors of identical hardware (two equal fans, two equal disks) are kept
  apart.
- The inspector no longer jumps when stepping number fields; the shared
  background is named in a member's face picker.
- The sensor catalogue is no longer reloaded at every sample.
- Updated rustls with its security fix.

### Platform: Windows only
- Bezel Evo supports **Windows 10 and 11 only**. The documentation, the user
  guide and its Portuguese translation now describe Windows only.
- The inherited CI pipeline, which built Linux packages, is gone; the Windows
  packages are built on demand.

## Inherited from Bezel

The entries below describe [Bezel](https://github.com/slipalison/bezel) up to
the point this fork started. They are kept for history: the Linux packages,
udev rules, systemd service and other Linux items no longer apply to Bezel Evo.

### Added
- Cargo workspace with the hexagonal core (`bezel-core`), the device adapter
  (`bezel-devices`), the `bezel` CLI and the Bezel Studio app.
- `bezel devices`: lists the connected smart screens without writing to them.
- Reverse-engineering specification of every supported protocol and file
  format in `docs/reverse-engineering/`.
- Screen drivers for every protocol family: Turing rev A, XuanFang rev B,
  Turing rev C (with the wake micro-controller), Kipye rev D, WeAct Studio
  Display FS, the Turing/TURZX USB generation (0x1CBE) and WCH panels
  (0x43A8); 44 models in the catalog, the Turing 8.8" validated on hardware.
- `bezel test-pattern`, `bezel brightness`, `bezel show <picture>`,
  `bezel off` and `bezel release`; orientations are `vertical`,
  `horizontal` (or `portrait`, `landscape`) and their flipped forms.
- A screen held by another program is refused with that program's name and
  PID instead of garbling both streams.
- `bezel sensors`: every sensor of the machine (CPU per core, temperatures of
  every hwmon chip, NVIDIA and AMD GPUs, memory, disks, network) as a table,
  JSON or a live `--watch`; a sensor that cannot be read says why instead of
  showing a guess. Windows reads LibreHardwareMonitor when it runs.
- `bezel render <theme> -o out.png` renders one frame of a theme with the
  machine's sensors (or the demo values with `--fake`) to a PNG of the canvas
  size; it previews `.turtheme` and turing-smart-screen-python themes too.
- `bezel run <theme>` shows a theme on the screen with live sensors at the
  theme's refresh rate until Ctrl+C (or SIGTERM), then hands the screen back to
  its standalone mode.
- `bezel import <src> -o <dst>` converts a TURZX `.turtheme` or a
  turing-smart-screen-python theme into a native `.bezeltheme` (or folder) and
  prints what could not be converted exactly.
- Bundled "Midnight" themes for the 8.8" (horizontal and vertical), 5", 3.5"
  (both ways) and 2.1"/2.8" round screens, with the Inter and JetBrains Mono
  fonts (SIL Open Font License 1.1); a theme can be named instead of a path.
- Bezel Studio, the desktop app: a single window with a drag-and-drop theme
  editor (widgets and sensors from the library onto a canvas that shows the
  real renderer's frame; snapping, alignment, layers, undo/redo, pt-BR and
  English, light and dark), vertical or horizontal themes with one click, live
  mode on the screen that keeps running from the tray, start at login, the theme
  library with bundled themes and import of other apps' themes.
- Bezel Studio: a Storage tab in the Screen panel — usage bars for the internal
  flash and the memory card, files per folder, sending by drag-and-drop with a
  progress bar and Cancel, play/stop, and delete or the boot media behind a
  confirmation dialog naming the file; a theme with a video background offers
  "Send to the screen" and then plays over the video the screen loops; for screens
  that cannot play videos, live mode decodes it on the computer, as `bezel run`
  does.
- `bezel storage info|ls|put|rm|play|stop|boot`: the screen's internal flash
  and memory card (sizes, `--json`), sending pictures and videos with a
  progress bar and Ctrl+C to cancel, device-side playback and the boot media.
  Deleting, replacing and changing the boot media print what they will do and
  need `--yes`; without it nothing reaches the screen.
- `bezel storage put` converts a video to the panel's format with an external
  ffmpeg (`--ffmpeg PATH`, else `PATH`): turned for `--orientation`, cropped to
  the panel's shape instead of stretched, optional `--fps`; without ffmpeg,
  clips already in the format still go and the install command is shown. A
  full screen lists what could be deleted and deletes nothing.
- `bezel run` with a video-background theme has the screen loop the stored
  video under the theme, shows the poster with the `bezel storage put` command
  when the video is missing, and decodes it on the computer for screens that
  cannot play videos (`--ffmpeg PATH`).
- Bezel Studio gives a theme a video background: **Add video…** in the Media
  tab, or a video or animated GIF dropped on the tab or on the editing area,
  copies it into the theme with a poster taken by ffmpeg (shown under the
  elements; without ffmpeg the video comes without one), and **Properties**
  shows it with whether the screen stores it. An animated GIF sent to a screen
  is converted to a video at a constant frame rate.
- A cancelled upload reports the incomplete file it left and the command that
  deletes it; an upload whose stored size differs from the file says to delete
  it and send it again.
- A Turing rev C screen that froze (it stopped reading what Bezel sent, or it
  is on the bus but answers nothing) is restarted through its wake chip
  without a USB replug: once, on its own, by the next connection (the next
  command, `bezel run` started again, turning Live on), with
  `bezel restart [-s SCREEN]`, or with **Restart screen…** in the studio's
  Screen panel, which errors meaning a frozen screen also offer. It is back in
  about 10 s; what it played stops, its stored files stay.
- Animated GIF elements move at their own pace, between the theme's
  refreshes: up to 30 frames a second on the screen (only the GIF's rectangle
  is sent; a slow link skips frames instead of lagging) and 15 in the studio's
  preview, while sensors keep the theme's refresh. Full-screen GIFs belong in
  a video background.
- Live mode recovers from a screen that stops taking frames: `bezel run`, the
  service and the studio connect it again after 2, 5 and 10 s (restarting a
  frozen rev C screen on the way, found again under its new port), say so
  meanwhile, and stop with the error after the third attempt.
- `bezel udev-rules` prints the Linux udev rule generated from the device
  catalog and the one-line sudo command that installs it, for AppImage,
  archive and source installs; Bezel never runs it. A refused port points at
  it, and the studio shows the command, ready to copy.
- Turing USB panels in the vendor's desktop mode (1a86:ad10–ad13) are listed by
  `bezel devices` as "desktop mode (not validated on hardware)", and
  `bezel monitor-mode --yes` switches one back to USB monitor mode; without
  `--yes` (or the studio's confirmation) nothing is sent.
- Game FPS (`gpu.fps`), read-only: the RivaTuner Statistics Server shared
  memory on Windows, the newest MangoHud CSV log on Linux (`--mangohud-dir`).
  Nothing measuring a game, or a reading older than 3 s, is unavailable with
  how to turn the source on. Not yet validated with a real game.
- `net.ping`, the round trip to `--ping-host` (default 8.8.8.8), measured on a
  thread of its own so a silent host never delays the other sensors, and only
  while a shown theme or the studio's sensor list uses it (no traffic
  otherwise); fans,
  pump, voltages, network totals and available memory from hwmon/sysfs on
  Linux and LibreHardwareMonitor on Windows; the sensor keys of imported themes
  map to Bezel's.
- Bezel Studio in Portuguese and English throughout, following the system
  language unless one is chosen in Preferences; errors and import warnings are
  translated; the ping target and the MangoHud log folder are Preferences.
- Bezel Studio's Themes tab shows each theme as a thumbnail drawn by the real
  renderer with sample values (a video background shows its poster), kept in
  the app's cache until the theme changes, and names the screen it was made
  for (`8.8″ · 1920×480`). **For this screen** (the default while one is
  connected) / **All** and **Vertical** / **Horizontal** filter the list; the
  choice is remembered.
- Linux packages: the deb and the rpm install the `bezel` command as
  `/usr/bin/bezel`, the `bezel-run@` systemd user service in
  `/usr/lib/systemd/user` (running `/usr/bin/bezel`) and the bundled themes
  where the command finds them, next to the udev rule; installing applies the
  rule to serial, USB and HID devices at once.
- The storage manager on the command line:
  `bezel storage mv|rename|restore|cleanup|catalog|cache`. Bezel keeps a local
  copy of every file it sends (the exact bytes, in `<data>/bezel/storage`:
  `~/.local/share` on Linux, `%APPDATA%` on Windows), since the screens cannot
  send a file back, rename or move it. `bezel storage mv PATH... --to internal|sd`
  and `rename` send that copy again, check the stored size and only then delete
  the source; a failure or Ctrl+C keeps the source and stops the batch.
  `restore internal|sd` sends back the files missing from a formatted or new
  card, after checking the free space and the 25 MiB rev C limit before the
  first byte, and never deletes. `cleanup` lists the vendor app's duplicate
  copies (`x.mp4.mp4`, `x.mp4<digits>.mp4`), interrupted uploads and files no
  theme plays, never the boot media nor a theme's video; `cleanup --dry-run`
  only lists, `--yes` deletes exactly the pre-checked files it printed.
  `catalog associate` gives a file Bezel did not send a local copy from its
  original; `cache clear` removes the copies of deleted files (`--all`: every
  copy), which count toward a 2 GiB limit (`cache --limit`). Each command
  prints the exact list first and changes nothing without `--yes`.
- Bezel Studio: the Storage tab opens full width as the storage manager —
  internal memory and SD card side by side with thumbnails, multiple selection
  with the mouse or the keyboard, sort and filter, **Move to the SD card** (or
  dragging to the other side), copy, rename, restore (also of files deleted
  through Bezel, listed unchecked), the cleanup assistant with its exact list
  to confirm (a file that changed since is not deleted), associating an
  original and the local copies with **Clear cache…**; progress and results
  are announced to screen readers.
  On Turing USB screens, what ends in a delete (move, rename, delete, cleanup)
  is disabled with the reason, and sizes come from the catalog or show as
  unknown.
- User guide in English (`docs/user/`) and Portuguese (`docs/user/pt-BR/`):
  installing on each system, screen permissions and Windows drivers (WinUSB
  with Zadig, LibreHardwareMonitor), the unsigned installers and SmartScreen,
  the first theme, vertical or horizontal use, sensors, game FPS, storage and
  video with the storage manager, ffmpeg, preparing an SD card, running at
  login, coming from turing-smart-screen-python, troubleshooting and the
  supported screens.
- Video background framing in Bezel Studio: **Properties → Framing** turns the
  video (Auto, or 0, 90, 180 or 270°), fills or fits it (with a pad color),
  zooms it (100 to 400%) and places it (Position X and Y, Center, Reset
  framing); **Frame on canvas**, or a double-click on the video, does it on the
  editing area with the mouse (drag, wheel) and the keys (arrows, + and −, 0,
  Esc or Enter), each change or gesture one undo step. The editor's preview
  plays the video background, framed, at up to 15 pictures a second (the
  poster when motion is reduced or without ffmpeg). A theme keeps an optional
  `framing` in its video background (`rotation`, `fit`, `zoom`, `position`,
  `padColor`), left out when it is the default; `bezel run` honours it. A
  re-framed video goes to the screen as a copy of its own, named with `_f` and
  8 hex digits, and Bezel deletes no earlier copy. See
  [Framing the video](docs/user/storage-and-video.md#framing-the-video).
- GIF and sticker search in Bezel Studio, from KLIPY with your own free key:
  the Media tab's **Collection** (next to **This theme**) opens **Search GIFs
  and stickers** — GIFs or stickers, by text or **Trending**, 24 at a time with
  **Load more**, explicit results only while **Show explicit results** is on
  (off at every start). The key stays on this computer (`klipy.json` in the
  app's config folder, readable only by you) and shows only its last 4
  characters; without a key, or before you search, nothing reaches KLIPY. A
  test key allows 100 requests an hour, and the help says how to ask for
  production in KLIPY's Partner Panel. **Add to collection** keeps the largest
  GIF of at most 25 MiB, once per content, in your local collection, where each
  item can be added as an image, used as the background, dragged onto the
  canvas, renamed or deleted (the confirmation names the themes that use it;
  they keep their own copy). A sticker's transparency shows the theme under
  it. See [GIFs and stickers](docs/user/gifs-and-stickers.md).
- What a Turing rev C screen does when the computer shuts down or restarts,
  instead of staying frozen on the theme's last picture: **Leave as it is**
  (the default, as before), **Turn the screen off**, **Play a video stored on
  the screen** in a loop, or the **Photo album from the card** (the screen's
  own album of `sd/image`, a photo every 3 to 5 s). It is chosen per screen in
  Bezel Studio (**Screen → Settings**) or with
  `bezel standby set keep|off|video|album --yes` (`bezel standby show` lists
  it), behind a confirmation, and Bezel Studio carries it out at shutdown: on
  Linux through a systemd-logind delay inhibitor, on Windows when the session
  ends; quitting the app applies nothing. Choosing also stores a plan B on the
  screen for when the studio cannot act: the screen's sleep timer (1 to 10
  minutes) with **Turn the screen off**, or what it starts with (the album, or
  the first video of `sd/video`, since the screen cannot be told which file);
  **Leave as it is** undoes it. Without the studio, only the sleep timer acts
  while the screen stays powered: the album or a video starts when the screen
  itself starts again. `--brightness` is recorded with the plan B, and the
  album's restart at shutdown starts at it. A choice that cannot be carried out (the
  video or the card is gone) turns the screen off. Photos added to the album
  (`bezel standby album add`, or the studio's album) are stood upright by their
  EXIF orientation and framed for the way the screen stands, filled or fitted.
  A live rev C screen gets a one-pixel update after 30 s without traffic, so
  its sleep timer does not fire while a theme is live. See
  [When the computer shuts down](docs/user/power-off.md).

### Changed
- Turing rev C screens take at most 25 MiB per file: their firmware keeps a
  whole upload in memory and froze past about 28 MiB. A larger file is refused
  before anything is sent, with the limit in MiB; a conversion caps the
  video's bitrate from its length so it fits, and a converted video still over
  the limit is refused with how to make it fit (a shorter clip, `--fps`).
  Turing USB screens keep the vendor's 120 MB.
- A theme file that cannot be read or does not fit its screen fails with
  `theme file: …` instead of a transport error; the studio opens a
  `theme.json` like the command line does.
- A file a Turing USB screen stores without reporting its size counts as
  present, with an unknown size, when listing, playing and in video
  backgrounds.
- `scripts/install-local.sh` points the `bezel-run@` service it installs at the
  `bezel` it installed.
- `bezel storage ls` shows, for the files Bezel sent, their state (stored, or
  pending: an upload that did not finish) and whether Bezel keeps a local copy;
  `put`, `rm` and `boot`, and the studio's sending, deleting and theme videos,
  record what they do in Bezel's catalog.
- Setting what a rev C screen shows at start (`bezel storage boot`,
  **Show at start…**) keeps the sleep timer of the shutdown choice: the
  screen's settings are written whole from what Bezel recorded for it.

### Fixed
- A rev C screen that another app just turned off (turing-smart-screen-python
  and the vendor app send TURNOFF on exit) is woken instead of failing with
  "Broken pipe".
- A theme video stored turned for the screen, as in the vendor's horizontal
  themes (Dragon Ball: 480x1920 in a 1920x480 theme on the 8.8"), is turned
  back by the framing's Auto instead of being zoomed past the screen, and the
  `dragon.mp4` the screen already stores is found (Bezel looked for
  `dragon_90.mp4`) and used without sending it again when its size matches;
  framed by default, such a video is sent as it is, without ffmpeg. The
  studio's preview plays a video background instead of showing its poster.
- With a screen live in Bezel Studio, its controls work again. An 8.8" that
  went live through its wake micro-controller's port (from the tray, or when
  the studio resumed after a restart) failed brightness changes with "Device or
  resource busy", hid its SD card and made the Themes tab's **For this screen**
  filter flicker. Now the brightness changes on the live screen, the **Storage**
  tab shows the internal memory and the SD card and manages them, and
  **For this screen** stays enabled and steady.
