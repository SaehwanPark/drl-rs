#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
legacy_dir=${1:-${DRL_LEGACY_REPO:-"$repo_root/../doom-the-roughlike-original"}}

hq_dest="$repo_root/assets/legacy/drlhq/sound"
lq_dest="$repo_root/assets/legacy/drllq/sound"

# Locate HQ sound source
hq_source=""
for candidate in \
  "$legacy_dir/bin/data/drlhq/sound" \
  "$legacy_dir/data/drlhq/sound" \
  "$legacy_dir/wav" \
  "$legacy_dir/sound"; do
  if [ -d "$candidate" ] && [ -n "$(find "$candidate" -maxdepth 1 -name '*.wav' -print -quit 2>/dev/null)" ]; then
    hq_source="$candidate"
    break
  fi
done

# Locate LQ sound source
lq_source=""
for candidate in \
  "$legacy_dir/bin/data/drllq/sound" \
  "$legacy_dir/data/drllq/sound"; do
  if [ -d "$candidate" ] && [ -n "$(find "$candidate" -maxdepth 1 -name '*.wav' -print -quit 2>/dev/null)" ]; then
    lq_source="$candidate"
    break
  fi
done

# Fallback: if only one sound source exists (e.g. wav/), use it for both HQ and LQ
if [ -n "$hq_source" ] && [ -z "$lq_source" ]; then
  lq_source="$hq_source"
elif [ -z "$hq_source" ] && [ -n "$lq_source" ]; then
  hq_source="$lq_source"
fi

if [ -z "$hq_source" ]; then
  printf '%s\n' "Error: Could not locate legacy sound files in '$legacy_dir'." >&2
  printf '%s\n' "Expected 'bin/data/drlhq/sound/' or 'data/drlhq/sound/' or 'wav/' containing .wav files." >&2
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
  printf '%s\n' "Prepared $hq_count HQ sound file(s) in $hq_dest (untracked by git)."
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
  printf '%s\n' "Prepared $lq_count LQ sound file(s) in $lq_dest (untracked by git)."
fi
