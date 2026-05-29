#!/usr/bin/env bash
#
# Generate crates/openlogi-gui/icon/AppIcon.icns from the master SVG.
#
# Rasterizer: prefers rsvg-convert or resvg (crisp per-size, CI-friendly);
# otherwise falls back to macOS built-ins — one 1024 master via QuickLook
# (qlmanage) downscaled with sips — so it works out of the box with no extra
# installs. The resulting .icns is what `[package.metadata.bundle].icon`
# points at; cargo-bundle copies a provided .icns verbatim.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SVG="$ROOT/design/icon/openlogi.svg"
OUT_DIR="$ROOT/crates/openlogi-gui/icon"
OUT="$OUT_DIR/AppIcon.icns"

[ -f "$SVG" ] || { echo "error: missing $SVG" >&2; exit 1; }
mkdir -p "$OUT_DIR"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
iconset="$work/AppIcon.iconset"
mkdir -p "$iconset"

if command -v rsvg-convert >/dev/null 2>&1; then
  render() { rsvg-convert -w "$1" -h "$1" "$SVG" -o "$2"; }
elif command -v resvg >/dev/null 2>&1; then
  render() { resvg --width "$1" --height "$1" "$SVG" "$2" >/dev/null; }
else
  echo "note: no rsvg-convert/resvg — using qlmanage + sips (built-in)"
  qlmanage -t -s 1024 -o "$work" "$SVG" >/dev/null 2>&1 || true
  master="$work/$(basename "$SVG").png"
  [ -f "$master" ] || { echo "error: qlmanage could not render $SVG" >&2; exit 1; }
  render() { sips -z "$1" "$1" "$master" --out "$2" >/dev/null; }
fi

# The 10 sizes Apple's .iconset expects (1x + @2x for each base).
for s in 16 32 128 256 512; do
  render "$s"       "$iconset/icon_${s}x${s}.png"
  render "$((s * 2))" "$iconset/icon_${s}x${s}@2x.png"
done

iconutil -c icns "$iconset" -o "$OUT"
echo "wrote $OUT"
