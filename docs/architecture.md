# Architecture

Postik is a Tauri 2 application: a Rust backend hosting two Svelte 5 frontends —
one for the controller window, one templated per note window. This file
explains the load-bearing decisions; see `multi-window-explained.md` for a
deeper dive on the multi-window model specifically.

## High-level layout

```
┌────────────────────────────────────────────────────────────┐
│                     Rust backend (Tauri)                   │
│                                                            │
│   ┌──────────┐  ┌──────────────┐  ┌─────────┐  ┌────────┐  │
│   │ Storage  │  │ WindowManager│  │ Timer   │  │ Tray + │  │
│   │ (SQLite) │  │ (per-note    │  │ Engine  │  │ Global │  │
│   │          │  │  windows)    │  │ (thread)│  │ Hotkeys│  │
│   └──────────┘  └──────────────┘  └─────────┘  └────────┘  │
│        ↑               ↑               ↑          ↑        │
│        └───────── #[tauri::command] ───┴──────────┘        │
└────────────────────────────────────────────────────────────┘
                          ↕  IPC + events
┌────────────────────────────────────────────────────────────┐
│              Frontend (Vite multi-page Svelte 5)           │
│                                                            │
│   index.html  ──→ src/main.ts ──→ Controller.svelte   │
│   note.html?id=…   ──→ src/note.ts ──→ Note.svelte         │
└────────────────────────────────────────────────────────────┘
```

## Stack choices

| Layer       | Choice                       | Why                                                  |
| ----------- | ---------------------------- | ---------------------------------------------------- |
| Shell       | Tauri 2                      | Single binary, native multi-window, ~MB bundle       |
| Backend     | Rust + tokio (sync features) | Memory-safe, fast, no GC pauses while ticking timers |
| Persistence | rusqlite (bundled SQLite)    | Embedded, zero-config, no sidecar daemon             |
| Frontend    | Svelte 5 + Vite              | Smallest runtime; runes give clean reactive state    |
| TS strict   | yes                          | Catches IPC payload drift between Rust ↔ TS at build |

## Why two HTML entry points?

Each note runs in its own WebView. Tauri lets you point each window at a
different URL or HTML file; we use `index.html` for the main panel and
`note.html?id=<uuid>` for note windows. The note's UUID arrives via the URL
query string and the Svelte mount reads it before instantiating `Note.svelte`
(see `src/note.ts`).

The HTML files live at the project root rather than under `src/pages/` because
Vite's `rollupOptions.input` resolves entries relative to the project root by
default. Putting them under `src/pages/` would force a `root: 'src/pages'`
override that complicates the `/src/main.ts` and asset paths.

## Why per-window state?

A single shared store across all windows would force every window's reactive
graph to recompute on every change anywhere. Postik instead keeps each note's
state local to its own window (a normal `$state` declaration in `Note.svelte`)
and uses Tauri events (`timer:tick`, `timer:done`) only when the backend needs
to push something. The controller listens for the same events to render its
list-view summaries.

## Why a Rust-side timer engine instead of `setInterval` in the WebView?

A WebView that loses focus on macOS/Windows is throttled or paused. A note's
countdown can't depend on that. The `TimerEngine` runs on a dedicated thread,
ticks once a second, persists state to SQLite every ~10 seconds, and emits
`timer:tick` events. The frontend re-renders only when an event arrives; the
clock is the backend.

## Persistence model

The schema is intentionally simple — two tables (`notes`, `timers`). Position
and size are persisted on every move/resize via Tauri's `WindowEvent::Moved`
and `Resized` callbacks; `content` is debounced 400ms in the frontend before
hitting `update_note_content`. There is no migrations system in v0.1; the
schema is created idempotently on first launch.

The DB path comes from `app.path().app_data_dir()`:

- macOS: `~/Library/Application Support/com.postik.app/postik.db`
- Windows: `%APPDATA%\com.postik.app\postik.db`

## Window management

`WindowManager` owns all per-note OS windows. On startup it lists notes from
SQLite and reopens a window for each. Closing a window does _not_ delete the
note — the user must explicitly trash a note via the close button (with a
confirmation if there's content). The controller window's close button is
intercepted: it hides the window rather than quits the app, mirroring how
macOS menubar apps behave. Quit happens via the tray menu's "Quit" item.

## IPC surface

Commands are defined in `src-tauri/src/commands.rs` and called from the
frontend via the typed wrapper in `src/lib/utils/tauri.ts`. Events flow the
other direction:

- Backend emits `timer:tick`, `timer:done`, `shortcut:new-note`, etc.
- Frontend `listen()`s on these from both windows.

If you add a new command, update both `commands.rs` _and_ `tauri.ts` — the
TypeScript wrapper is the single point where IPC payload types are checked.

## Testing strategy

- **Frontend**: pure functions like `time-parser.ts` are covered by Vitest.
  Component-level tests are intentionally omitted in v0.1 (the cost of mocking
  Tauri events outweighs the value at this stage); v0.2 adds Playwright E2E.
- **Backend**: `storage.rs` has CRUD coverage with an in-memory SQLite. The
  timer engine isn't unit-tested (its 1Hz ticking thread isn't easy to make
  deterministic in unit tests); v0.2 introduces a virtual-clock test harness.
