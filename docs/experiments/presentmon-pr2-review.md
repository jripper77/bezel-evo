# PR #2: draft response to DarkK3y-v2

Prepared for review before posting. This is a local integration branch, not a
published release or a merge into the main branch.

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

Tests and clippy pass for the five changed Rust crates; the affected UI checks
include 55 unit tests and 28 distinct browser cases. The earlier hardware runs
tested standalone acquisition probes, so they do not validate every lifecycle
change in this integrated adapter. Next is a local Windows executable trial
with Windrose. Real RTSS fallback, extended idle/leak behaviour, other games and
FG configurations remain to be validated. The full pre-push suite is still
required before publication.

The original hardware-validation record remains attributed to your contribution;
we are tracking the rewritten adapter's validation separately. Generic CLI
diagnostics also need a presentation review before release.

We wanted to share the changes and the decision with you before merging into
the base branch, as agreed. Feedback on these choices would be welcome.
