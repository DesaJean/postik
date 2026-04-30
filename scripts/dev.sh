#!/usr/bin/env bash
# Launch the Tauri dev loop with extra logging enabled.
set -euo pipefail
cd "$(dirname "$0")/.."
RUST_BACKTRACE=1 RUST_LOG=postik=debug,tauri=info npm run tauri dev "$@"
