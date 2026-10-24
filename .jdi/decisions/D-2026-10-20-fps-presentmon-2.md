# D-2026-10-20-fps-presentmon-2: PresentMon integration on the v0.1.59 base

**Context**: The FPS module was reworked for v0.1.59 (Windows-only, RTSS as
the single source). The hardware-validated PresentMon integration from the
`presentmon` branch needs to be rebuilt on this base, including the two
changes agreed with the maintainer in PR #1.

**Decision**: Re-add PresentMon as a full `FrameRateSource` on Windows,
started and stopped on demand, with two agreed changes to the earlier
design:

1. **No `PATH` lookup**: the executable is discovered only via the
   configured path, `tools\` and `sensors\` next to `bezel.exe`, and Intel's
   installation directory. Discovery stays predictable.
2. **A fixed, Bezel-specific ETW session name** (`BezelPresentMon`), so
   `--stop_existing_session` never touches unrelated PresentMon sessions,
   while a leaked session after a crash is still cleaned up on the next
   start.

Additionally, versioned release names such as `PresentMon-2.6.0-x64.exe`
are accepted (plain `PresentMon.exe` wins), and placeholder process names
in angle brackets (`<unknown>`) are never treated as a game.

**Rationale**: See D-2026-10-15-fps-presentmon-1 for the original
rationale (on-demand console app, safe CSV parsing, job object lifecycle,
RTSS fallback). This record only updates it for the new base and the two
maintainer-requested changes.

**Consequences**:
- Windows-only; RTSS remains the fallback for `gpu.fps`.
- Requires Administrator rights or "Performance Log Users" membership.
- The 9 FrameMetrics sensors (`gpu.fps`, `gpu.frametime`, `gpu.fps_low_1p`,
  `gpu.fps_low_01p`, `gpu.fps_min`, `gpu.fps_max`, `gpu.game`,
  `gpu.limiter`, `gpu.fps.source`) come back with this integration.
- Hardware-validated on the previous base (E1–E9); re-validated per build.
