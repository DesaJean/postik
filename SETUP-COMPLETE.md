# Postik v0.1.0 — Setup Complete

The project is scaffolded, all checks pass locally, and the initial commit
plus `v0.1.0` tag are in place. This file explains what's left for **you** to
do before publishing.

## What works (verified)

- ✅ `npm run check` — svelte-check + tsc, 0 errors / 0 warnings
- ✅ `npm test` — 13/13 Vitest tests passing (`time-parser`)
- ✅ `npm run lint` — ESLint + Prettier clean
- ✅ `cargo fmt --check` — clean
- ✅ `cargo clippy --all-targets -- -D warnings` — clean
- ✅ `cargo test --lib` — 4/4 storage tests passing

## What you must do manually before publishing

### 1. Generate the real app icons

The default Tauri placeholder icons in `src-tauri/icons/` work for development
but should be replaced with the Postik logo:

```sh
# Install one of:
brew install librsvg          # macOS - recommended (rsvg-convert)
# or:
brew install imagemagick

# Then:
./scripts/generate-icons.sh                       # produces docs/screenshots/icon-512.png
npm run tauri icon docs/screenshots/icon-512.png  # populates src-tauri/icons/
git add src-tauri/icons docs/screenshots/icon-512.png
git commit --amend --no-edit                      # (if amending the v0.1.0 commit)
```

If you've already pushed/tagged, do this on a follow-up commit instead and
re-tag (or roll forward to v0.1.1).

### 2. Run the app interactively

```sh
npm run tauri dev
```

