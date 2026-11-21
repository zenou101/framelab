# FrameLab

**A local-first Windows performance monitor and benchmark recorder for PC games.**

FrameLab shows what your machine is doing while a game runs, records real frame timings, and stores
the results on your own computer. It has no accounts, no cloud component and no telemetry.

> **Status: early development preview (0.1.0).** FrameLab is usable but incomplete. Features, user
> interface and stored data formats may change without notice, and no compatibility or stability
> promise is made for this preview. See [Limitations](#limitations) before relying on any measurement.

---

## What it does today

- **Live system monitoring** — CPU utilisation and clock, per-core load, memory, GPU utilisation and
  VRAM use, disk and network throughput where Windows exposes the counters, plus your display mode
  and refresh rate.
- **Frame capture** — attaches to a game that is already running and reads frame timings through
  Intel's PresentMon. *Displayed* FPS (frames that reached the screen) and *Presented* FPS (frames the
  game submitted) are captured and reported separately.
- **Benchmark recording** — saves frame timings and system samples for a session to a local SQLite
  database, in durable chunks, so an interrupted session keeps what was already written.
- **Results** — average, median, 1% low and 0.1% low FPS, frame-time percentiles, stutter counts and
  frame-time charts, with a clear note of which measurement each figure comes from.
- **History** — every session stays on your machine until you delete it, and can be exported as CSV
  or JSON.
- **Analysis** — configurable, clearly-labelled heuristic observations (pacing instability, spike
  clusters, memory or VRAM pressure and similar) that describe what was measured and never claim a
  definitive cause.
- **Games library** — reads Steam and Epic installations from their local manifests, or you can add
  an executable yourself. Launching only ever runs an executable already in your library.

Anything a machine does not expose is shown as **unavailable with the reason**, never as a zero.

## System requirements

| Requirement | Detail |
| --- | --- |
| Operating system | Windows 10 (22H2 or later) or Windows 11, 64-bit. ARM64 is not supported. |
| Runtime | Microsoft Edge WebView2 Evergreen Runtime (present on current Windows 10/11; the installer can bootstrap it). |
| Memory | 8 GB RAM minimum; 16 GB recommended for modern games. |
| GPU | A DirectX 12 capable GPU with current drivers. |
| Disk | About 250 MB for the application, plus roughly 1–2 MB per minute of recorded session. |
| Permissions | Frame capture uses Windows ETW. Running unelevated usually works; some systems require administrator rights or membership of the local **Performance Log Users** group. |

FrameLab has been exercised on Windows 10 22H2 with an AMD GPU. It has not been tested on every
hardware, driver or anti-cheat configuration; see [Limitations](#limitations).

## Installation

Download the current build from [Releases](https://github.com/zenou101/framelab/releases):

- **Installer** (`FrameLab_0.1.0_x64-setup.exe`) — installs per user; administrator rights are not
  required. Recordings and settings go to `%APPDATA%\app.framelab.desktop`.
- **Portable** (`FrameLab_0.1.0_x64-portable.zip`) — extract anywhere and run `framelab.exe`.
  Settings and recordings are kept in a `data` folder beside the executable, so the whole folder can
  be moved or deleted as one unit.

Builds are unsigned, so Windows SmartScreen may warn about an unknown publisher.

To build from source you need:

- Windows 10/11 x64
- [Rust](https://rustup.rs) with the MSVC toolchain (`stable-x86_64-pc-windows-msvc`)
- Visual Studio Build Tools with the **Desktop development with C++** workload
- Node.js 20 or newer with npm

```bash
git clone https://github.com/zenou101/framelab.git
cd framelab
npm install
npm run tauri build
```

The installer is written to `src-tauri/target/release/bundle/nsis/`. The application is installed per
user by default; administrator rights are not required to install it.

For development:

```bash
npm install
npm run tauri dev
```

PresentMon is bundled as a resource and is verified against a pinned SHA-256 hash before FrameLab
runs it. It is not downloaded at runtime.

## Privacy

FrameLab is designed to keep everything local:

- no account, sign-in or cloud sync;
- no telemetry, analytics or crash reporting;
- no upload of measurements, hardware details or session data;
- no network access required for monitoring, capture, recording or export;
- session data, settings and exports are stored only in your own user profile.

The only outbound request FrameLab can make is the optional WebView2 runtime download performed by
the Windows installer when the runtime is missing. Details, including where data is stored and how to
delete it, are in [PRIVACY.md](PRIVACY.md).

## Limitations

- **Measurements are estimates derived from operating-system counters.** They can be inaccurate,
  delayed or unavailable, particularly on virtual machines, remote desktops, laptops with hybrid
  graphics, or systems with Hardware-Accelerated GPU Scheduling enabled.
- **Some counters are not exposed by every Windows installation.** Disk and network throughput in
  particular depend on Windows performance counter providers that may be disabled or absent; those
  values are reported as unavailable rather than guessed.
- **Temperatures are not read** unless a supported sensor source is present. FrameLab does not
  present ACPI thermal-zone readings as CPU or GPU temperature.
- **Frame capture requires ETW access** and may fail without elevation on some systems, and cannot
  attach to a process that has already exited.
- **Exclusive fullscreen** presentation modes can hide or restrict monitoring overlays; FrameLab
  currently has no in-game overlay.
- **Anti-cheat and protected processes** may block or penalise external monitoring. FrameLab does not
  bypass, disable or interfere with anti-cheat software, and you are responsible for using it in line
  with each game's terms.
- **Not all games, engines or launchers are supported.** PresentMon only sees frames from processes
  that present through supported Windows presentation APIs.
- Session data is only as reliable as the machine's clock and counter behaviour while recording.

## Responsible use

Use FrameLab only on systems and software you are entitled to monitor. Do not use it to interfere
with, circumvent or gain advantage in multiplayer games, and comply with the terms of service of any
game, launcher or platform you use it with.

## Licence

FrameLab is released under the [MIT License](LICENSE). Copyright (c) 2026 FrameLab.

Bundled and third-party components remain under their own licences, including Intel PresentMon (MIT)
and the Inter typeface (SIL Open Font License 1.1). See
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for the complete list and required notices.

## Disclaimer

FrameLab is provided **"as is"**, without warranties of any kind, and is used at your own risk.
Measurements are informational and must not be treated as professional, diagnostic or safety advice.
Nothing in the project's documentation excludes or limits rights that cannot lawfully be excluded,
including guarantees under the Australian Consumer Law where applicable.

Read [DISCLAIMER.md](DISCLAIMER.md) for the full statement.

## Reporting problems and security issues

- Bugs and feature requests: open an issue at https://github.com/zenou101/framelab/issues.
- Security vulnerabilities: please report them privately to nuxxzul@gmail.com rather than in a public
  issue.

These documents are provided for transparency and do not constitute legal advice.
