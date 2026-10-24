# Game FPS

[Português (Brasil)](pt-BR/fps.md)

The game FPS sensors (`gpu.fps` and its related metrics) show the frame rate
of the game you are playing. Bezel does not hook into games: on Windows it
reads Intel PresentMon, and falls back to what RivaTuner Statistics Server, a
frame-rate tool you may already run, publishes. Drag a sensor onto a theme
like any other.

> The FPS readers follow the published formats of these tools and were
> checked against a running game on real hardware (2026-10-18). If a value
> looks wrong, please report it.

## Windows: Intel PresentMon (preferred)

Bezel reads frame metrics directly from Intel PresentMon's console
application. It starts PresentMon on demand, only while a game is running and
a theme requests the sensors, and stops it about 60 seconds after the last
frame. It is never installed as a service.

1. Download the Intel PresentMon console application (v2.6.0 or newer). Both
   the plain `PresentMon.exe` and versioned release names such as
   `PresentMon-2.6.0-x64.exe` work.
2. Place it in one of these locations (checked in this order):
   - The path configured in Bezel: the `presentmonPath` key in `settings.json`,
     or `--presentmon-path` on the command line.
   - A `tools\` or `sensors\` folder next to the Bezel executable.
   - `C:\Program Files\Intel\PresentMon\`.
3. **Administrator rights required**: PresentMon reads ETW events. Run Bezel
   as Administrator, or add your user to the "Performance Log Users" group. If
   the rights are missing, the sensors read as unavailable with a message
   explaining the fix.
4. Start the game. Bezel automatically starts PresentMon, excludes
   compositor and overlay processes (like `dwm.exe`, `explorer.exe`,
   `discord.exe`), and shows `gpu.fps`, `gpu.frametime`, `gpu.fps_low_1p`,
   `gpu.fps_low_01p`, `gpu.fps_min`, `gpu.fps_max`, `gpu.fps.source`,
   `gpu.game` and `gpu.limiter`.

If PresentMon is not found or fails to start, `gpu.fps` falls back to
RivaTuner Statistics Server (RTSS) exactly as before, and the catalog says
which source is active.

## Windows: RivaTuner Statistics Server (fallback)

1. Install RivaTuner Statistics Server (RTSS). It comes with MSI Afterburner, or
   on its own.
2. Start RTSS and leave it running (it can sit in the tray).
3. Start the game. Bezel shows the frame rate of the game RTSS measured last.

While RTSS is not running the sensor is unavailable and says so ("RivaTuner
Statistics Server is not running: start it ...").

## What you see

- A number while the game runs. A 0 reported by the tool is shown as 0.
- `—` when no tool is measuring a game, or when the last value is older than
  3 seconds (game paused, minimized or closed). Bezel never keeps showing an
  old value.
