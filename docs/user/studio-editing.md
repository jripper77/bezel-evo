# Editing in Studio

[Your first theme](first-theme.md) walks through the basics. This page covers
every editing tool of Bezel Studio.

## The workspace

![Bezel Studio: icon rail, widget library, canvas with floating bars and the inspector](../images/studio-hero.png)

- **Icon rail.** **Widgets**, **Sensors**, **Layers**, **Themes**, **Media**
  (Design) and **Screen** (Device). The library panel shows the one you pick.
- **Top bar.** The theme's path (*Themes › Ember*), an **Unsaved** badge while
  there are changes, the screen selector, **Preferences** (the sliders icon),
  **Live** and **Save** (**Ctrl+S**).
- **Canvas bars.** At the top: **Vertical** / **Horizontal**, rotate 180°, undo,
  redo and **Show guides and grid**. At the bottom: zoom out, the zoom level,
  zoom in and **Fit**.
- **Status bar.** Whether everything is saved and when, the
  [sensor chip](sensors.md#the-sensor-status), the memory chip, the time the
  last frame took to draw and whether the screen is awake.
- **Panels.** Drag the edge of the library or of the inspector to resize it;
  Studio remembers the widths.

**Moving around.** The mouse wheel zooms around the pointer (a touchpad pinch
too), from 5% to 800%. Hold the middle button and drag to pan. **Fit** shows the
whole screen. Clicking the empty space around the screen deselects everything.

**Light and dark.** Studio follows Windows' light or dark mode, and its language
(English, Italian or Portuguese) unless you pick one in **Preferences →
Language**.

## Adding and selecting objects

- Drag a widget from **Widgets** onto the canvas, or select it and press
  **Enter** to add it in the middle. Drag a sensor from **Sensors** onto a widget
  to bind it, or onto empty space to create a new value.
- Click an object to select it; **Ctrl+click** or **Shift+click** adds to the
  selection; drag on empty space to select with a rectangle. Objects outside the
  screen's edges stay reachable and show their outline and name.
- Arrow keys move the selection by 1 px (with **Shift**, 10 px).
- **Double-click** an object's frame to fit the frame to what it draws: the
  text of a text, value or clock in its current font, an image or icon at its
  natural proportions, a weather or media player block. **Fit frame to content**
  in the inspector does the same. Each is one undo step.
- Right-click an object on the canvas or in **Layers** (or **Shift+F10**) for
  cut, copy, paste, duplicate, group and ungroup, make a Card, bring to front or
  send to back, detach from a group or Card, hide, lock and delete.
- **Ctrl+C**, **Ctrl+X** and **Ctrl+V** copy, cut and paste objects inside the
  theme, with their settings; **Ctrl+G** groups and **Ctrl+Shift+G** ungroups.

With several objects selected, the alignment buttons line them up on the first
one you selected; with one, they align it on the screen.

In **Layers**, combine the name/sensor search with **Object type** to find
text, values, clocks, graphs, images, cards, groups or other widget types.
**All types** clears the type filter. Matching children keep their card/group
ancestry visible; folded containers expand temporarily, then return to their
previous fold state when filters are cleared. A matching card or group shows
its complete contents. Filtering leaves the document and selection unchanged.

![Layers filtered to rings, retaining their parent group](../images/studio-layers-filter.png)

## Removing imported media

In **Media → This theme**, use the trash button on a file to remove it from
the theme's media list. Files still used by images, needles, fonts or the
background are protected: replace or remove those references first, including
objects on hidden card faces. The original file on your computer is preserved.
Saving includes only assets the theme uses. Session bytes remain available so
Undo can restore a previously removed image object; importing the file again
also restores it to the list.

## The inspector

With nothing selected, the inspector shows the **theme**: its name, **Vertical**
or **Horizontal**, the [background](#the-background) and how often it refreshes
(quick values 0.5, 1, 2 and 5 s).

With an object selected, it shows from top to bottom:

- a **header** with the object's name, kind and layer, a visible object-type
  selector, and buttons to hide it, lock it and open more actions;
- a **live preview** of the object on its own, drawn by the same renderer as the
  screen, with its real readings: you see each change at once, even when the
  object is small on the canvas;
- **Membership**, then **Position and size**, including rotation for supported
  independent objects; these stay above the tabs;
- the **Data** tab: what it shows (sensor and range, text, clock format,
  weather location, graph window, media app);
- the **Look** tab: colours, font, corners, thickness, transitions and the
  other visual settings.

Number fields take the arrow keys and keep their focus while you step.
Scrollable panels use slim rounded scrollbars; source descriptions in Sensors
wrap within the panel rather than forcing horizontal scrolling.

![The inspector of a CPU ring, Look tab, with the colour picker open](../images/studio-color-picker.png)

### Choosing a sensor

Click the source card to open the same sensor picker used for widget data,
colour rules and sensor-driven motion. Search by name, key, category or probe.
Results are grouped by category and show the probe, current reading and status;
pending or unavailable readings remain visible. Numeric controls exclude text
sensors. Use the arrow keys and Enter to choose, or Escape to cancel.

### Sensor-driven motion

Select an image, icon, shape or another ordinary widget and open **Look →
Sensor-driven motion**. Turn on **Rotate with the sensor**, then choose a
numeric sensor. Containers and device-video cutouts do not offer this control;
ordinary objects inside cards and groups do.

For a fan graphic, choose its RPM sensor, set **Full speed at** to
3000 RPM, **One turn every** to 1 s and **Start and stop** to **Gradual**.
A reading of 1500 approaches one turn every 2 s. This is a visual scale: the animation
does not try to reproduce thousands of physical revolutions per minute.

**Start and stop** offers **Immediate**, **Gradual** and **Soft** responses.
Existing custom inertia settings are preserved and shown as **Custom**.
Zero, negative or unavailable readings slow the graphic to a stop, holding its
last angle. **Counterclockwise** reverses the direction. Motion applies around
the frame centre, so centre the artwork within its image for a balanced rotor.

The live preview uses the same motion as the display, with enough transparent
space for every angle. Reduced motion shows the authored static orientation.
Sensor sampling stays unchanged; each display renders at its own output pace,
requesting up to 50 frames/s while the graphic moves, subject to rendering and
transfer costs. The stored rotation and object frame are not changed by playback.

### Colours

Every colour in Studio opens the same **colour picker**:

- a saturation and brightness area, a hue bar and an opacity bar (all usable with
  the arrow keys), the **hex code** and the opacity in %;
- an **eyedropper** to pick a colour on any monitor, with a magnifier, marked
  centre pixel, colour swatch and HEX value beside the cursor;
- **Theme colours**: click **Save** to keep the current colour in the theme, then
  click a swatch to use it again. **Manage** renames, reorders and deletes them.
  A saved colour is a starting point: changing it does not repaint what already
  uses it;
- **Used in this theme**: the colours already in the theme, the most used first;
- **Recent**: the colours you picked lately.

Each change is one undo step; dragging in the area is one step too.

During a desktop pick, the selection surface freezes temporarily so the preview
and selected pixel match exactly. Move to inspect pixels and click to select;
Escape or right-click cancels. The capture stays in memory only until the pick
ends. The existing opacity is preserved.

### Fonts

The **Family** button opens the **font picker**:

![The font picker of a clock, with its sections and preview](../images/studio-font-picker.png)

- search by name, and filter **All**, **Sans**, **Serif** or **Mono**;
- sections **In this theme**, **Recent**, **Theme fonts**, **Included with
  Bezel**, **My fonts** and **System fonts**, each name written in its own font;
- a preview with the object's real text (its value, the time, the song playing);
- **Open fonts folder** and **Reload fonts** for your own fonts.

**Style** then offers only the weights and italics that font really has. A theme
that asks for a font this PC does not have shows **Not installed** and the font
the screen uses instead. See [Fonts](fonts.md) for the three places fonts come
from and how to pack one inside a theme.

### Smooth values

Rings, bars, needles and values can glide to each new reading instead of
jumping. In **Look → Transition**, pick **Smooth** and a duration (100 to
2000 ms), or **Inertia** and a settling time. Inertia preserves the current
position and velocity when a new reading arrives, then slows towards the new
target. **None** updates immediately. Older Bezel builds open inertial themes
using a smooth transition. New objects start smooth at 400 ms; themes made before keep jumping
until you turn it on. Studio and Light send extra frames only while something
moves, and fewer to a slow screen; if a screen cannot keep up, it shows the new
value at once. The editor honours Windows' reduced-motion setting.

### Colour by value

Texts, values, rings, bars, needles, graphs, shapes and SVG icons can change
colour with a sensor's value. In **Look**, turn on **Colour by value**, choose
the sensor (a bound object uses its own), a **Base colour**, then **Add
threshold**: *From ≥ 70* orange, *From ≥ 90* red. Values use the sensor's own
units; the highest threshold reached wins.

### Corners

Shapes, Cards, images and the player's cover share one corners control:
**Equal** (one radius) or **Individual** (each corner), with a drawing of the
four corners that shows what you set at any size.

### Graphs

A graph has its own time window, independent of the theme's refresh. In
**Data → Time window**, pick a quick value (30 s, 1 min, 5 min, 15 min, 1 h) or
write the sentence *Show the last [1] [minute], with one point every [1]
[second]*; a ruler shows the window and how many points it draws (2 to 3600).
**Vertical scale** is **Automatic** (fits the highest value in view) or
**Fixed** (min and max).

## Cards

A **Card** shows one of several **faces** at a time, with a transition between
them. Add one from **Widgets**, or select objects and choose **Make card** in
**Layers** or in the right-click menu.

![A Card selected: its faces list, shared background and transition](../images/studio-card-faces.png)

**Faces tab.**

- The faces are a list: number, name, **in view** on the face the screen shows,
  **Edit** to show that face on the canvas and edit its objects, and a **⋯** menu
  with **Rename**, **Duplicate**, move earlier or later, and **Delete**.
  **+ Add** adds a face.
- **Shared background** holds the objects visible under every face; **Edit** it
  like a face.
- **Transition between faces**: one effect for the whole Card (none, fade, slide
  or flip), its duration (150 to 1500 ms) and, for slide and flip, its direction.
  **Animate next face** tries it.

**Motion tab.** The same transition with all its options, including whether the
shared background moves with the faces.

**Triggers tab.**

- **Timed triggers** change the face at regular intervals: *every 30 s → show
  the next face* (or a given face). **Add timed trigger** adds one.
- **App triggers** show a face for a while: when an application is running, is
  closed, is in the foreground, or when media is playing (in any app or in one
  you name). Each rule has a **priority** and a **return** delay; drag the
  handle, or use its arrow keys, to reorder. **When a trigger ends** chooses the
  face to go back to.
- Timed changes pause while you edit the Card or while an app trigger is
  active. Deselect the Card to see them run.

Themes made with the older "rotate every N seconds" setting become a timed
trigger when opened.

## Media player

The **Media player** shows what Windows' media sessions publish: Spotify, a
browser, a video player. Covers and titles depend on what each app publishes.

![The media player's parts and progress bar options](../images/studio-player.png)

- **Data**: **Which app** (automatic, an app playing now, or **Other…** by name),
  **When nothing plays** (hide the player or show a text) and the **Times
  shown** (elapsed / total, elapsed / remaining, elapsed, remaining or total).
- **Look → What to show**: turn **Cover**, **Title**, **Artist**, **Time**,
  **Progress** and **App name** on or off. Several players with different parts
  make a layout of your own (a big cover here, the title there).
- **Look → Progress bar**: fill and track colours, thickness, round ends, a knob
  at the position, **Continuous**, **Segments** or **Dots**, and where the times
  go (**In the text**, **At the sides**, **Above**).
- **Cover corners** and the spacing between cover and text.

## The background

With nothing selected, the theme's **Background** card shows the background in
use: a preview, its kind, the file, and whether the screen stores it.

**Change background…** opens three sources inside the inspector, one at a time:

- **Solid colour**, transparent too;
- **From this computer**: a picture or a video, saved with the theme;
- **Video from the screen**: a video already stored on the display, played by the
  screen itself. A source the screen cannot use is greyed out with the reason.

Nothing changes until **Use this background**; the footer shows the old and the
new background. **Cancel**, **Back** or Esc leave everything as it was, and the
change is a single undo step. Video framing (rotation, fill or fit, zoom,
position) is described in [Framing the video](storage-and-video.md#framing-the-video).

## Layers

**Layers** lists the objects, the topmost first, with their kind or sensor.
**Filter by name or sensor** finds one. Drag rows to reorder them, or into a
group, a Card face or its shared background. Select several (Ctrl+click) to
**Group** them (**Ctrl+G**) or **Make card**. Each row can be hidden, locked and
renamed (double-click the name). Cards show their shared background and each
face, with **in view** on the shown one.

## Saving

**Save** (**Ctrl+S**) saves to the theme's file, or to your library for a new
theme. **Themes → Save as .bezeltheme…** (**Ctrl+Shift+S**) saves a copy, with
its pictures, videos and included fonts, wherever you choose. A background video
stored only on the screen stays a reference to the screen: it is not downloaded
when saving.

## Preferences

The sliders icon in the top bar opens **Preferences**:

- **Language** of Studio and of Light's tray menu;
- **Debug**: diagnostic logs and on-screen readings, for reporting a problem;
- **Logs**: where the logs are, with **Open logs folder**
  ([Troubleshooting](troubleshooting.md#logs));
- **Sensors**: the ping target.

Implementation notes and validation for each feature are in
[the fork notes](../../FORK.md).
