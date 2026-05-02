# Postik

[![CI](https://github.com/DesaJean/postik/actions/workflows/ci.yml/badge.svg)](https://github.com/DesaJean/postik/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/built_with-Tauri%202-orange.svg)](https://v2.tauri.app/)
[![Svelte](https://img.shields.io/badge/built_with-Svelte%205-ff3e00.svg)](https://svelte.dev/)

Floating sticky notes with per-window timers, calendar integration, recurring
reminders, and a pinch of AI. Each note is a real OS window — pin them, move
them across spaces, run independent timers on each, and they'll stay exactly
where you left them.

![Postik demo](docs/screenshots/demo.gif)

<!-- ↑ recorded after install — see docs/screenshots/README-PLACEHOLDER.txt -->

## Why Postik?

- **Multi-window first.** Every note is its own OS window. Spread them across
  monitors; they stay where you put them. Other sticky-note apps render notes
  inside one big panel — Postik treats each note as a first-class OS citizen.
- **Per-note timers.** Run a 25-minute pomodoro in one note while a 90-second
  countdown ticks in another. The timer engine runs in the Rust backend, so it
  keeps ticking even when the WebView is suspended.
- **Always-on-top, per window.** Pin a single reference note over a meeting;
  let the rest sit normally.
- **Local-first.** Everything lives in a single SQLite file under your app data
  dir. No account, no telemetry, no backend. Point it at iCloud / Dropbox /
  Syncthing for cross-device sync.
- **Lightweight.** Bundle is in the single-digit MBs and idle RAM with 6 notes
  open is around 15 MB on macOS. No Electron, no V8 per window.
- **Open source, MIT.** Forkable, auditable.

## Features

### Notes & editing

- Multi-window: one OS window per note, position/size persisted
- Markdown preview toggle (live render with sanitized HTML, code blocks,
  inline images, clickable links)
- Inline checklists (`- [ ]` → tickable in preview mode)
- Image paste — screenshots become inline data-URL images
- Search across all notes (`⌘K`), tag filter, and stack filter
- Tags + per-note color, text color, opacity (20–100%), always-on-top
- Templates: blank · daily journal · meeting notes · todo
- Drag-to-reorder + archive (soft delete) with restore

### Timers

- Per-note countdown / pomodoro / stopwatch
- Custom syntax (`25m`, `1h30m`, `14:30`, `2:30pm`, `pomo`, `stopwatch`)
- Pomodoro statistics: today / week / 7-day mini-chart
- Recurring reminders (every weekday at 9am, etc.)
- Snooze on fire (`+5m / +15m / +1h`)
- Post-timer actions: open an app, hit a URL, fire a webhook
- Distraction blocker (soft): pomodoro work session intercepts URLs to
  configured hosts with a "Stay focused?" prompt

### Organization

- Tags with chip-row filter on the controller
- Stacks (note groups) — Work / Personal / etc., with color, filter chips,
  per-note picker
- Focus mode: hide all but one note for deep-work sessions

### Integrations

- **Google Calendar.** OAuth (PKCE), per-event countdowns, auto-sync every
  15 min, pre-event alarms.
- **Outlook Calendar** via Microsoft Graph (same flow).
- **Google Tasks** read-only sync into a managed checklist note.
- **postik:// URL handler** — `postik://new?content=…` creates pre-filled
  notes from external tools.
- **Webhook on timer fire** — POSTs `{ note_id, mode, fired_at }` to any URL.

### AI (bring-your-own-key)

- Per-note ✨ summarize via Claude Haiku
- Settings → AI "Organize" — Claude proposes tags + stack assignments;
  confirm-before-apply
- Smart timer duration suggestion in the timer popover

Set `ANTHROPIC_API_KEY` in Settings → AI. No background calls — every
request is manual.

### Persistence & ops

- Encrypted-zip backup export / import
- Custom DB path (drop the SQLite file into iCloud / Dropbox / Syncthing for
  cross-device sync — single-writer-at-a-time)
- Auto-update via the Tauri updater + signed bundles
- Customizable global shortcuts
- Sidebar mode (slim always-on-top dock)

## Install

### From a release

Download the latest `.dmg` (macOS, universal) or `.msi` (Windows) from
[Releases](https://github.com/DesaJean/postik/releases).

### From source

```sh
git clone https://github.com/DesaJean/postik.git
cd postik
npm install
npm run tauri dev      # development
npm run tauri build    # production binary in src-tauri/target/release/bundle/
```

Requires Node 22+, Rust 1.77+, and the platform toolchain Tauri requires
([Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)).

#### Optional environment variables (compile-time)

| Variable                      | Purpose                                              |
| ----------------------------- | ---------------------------------------------------- |
| `POSTIK_GOOGLE_CLIENT_ID`     | Enables Google Calendar / Tasks. Get from GCP.       |
| `POSTIK_GOOGLE_CLIENT_SECRET` | Same — used in the PKCE token exchange.              |
| `POSTIK_OUTLOOK_CLIENT_ID`    | Enables Outlook Calendar. Public client (no secret). |

When unset, the corresponding Calendar section falls back to a "Setup
needed" empty state. Postik still runs fine without them.

## Default keyboard shortcuts

All four are rebindable in Settings → Keyboard shortcuts.

| Action                               | Shortcut           |
| ------------------------------------ | ------------------ |
| New note                             | `⌘⇧N` / `Ctrl+⇧+N` |
| Hide / show all notes                | `⌘⇧H` / `Ctrl+⇧+H` |
| Start timer in focused note          | `⌘⇧T` / `Ctrl+⇧+T` |
| Toggle always-on-top in focused note | `⌘⇧P` / `Ctrl+⇧+P` |
| Search notes (controller)            | `⌘K` / `Ctrl+K`    |
| Snap focused note to edge            | `⌘⇧←` / `⌘⇧→`      |
| Close focused note                   | `⌘W` / `Ctrl+W`    |

## Timer syntax

In any note's timer field, type:

| Input       | Result                                 |
| ----------- | -------------------------------------- |
| `25m`       | 25-minute countdown                    |
| `1h30m`     | 1h 30m countdown                       |
| `90s`       | 90-second countdown                    |
| `14:30`     | Countdown to 14:30 today (or tomorrow) |
| `2:30pm`    | Same, 12-hour notation                 |
| `pomo`      | 25/5 pomodoro (auto-cycles)            |
| `stopwatch` | Counts up from 0                       |

## How does it compare?

| Feature                    | Postik | Antinote | macOS Stickies | Notezilla |
| -------------------------- | :----: | :------: | :------------: | :-------: |
| Real OS window per note    |   ✅   |    ❌    |       ❌       |    ❌     |
| Per-note timers + pomodoro |   ✅   |    ❌    |       ❌       |    ❌     |
| Calendar integration       |   ✅   |    ❌    |       ❌       |    ❌     |
| Recurring reminders        |   ✅   |    ❌    |       ❌       |    ❌     |
| Markdown + checklists      |   ✅   |    ✅    |       ❌       |    ✅     |
| Tags + stacks              |   ✅   |    ✅    |       ❌       |    ✅     |
| Always-on-top per window   |   ✅   |    ❌    |       ❌       |    ✅     |
| Local-first / no account   |   ✅   |    ✅    |       ✅       |    ✅     |
| Open source                |   ✅   |    ❌    |       —        |    ❌     |
| Cross-platform (Mac + Win) |   ✅   |    ❌    |       ❌       |    ✅     |

## Roadmap

The v0.x roadmap is fully shipped (markdown, search, snooze, auto-update,
tags, stacks, recurring, calendar, AI, …). What's next:

- **v1.0** — stabilization release, no new features beyond polish
- **v1.x** — real cloud sync (account-backed, conflict-resolved), mobile
  companion, plugin SDK, theming

See [CHANGELOG.md](CHANGELOG.md) for the per-release breakdown.

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
