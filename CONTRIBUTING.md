# Contributing to Postik

Thanks for your interest! Postik is in early development; the codebase is small and
welcoming to contributions.

## Development setup

```sh
git clone https://github.com/jeandesa/postik.git
cd postik
npm install
npm run tauri dev
```

Tauri prerequisites (Rust toolchain, platform SDK): see
<https://v2.tauri.app/start/prerequisites/>.

## Running the test suite

```sh
npm test           # frontend unit tests (Vitest)
npm run check      # svelte-check + tsc, no warnings allowed
npm run lint       # ESLint + Prettier check
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
```

CI runs all of the above on every PR.

## Style

- TypeScript: strict mode. Prefer named exports.
- Rust: `cargo fmt` + `clippy` clean. Keep modules focused.
- Comments: write them only when the _why_ isn't obvious from the code.
- Frontend identifiers: `camelCase` for TS, `kebab-case` for filenames of UI components.

The project's `eslint.config.js`, `.prettierrc.json`, and `Cargo.toml` are the
authoritative configs — defer to them.

## Commit convention

Use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat: add color picker keyboard navigation`
- `fix: timer pause not persisting`
- `docs: explain multi-window startup flow`
- `chore: bump tauri to 2.x.y`
- `refactor: extract timer state machine`
- `test: cover pomodoro phase transition`

## Branches

- `feature/<short-name>` for new features
- `fix/<short-name>` for bug fixes
- `docs/<short-name>` for docs-only changes

## Pull requests

Each PR should include:

- A short description of _why_ the change is needed
- Screenshots for any UI change
- A checklist confirming:
  - [ ] `npm run lint` passed
  - [ ] `npm test` passed
  - [ ] `cargo clippy --all-targets -- -D warnings` passed
  - [ ] `cargo test` passed
  - [ ] Docs updated when behavior changed

## Reporting bugs

Open an issue using the **bug report** template under `.github/ISSUE_TEMPLATE/`.

## Code of conduct

This project follows the [Contributor Covenant 2.1](CODE_OF_CONDUCT.md).
