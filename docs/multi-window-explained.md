# Multi-Window in Postik (Deep Dive)

This is the differentiator. Other sticky-note apps either render every note
inside a single window (Antinote, Notezilla's main view) or use the OS's
built-in API for sticky notes (macOS Stickies). Postik instead creates a
real OS window per note. This document explains how that works in Tauri 2.

## What "multi-window" means here

Each note in Postik is an OS window in every sense the operating system cares
about:

- It has its own entry in the window list / Mission Control / Alt-Tab.
- It can live on a different Space or virtual desktop from other notes.
- It can be moved across monitors with persisted coordinates per note.
- Its `always_on_top` is independent — pin one without pinning the rest.
- macOS's window-manager features (Stage Manager, Spaces, App Exposé) treat
  each note as a separate window of the same app.

This is different from a "child window" or "panel" inside the controller.
There is no parent window; the controller window can be closed without
affecting the notes.

## The Tauri 2 multi-window model

Tauri 2 splits "windows" into two concepts:

- **Window**: an OS window (titlebar, position, size, native chrome).
- **Webview**: a WebKit/WebView2 surface inside a window.

A `WebviewWindow` is a window with exactly one webview filling it — that's
what Postik uses. Each `WebviewWindow` is independent: separate JavaScript
context, separate event loop on the renderer side, and separate
`WindowEvent` stream on the backend side.

```rust
WebviewWindowBuilder::new(
    app,
    "note-<uuid>",                              // unique label
    WebviewUrl::App("note.html?id=<uuid>".into())   // entry HTML + query
)
    .inner_size(width, height)
    .position(x, y)
    .decorations(false)            // no native titlebar
    .transparent(true)             // for opacity
    .always_on_top(pinned)
    .build()?;
```

The Rust side holds the only authoritative reference to which windows exist.
Frontends never create windows themselves — they call the
`create_note` / `delete_note` / `focus_note` IPC commands.

## Frontend entry-point routing

Each Tauri window loads a URL. Vite's multi-page mode produces multiple HTML
files at build time; the URL each window opens decides which Svelte app
mounts:

| Window          | URL loaded            | Svelte component mounted |
| --------------- | --------------------- | ------------------------ |
| controller      | `index.html`          | `Controller.svelte`      |
| note (per note) | `note.html?id=<uuid>` | `Note.svelte`            |

`src/note.ts` reads the `id` query parameter and passes it as a prop to
`Note.svelte`. This is the only thing telling each note window _which note_
it represents.

## Why not iframes?

Iframes share the parent process and event loop. They can't go always-on-top
independently, can't appear on a separate Space, and can't be moved by the
OS — they're a `display: block` rectangle. Real windows are the only way to
get the multi-window UX Postik wants.

## Why not use Tauri's static `windows[]` config?

`tauri.conf.json` has a `windows` array for declaring windows at config time.
That's fine when you know up front how many windows you need, but Postik
creates one per note dynamically. The only window declared statically would
be the controller, and even that is registered programmatically so the
backend can attach a custom `CloseRequested` handler that hides instead of
quits.

We set `app.windows = []` in `tauri.conf.json` and create everything from
Rust during `setup()`.

## Cost: memory and CPU

Each WebView has a non-zero baseline footprint — the WebKit/WebView2 process
itself isn't free. Rough numbers on macOS:

- **Idle controller alone**: ~30–40 MB resident
- **Each additional note window**: ~5–10 MB resident
- **Six notes open**: ~15 MB _additional_ over the controller baseline

CPU at idle is essentially zero — WebViews don't paint when nothing changes,
and the backend timer thread sleeps between 1 Hz ticks. The baseline cost is
why we don't recommend opening hundreds of notes; the design point is
"5–20 notes alive at once".

## Window-state recovery on startup

`WindowManager::restore_persisted` runs once during `setup()`:

1. List all rows from `notes`.
2. For each row, call `open_window_for(record)`, which builds a
   `WebviewWindow` with the saved `(x, y, width, height, always_on_top)`.
3. The frontend boots, reads `?id=<uuid>` from `window.location.search`,
   calls `list_notes` to find its own row, and hydrates from there.

If the user changed monitor configuration since last launch and a note's
saved coordinates land off-screen, `clamp_to_monitor` repositions the window
inside the primary monitor's bounds before opening.

If two notes had identical positions persisted (rare, but possible after a
reset), `next_position` walks the existing windows and offsets each new
window by `+20px` in both axes until there's no exact overlap.

## Position/size persistence

`WindowManager::open_window_for` attaches an event handler:

```rust
win.on_window_event(move |event| match event {
    WindowEvent::Moved(p) => { storage.update_position(&id, p.x, p.y); }
    WindowEvent::Resized(s) => { storage.update_size(&id, s.width, s.height); }
    _ => {}
});
```

These fire on every drag/resize, so the DB stays current — there's no
dedicated "save layout" step.

## Hide vs close

Postik distinguishes between **hiding** a note window and **deleting** the
note:

- The × button on a note hides the window, not deletes the note. The note
  stays in SQLite and reappears in the controller list.
- "Hide all notes" (`⌘⇧H`) hides every note window without touching the DB —
  great for clearing the screen during a presentation.
- The only path to actual deletion is the controller list's per-note delete
  action (added in v0.2; for v0.1 the user can manually delete via the
  hidden controller list — implement before tagging if you want to ship).

This decoupling lets users cmd-W a note window away without losing its
content.
