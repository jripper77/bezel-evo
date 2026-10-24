# PresentMon evaluation (Windows, development only)

These probes do not change Bezel, its settings, or connected displays. They use
the official PresentMon **v2.6.0** console and matching service/API, extracted
from the signed MSI into `target/scratch/fps-evaluation`. No Windows service is
installed. The synthetic workload opens a small, nonactivating, topmost D3D11
window so another application cannot occlude its swap chain during the probe.

## Inputs

Get the console EXE and MSI from
<https://github.com/GameTechDev/PresentMon/releases/tag/v2.6.0>.
Verify Intel Authenticode signatures and these release hashes:

| File | SHA-256 |
| --- | --- |
| PresentMon-2.6.0-x64.exe | B2A706BC6AD475749E3B7E3409263AA1E6906D45BDCF993F6DBC0F660188F1AF |
| PresentMon-2.6.0.msi | 0CB43D2622356C1E277A77F840887F9000CEA4E2D4236C0850922FC204C6F820 |

Use `msiexec /a` to create an administrative image under the scoped scratch
directory, rather than installing the MSI. Its AdminExecuteSequence contains
only standard image extraction actions. Keep the DLL with its matching service
and use the extracted `SDK/PresentMonAPI.h` to resolve metric IDs and signatures.
Production integration would instead use Intel's installed API loader.

Compile `workload.cpp` using MSVC x64, `/std:c++20 /EHsc /W4 /WX`, linking
`d3d11.lib dxgi.lib user32.lib /SUBSYSTEM:WINDOWS`. Place the executable at
`target/scratch/fps-evaluation/fps-workload-v2.exe` and compiler intermediates in
the same scoped scratch directory. The workload repeats 60/30/60 FPS in four
second phases. Actual successful presents are written to its redirected stdout.

## Runs

`run-admin.ps1 -SmokeOnly` performs the initial console/API test. Without the
switch it also runs three repetitions with baseline/console/API, sequentially.
ETW capture requires administrative rights or appropriate ETW group membership.
The default local repository and Python paths can be overridden by parameters.

`run-admin.ps1 -NormalClientProbe` starts only the elevated collector and writes
`api-server/connection.json` under its timestamped log directory. Run a normal
user client with `evaluate.py --connect <that-file> --modes api` and the required
paths. It writes a stop marker after it has finished; the collector also has a
bounded lifetime if the client never arrives. `run-admin.ps1 -GameProcessId <PID>`
checks the expected Windrose process, records 45 seconds of baseline and console
each, then prepares that same separate collector for a normal-user API client.
For a shorter retry, use `-SkipGameBaseline -GameSeconds 25 -StartDelaySeconds 10`,
then pass `--seconds 25` to the separate normal-user API client. The delay lets
the player return from the chat/UAC prompt to the game before measurement.

For a running game, use `evaluate.py --pid <PID>` with the same required paths;
it never launches or terminates the game. `--modes console,api --seconds 60`
records each source sequentially. Use the same reproducible scene and settings,
then repeat without/with Frame Generation only if supported by the game.

Each API test uses a unique named pipe, ETW session and temporary directory.
The service is a hidden, disposable child. Its test-control `%ping`/`%quit`
protocol requests graceful shutdown; a bounded timeout terminates only the
owned child and records the failure. The driver removes only newly-created,
still-empty PresentMon configuration registry keys. It preserves pre-existing
or nonempty keys. A crash or forced driver exit may leave resources behind;
inspect the run's evidence before removing anything.

## Analysis and limits

Run `python analyze.py <run-directory>` for `summary.json` and
`python -m unittest discover -s scripts/windows/fps-evaluation -p test_analysis.py`
from the repository for attribution/window tests.

- Console windows use source timestamps, grouped by PID and swap chain.
- API queries distinguish Presented, Displayed and Application FPS. Unsupported
  or nonfinite data stays missing. CPU frame duration is a separate metric.
- The API requests a 100 ms ETW flush period. Record that configuration when
  comparing CPU use and first-data delay.
- Resource reports separate the collector/service, reader and workload. CPU is
  reported both relative to one core and normalized over logical processors.
- The synthetic workload validates acquisition and rate changes. Its paced,
  lightweight rendering cannot establish game performance overhead.
- Game runs record target CPU/memory counters, total system CPU and whether
  the target PID owns the foreground window at every sample. A background-game
  capture must not be compared with a foreground capture. A baseline without
  ETW contains CPU/memory counters, not an FPS measurement.
- Dynamic API queries may return a cached nonzero value after Alt-Tab. Success
  of a query alone does not establish fresh data. Focus, source-frame age and
  capture lifecycle need explicit handling in any product integration.
- Analysis preserves unqualified statistics and also emits
  `foreground_window_*` distributions. These use only one-second windows
  fully inside observed foreground intervals, excluding one second after a
  return to the game and never bridging missing focus samples. These filters
  improve the comparison; they do not replace source-frame freshness checks.
- Startup-inclusive short-run memory growth is not a memory-leak test.
- A few valid rows are not enough: inspect source coverage and rate changes.
  Zero/absent samples, capture failures and occlusion invalidate a comparison.
- A service run in console mode is a feasibility test. Installed-service ACLs,
  normal-user clients, upgrade behaviour, extended idle and fallback RTSS still
  need separate validation before a product architecture decision.

The official 2.6.0 service may report an access error for its optional secure ETL
logging directory when run as an elevated user instead of LocalSystem. The live
queries in this experiment succeeded despite that warning. It is preserved in
the logs; the probes do not change ACLs or delete the component's shared temp
directory. Explicit per-run TEMP/TMP do not redirect this internal directory.
