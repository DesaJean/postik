#!/usr/bin/env bash
# Generate Tauri app icons from docs/screenshots/logo.svg.
#
# Requires one of:
#   - rsvg-convert  (brew install librsvg)   — recommended
#   - ImageMagick   (brew install imagemagick)
#
# After running this script, run `npm run tauri icon docs/screenshots/icon-512.png`
# to let Tauri generate every platform-specific format from the 512×512 PNG.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SVG="$ROOT/docs/screenshots/logo.svg"
OUT_DIR="$ROOT/docs/screenshots"
ICON_512="$OUT_DIR/icon-512.png"

if [ ! -f "$SVG" ]; then
  echo "error: logo SVG not found at $SVG" >&2
  exit 1
fi

if command -v rsvg-convert >/dev/null 2>&1; then
  rsvg-convert -w 512 -h 512 -o "$ICON_512" "$SVG"
elif command -v magick >/dev/null 2>&1; then
  magick -density 512 -background none "$SVG" -resize 512x512 "$ICON_512"
elif command -v convert >/dev/null 2>&1; then
  convert -density 512 -background none "$SVG" -resize 512x512 "$ICON_512"
else
  echo "error: install rsvg-convert or ImageMagick first." >&2
  echo "  macOS: brew install librsvg   (or: brew install imagemagick)" >&2
  echo "  linux: apt install librsvg2-bin" >&2
  exit 1
fi

echo "generated $ICON_512"
echo
echo "Next: run"
echo "  npm run tauri icon $ICON_512"
echo "to populate src-tauri/icons/."
