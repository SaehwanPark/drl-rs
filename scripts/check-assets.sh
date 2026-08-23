#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
asset_dir=${DRL_GRAPHICS_DIR:-$repo_root/assets/legacy/drl/graphics}
manifest="$asset_dir/SHA256SUMS"
expected_revision=17d9be1204751899b2d69d8d3a2dde247bd0cc5c

test -s "$asset_dir/LICENSE" || { printf '%s\n' 'Missing graphics license.' >&2; exit 1; }
test -s "$asset_dir/MANIFEST.txt" || { printf '%s\n' 'Missing graphics provenance manifest.' >&2; exit 1; }
test -s "$manifest" || { printf '%s\n' 'Missing graphics checksums.' >&2; exit 1; }
grep -qx "legacy_revision=$expected_revision" "$asset_dir/MANIFEST.txt" || {
  printf '%s\n' 'Graphics provenance revision does not match the pinned import.' >&2
  exit 1
}
grep -qx 'source_path=bin/data/drl/graphics' "$asset_dir/MANIFEST.txt" || {
  printf '%s\n' 'Graphics provenance source path is missing or incorrect.' >&2
  exit 1
}

expected_files=$(mktemp)
actual_files=$(mktemp)
trap 'rm -f "$expected_files" "$actual_files"' EXIT HUP INT TERM
awk '{ print $2 }' "$manifest" | LC_ALL=C sort > "$expected_files"
find "$asset_dir" -type f ! -name MANIFEST.txt ! -name SHA256SUMS -print |
  while IFS= read -r path; do
    printf '%s\n' "${path#"$asset_dir"/}"
  done | LC_ALL=C sort > "$actual_files"
if ! diff -u "$expected_files" "$actual_files"; then
  printf '%s\n' 'Asset directory contains missing or unrecorded files.' >&2
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  (cd "$asset_dir" && sha256sum -c SHA256SUMS)
else
  (cd "$asset_dir" && shasum -a 256 -c SHA256SUMS)
fi

actual=$(find "$asset_dir" -maxdepth 1 -type f -name '*.png' | wc -l | tr -d ' ')
test "$actual" -eq 32 || {
  printf '%s\n' "Expected 32 imported graphics, found $actual." >&2
  exit 1
}

printf '%s\n' "Legacy asset checks passed ($actual PNG files; provenance recorded)."
