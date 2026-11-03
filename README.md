## Changes in this fork (mrufekmk/Aperture, branch `SimpleTrofeoDisplay`) — 3 November 2026

Tested on a Ryzen 7 9800X3D, Radeon RX 9070 XT and ASUS TUF GAMING B650-PLUS. Every reading now comes straight from the hardware drivers; HWiNFO is no longer needed (it is still used as a fallback if it is running).

- **New `mruf` layout** (based on Duet): no music box, a larger weather panel with the forecast, memory shown as usage only (ring + Used / Free), GPU **Hotspot** in place of GPU clock with its own trend line, and the CPU model name (e.g. "AMD Ryzen 7 9800X3D") in the CPU header instead of cores/threads. Available in the settings app's layout gallery.
- **AMD GPU on Ryzen systems with integrated graphics:** the program now picks the discrete card (most video memory, reports temperatures) instead of the first AMD device, which could be the CPU's built-in graphics. GPU temperature and hotspot now work on the RX 9070 XT.
- **RX 9000 (RDNA4) support:** GPU temperature falls back to the core sensor when the edge sensor is missing, and GPU power is read through AMD ADLX (total board power), as these cards no longer report power through ADL.
- **Motherboard fans without HWiNFO:** CPU fan speed is read directly from Nuvoton NCT679x Super I/O chips (NCT6799D on this board) through PawnIO's signed LpcIO module, clearing the chip's I/O lock before each read as ASUS firmware re-locks it. `pro_fan_sensor = Fan #N` selects the header (default Fan #2 = CPU_FAN on ASUS).
- **Volume pop-up settings** (also in the settings app, under Music): `pro_volume_secs` sets how long it stays on screen (0 = off), `pro_volume_device` picks the playback device; the Windows default device is now followed when output is switched.
- **Diagnostics:** `aperture-display.exe --diag` (run as administrator) lists every AMD GPU and its sensors, ADLX power, PawnIO CPU temperature/power, all motherboard fan headers and the playback devices. `trofeo-status.txt` also reports the GPU temperature source.
- **Build:** `.github/workflows/build-simpletrofeodisplay.yml` builds `aperture-display.exe` and `aperture.exe` on GitHub (fetching the LpcIO module from the PawnIO.Modules release) and offers them as the **SimpleTrofeoDisplay-windows** download on each run.

---

<p align="center"><img src="docs/img/aperture/brand.png" alt="Aperture â€” a quieter view of your system" width="100%"></p>

<p align="center"><b>A native system display for the Thermalright Trofeo Vision 9.16â€³.</b><br>Large type. Complete readings. A little less noise.</p>

<p align="center"><a href="#the-collection">The collection</a> Â· <a href="#get-started">Get started</a> Â· <a href="docs/CONFIGURATION.md">Configuration</a> Â· <a href="docs/DEVELOPMENT.md">Build & contribute</a></p>

## The collection

Five new compositions built for the panel's unusually wide 1920 Ã— 462 canvas. A shared information model keeps CPU fan, GPU hotspot, VRAM and FPS in dedicated positions. Focus and Grid remain available alongside them.

| Layout | Composition |
| :-- | :-- |
| **Axis** | A shallow context bar above three equal hardware instruments. |
| **Ledger** | Three precise hardware rows with a separate context column. |
| **Duet** | CPU and GPU across two wide stages; memory has its own rail. |
| **Orbit** | Fine gauge arcs around the hardware numbers; a central clock and music column. |
| **Atelier** | Album art leads, with a complete hardware desk alongside it. |

![Axis](docs/img/aperture/axis.png)
![Ledger](docs/img/aperture/ledger.png)
![Duet](docs/img/aperture/duet.png)
![Orbit](docs/img/aperture/orbit.png)
![Atelier](docs/img/aperture/atelier.png)

*Images are rendered by the program using sample data, not measurements from a live PC.*

**Five quiet palettes:** Mineral, Frost, Blush, Sand and Iris. Neutral surfaces, readable secondary text and one coordinated accent. Temperature warnings keep their own semantic colors. Pick a custom accent if you prefer; dark accents are lifted for legibility. Existing layout and palette keys still work, with older landscape designs grouped separately in the settings app.

## What you can see

- **CPU:** temperature, load, power, clock, fan RPM, uptime and network throughput.
- **GPU:** temperature, hotspot, load, power, fan, clock, VRAM and FPS with frame time.
- **Memory & storage:** memory use, free memory, SSD temperature, disk capacity use and read/write throughput.
- **Context:** clock, date, current weather, a three-day forecast, album art, track title, artist and elapsed/total time.

Readings depend on the hardware and sensor source. Unavailable sensors show `--`; they are never replaced with invented values. The display does not replace VRAM with FPS when a game starts.

## Get started

### Windows

1. Download **`aperture-windows.zip`** from [Aperture releases](https://github.com/HuyBruh/Aperture/releases/latest). The source ZIP is for developers and contains no executable.
2. Extract the entire package into a permanent folder. Close Thermalright TRCC so it releases the USB display.
3. Run **`Aperture.bat`** to open settings. Run **`scripts/install.bat`** if you want sensor-driver setup and automatic startup. Windows may ask for elevation.
4. Select a layout and palette. Settings are saved in `trofeo.conf` and picked up by the display program.

The package contains two executables: `aperture.exe` for settings, and `aperture-display.exe` for the USB display. The display service is now named `ApertureDisplay`. Existing `trofeo.conf` settings remain supported; the installer replaces the previous scheduled task.

**Upgrading from HuyBruh Dash:** back up your current folder and keep your existing `trofeo.conf`. The new default configuration is only for fresh installs. See [migration](docs/MIGRATION.md) before replacing a running installation.

### Sensors

| Reading | Source |
| :-- | :-- |
| CPU temperature / power | PawnIO on supported AMD/Intel CPUs; external sensor fallback |
| CPU fan / additional temperatures | LibreHardwareMonitor or HWiNFO |
| GPU telemetry | GPU interfaces and supported external sensor sources |
| DirectX FPS | ETW; elevated permissions may be required |
| Weather | Open-Meteo; a configured city or IP-based location |
| Now playing | Windows media controls; `playerctl` on Linux |

Open **Sensors & help** to inspect status and select the CPU fan. Availability varies by machine. Weather needs an internet connection.

### Linux / build from source

See [development](docs/DEVELOPMENT.md) for prerequisites and commands. The new collection shares the existing software renderer and USB driver; this rework does not introduce a web runtime.

## Project structure

```text
src/dashboard/       Aperture collection renderer and generated geometry
design/              Canonical layout geometry and palette specifications
assets/brand/        Vector mark and application icons
windows/             Portable defaults, launchers and installation scripts
tools/               Design generation, checks, rendering and packaging
docs/                Configuration, migration, development and release evidence
```

Legacy renderers and upstream integrations remain in `src/` for compatibility. The new collection is isolated so a layout change does not require changes to sensor collection or the USB protocol.

## Performance & validation

The 3.x performance measurements were for older layouts on one PC at idle. They are [preserved as history](docs/history/v3.3-README.md), not presented as benchmarks of this rework. A fresh measurement is required before claiming a CPU or RAM budget for Aperture.

See [validation](docs/VALIDATION.md) for the exact checks performed and remaining hardware coverage. A successful preview render verifies the renderer, not every physical LCD or sensor configuration.

## Credits & license

By **Huy**, previously HuyBruh Dash. Built on [haven80/trofeo-lcd-rs](https://github.com/haven80/trofeo-lcd-rs), [sukualam/trofeo-lcd](https://github.com/sukualam/trofeo-lcd) and the protocol work in [Lexonight1/thermalright-trcc-linux](https://github.com/Lexonight1/thermalright-trcc-linux). AI-assisted implementation includes Claude and Codex. Full provenance is in [NOTICE.md](NOTICE.md).

Inter and Chakra Petch are distributed under the SIL Open Font License; their notices remain in `assets/`. **[GPL-3.0-or-later](LICENSE)**. Not affiliated with Thermalright.
