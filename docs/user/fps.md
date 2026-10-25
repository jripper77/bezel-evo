# Game FPS

[Português (Brasil)](pt-BR/fps.md)

On Windows, Bezel reads Intel PresentMon's separately installed service, with
RivaTuner Statistics Server (RTSS) as the FPS fallback. Bezel does not inject
code into a game. Drag the sensors onto a theme like any other readings.

## Enable game FPS

1. Select **Enable game FPS** in Windows Setup, or run **Setup game FPS** from
   the Bezel Evo Start menu. Portable users run `setup-game-fps.cmd`.
2. Approve the administrator prompt once to install the optional, signed Intel
   collector. Studio and Light continue to run as your normal Windows user.
   Your account groups are unchanged; signing out is unnecessary.
3. Add an FPS sensor and bring the game to the foreground. The foreground
   candidate process is selected, excluding common shell/browser/launcher apps.
   This is a graphics-app heuristic, not a complete database of games.

Setup includes the verified PresentMon 2.6.0 service and API; the Intel GUI is
not installed. An existing compatible, signed Intel service is reused without
overwriting its files. Incompatible existing services are preserved and setup
reports the conflict. A failed or declined setup leaves FPS unavailable (or
uses RTSS when available); retry through **Setup game FPS**.

Bezel reads frame events on a background worker and requests tracking only
while FPS sensors are needed. It measures one swap chain of the chosen PID.
Stopping FPS requests closes that client's tracking session; it does not stop
the shared Windows service or another application's sessions. Missing frames
expire after three seconds. Idle tracking pauses after 10 seconds initially,
or 60 seconds after receiving frames. Returning to a foreground game bypasses
idle delays; connection failures keep bounded retry delays.

The optional component has its own **Bezel Evo - Game FPS support (Intel
PresentMon)** entry in Windows Apps. It stays installed when Bezel is removed
so other PresentMon clients are not disrupted. Close its clients before removing
it. Removal only deletes the service owned by this component and known files;
modified or unknown files and licence/maintenance files are retained.

### Advanced: legacy console override

An explicit `presentmonPath` in Studio's `settings.json`, or CLI
`--presentmon-path`, selects the older console backend instead of the service.
This override still requires ETW rights (administrator or Performance Log Users).
Clear it to use the normal service setup. Bezel does not automatically elevate
the editor or change Windows groups. The console runs hidden on demand, with
bounded queues and retries; only one Bezel instance owns its fixed capture
session. Neither backend searches `PATH` for a DLL or injects a game hook.

## If the game counter stays unavailable

Check that FPS support is installed, an FPS sensor is used by the theme, and
the game is rendering in the foreground. A paused or minimized game can stop
producing frames. Retry **Setup game FPS** if the service is missing or stopped.

If acquisition remains unavailable even in standalone PresentMon, save your
work and try restarting Windows before reinstalling Bezel or changing account
permissions. In a local DRG Survivor test, both readers stopped producing frames
and standalone capture reported lost ETW events; the user confirmed recovery
after a Windows restart. This is a recovery observation, not an isolated cause
or a guarantee that every capture failure has the same solution.

## Available sensors

| Key | Meaning |
|---|---|
| `gpu.fps` | Average FPS over the latest measured second, using present intervals when available |
| `gpu.frametime` | Mean frame interval over the latest measured second, in milliseconds |
| `gpu.fps_low_1p` | Reciprocal of the mean of the slowest 1% of frame intervals in up to 30 seconds |
| `gpu.fps_low_01p` | The same calculation for the slowest 0.1% |
| `gpu.fps_min` / `gpu.fps_max` | Lowest / highest one-second mean FPS in up to 30 seconds of source history |
| `gpu.game` | Executable name of the measured process |
| `gpu.fps.source` | PresentMon or RTSS |
| `gpu.limiter` | Conservative CPU / GPU / frame-cap estimate; unavailable when evidence is ambiguous |

The collector delivers frames in batches. The averaging window ends at the latest
received source frame, rather than the time Bezel reads its output. A reading
expires when that source frame is more than three seconds old, even if buffered
output has just arrived. The value is not an instantaneous display refresh count.

Min/max use complete one-second source windows with at least two frames. They
remain unavailable until there is a full window, and partial windows are excluded.
They do not take the reciprocal of the fastest individual frame: a 0.2 ms
present interval would misleadingly look like 5,000 FPS. Lows below remain
frame-interval statistics and describe a different aspect of frame pacing.

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
