#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
legacy_dir=${1:-${DRL_LEGACY_REPO:-"$repo_root/../doom-the-roughlike-original"}}
destination="$repo_root/assets/legacy/drl/fonts"

fonts_source=""
for candidate in \
  "$legacy_dir/bin/data/drl/fonts" \
  "$legacy_dir/data/drl/fonts" \
  "$legacy_dir/fonts" \
  "$legacy_dir"; do
  if [ -d "$candidate" ] && { [ -f "$candidate/font10x19.png" ] || [ -f "$candidate/default" ]; }; then
    fonts_source="$candidate"
    break
  fi
done

font_dat_source=""
for candidate in \
  "$legacy_dir/bin/font.dat" \
  "$legacy_dir/font.dat" \
  "$legacy_dir/bin/data/drl/fonts/font10x19.png" \
  "$legacy_dir/data/drl/fonts/font10x19.png"; do
  if [ -f "$candidate" ]; then
    font_dat_source="$candidate"
    break
  fi
done

if [ -z "$fonts_source" ] && [ -z "$font_dat_source" ]; then
  printf '%s\n' "Error: Could not locate legacy fonts in '$legacy_dir'." >&2
  printf '%s\n' "Expected 'bin/data/drl/fonts/' or 'data/drl/fonts/' containing 'font10x19.png'." >&2
  exit 1
fi

mkdir -p "$destination"

copied_count=0
if [ -n "$fonts_source" ]; then
  for item in "$fonts_source"/*; do
    if [ -f "$item" ]; then
      cp "$item" "$destination/"
      copied_count=$((copied_count + 1))
    fi
  done
fi

if [ -n "$font_dat_source" ] && [ ! -f "$destination/font.dat" ]; then
  cp "$font_dat_source" "$destination/font.dat"
  copied_count=$((copied_count + 1))
fi

printf '%s\n' "Prepared $copied_count legacy font file(s) in $destination (untracked by git)."
