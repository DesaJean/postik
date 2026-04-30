# Postik

> 🚧 Postik is in early development. v0.1.0 is the first public release.
> Star the repo to follow progress.

[![CI](https://github.com/jeandesa/postik/actions/workflows/ci.yml/badge.svg)](https://github.com/jeandesa/postik/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/built_with-Tauri%202-orange.svg)](https://v2.tauri.app/)
[![Svelte](https://img.shields.io/badge/built_with-Svelte%205-ff3e00.svg)](https://svelte.dev/)

Floating sticky notes with per-window timers, colors, and always-on-top control.
Each note is a real OS window — pin them, move them across spaces, run independent
timers on each, and they'll stay exactly where you left them.

![Postik demo](docs/screenshots/demo.gif)

<!-- ↑ recorded after install — see docs/screenshots/README-PLACEHOLDER.txt -->

## Why Postik?

- **Multi-window first.** Every note is its own window. Spread them across monitors;
  they stay where you put them. Other sticky-note apps render notes inside one big
  panel — Postik treats each note as a first-class OS citizen.
- **Per-note timers.** Run a 25-minute pomodoro in one note while a 90-second
  countdown ticks in another. The timer engine runs in the Rust backend, so it
  keeps ticking even when the WebView is suspended.
- **Always-on-top, per window.** Pin a single reference note over a meeting; let
  the rest sit normally.
- **Lightweight.** Bundle is in the single-digit MBs and idle RAM with 6 notes
  open is around 15 MB on macOS. No Electron, no V8 per window.
- **Open source, MIT.** Forkable, auditable, no telemetry.

## Features (v0.1.0)

- ✅ Multi-window: one OS window per note, with persisted position/size
- ✅ Per-note timers — countdown, stopwatch, and pomodoro modes
- ✅ Six color presets + transparent mode with adjustable opacity (20–100%)
- ✅ Always-on-top toggle per window
- ✅ Global keyboard shortcuts
- ✅ Native system notifications when timers complete
- ✅ Local SQLite persistence — your notes never leave your machine
- ✅ macOS and Windows builds

## Install

### From a release

Download the latest `.dmg` (macOS) or `.msi` (Windows) from
[Releases](https://github.com/jeandesa/postik/releases).

### From source

```sh
git clone https://github.com/jeandesa/postik.git
cd postik
npm install
npm run tauri dev      # development
npm run tauri build    # production binary in src-tauri/target/release/bundle/
```

Requires Node 20+, Rust 1.77+, and the platform toolchain Tauri requires
([Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)).

## Keyboard shortcuts

| Action                               | Shortcut           |
| ------------------------------------ | ------------------ |
| New note                             | `⌘⇧N` / `Ctrl+⇧+N` |
| Hide / show all notes                | `⌘⇧H` / `Ctrl+⇧+H` |
| Start timer in focused note          | `⌘⇧T` / `Ctrl+⇧+T` |
| Toggle always-on-top in focused note | `⌘⇧P` / `Ctrl+⇧+P` |
| Close focused note                   | `⌘W` / `Ctrl+W`    |

## Timer syntax

In any note's timer field, type:

| Input       | Result                      |
| ----------- | --------------------------- |
| `25m`       | 25-minute countdown         |
| `1h30m`     | 1h 30m countdown            |
| `90s`       | 90-second countdown         |
| `pomo`      | 25/5 pomodoro (auto-cycles) |
| `stopwatch` | Counts up from 0            |

## How does it compare?

| Feature                  | Postik | Antinote | macOS Stickies | Notezilla |
| ------------------------ | :----: | :------: | :------------: | :-------: |
| Real OS window per note  |   ✅   |    ❌    |       ❌       |    ❌     |
| Per-note timers          |   ✅   |    ❌    |       ❌       |    ❌     |
| Pomodoro mode            |   ✅   |    ❌    |       ❌       |    ❌     |
| Always-on-top per window |   ✅   |    ❌    |       ❌       |    ✅     |
| Adjustable opacity       |   ✅   |    ❌    |       ❌       |    ✅     |
| Open source              |   ✅   |    ❌    |       —        |    ❌     |
| Cross-platform (Mac+Win) |   ✅   |    ❌    |       ❌       |    ✅     |

## Roadmap

- v0.2 — Settings panel, sound toggle, custom shortcuts, timer history
- v0.3 — Categories/tags, search across notes
- v0.4 — Markdown rendering toggle
- v1.0 — i18n, theming, plugin SDK
- Future — Optional sync (E2E encrypted), mobile companion

## Tech stack

- **[Tauri 2](https://v2.tauri.app/)** — desktop shell, IPC, multi-window
- **[Svelte 5](https://svelte.dev/)** — UI with runes
- **TypeScript** strict mode + **Rust** edition 2021
- **[rusqlite](https://github.com/rusqlite/rusqlite)** (bundled SQLite)
- **Vite** as the bundler

## Contributing

PRs welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE) © 2026 Postik Contributors
