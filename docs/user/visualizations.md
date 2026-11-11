# Native visualization presets

Select a bar, ring or graph and open **Shape → Visualization**. Choose a preset
from its schematic preview tile. The preview uses
the same native renderer as connected displays. **Classic** keeps the previous
appearance and is the default for existing themes.

- **Block matrix** (bar): columns, rows, spacing and corner radius. The existing
  fill direction controls cell order. Cells light completely, one by one; an
  unavailable source shows only the empty track. Keep the grid sparse enough
  for the physical display's resolution. **Fit matrix cells** adjusts the height
  for near-square cells, bounded by the remaining screen height, without moving
  the object. It is an explicit action and can be undone once.
- **Pixel arc** (ring): square cells along the selected arc, using the existing
  fill, track and arc gradient. Scale marks controls the number of pixels;
  ring thickness sets their size. Cells light completely rather than partially.
- **Modern dial** (ring): inset arc, outer scale marks and an optional outlined
  endpoint marker. Arc colors, range, direction and value transitions still apply.
- **Rounded bars** (graph): rounded historical columns with adjustable spacing.
- **Pixel bars** (graph): each column is a stack of square cells, as wide as
  a bar and as far apart as the bars; **Thickness** and **Spacing** set both.
  A cell lights completely or not at all. **Show unlit cells** draws the empty
  cells in a colour of your choice; switched off, only lit cells are drawn.
  Pixel bars also work in a mirrored graph.
- **Mirrored graph**: the primary source above a central gap and a separately
  selected lower source below it. Choose the lower source and color in Data.
  This works with lines, filled areas or bars; the lower source is not a
  decorative reflection or a negative reading. Without a lower source it stays
  empty. Missing readings break their respective series.

In a mirrored graph, **Source** lists the upper and lower series' sensors and
**Vertical scale** has one block per series, each **Automatic** or **Fixed**:

- **Fixed** uses that series' own minimum and maximum.
- **Automatic** starts at 0 and reaches the highest reading in the window.
  The greyed min and max show the range in use.
- With both automatic, **Same scale for both series** gives them one peak, the
  higher of the two, so their sizes stay comparable (download and upload).
  Turned off, each series fills its own half (temperature above, load below).
  Choosing a sensor turns it on when both series share a unit and off when
  they do not; you can change it afterwards.

Themes made before this keep their look: the lower series follows the upper
one's choice and both share the peak. The sample interval and history window
remain shared; frame pacing remains specific to each display. This does not
require H.264 support.

Value labels and captions remain ordinary text objects, so they can use any
font, placement or formatting. Group them with the visualization when useful.

![Native rendering of the five visualization presets](../images/native-visualizations.png)

The editable reference manifest is in
[`examples/native-visualizations/theme.json`](../examples/native-visualizations/theme.json).
It includes illustrative labels and a placeholder lower sensor; choose your
own sources before using it on a display. Reproduce the reference image with:

```powershell
cargo run -p bezel-render --example visualization_preview -- docs/examples/native-visualizations/theme.json docs/images/native-visualizations.png
```
