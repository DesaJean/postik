# Changelog

All notable changes to Postik will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
