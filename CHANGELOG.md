# Changelog

All notable changes to Postik will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-05-01

### Added — organisation

- **Archive (soft delete)** (B3). The trash icon now archives instead of
  permanently deleting. Archived notes live in a "View archived" section
  reachable from a footer link in the Notes pane. From there you can
  Restore or Delete forever (with confirmation).
- **Auto-start next pomodoro phase** (C5). New Settings toggle
  "Auto-start next pomodoro phase" (Settings → Playback). When off, the
  timer pauses at each phase boundary so you can acknowledge before the
  next phase begins; defaults on (preserves existing behaviour).
- **Snap zones** (E3). `⌘`/`Ctrl` + `Shift` + `←/→/↑/↓` snaps the
  current note window to half the screen along that edge.
- **Tags / labels per note** (B1). New "Tags" section at the bottom of
  the appearance popover. Type a tag and press Enter (or comma) to add;
  click × on a chip to remove. Tags are lowercase and deduplicated.
  The Notes tab shows a chip row of all distinct tags above the search;
  clicking a tag filters the list.

## [0.3.0] - 2026-05-01

### Added — content layer

- **Click-to-open links inline** (A2). `⌘`/`Ctrl` + click any URL inside
  a note's textarea opens it in the default browser. `www.foo.com` is
  auto-prefixed with `https://`. Backend reuses the existing
  `launcher::open_url` cross-platform helper.
- **Inline checklist toggle** (A3). Click on the `[ ]` or `[x]` of any
  `- [ ]` / `- [x]` line — including indented and `*`-bulleted variants
  — to flip it. The cursor stays where it was. Persisted immediately, no
  debounce wait.
- **Markdown preview toggle** (A1). New 📄 / ✏️ button in the title bar
  swaps the textarea for a rendered markdown view. Headings, lists,
  code, blockquotes, GFM checkboxes, and auto-links all render. Links
  open via `tauri.openUrl`; checkboxes are clickable and write back to
  the source. Output is sanitised through DOMPurify.

### Notes

- The Daily / Meeting / Todo templates from v0.2.0 now light up: their
  `- [ ]` lines toggle inline, and Preview mode shows them as proper
  checkboxes.
- 8 new unit tests (`textarea-actions.test.ts`) cover URL detection,
  scheme normalisation, and checklist toggle across indentation +
  bullet variants.

## [0.2.1] - 2026-05-01

### Added

