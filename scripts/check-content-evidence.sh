#!/bin/sh

set -eu

cd "$(dirname "$0")/.."
legacy_repo=${DRL_LEGACY_REPO:-../doom-the-roughlike-original}
revision=17d9be1204751899b2d69d8d3a2dde247bd0cc5c

if [ ! -d "$legacy_repo/.git" ]; then
  printf '%s\n' 'Content evidence pinned probe: NOT_RUN (legacy checkout unavailable).'
  exit 0
fi

temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/drl-content-evidence.XXXXXX")
trap 'rm -rf "$temp_dir"' EXIT HUP INT TERM

python3 scripts/convert-legacy-content-bundle.py \
  --kind being \
  --legacy-repo "$legacy_repo" \
  --revision "$revision" \
  --output "$temp_dir/being.json" >/dev/null
python3 scripts/convert-legacy-content-bundle.py \
  --kind item \
  --legacy-repo "$legacy_repo" \
  --revision "$revision" \
  --output "$temp_dir/item.json" >/dev/null
python3 scripts/convert-legacy-content-bundle.py \
  --kind cell \
  --legacy-repo "$legacy_repo" \
  --revision "$revision" \
  --output "$temp_dir/cell.json" >/dev/null
python3 scripts/convert-legacy-level-index.py \
  --legacy-repo "$legacy_repo" \
  --revision "$revision" \
  --output "$temp_dir/level.json" >/dev/null

python3 scripts/check-content-evidence.py \
  --config docs/content/evidence-coverage.json \
  --bundle "being=$temp_dir/being.json" \
  --bundle "item=$temp_dir/item.json" \
  --bundle "cell=$temp_dir/cell.json" \
  --bundle "level=$temp_dir/level.json" \
  --rust-catalog crates/drl-core/src/special_level_definition.rs
