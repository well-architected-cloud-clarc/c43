#!/usr/bin/env bash
# Render generated drawio files to PNG and SVG with the drawio CLI (dev aid only).
# Usage: scripts/render.sh [file.drawio | dir ...]   (default: the golden cases under fixtures/layout-cases)
# The goldens are rendered as they are; FULL_REBUILD=1 rebuilds them first (cargo build, UPDATE_GOLDEN=1 drawio_golden tests,
# about 2 minutes), so edits to the code, the icons under media/ or the fixtures show up. Only without arguments.
# PNGs are at least 1024 px wide (scaled up when smaller, the 4000 px cap on the longer side wins). Output: out/<name>.{png,svg}; <name> = case dir for */expected.drawio, else the file's basename
set -euo pipefail
cd "$(dirname "$0")/.."
command -v drawio >/dev/null || { echo "drawio CLI not found (nix-shell provides drawio-headless)" >&2; exit 2; }

rebuild=0
[ $# -gt 0 ] || [ -z "${FULL_REBUILD:-}" ] || rebuild=1
if [ "$rebuild" = 1 ]; then
  echo "FULL_REBUILD: rebuilding goldens first"
  cargo build --quiet
  UPDATE_GOLDEN=1 cargo test --quiet --test drawio_golden >/dev/null
elif [ $# -eq 0 ]; then
  echo "rendering the goldens as they are (FULL_REBUILD=1 rebuilds them first)"
fi
mkdir -p out

targets=("$@")
[ ${#targets[@]} -gt 0 ] || targets=(fixtures/layout-cases)
mapfile -t files < <(find "${targets[@]}" -name '*.drawio' -not -path '*/node_modules/*' -not -path './target/*' -not -path './out/*' | sort)
[ ${#files[@]} -gt 0 ] || { echo "no drawio files found" >&2; exit 1; }

for f in "${files[@]}"; do
  name=$(basename "$f" .drawio)
  [ "$name" != expected ] || name=$(basename "$(dirname "$f")")
  # PNGs are at least 1024 px wide (small diagrams are scaled up); very large diagrams come out blank or fail at full scale,
  # so the cap of ~4000 px on the longer side wins over the minimum width (the margin cell spans the whole drawing)
  size=$({ grep -o '<mxCell id="margin"[^>]*><mxGeometry [^>]*' "$f" || true; } | sed -E 's/.*width="([0-9.]+)" height="([0-9.]+)".*/\1 \2/')
  scale=$(awk -v s="${size:-0 0}" 'BEGIN { split(s, a, " "); w = a[1]; h = a[2]; long = (w > h ? w : h)
    sc = (w > 0 && w < 1024) ? 1024 / w : 1; if (long * sc > 4000) sc = 4000 / long; print sc }')
  echo "$f -> out/$name.{png,svg} (scale $scale)"
  drawio -x -f png -s "$scale" -o "out/$name.png" "$f" --no-sandbox >/dev/null 2>&1 \
    || echo "  png render failed for $f" >&2
  # full-size vector copy: zoomable, searchable text; light theme so dark-mode viewers do not invert it
  drawio -x -f svg --svg-theme light -o "out/$name.svg" "$f" --no-sandbox >/dev/null 2>&1 \
    || echo "  svg render failed for $f" >&2
done
