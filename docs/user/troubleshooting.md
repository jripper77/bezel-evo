# Troubleshooting

[Português (Brasil)](pt-BR/troubleshooting.md)

Start with Studio's screen selector, or `bezel devices` in a terminal: both list
the connected screens and their state without sending them anything. Add `-v`
to any command to see what Bezel sends and receives. When you need more detail,
the [logs](#logs) record what Studio and Light did.

## "No smart screen found"

- Check the USB cable (some cables only charge) and try another port.
- Turing USB (`1CBE`) and WCH (`43A8`) screens need the WinUSB driver:
  [Let Bezel open the screen](permissions.md).
- If Windows gave the screen a new COM port, pick it again in Studio's screen
  selector.

## The screen is in use ("access denied")

Only one program can use a screen at a time. Windows refuses a serial screen
that another program holds, and Bezel says so, for example:

```text
access denied to COM5: …
```

Stop that program: turing-smart-screen-python, the vendor app, or a second
`bezel run` in a terminal. Studio and Light share the screens between them on
their own. See [Coming from turing-smart-screen-python](migrating.md#1-stop-it).

## The screen is dark, or listed as "asleep"

Rev C screens sleep when another program turns them off (turing-smart-screen-python
and the vendor app do it when they quit) and after `bezel off`. Anything that
draws wakes them: **Live** in Studio, `bezel run`, `bezel show`,
`bezel test-pattern`. Waking takes a few seconds.

## The screen froze / stopped responding

Symptoms: *"the screen stopped responding: it stopped reading what was sent"*,
"timeout talking to …", an upload or the live theme that stalls. A Turing rev C
screen (the 8.8" and the other serial models with a wake chip) can freeze, for
example on an upload over its 25 MiB limit. **No replug is needed**: Bezel
restarts it through its wake chip, and it comes back in about 10 seconds.

- **While live**: Light, `bezel run` and **Live** in Studio
  connect the screen again by themselves, 2, 5 and 10 seconds after it
  stopped; connecting restarts a frozen screen first, then the theme goes on.
  Studio's status bar says *Reconnecting to the screen (attempt 1 of 3)…*,
  then *The screen is back and shows the theme live again.*; `bezel run`
  writes the same in the terminal. After the third attempt live mode stops
  with the error. Turning **Live** off (or Ctrl+C) meanwhile stops at once.
- **On its own**: the next command, or turning **Live** on again, finds the
  frozen screen and restarts it once before connecting. Nothing is restarted
  while the screen answers.
- **Command line**: `bezel restart` (or `bezel restart -s COM5` for one
  screen). It says that it restarts the screen, waits for it and prints where
  it is back. The terminal also prints this hint after an error that means the
  screen froze.
- **Studio**: **Screen → Settings → Advanced → Restart screen…**, after a short
  confirmation.
  A storage error that means the screen froze has the same button, and so has
  the screen's card when live mode stopped because of it.

A restart stops what the screen shows or plays; the files it stores stay. If
the screen still does not answer after a restart, or it has no wake chip
(Bezel then says it cannot restart it), unplug its USB cable, wait a few
seconds and plug it back in.

## After cancelling an upload

Delete the incomplete file before sending again: Studio offers **Delete the
incomplete file**, and the terminal prints the `bezel storage rm … --yes`
command. If the next upload ends with *"the stored size differs; delete it and
send it again"*, delete that file and send it once more. See
[Cancelling an upload](storage-and-video.md#cancelling-an-upload).

## "refused: the file is … MiB and this screen takes files up to 25 MiB each"

Turing rev C screens take at most 25 MiB per file: their firmware keeps the
whole upload in memory and freezes on a larger one. Bezel refuses the file
before sending anything. Send a shorter clip, or let Bezel convert the video at
a lower frame rate (`--fps 24`): a conversion caps the bitrate so the result
fits. See [How large a file can be](storage-and-video.md#how-large-a-file-can-be).

## A video will not send

"needs ffmpeg" or "ffmpeg not found": install ffmpeg with `libx264`
([Installing ffmpeg](ffmpeg.md)). Pictures, and videos already in the screen's
format, go without it.

## "No SD card in the screen"

The card is missing or not FAT32 on an MBR partition table:
[Preparing an SD card](sd-card.md).

## A sensor shows `—`

`bezel sensors` prints why. See [Sensors](sensors.md#when-a-value-is-unavailable).

Studio's status bar has a sensor chip: green when everything reads, yellow when
at least one piece of hardware gives no readings, red when nothing reads. Click
it to see which hardware is affected. Optional missing sensors do not turn a
healthy device yellow. **Restart Libre** restarts the bundled hardware sensor
reader; the chip updates when readings return. This can recover a Corsair PSU
whose USB readings stopped, even when its computed Total Output still reports
`0 W`. The restart needs the `Bezel-Sensors` task that Setup's **Configure
hardware sensors** option (or `setup-evo-sensors.cmd` in the portable ZIP) creates.
See [The sensor status](sensors.md#the-sensor-status).

## Game FPS shows `—`

No tool is measuring a game, or the game is paused: [Game FPS](fps.md).

## "Windows protected your PC"

Bezel's installers are not signed. Click **More info**, then **Run anyway**:
[Install Bezel](install.md#releases-are-not-signed).

## A Turing USB panel shows as "desktop mode"

It is in the vendor's desktop (second monitor) mode:
[Supported screens](devices.md#desktop-mode).

## Studio does not open

Studio opens no console. To see what it prints, open a Command Prompt in the
folder Bezel Evo is installed in and run `bezel-studio.exe 2> bezel-studio.txt`,
then open `bezel-studio.txt`. The [logs](#logs) may tell more.

Studio's messages (`bezel-studio: …`) are fixed sentences from a closed list:
none of them holds the text of the error behind it, nor a file or folder of
yours. This is by design, so that nothing Studio prints can carry a secret, such
as your KLIPY key, or a personal path. When Studio stops on an error (it
panics), the one line it prints adds where in the source code that happened: a
file of Bezel's or of a library's code, from its crate's folder on
(`tauri-2.…/src/…`), and a line and a column, never the error's message.
When Studio cannot start, or starts without a part, it says which part failed:

| It says | What failed |
| --- | --- |
| `bezel-studio: the app's folders were not found`, then `bezel-studio: the app panicked at tauri-…/src/app.rs:…` | Where Studio keeps its settings, data and cache (your user's `AppData` folders). |
| `bezel-studio: the tray icon was not added`, then `bezel-studio: the app panicked at tauri-…/src/app.rs:…` | The tray icon. |
| `bezel-studio: the app panicked at tauri-…/src/app.rs:…`, with no line before it | The window or the page inside it: the WebView2 runtime. Run Setup again, or install WebView2 from Microsoft. |
| `bezel-studio: the app panicked at …`, another place | An error inside Studio or a library it uses. Report it, with the whole line. |
| `bezel-studio: the app did not start: a plugin did not start` | One of its plugins: a single window, file dialogs, opening links, starting at login. |
| `bezel-studio: the app did not start` | Another cause. |
| `bezel-studio: refresh loop not started: no thread or memory to spare` | Studio opens, but sensors and the live screen do not update: Windows is out of threads or memory. Close some programs and open Studio again. |
| `bezel-studio: refresh loop not started` | The same, for another cause. |

In a place, `…` stands for the library's version, the line and the column,
which change from one Bezel release to the next. When you report the problem,
copy these lines as they are.

## Logs

Studio and Light write their logs to one folder:

```text
%LOCALAPPDATA%\io.github.slipalison.bezel\logs
```

To open it, use **Preferences → Logs → Open logs folder** in Studio (**⋯ →
Preferences** in the top bar). Light's log is `light.log`. The logs are written also when
**Debug** is off; **Preferences → Debug → Enable diagnostic logs** adds detail
about rendering, transfers and sensors while you reproduce a problem. Logs rotate
at 2 MiB and never record network requests. The hardware sensor reader keeps its
own logs ([sensor reader guide](../../apps/bezel-sensors-helper/README.md)).

## Reporting a problem

Include the version (shown in Studio's header and by `bezel --version`), the
output of `bezel -v devices`, and the [logs](#logs) of the moment it failed.
Remove serial numbers and personal paths before posting.
