# Studio UX v2 templates

Four editable horizontal templates (1920×480) reproduce the offline Studio UX v2 designs using native widgets and local vector artwork:

| Template | Appearance | Live content |
| --- | --- | --- |
| Pixel | Black dotted panels, Doto digits, orange block meters | Clock, CPU/GPU/RAM, histories, mirrored network traffic, output volume |
| Nebula | Glass panels, violet/cyan/pink dials, Space Grotesk | Clock, current Milan weather, telemetry, usage rings and network history |
| Paper | Cream newspaper layout, Fraunces and Instrument Sans | Clock/date, current Milan weather, CPU temperature history, system indices, media and disk activity |
| Now Playing | Magenta listening room, geometric standby cover and vinyl, Bricolage Grotesque | Actual session cover/title/artist/source, remaining time, timeline, system-output spectrum and telemetry |

Weather coordinates are editable. There are no stored sample readings, hardware model names, forecast temperatures or track metadata. Missing sensors remain unavailable; no media session shows a waiting label and local standby artwork. Audio capture follows the existing system-audio spectrum settings. Graphs fill as readings arrive. Template labels are editable Italian text; dates follow the system language.

The four additional type families ship only the faces these templates use, with their original OFL licences in `themes/fonts`. Their exact native optical-family names are **Fraunces 144pt** and **Bricolage Grotesque 96pt**. See [font sources](fonts/UX-V2-SOURCES.md).

Regenerate these four templates only with `python scripts/create-ux-v2-themes.py`. The script never rewrites other templates or user documents. For labelled native review frames, set `BEZEL_UX2_THEME_PREVIEWS` to an absolute task output folder and run `cargo test -p bezel --test bundled_themes studio_ux_v2_templates_render_live_and_unavailable_inputs`.

# Modern template pairs

## Animated graphic collections

The six modern originals below remain available. The library also includes six
additional graphic-only animated themes, with matching Wide and Round layouts:

| Collection | Art direction | Live elements |
| --- | --- | --- |
| Synapse / Kinetic | Dark mint/cyan signal architecture, orbiting particles, counter-rotating arcs and large typography | Segmented GPU gauge, CPU/RAM readings and meters, thermals, media metadata and system-audio frequency field on Wide |
| Prism / Kinetic | Cream, orange and lilac poster, continuously deforming geometric knot | Clock, CPU/GPU/RAM readings and histories on Wide; compact CPU dial inside the clock on Round |
| Morph / Sound | Magenta/cyan motion graphics, full-width layered audio sculpture | Media session, live system-audio curves and telemetry on Wide; radial audio around a central media display on Round |

The animated artwork is drawn from geometric primitives, with no photographs.
Forty transparent frames form a seamless local loop lasting approximately
4.8 seconds (about 8 frames/second). The display's animation scheduler skips
frames when necessary; audio and sensor widgets use actual readings. No video
decoder or media conversion is needed for the decorative loops.

For a still layout, hide the layer named **Motion / hide this layer for a still
theme**. The clock, sensor readings and live audio remain independent.

Regenerate these six new themes with `python scripts/create-kinetic-themes.py`
(requires Pillow). This script does not rewrite the six modern originals.

To export animated review pages from actual renderer frames, set
`BEZEL_THEME_PREVIEWS=dist/kinetic-theme-previews` and
`BEZEL_KINETIC_PREVIEW_FRAMES=target/kinetic-preview-frames` to **absolute paths**,
run `cargo test -p bezel-render coordinated_templates -- --nocapture`, then
`python scripts/create-kinetic-previews.py`. The result is
`dist/kinetic-theme-previews/index.html`, with labelled, simulated preview data.

## Modern originals

Open these from Studio's **Themes** library. Included themes are read-only;
save a copy to make your own. Each collection has a **Wide** canvas (1920×480)
and a **Round** canvas (480×480, with important content inside the circular
safe area). Theme selection is filtered by the connected display's model.

| Collection | Design | Features |
| --- | --- | --- |
| Pulse / Gaming | Instrument cockpit, olive grid, lime load and orange thermal rings | Cut-corner CPU Card rotates between load and thermals every 12 seconds; calibrated GPU dial, smooth values, thermal array, warning colours and 90-second histories. Round becomes a matching instrument face. |
| Resonance / Music | Warm listening room, large artwork and a grooved vinyl disc | Standby/Playing Card follows media playback with a crossfade. Asymmetric album cover on Wide; circular cover on Round; live radial system-output spectrum surrounds the record, with a styled timeline and output volume on Wide. |
| Clarity / Monitor | Copper clock column, cream editorial telemetry and dark memory module | CPU/GPU histories, RAM dial and history, uptime. Round pairs a copper clock face with compact CPU/GPU meters. |

Graphs fill as readings arrive. Unavailable sensors show a dash. Music templates
use actual Windows media sessions and audio output; without music they show a
quiet spectrum and a waiting label. No demo data is stored in the themes.
The clock follows the system language. Template labels and names are editable.

To use a pair, create a profile in **Themes → Profiles and automation**, assign
the Wide template to the wide display and the Round template to the square or
round display, then choose its event rules. Configure the profile transition in
the same dialog. Instant, Fade, Slide and Flip are available from 150 to 3000 ms;
native device video changes immediately because its picture is not on the host.

All templates use Inter and JetBrains Mono included with Bezel. Their architectural
artwork is local SVG, with no external image downloads. Album artwork comes from
the actual media session. `python scripts/create-modern-themes.py` regenerates these six
bundled originals; it does not touch saved user copies.
