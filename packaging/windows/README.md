# Bezel Evo for Windows x64

Download the Setup executable for a normal installation, or extract the
portable ZIP into a writable folder. Start `bezel-studio.exe`, choose a display
and save a theme. Studio and Light run as your normal Windows user.

Setup offers optional **Light at Windows sign-in**, **hardware sensors**, and
**game FPS support**. Configuring sensors or the FPS collector asks for
administrator approval; Studio and Light remain normal user applications.
Game FPS installs the verified Intel PresentMon 2.6.0 service/API, without its
GUI or changes to account groups. Retry with **Setup game FPS** in the Start menu
or `setup-game-fps.cmd` for portable installations. The separately registered
FPS component remains available to other clients when Bezel is uninstalled;
remove its own Windows Apps entry separately after closing those clients.
An existing compatible Intel service is reused; conflicting services are preserved.

Sensor configuration preserves vendor tasks and other Libre installations. If
another Bezel installation owns `Bezel-Sensors`, its task is left unchanged.
The installer does not install hardware drivers.

For the portable version, run `setup-evo-sensors.cmd` once to configure the
reader. Double-click `start-light.cmd` for Light after saving a theme and choosing
a screen in Studio. The installer-maintenance script can also enable Light at
sign-in: `powershell -NoProfile -ExecutionPolicy Bypass -File
installer-maintenance.ps1 -Mode Startup -AppDirectory "C:\path\to\Bezel Evo"
-Startup 1`. The command-line execution policy applies only to that process.
Setup's Light option and Studio's own editor-autostart setting are separate.

Save and fully exit Studio and quit Light from the tray before updating or
uninstalling. The sensor helper exits within 30 seconds after its clients close.
The installer blocks open Studio/Light clients and waits up to 45 seconds for
the reader's sensor files to become writable, even when Windows hides its
elevated process path. It does not kill applications. Uninstall removes only
the sensor task and Light login entry that point
to this installation. User themes, preferences and caches are retained, including
the inherited internal `io.github.slipalison.bezel` data directories used for
compatibility. Bezel Evo's installed product name and publisher identify this fork.

Setup can download Microsoft's WebView2 runtime when absent. Portable Studio
requires WebView2 already installed. The sensor helper uses .NET Framework 4.7.2
or newer. Video preview/transcoding requires an available FFmpeg installation;
FFmpeg is not bundled in this first package.

Silent Setup (`/S`) preserves optional-service choices on updates and does not
enable new login entries or display a sensor elevation prompt. For a first silent
installation, configure those services afterwards. Setup and binaries currently
have no Authenticode signature.

Bezel Evo derives from Bezel by [Alison Amorim (@slipalison)](https://github.com/slipalison).
Original authorship and copyright notices are preserved. Bezel Evo is GPL-3.0-or-later;
source and release history are at https://github.com/jripper77/bezel-evo.
The embedded LibreHardwareMonitorLib 0.9.6 and dependencies retain their licenses
under `sensors/`. Tabler and Material Design Icons retain MIT and Apache notices.
IBM Plex: SIL OFL 1.1 (`LICENSE-IBM-Plex-OFL.txt`).
The installer template is adapted from Tauri CLI 2.12.1 under MIT; its license is
included. PresentMon MIT and dependency notices are included under `fps/`.
No personal logs, hardware aliases or development backups are shipped.

## Maintainer build

Run `scripts/windows/package-evo.ps1` from a shell with Rust 1.98, the MSVC build
tools, Node/npm and the Windows Framework C# compiler. It builds both binaries,
downloads a SHA-256-pinned LibreHardwareMonitor release, rebuilds the helper,
prepares the pinned and signature-verified Intel PresentMon service payload,
and uses the pinned Tauri CLI 2.12.1 to produce NSIS and ZIP packages under `dist/`.
Use `-Version X.Y.Z` to choose a version or retry; `-SkipBuild` bundles already-built
binaries matching `VERSION`. The adapted template retains Tauri's WebView2 and
upgrade handling, adds the options page, and omits user-data deletion.
