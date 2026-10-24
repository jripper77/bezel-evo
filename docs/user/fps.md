# Game FPS

[Português (Brasil)](pt-BR/fps.md)

On Windows, Bezel can read Intel PresentMon's console application, with
RivaTuner Statistics Server (RTSS) as the fallback for `gpu.fps`. Neither
reader makes Bezel inject code into a game. Drag the sensors onto a theme
like any other readings.

## PresentMon setup

1. Download the tested [PresentMon 2.6.0 console application](https://github.com/GameTechDev/PresentMon/releases/tag/v2.6.0).
   Use the console executable, not Intel's graphical capture application.
2. Set `presentmonPath` in Studio's `settings.json` or pass
   `--presentmon-path` to the CLI. Alternatively, put the console executable
   in `tools\` or `sensors\` beside Bezel. Discovery also checks Intel's
   `PresentMon\PresentMonConsoleApplication\` installation folder, then
   the older `PresentMon\` layout. It never searches `PATH`.
   `PresentMon.exe` and versioned console names such as
   `PresentMon-2.6.0-x64.exe` are accepted; numbered versions sort numerically.
3. ETW capture needs administrator rights or membership in Windows'
   **Performance Log Users** group. Group changes can require signing out
   and back in. Bezel does not grant these rights or elevate itself.
4. Request an FPS sensor and bring the game to the foreground. Automatic
   discovery chooses that process ID, filtering common shell, browser and
   launcher programs. It is a foreground graphics-app heuristic, not a
   complete database of games.

PresentMon starts only on demand, with its console hidden. Bezel targets the
chosen PID and measures one swap chain within it, selected by recent frame
count. Another foreground candidate can replace the target. Stored samples
keep PresentMon's QPC time, so delayed pipe output cannot refresh old frames.

Stopping all FPS requests closes the child. Without frames, a probe stops
after 10 seconds; after it has measured frames, it tolerates a pause of up to
60 seconds. Failures use RTSS when available and retry PresentMon with bounded
backoff. Missing tools and idle probes retry at most once per minute.
Only one Bezel instance owns its fixed capture session at a time; another
uses the fallback instead of stealing it. The capture child is assigned to
a kill-on-close Windows job before capture startup completes.

## Available sensors

| Key | Meaning |
|---|---|
| `gpu.fps` | Average FPS over the last second, using present intervals when available |
| `gpu.frametime` | Mean frame interval over the last second, in milliseconds |
| `gpu.fps_low_1p` | Reciprocal of the mean of the slowest 1% of frame intervals in up to 30 seconds |
| `gpu.fps_low_01p` | The same calculation for the slowest 0.1% |
| `gpu.fps_min` / `gpu.fps_max` | Slowest / fastest individual frame in that history, expressed as FPS |
| `gpu.game` | Executable name of the measured process |
| `gpu.fps.source` | PresentMon or RTSS |
| `gpu.limiter` | Conservative CPU / GPU / frame-cap estimate; unavailable when evidence is ambiguous |

Lows need enough frames: at least 100 for 1% and 1000 for 0.1% inside the
history window. Until then, Studio says it is collecting data. A very low
frame rate may never supply 1000 frames in 30 seconds. Queue overflow makes
history statistics unavailable while a complete window is collected again.

These FPS do not guarantee a separate count of native and generated frames.
The console and service paths were compared in Windrose with Frame Generation
on and off; the service's application/displayed metrics did not establish a
verified native/generated split in that configuration. Do not divide a reading
by a presumed multiplier to invent a native frame rate.

## RTSS fallback

Run RTSS (available separately or with MSI Afterburner), then start the game.
It supplies the FPS of the application it measured most recently. Other new
frame statistics remain unavailable; `gpu.fps.source` reports RTSS. If neither
tool can read, Studio shows a reason and keeps the raw diagnostic detail.

A live zero from RTSS remains zero. Missing or stale data is unavailable,
never the last successful value held indefinitely. PresentMon's one-second
statistics expire without recent frames even while its process remains alive.
