#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
legacy_dir=${1:-${DRL_LEGACY_REPO:-"$repo_root/../doom-the-roughlike-original"}}

hq_dest="$repo_root/assets/legacy/drlhq/music"
lq_dest="$repo_root/assets/legacy/drllq/music"

# Locate HQ music source (MP3)
hq_source=""
for candidate in \
  "$legacy_dir/bin/data/drlhq/music" \
  "$legacy_dir/data/drlhq/music" \
  "$legacy_dir/mp3" \
  "$legacy_dir/music"; do
  if [ -d "$candidate" ] && [ -n "$(find "$candidate" -maxdepth 1 -name '*.mp3' -print -quit 2>/dev/null)" ]; then
    hq_source="$candidate"
    break
  fi
done

# Locate LQ music source (MIDI)
lq_source=""
for candidate in \
  "$legacy_dir/bin/data/drllq/music" \
  "$legacy_dir/data/drllq/music" \
  "$legacy_dir/music"; do
  if [ -d "$candidate" ] && [ -n "$(find "$candidate" -maxdepth 1 -name '*.mid' -print -quit 2>/dev/null)" ]; then
    lq_source="$candidate"
    break
  fi
done

if [ -z "$hq_source" ] && [ -z "$lq_source" ]; then
  printf '%s\n' "Error: Could not locate legacy music files in '$legacy_dir'." >&2
  printf '%s\n' "Expected 'bin/data/drlhq/music/' (.mp3) or 'bin/data/drllq/music/' (.mid)." >&2
  exit 1
fi

hq_count=0
if [ -n "$hq_source" ]; then
  mkdir -p "$hq_dest"
  for item in "$hq_source"/*; do
    if [ -f "$item" ]; then
      cp "$item" "$hq_dest/"
      hq_count=$((hq_count + 1))
    fi
  done
  printf '%s\n' "Prepared $hq_count HQ music file(s) in $hq_dest (untracked by git)."
fi

lq_count=0
if [ -n "$lq_source" ]; then
  mkdir -p "$lq_dest"
  for item in "$lq_source"/*; do
    if [ -f "$item" ]; then
      cp "$item" "$lq_dest/"
      lq_count=$((lq_count + 1))
    fi
  done
  printf '%s\n' "Prepared $lq_count LQ/MIDI music file(s) in $lq_dest (untracked by git)."
fi