- **Auto-update via the Tauri updater plugin.** Settings → Updates → "Check
  now" pings GitHub releases for a newer version. If signed verification
  succeeds, the bundle downloads in the background and a "Restart" button
  appears. The verification public key is baked into `tauri.conf.json`;
  every release is signed in CI from the matching private key (stored as
  GitHub secrets `TAURI_SIGNING_PRIVATE_KEY` /
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`).
- The release workflow now produces a `latest.json` manifest at the root
  of each release (the updater endpoint Postik queries).

### Notes

- v0.2.1 is the cut-over: existing users still need to manually install
  this version. Every subsequent tag from now on is auto-deliverable.

## [0.2.0] - 2026-05-01

### Added

- **Search across notes** — search bar at the top of the controller's
  Notes tab. Filters as you type. `⌘K` / `Ctrl+K` from anywhere focuses
  the search input.
- **Snooze on timer fire** — `+5m`, `+15m`, `+1h` chips in the Done
  state. Cancels the just-fired timer and starts a fresh countdown for
  the chosen offset.
- **Note templates** — three quick chips below "+ New note":
  - **Daily** (today's date + opening checklist, blue color)
  - **Meeting** (header + Attendees/Notes scaffold, purple)
  - **Todo** (three empty `- [ ]` lines, amber)
    Templates render as plain text today; once inline checklists ship in
    v0.3 they'll come alive automatically.
- **Drag-to-reorder** — drag any row in the controller's Notes list to
  move it. New `sort_index` column stores the manual order; rows that
  were never reordered keep falling back to `updated_at DESC`. Disabled
  while a search filter is active.
- **Focus mode** — eye icon in the title bar hides every other note
  window, so the active one is the only thing visible. Click again to
  show all.
- **Opacity presets + hover-to-100%** — `Ghost / Normal / Opaque` chips
  in the appearance popover. Hovering a faded note temporarily bumps it
  to 100% so you can read it without permanently undoing the fade.

## [0.1.10] - 2026-05-01

### Fixed

- Calendar event timer alarm could not be silenced from the Calendar
  tab's bell toggle: the chime in the open note window kept looping
  because nothing told the frontend that the backend had cancelled the
  timer. `TimerEngine::cancel` now emits a `timer:cancelled` event and
  the note window listens to it, dropping its local timer state and
  stopping the chime. The Dismiss button inside the note window also
  benefits — any code path that cancels a timer (Dismiss, calendar
  bell, auto-sync prune) now propagates to every open window.

## [0.1.9] - 2026-05-01

### Fixed

- Release workflow now passes `POSTIK_GOOGLE_CLIENT_ID` and
  `POSTIK_GOOGLE_CLIENT_SECRET` from repository secrets through to
  `tauri-action`, so published binaries actually have Google Calendar
  credentials baked in. v0.1.8 was published before this wiring, so its
  binaries showed "Setup needed" in the Calendar tab; v0.1.9 is the
  first build with credentials in CI.

## [0.1.8] - 2026-05-01

### Added

- **Google Calendar integration.** New "Calendar" tab in the controller
  syncs your primary calendar's events into Postik as read-only notes.
  Each event has a configurable alarm offset (at start / 5m / 10m / 15m
  before) and a per-event timer toggle; the existing notification + chime
  fires at the offset time. Default sync range is today; "7 days" chip
  switches it. Auto-sync runs every 15 min when enabled (in the Calendar
  tab). Disconnecting clears the tokens and all event-backed notes;
  regular notes are untouched.
- Read-only mode for note windows: when a note is event-backed, the
  textarea becomes read-only.
- One-time Google Cloud setup is documented in
  `docs/google-calendar-setup.md`. Until the env vars
  `POSTIK_GOOGLE_CLIENT_ID` and `POSTIK_GOOGLE_CLIENT_SECRET` are
  configured at build time, the Calendar tab shows a "Setup needed"
  state — the rest of the app is unaffected.

### Implementation notes

- Auth uses standard OAuth 2.0 with PKCE and a one-shot loopback
  redirect listener. No browser embed inside Tauri.
- Tokens persist in the existing SQLite DB under `app_data_dir/postik.db`.
- Per-event countdowns reuse the existing `TimerEngine`; no new alarm
  system was introduced.

## [0.1.7] - 2026-05-01

### Fixed

- Controller window stayed behind pinned notes even when clicked. Pinned
  notes use `always_on_top` (the macOS floating window level), and the
  OS resolves levels before focus, so the controller could never come
  above them. Now the controller toggles its own `always_on_top` on
  focus/blur — clicking it raises it above any pinned note; clicking a
  note drops it back so the note can stay on top while in use.

## [0.1.6] - 2026-04-30

### Added

- Clock-time input in the Set-timer custom field. Type `14:30`, `@14:30`,
  `at 14:30`, `2:30pm`, or `9am` to fire the timer at that wall-clock
  time. If the time has already passed today, the timer rolls to the
  same time tomorrow. Internally still a countdown — pause/resume shifts
  the fire time as with any countdown.

## [0.1.5] - 2026-04-30

### Fixed

- Windows release build still failing in v0.1.4 because `read_bool_setting`
  was unused on Windows (its only callers are inside the non-Windows
  `cfg` branch), and the release `deny(warnings)` lint promotes dead-code
  to a hard error. Gate the function with `#[cfg(not(target_os = "windows"))]`
  too.

## [0.1.4] - 2026-04-30

### Added

- "When timer ends, open…" per-timer action in the Set-timer popover. Pick
  an app (.app on macOS, .exe on Windows), an optional URL/args, or both.
  Configurations include:
  - Countdown: launches the action when the timer reaches 0:00.
  - Pomodoro: launches the action after the configured cycle count
    completes (default 4 work sessions). Pomodoros without a configured
    action keep cycling indefinitely as before.
  - Stopwatch: launches the action when the user dismisses/cancels.
    Last-used app + URL + cycle count are remembered across sessions.

### Fixed

- Windows release build failing with `unused_mut` errors (the
  `deny(warnings)` lint promoted the warning to a hard error on the
  branch where the variable wasn't reassigned). Window builders are now
  shadowed via `#[cfg]` instead of mutated. v0.1.2 through v0.1.3 had
  this same issue; v0.1.4 is the first Windows-publishable build with
  the transparent/content-protected fix.

## [0.1.2] - 2026-04-30

### Fixed

- Windows: note windows rendered as a blank white box because the
  transparent-window + content-protection combo was unreliable on WebView2.
  Note and controller windows are now opaque on Windows (Win11 still rounds
  frameless windows at the OS level), and content protection is skipped
  there.
- Default `--note-fill`/`--note-border`/`--note-text` CSS vars to amber so
  the first paint has a valid color before Svelte's effect runs — avoids a
  no-background flash on slow renders.

## [0.1.1] - 2026-04-30

### Added

- Sound chime when a timer completes (synthesized two-note bell via Web Audio),
  loops every ~2s until the user dismisses.
- Distinct "Done" state in the timer bar with a Dismiss button — replaces the
  Pause/Cancel pair once a countdown reaches zero, so finished timers can be
  acknowledged and cleared.

### Fixed

- Windows: app launching and closing immediately when a global keyboard
  shortcut collided with a system-reserved combo. Registration failures are
  now logged instead of aborting startup.
- Windows installer now bundles the WebView2 bootstrapper, so the app starts
  on Windows 10 machines that don't ship with WebView2 preinstalled.

## [0.1.0] - 2026-XX-XX

### Added

- Multi-window architecture: each note is an independent OS window
- Per-note timers: countdown, stopwatch, and pomodoro modes
- Six color presets plus transparent mode with adjustable opacity
- Always-on-top toggle per window
- Global keyboard shortcuts (new note, hide all, start timer, toggle pin)
- Native system notifications when timers complete
- Local SQLite persistence
- macOS and Windows builds
