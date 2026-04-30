# Changelog

All notable changes to Postik will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
