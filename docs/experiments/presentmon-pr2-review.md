# PR #2: draft response to DarkK3y-v2

Prepared for review before posting. The integration is included in the 0.2.12
release preparation; this document has not been sent to the contributor.

The accepted deployment design now uses the separately installed PresentMon
service by default, keeping Studio and Light unelevated. An explicit console
path remains an advanced override. The service adapter and optional installer
component are implemented. The user confirmed DRG Survivor reads FPS on the
service build after restarting Windows; see the qualified validation record below.

---

Thanks for the proposal and the original implementation! We have prepared a
local integration branch while preserving your commits and authorship. We also
added explicit credit linking your profile and PR #2 in the README and changelog.

We compared the official PresentMon 2.6.0 console with its service/API using
a controlled workload and Windrose with Frame Generation on and off. The console
was lighter (about 16–17 MiB versus 109 MiB service working set in the short runs),
so we initially used it locally. For distribution, we subsequently chose the
service API: one-time authorization belongs to the collector, while the editor
and runtime remain normal applications. This is a deployment tradeoff, not an
overhead benchmark or a claim that the service is lighter.

The adapter consumes individual frame events instead of polling a cached FPS
number. Two normal-user controlled 60/30/60 FPS trials decoded over 550 frames
each, followed the rate changes and expired readings after the workload ended.
The account was not elevated and was not in Performance Log Users. Initial
attempts produced no events until the owned service was restarted; that evidence
is retained. The user later confirmed DRG Survivor reads FPS after a Windows restart.
Before that restart, an in-foreground API probe returned no frames and standalone
PresentMon also lost ETW events. We have not isolated the cause or established
extended stability across games.

The integration changes are:

- Resolve conflicts with the current sensor catalogue and Windows features;
  fix fixture loading and new runtime option initializers so the tests compile.
- Target the foreground candidate process and measure one swap chain, rather
  than mixing frames from different applications or chains.
- Preserve source QPC timestamps and expire stale readings. Buffered output
  must not make old frames look fresh after an Alt-Tab or pause.
- Separate source freshness from the averaging window. Windrose console traces
  delivered some frames 1–1.5 seconds late, so a window anchored to the polling
  clock incorrectly rejected active data. Average the latest measured second,
  expiring readings once the source frame is more than three seconds old.
- Hide the collector console, bound incoming queues and frame history, isolate
  reader state between captures, and join the readers during shutdown.
- Retain the fixed ETW session name, with a global ownership guard so concurrent
  Bezel instances do not stop each other's collector. Assign the child to a
  kill-on-close job during startup; spawn and job assignment are not atomic.
- Fall back to RTSS when a collector exits after spawning, as well as when
  startup fails. Bound retries/backoff and stop capture when no FPS sensors
  are requested.
- Resume promptly after Alt-Tab: returning to a foreground game bypasses idle
  retry delays and re-arms a retained collector's first-frame watchdog, while
  actual capture failures and permission errors retain their retry backoff.
- Define lows explicitly, require 100/1000 samples for 1%/0.1%, and temporarily
  withhold history statistics after overflow. Keep the CPU/GPU/cap estimate
  unavailable when the evidence is ambiguous.
- Following the local hardware trial, define minimum/maximum FPS as extrema of
  complete one-second averages in up to 30 seconds of source history. Raw
  single-present reciprocals produced values around 4,773 FPS from a 0.2095 ms
  interval, which was misleading for the named FPS sensors. The UI labels now
  identify the one-second interval; lows keep their frame-interval definition.
- Add translated sensor labels and availability reasons in English, Portuguese
  and Italian; identify PresentMon correctly in the sensor picker. Restore the
  unrelated Support section removed from the README.

We are keeping the main FPS reading as a numeric measurement. We are **not**
adding an automatic “DLSS” label or a claimed native/generated split. In the
Windrose runs, the API's application/displayed/presented measurements did not
establish that split, even though the readings changed with the FG settings.
We will not infer native FPS by dividing by a presumed multiplier. The service API is used for deployment and frame events; this does not establish
a verified native/generated split.

A follow-up console capture explicitly enabled `--track_frame_type` with FG
enabled in Windrose. It returned 3,081 target rows, all labelled `Application`;
the qualified foreground FPS median was about 212. Only 33/80 focus samples
were in-game, so this is a limited instrumentation check, not a continuous
20-second foreground stability test. It does not give us a trustworthy DLSS
label, and it does not establish that DLSS detection is impossible by another
method. We have kept the experimental tracking flag out of production.

The final configuration review also found and fixed a missing propagation:
changing the configured PresentMon path must rebuild the sensor source,
whereas changing diagnostics or audio consent must not. Oversized path values
are rejected before reaching the adapter.

Tests and clippy pass for the five changed Rust crates; the affected UI checks
include 55 unit tests and 28 distinct browser cases. The earlier hardware runs
tested standalone acquisition probes, so they do not validate every lifecycle
change in this integrated adapter. In the local Windows trial with Windrose,
the user has now confirmed that Bezel's game-FPS widget on the
hardware display was continuous and close to Steam during the short FG-on trial
of local build 0.2.9. This is a useful acceptance check, not an extended benchmark.
Real RTSS fallback, extended idle/leak behaviour, other games and
FG configurations remain to be validated. Full publication checks are recorded
in the release hand-off; targeted results above describe the earlier integration.

The original hardware-validation record remains attributed to your contribution;
we are tracking the rewritten adapter's validation separately. Generic CLI
diagnostics also need a presentation review before release.

We preserved your commits, credited the proposal in the README and changelog,
and integrated the final changes for 0.2.12. Feedback on the deployment and
statistics choices would be welcome.
