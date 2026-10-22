# Sensors

[Português (Brasil)](pt-BR/sensors.md)

Bezel reads the computer's sensors itself: processor, graphics card, memory,
disks, network, motherboard and fans. In Studio they are in the **Sensors**
panel, grouped by category, each with its current value and a small history
line. Drag one onto the canvas to show it, or onto a widget to bind it.

Combine the search and category choices with **Probe** (Windows,
LibreHardwareMonitor, NVML or another reader) and **Status**: all, available,
unavailable or not checked yet. The result count shows how many sensors match
out of the managed catalog. Valid zero values and text readings are available;
not checked yet means no reading has arrived, rather than a failed sensor.
Long source descriptions and hardware paths wrap within the panel.

Widget source cards, colour-by-value rules and sensor-driven motion use the
same [searchable sensor picker](studio-editing.md#choosing-a-sensor). Search
names, keys, categories or probes; results show live readings and availability.

From a terminal:

```powershell
bezel sensors                    # every sensor, with its key
bezel sensors --watch 1          # refresh every second, until Ctrl+C
bezel sensors --json             # for scripts
```

## The hardware sensor reader

CPU usage, memory, disks and network work out of the box, and NVIDIA cards
through their driver. Temperatures, fans, voltages and power come from the
**hardware sensor reader** bundled with Bezel Evo, built on
LibreHardwareMonitorLib. You do not need to keep a separate monitoring app open.

- **Setup** configures it when you tick **Configure hardware sensors**; **Setup sensors** in the Start
  menu does it again later. The portable ZIP has `setup-evo-sensors.cmd` for the same step.
- It is the only part of Bezel that runs with administrator rights, as a task of
  your Windows user; Studio and Light stay unelevated and start it when they need
  readings. It stops by itself 30 seconds after the last of them closes.
- It only reads: it never controls fans or lighting, and installs no drivers.
  Which values exist depends on your hardware and its drivers.

An external LibreHardwareMonitor already running with its web server
(`http://127.0.0.1:8085`) still works as a fallback. Setup, logs and rollback are
in the [sensor reader guide](../../apps/bezel-sensors-helper/README.md).

## The sensor status

The **sensor chip** in Studio's status bar shows how the readings go:

- **sensors · 3 ms**, with a green dot: everything reads (the time is how long the
  last reading took);
- **sensors · 1 missing**, yellow: at least one sensor reading is unavailable;
- a red state: none of the checked sensors can be read, or sampling failed;
- **sensors not checked yet**: no checked readings are available yet.

Click it for a compact **Probes and sensors** summary. One row per probe shows
its state and sensor counts. Click the probe to open its sensors, or its
available/unavailable count to open that filtered view. Overall counts work the
same way. Hardware paths and individual values stay in the Sensors panel.

GPU aliases belong to their actual reader (for example NVML), including
unsupported metrics; they do not appear as separate probes.
The last-reading time remains available. **Restart** appears on each probe
that supports it: currently only the managed LibreHardwareMonitor reader.
Windows and NVML do not expose a restart action. Filters continue checking their selected
scope so a recovered sensor leaves the unavailable view automatically.
**Open Sensors** clears the filters and opens the whole catalog.

![Sensors grouped by category, and the sensor chip popover with per-probe counts and Restart](../images/studio-sensor-probes.png)

## When a value is unavailable

Bezel never makes up a number. A sensor it cannot read is *unavailable*: the
theme shows `—`, and `bezel sensors` prints the reason next to it.

| Sensor | Why it is unavailable | What to do |
|---|---|---|
| CPU temperature, fans, power | the hardware sensor reader is not configured or not reading | run Setup's **Configure hardware sensors** option, then **Restart** on the LibreHardwareMonitor row ([above](#the-sensor-status)) |
| Game FPS | no overlay tool is measuring a game | see [Game FPS](fps.md) |
| Ping | no answer from the ping target yet | check the network or change the target (below) |
| Output volume, mute and audio device | no default output device is available | connect or select an output in Windows sound settings |
| Battery and remaining time | no battery, or Windows has no estimate | desktop PCs normally have no battery; an estimate may be absent while charging |

A sensor that is simply not there (no second GPU) is unavailable too.

The **System** category groups system information, audio and power. It includes
uptime, the full host name, Windows version, running process count, default output
volume, mute state and device name, AC power, battery percentage, charging state
and estimated remaining seconds. Audio follows changes to the Windows default
output device. State sensors use 0/1 values for widgets; the sensor list shows
localized Yes/No labels. Hover a value for its full text or unavailability reason.

## Ping

`net.ping` measures the round trip to `8.8.8.8` by default, with a TCP
connection to port 53 or 443 (Windows does not let a program send an ICMP echo).
Bezel measures once a second, and only while something you see uses the ping:

- a theme with a `net.ping` element, shown by Light, `bezel run` or Studio (live
  on the screen, or in the editor);
- Studio's sensor list while its panel is open and ping is in the selected
  search/category/probe scope. A status filter keeps checking that scope to
  detect failures and recovery;
- `bezel sensors`, which lists every sensor.

Otherwise nothing is sent to the ping target, and `net.ping` reads unavailable
(*"measured only while shown"*).

Change the target in **Preferences → Sensors** in Studio, or with `--ping-host`
on the command line (`bezel run --ping-host 1.1.1.1 ...`).
