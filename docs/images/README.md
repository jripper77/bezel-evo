# README images

All screenshots come from Studio's browser demo, with no Tauri backend, connected
hardware, personal files or local settings. Most show the example Ember layout,
assembled from Studio's existing text, clock, ring, graph, shape, bar, Card and
media player widgets. This is a custom example, not a claim that Ember is a
bundled theme. Demo rendering is approximate; sensor values, the media session
("Demo track", "Demo artist"), the sensor-reader status and the memory figures
are simulated, not measurements.

| Asset | Source |
| --- | --- |
| `pixel.svg` | Unmodified copy of `apps/bezel-studio/src/icon.svg`, the existing Pixel mark |
| `studio-hero.png` | Dark Studio: widget library, Ember document, CPU ring selected with its handles, floating bar, live preview and Data section |
| `studio-color-picker.png` | Dark Studio: the ring's Color section with the colour picker open, theme colours and colours used in the theme |
| `studio-font-picker.png` | Dark Studio: the clock's font picker with the theme, bundled, user and system sections |
| `studio-card-faces.png` | Dark Studio: a two-face Card, its faces list, shared background and one transition |
| `studio-player.png` | Dark Studio: media player parts and a segmented progress bar with knob and side times |
| `studio-background.png` | Dark Studio: Media panel and the "Change background" chooser |
| `studio-sensor-probes.png` | Sensors panel with probe/status filters and per-probe availability counts; built-in Demo theme, captured by `tests/e2e/libre.spec.mjs` at 1280x800 with the live simulated clock |
| `studio-sensors.png` | Dark Studio: Sensors panel and the sensor status popover, with one simulated missing device |
| `studio-themes.png` | Dark Studio: the Themes gallery with every demo theme and Ember selected |
| `studio-profiles.png` | Dark Studio: Profiles and automation with two example profiles (Night by schedule, Music when media plays) created in the page only, and the Curtain transition |
| `studio-light.png` | Light Studio: Layers with a Card, the ring's colour-by-value thresholds |
| `screen-render.png` | Generic illustrative device, composed with HTML/CSS around the exact demo canvas frame; not a product photograph or hardware validation |

Except for the probe overview above, capture at 1440 x 900 (Studio) and 1200 x 900 (device illustration), English UI,
UTC, dark/light system theme, with a fixed demo clock. The only device shown is
the demo's Turing Smart Screen 8.8"; addresses in the UI are the demo fixtures,
shown as Windows ports (COM5, COM4) by the capture script. The example uses
Inter, the font Bezel includes, loaded into the demo page from `themes/fonts`.
The device illustration uses the brand palette and bundled IBM Plex Sans.

From the repository root:

```powershell
npm --prefix apps/bezel-studio ci
node docs/images/capture.mjs
node docs/images/capture.mjs studio-player studio-light   # only these pictures
```

The capture script starts its own local server, loads only `?demo=turing88`,
provides the example document in memory, takes screenshots, then closes the
browser and server. It does not modify application source or real themes.

PNGs are RGB and losslessly recompressed with Pillow (`optimize=True`,
`compress_level=9`). To repeat that optimization after capture:

```powershell
python -c "from pathlib import Path; from PIL import Image; [(lambda p: Image.open(p).convert('RGB').save(p, optimize=True, compress_level=9))(p) for p in Path('docs/images').glob('*.png')]"
```

The images share the project's [GPL-3.0-or-later licence](../../LICENSE).
Component notices remain in [THIRD_PARTY.md](../../THIRD_PARTY.md).
