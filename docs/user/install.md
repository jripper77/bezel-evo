# Install Bezel Evo

[Português (Brasil)](pt-BR/install.md)

Bezel Evo runs on **Windows 10 and 11, x64**. Download it from
[the Releases page](https://github.com/jripper77/bezel-evo/releases):

| File | What it is |
|---|---|
| **Setup** (`Bezel-Evo-<version>-windows-x64-setup.exe`) | Installs Studio, Light, the bundled themes and the hardware sensor reader for your Windows user |
| **Portable ZIP** (`…-windows-x64-portable.zip`) | The same programs, to extract into a writable folder |
| **Checksums** (`…-SHA256SUMS.txt`) | SHA-256 of each file, to check a download |
| **Source** | The source code of that release |

Before installing, quit any other program that drives the screen (the vendor
app, turing-smart-screen-python): only one program can use it at a time. See
[Coming from turing-smart-screen-python](migrating.md).

## Setup

Run the Setup executable. It installs for your Windows user and, on its
**Startup and hardware sensors** page, offers two options:

- **Start Bezel Evo Light when I sign in**: your saved themes come back on the screens when
  you sign in, without opening Studio ([Running at sign-in](run-at-login.md)).
- **Configure hardware sensors**: registers the bundled sensor reader, so CPU
  temperatures, fans and power can be read ([Sensors](sensors.md)). Only this
  step asks for administrator approval; approve it with the same Windows
  account. Studio and Light always run as your normal user.

If Microsoft Edge WebView2 is missing, Setup downloads it. Hardware sensors use
.NET Framework 4.7.2 or newer, which Windows 10 and 11 already include.

Then open **Bezel Studio** from the Start menu and [make your first theme](first-theme.md).

## Portable ZIP

Extract the ZIP into a folder you can write to, then start `bezel-studio.exe`.
The portable version needs WebView2 already installed.

- Run `setup-evo-sensors.cmd` once to configure the hardware sensor reader.
- Double-click `start-light.cmd` to start Light, after saving a theme and
  choosing a screen in Studio.

## Releases are not signed

Bezel Evo's Setup and programs carry no code signature, so Windows SmartScreen
may stop Setup with "Windows protected your PC". Check that the file is the one
you downloaded, click **More info**, then **Run anyway**.

To check that a download is intact, compare its SHA-256 with the line for that file in
`Bezel-Evo-<version>-SHA256SUMS.txt`, on the same release (lower or upper case
does not matter):

```powershell
Get-FileHash .\Bezel-Evo-<version>-windows-x64-setup.exe
```

## Updating

Once a day Light asks GitHub for the latest published release (it sends only
its version, nothing else). When a newer one exists, the tray menu shows
**Update available: X.Y.Z** and Studio shows it in **Preferences › Updates**,
with a button to the download page. Nothing is downloaded or installed for
you; the switch in the same place turns the check off.

Save your theme and quit Studio, then quit Light from its tray icon (**Quit**).
Run the new Setup, or extract the new ZIP over the old folder. Setup refuses to
update while Studio or Light is running; it never closes them for you.

Updates keep your themes, preferences, fonts and caches. A silent update
(`/S`) keeps the options you chose before and does not turn on new ones.

## Uninstalling

Use **Settings → Apps → Installed apps** in Windows. Uninstalling removes the
programs, and the sensor task and Light sign-in entry that belong to this
installation. Your themes, preferences and caches stay.

## What is not included

- **ffmpeg**, to convert videos: [Installing ffmpeg](ffmpeg.md).
- **Hardware drivers**: most screens need none; the Turing USB generation and
  WCH panels need WinUSB ([Let Bezel open the screen](permissions.md)).

Details for maintainers, silent installation and the portable scripts are in the
[Windows packaging guide](../../packaging/windows/README.md). To build from
source, see [Build on Windows](../build-windows.md).