Verify by hand (I couldn't test the UI here):

- [ ] Click "+ New note" — a new floating window appears
- [ ] Type into the note — content persists across restarts
- [ ] Pin (📌), open timer (⏱), and close (×) buttons work
- [ ] Type `25m` in the timer field, press Set — countdown starts
- [ ] Type `pomo` — pomodoro mode auto-cycles between work/break
- [ ] Type `stopwatch` — counts up
- [ ] Color picker swaps the note's color
- [ ] Opacity slider fades the fill (text stays opaque)
- [ ] `⌘⇧N` creates a new note from anywhere
- [ ] `⌘⇧H` hides/shows all notes
- [ ] `⌘⇧P` toggles pin on the focused note
- [ ] `⌘W` closes (hides) the focused note window
- [ ] Tray icon → menu items work; left-click toggles the controller
- [ ] Quit and relaunch — every note reappears with its position, size, color

### 3. Test the production build

```sh
npm run tauri build
```

This produces a `.dmg` (macOS) or `.msi` (Windows) under
`src-tauri/target/release/bundle/`. First-time builds may take 5-10 minutes.

### 4. Record the demo GIF

Once the UI looks right, record a 4-5 second demo:

- Tool: **LICEcap** (macOS) or **ScreenToGif** (Windows)
- Recommended: 800×500, 12 fps
- Save as `docs/screenshots/demo.gif`
- Optional: take static screenshots (`controller.png`, `timer-running.png`,
  `colors.png`) for your GitHub social card and any blog post

### 5. Create the GitHub repo and push

Replace `<USERNAME>` everywhere:

- `README.md` (badges, install URL, social card)
- `CONTRIBUTING.md` (clone URL)
- `.github/FUNDING.yml` (sponsor handle)
- `src-tauri/Cargo.toml` (`repository = …`)

```sh
# Quick replace (verify first):
grep -rln '<USERNAME>' . --exclude-dir=node_modules --exclude-dir=src-tauri/target
# Then either edit by hand or:
find . -type f \( -name '*.md' -o -name '*.toml' -o -name '*.yml' \) \
  -not -path './node_modules/*' -not -path './src-tauri/target/*' \
  -exec sed -i '' 's/<USERNAME>/your-handle/g' {} +

# New commit after the v0.1.0 tag, or amend & re-tag — your call.
git add -A
git commit -m "chore: replace <USERNAME> placeholder with $YOUR_HANDLE"

# Create the repo on GitHub (via gh CLI or web), then:
git remote add origin git@github.com:<USERNAME>/postik.git
git push -u origin main
git push origin v0.1.0
```

### 6. (Optional) Configure GitHub Sponsors

- Apply at <https://github.com/sponsors>
- Once approved, your `.github/FUNDING.yml` `github:` line activates the
  sponsor button at the top of every repo page

## Decisions I made when the spec was ambiguous

These are spots where the prompt didn't fully specify behavior and I picked a
reasonable default. Worth a quick read so you can adjust if you disagree.

| Decision | What I chose | Why |
| --- | --- | --- |
| HTML entry-point location | Project root (`controller.html`, `note.html`) | Vite's `rollupOptions.input` resolves relative to project root by default; putting them under `src/pages/` would have forced a `root: 'src/pages'` override. Documented in `docs/architecture.md`. |
| `×` button on a note | Hides the window after a confirmation dialog if there's content | The spec said "delete only via explicit ×, with confirmation" but also "closing should not delete from DB." I read this as: hide on close; explicit delete becomes a controller-list action (planned for v0.2). The note is never lost from SQLite. |
| Settings modal | Empty modal with "Coming in v0.2" copy | Spec said exactly this. |
| Pomodoro phase length | 25 min work / 5 min break, hardcoded | Standard Pomodoro values; configurable in v0.2. |
| Notification sound | Disabled (no sound) | Spec said "default off, config in v0.2." |
| Timer persistence cadence | Every 10 seconds while running | Compromise between durability and SQLite write volume. Pause/resume/cancel persist immediately. |
| Tray left-click | Toggle controller visibility | Standard menubar-app convention. |
| First-launch behavior | Controller hidden until tray click or `⌘⇧N` | Spec said the controller starts hidden. Tray icon is the entry point. |
| `tauri-plugin-dialog` | Added (wasn't in the spec's plugin list) | Needed for the close-with-content confirmation. |
| `parking_lot::Mutex` | Used instead of `std::sync::Mutex` | Faster and never poisons; standard in the Rust ecosystem. |
| Code of Conduct text | Brief intro that links to the canonical Contributor Covenant 2.1 | The full text is long; linking keeps the file maintainable when the Covenant updates. |

## Parts that may need a closer look

These are areas where the code compiles and tests pass but where you should
manually verify behavior in dev:

- **Global shortcuts on macOS** — system-wide hotkeys require accessibility
  permission on first launch. The OS will prompt automatically; the user
  needs to grant it before `⌘⇧N` etc. work.
- **Window transparency** — `macOSPrivateApi: true` is enabled, but if you
  see fully opaque windows, check that the bundle is properly signed in
  release builds (unsigned builds may have stricter sandboxing).
- **Always-on-top across Spaces** — On macOS, `always_on_top` keeps a
  window above its own Space's other windows but doesn't cross Spaces by
  default. If you want note windows on every Space, you'd need to set the
  `NSWindow.collectionBehavior` to `.canJoinAllSpaces` — that's a Tauri 2
  feature gap as of 2.10 and would need a `tauri-plugin-positioner`-like
  workaround.
- **Tray icon** — uses `app.default_window_icon()`. Replace once you've run
  `npm run tauri icon …` so the tray shows the Postik P.
- **Settings modal a11y** — basic only; full keyboard navigation is on the
  v0.2 list.
- **Timer engine determinism** — uses `std::thread::spawn` + 1Hz sleep. Drift
  on a wedged thread is bounded but not zero. If you see a timer falling
  behind a wall clock, that's where to look.

## Suggested next features (v0.2 backlog)

- Settings modal that actually does something: sound toggle, custom shortcuts,
  default new-note color.
- Per-note delete from the controller list (right-click → Delete, or trash
  icon on hover).
- Global Pomodoro presets (different work/break durations).
- Search across notes by content.
- Light/dark themes for note bodies (right now only the controller respects
  `prefers-color-scheme`).
- Timer history view per note.
- Playwright E2E tests covering the multi-window happy path.

## Useful commands

```sh
# Frontend
npm run dev               # Vite alone (no Tauri)
npm run tauri dev         # full app, hot reload
npm test                  # vitest run
npm run check             # svelte-check + tsc
npm run lint              # eslint + prettier
npm run format            # prettier --write

# Backend
cd src-tauri
cargo check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo fmt --all -- --check
```

## File map

```
postik/
├── controller.html / note.html      # Vite multi-page entry HTMLs
├── src/                             # Svelte 5 + TypeScript frontend
│   ├── main.ts / note.ts            # mount points for the two windows
│   ├── lib/components/              # Controller, Note, Timer, TitleBar, ColorPicker, OpacitySlider
│   ├── lib/stores/notes.svelte.ts   # global notes store
│   ├── lib/utils/                   # tauri (typed invoke), time-parser, colors
│   └── styles/global.css
├── src-tauri/src/                   # Rust backend
│   ├── lib.rs                       # tauri::Builder, tray, plugin wiring
│   ├── main.rs                      # entry point
│   ├── commands.rs                  # all #[tauri::command] handlers
│   ├── window_manager.rs            # multi-window lifecycle
│   ├── timer.rs                     # background tick engine
│   ├── shortcuts.rs                 # global hotkey registration
│   └── storage.rs                   # SQLite wrapper + tests
├── docs/
│   ├── architecture.md
│   ├── multi-window-explained.md
│   └── screenshots/                 # logo.svg, logo-dark.svg, placeholder
├── scripts/                         # dev.sh, generate-icons.sh
└── .github/                         # CI, release, issue templates, FUNDING
```

That's it. Have fun shipping Postik. 🍞
