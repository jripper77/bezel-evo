# Coming from turing-smart-screen-python

[Português (Brasil)](pt-BR/migrating.md)

Bezel drives the same screens as
[turing-smart-screen-python](https://github.com/mathoudebine/turing-smart-screen-python)
and reads its themes. The two cannot run at the same time.

## 1. Stop it

Two programs on one screen mix their data, so Bezel refuses a port that another
program holds, for example `access denied to COM5`.

- **Started in a terminal:** press Ctrl+C there.
- **Started with Windows:** quit it from its tray icon, and remove it from
  **Task Manager → Startup apps** (or from the Task Scheduler, if you added it
  there).

The vendor app (TURZX / Turing) also holds the screen: quit it from its tray
icon before using Bezel.

## 2. Bring your themes

In Studio: **Themes → Import…**, then choose the theme's folder (for example
`res/themes/MyTheme` inside turing-smart-screen-python) or its `theme.yaml`.
The imported theme opens in the editor; click **Save** to keep it in your
library. What could not be converted exactly (a sensor Bezel does not have, a
font it cannot find) is listed after the import.

From a terminal:

```powershell
bezel import turing-smart-screen-python\res\themes\MyTheme -o mytheme.bezeltheme
bezel run mytheme.bezeltheme
```

`bezel run` and `bezel render` also take the theme folder directly and convert
it on the fly. Themes of the vendor app (`.turtheme`) import the same way.

Keep a `.turtheme` in its vendor folder when importing: Bezel looks for fonts
in a nearby `fonts` folder, including `<vendor app>/fonts`, and includes the
files it finds in the imported theme. Studio also checks the PC's installed
fonts; missing families use Inter and appear in the import report. Optional
fan and memory model text keeps the vendor's saved text when the PC has no
matching sensor. Imported weather starts with Rome; choose your city in the
inspector after importing.


## 3. Your settings

| `config.yaml` | In Bezel |
|---|---|
| `COM_PORT` | found automatically; Studio's screen selector (or `--screen`) picks one when several are connected |
| `REVISION` | detected from the screen |
| `DISPLAY_REVERSE` | the rotate 180° button on Studio's canvas bar; the `-flipped` orientations on the command line |
| `BRIGHTNESS` | **Screen → Settings → Brightness**, or `bezel brightness 40` |
| `THEME` | the theme you save in Studio, which Light runs: [Running at sign-in](run-at-login.md) |
| `HW_SENSORS` | Bezel reads the sensors itself, with its bundled hardware sensor reader ([Sensors](sensors.md)) |
| `PING` | **Preferences → Sensors**, or `--ping-host` |
