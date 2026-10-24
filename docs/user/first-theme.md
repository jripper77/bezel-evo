# Your first theme

[Português (Brasil)](pt-BR/first-theme.md)

Open **Bezel Studio** from the Start menu. The top bar shows the screen in use
(for example *Turing Smart Screen 8.8" · 480×1920 · COM5*); click it to choose
another screen. With no screen connected, Studio says so and you can still edit.

![Bezel Studio with its icon rail, widget library, canvas and inspector](../images/studio-hero.png)

Studio has four areas:

- the **icon rail** on the far left: **Widgets**, **Sensors**, **Layers**,
  **Themes** and **Media** (Design), and **Screen** (Device);
- the **library** next to it, showing the panel you picked on the rail;
- the **canvas** in the middle, on a dot grid, with floating bars for the
  orientation, undo/redo and guides on top and the zoom at the bottom;
- the **inspector** on the right: the theme's settings, or those of the object
  you selected.

Drag the edge between two areas to make a panel wider or narrower. The canvas
shows exactly what the screen gets: it is drawn by the same renderer.

## 1. Start from a theme

Click **Themes** on the rail. The panel lists your themes and the built-in ones,
each with a picture drawn with sample values and the screen it was made for
(for example `8.8″ · 480×1920`). **For this screen** / **All** and
**All** / **Vertical** / **Horizontal** filter the list, and **Search themes**
finds one by name.

- Click a theme to open it.
- **New theme** starts an empty theme of your screen's size, **Vertical** or
  **Horizontal**.
- **Open theme gallery** shows the same themes in a larger window, with
  **Duplicate** and **Open in the editor**.
- **Import…** converts a theme of another app ([Themes from other apps](#themes-from-other-apps)).

## 2. Edit it

- **Widgets**: text, value, clock, weather, ring, bar, needle, graph, image,
  media player, shape, Card and group, in three groups with a search box. Drag
  one onto the canvas, or select it and press **Enter** to add it in the middle.
- **Sensors**: every sensor of your PC, by category, with its current value and a
  small history line. Drag a sensor onto a widget to bind it, or onto empty space
  to create a new value.
- Click an object to edit it in the **inspector**: a live preview of the object
  on its own sits at the top, then its settings in two tabs, **Data** (what it
  shows: sensor, range, text, time window) and **Look** (how it looks: colours,
  font, size, corners, transitions).
- **Layers**: order, hide, lock, group and rename objects, and see the faces of
  your Cards.
- **Media**: pictures, GIFs and videos of the theme, your collection and the
  icon library.

Undo is **Ctrl+Z** and redo **Ctrl+Shift+Z**, also on the canvas bar. The mouse
wheel zooms around the pointer and the middle button pans; **Fit** shows the
whole screen. [Editing in Studio](studio-editing.md) covers every tool.

## 3. Show it on the screen

Click **Live** in the top bar. The screen follows your edits as you make them. Opening or importing another theme turns Live off for that display, so you
can check the new theme in Studio first. The Live control briefly highlights
when sending stops; click it again when you are ready to send the new theme.

Click **Save** (**Ctrl+S**) to keep the theme in your library.

If the screen stops taking frames while live (a cable glitch, a frozen
screen), Bezel connects it again by itself and goes on; see
[The screen froze](troubleshooting.md#the-screen-froze--stopped-responding).

**Closing Studio** hands your saved theme to **Bezel Light**, which keeps it
running from the tray: the picture does not flicker, graphs keep their history
and a background video keeps playing. If you close with unsaved changes, Studio
asks first; the screen then shows the saved theme. Light's tray menu has
**Open Studio** and **Quit**. To have it start when you sign in, see
[Running at sign-in](run-at-login.md).

## 4. Screen settings

Click **Screen** on the rail. It has two tabs:

- **Settings**: the selected screen with its state, **Brightness**, **Start with
  the computer**, what the screen does [when the computer shuts down](power-off.md),
  and **Advanced** (hand the screen back to its own mode, restart it).
- **Storage**: pictures and videos kept on the screen
  ([Screen storage and video](storage-and-video.md)).

## A video in the background

1. Select nothing (click an empty part of the workspace): the inspector shows the
   theme. Its **Background** card shows the background in use.
2. Click **Change background…**. Three sources open in the inspector:
   **Solid colour**, **From this computer** (a picture or a video, saved with the
   theme) and **Video from the screen** (a video already stored on the display).
   Nothing changes until you confirm with **Use this background**; **Cancel** or
   Esc leaves everything as it was.

   ![The Change background panel with its three sources](../images/studio-background.png)

3. You can also drop a video or an animated GIF straight on the editing area: it
   becomes the background at once. Or add it in **Media** (**Add video…**) and
   pick **Use as background** on it.

Bezel copies the video into the theme and takes a still picture from it, the
*poster*, cropped to the theme's shape. Taking it needs ffmpeg
([Installing ffmpeg](ffmpeg.md)); without it the video is still added, without
a poster. Undo (Ctrl+Z) brings the previous background back.

What the screen does with it:

- Screens that play videos themselves (Turing rev C, such as the 8.8", and the
  Turing USB generation) loop a copy stored on them, and Bezel draws the theme
  over it. The background card says whether the screen already stores it; if
  not, **Open Screen › Storage** leads to **Send to the screen**, which converts
  the video for the screen (an animated GIF becomes an MP4) and sends it.
- Turing rev C screens take at most **25 MiB per file**. The conversion lowers
  the bitrate from the video's length so that it fits, so a long clip loses
  some quality; if it still does not fit, Bezel says so before sending
  anything ([How large a file can be](storage-and-video.md#how-large-a-file-can-be)).
- Screens that cannot play videos get the video decoded on the computer, which
  also needs ffmpeg.

Saving the theme keeps the video and its poster inside the `.bezeltheme` file.
To turn, zoom or place the video, see
[Framing the video](storage-and-video.md#framing-the-video).

## Animated GIFs

An animated GIF added as an **image** moves at its own pace, also between the
theme's refreshes: up to 30 frames a second on the screen, up to 15 in the
editor. Sensors keep the theme's refresh. Only the GIF's rectangle goes to the
screen, so a small GIF (a logo, an icon, a 240x240 animation) moves smoothly. A
GIF as large as the screen changes only as fast as the USB link carries whole
frames (about 2 a second on the 8.8"): Bezel then skips frames to keep time. For
a full-screen animation, use the GIF as a video background
([A video in the background](#a-video-in-the-background)): the screen plays it
itself. A hidden GIF does not animate.

## Themes from other apps

**Themes → Import…** converts a theme of the vendor app (`.turtheme`) or of
turing-smart-screen-python (its theme folder or `theme.yaml`). Whatever could
not be converted exactly is listed after the import.

## The same from the command line

```powershell
bezel run turing-8.8-horizontal                       # a built-in theme, live, until Ctrl+C
bezel run "$HOME\Documents\mine.bezeltheme"           # a theme file
bezel render turing-8.8-vertical -o preview.png       # one frame to a picture, no screen needed
bezel import AMD.turtheme -o amd.bezeltheme           # convert another app's theme
```

A theme is a `.bezeltheme` file: a zip with `theme.json` and an `assets/`
folder.
