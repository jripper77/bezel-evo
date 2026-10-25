# PR #2: draft response to DarkK3y-v2

Prepared for review before posting. This is a local integration branch, not a
published release or a merge into the main branch.

Production deployment is still under review: the console choice below concerns
the local capture implementation. The intended user experience keeps Studio
and Light unelevated, with a separately authorized collector component set up
during installation. The local administrator/group workaround is not a completed
distribution design, and no service/helper migration has been implemented yet.

---

Thanks for the proposal and the original implementation! We have prepared a
local integration branch while preserving your commits and authorship. We also
added explicit credit linking your profile and PR #2 in the README and changelog.

We compared the official PresentMon 2.6.0 console with its service/API, first
using a controlled graphics workload, then Windrose with Frame Generation on
and off. We decided to keep your console-based approach, with RTSS as the FPS
fallback. The collector's working set was about 16–17 MiB for the console versus
109 MiB for the service in these short game runs. This is not a game-overhead
benchmark, a long-term leak test, or a proof that the API cannot offer benefits
in other configurations.

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
We will not infer native FPS by dividing by a presumed multiplier. The API
remains a possible later addition if it provides verified extra metrics worth
the deployment and resource cost.

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
FG configurations remain to be validated. The full pre-push suite is still
required before publication.

The original hardware-validation record remains attributed to your contribution;
we are tracking the rewritten adapter's validation separately. Generic CLI
diagnostics also need a presentation review before release.

We wanted to share the changes and the decision with you before merging into
the base branch, as agreed. Feedback on these choices would be welcome.
