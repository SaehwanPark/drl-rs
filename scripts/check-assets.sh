#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
asset_dir=${DRL_GRAPHICS_DIR:-$repo_root/assets/legacy/drl/graphics}
manifest="$asset_dir/SHA256SUMS"
expected_revision=17d9be1204751899b2d69d8d3a2dde247bd0cc5c

require_all=0
for arg in "$@"; do
  if [ "$arg" = "--all" ]; then
    require_all=1
  fi
done
if [ "${DRL_REQUIRE_ALL_ASSETS:-0}" = "1" ]; then
  require_all=1
fi

# 1. Verify tracked graphics bundle
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

actual_graphics=$(find "$asset_dir" -maxdepth 1 -type f -name '*.png' | wc -l | tr -d ' ')
test "$actual_graphics" -eq 32 || {
  printf '%s\n' "Expected 32 imported graphics, found $actual_graphics." >&2
  exit 1
}

# 2. Verify git tracking safety: no audio, music, or fonts may be tracked in git
tracked_violations=$(
  git -C "$repo_root" ls-files assets | grep -E '\.(mp3|wav|mid|midi|mod|s3m|xm|ogg|flac|aac|woff|woff2|ttf|otf|pas|wad|dat)$|^assets/legacy/(drl/fonts|drlhq|drllq)/' || :
)
if [ -n "$tracked_violations" ]; then
  printf '%s\n' 'ERROR: Untracked assets are tracked by git in visible repository space:' >&2
  printf '%s\n' "$tracked_violations" >&2
  printf '%s\n' 'Sound, music, and font binaries must not be committed to git. Run git rm --cached to un-track.' >&2
  exit 1
fi

# 3. Inspect optional untracked local assets (fonts, sound, music)
fonts_dir="$repo_root/assets/legacy/drl/fonts"
hq_sound_dir="$repo_root/assets/legacy/drlhq/sound"
lq_sound_dir="$repo_root/assets/legacy/drllq/sound"
hq_music_dir="$repo_root/assets/legacy/drlhq/music"
lq_music_dir="$repo_root/assets/legacy/drllq/music"

fonts_found=0
if [ -d "$fonts_dir" ] && [ -f "$fonts_dir/font10x19.png" ]; then
  fonts_found=$(find "$fonts_dir" -maxdepth 1 -type f | wc -l | tr -d ' ')
fi

hq_sound_found=0
if [ -d "$hq_sound_dir" ]; then
  hq_sound_found=$(find "$hq_sound_dir" -maxdepth 1 -type f -name '*.wav' | wc -l | tr -d ' ')
fi

lq_sound_found=0
if [ -d "$lq_sound_dir" ]; then
  lq_sound_found=$(find "$lq_sound_dir" -maxdepth 1 -type f -name '*.wav' | wc -l | tr -d ' ')
fi

hq_music_found=0
if [ -d "$hq_music_dir" ]; then
  hq_music_found=$(find "$hq_music_dir" -maxdepth 1 -type f -name '*.mp3' | wc -l | tr -d ' ')
fi

lq_music_found=0
if [ -d "$lq_music_dir" ]; then
  lq_music_found=$(find "$lq_music_dir" -maxdepth 1 -type f -name '*.mid' | wc -l | tr -d ' ')
fi

if [ "$require_all" -eq 1 ]; then
  test "$fonts_found" -ge 1 || { printf '%s\n' 'Missing optional fonts in assets/legacy/drl/fonts.' >&2; exit 1; }
  test "$hq_sound_found" -ge 91 || { printf '%s\n' "Expected >= 91 HQ sound files, found $hq_sound_found." >&2; exit 1; }
  test "$lq_sound_found" -ge 90 || { printf '%s\n' "Expected >= 90 LQ sound files, found $lq_sound_found." >&2; exit 1; }
  test "$hq_music_found" -ge 21 || { printf '%s\n' "Expected >= 21 HQ music files, found $hq_music_found." >&2; exit 1; }
  test "$lq_music_found" -ge 31 || { printf '%s\n' "Expected >= 31 LQ music files, found $lq_music_found." >&2; exit 1; }
fi

printf '%s\n' "Legacy asset checks passed ($actual_graphics PNG graphics; provenance recorded; git tracking boundaries clean)."
if [ "$fonts_found" -gt 0 ] || [ "$hq_sound_found" -gt 0 ] || [ "$hq_music_found" -gt 0 ]; then
  printf '%s\n' "  - Local untracked fonts: $fonts_found file(s)"
  printf '%s\n' "  - Local untracked HQ sound: $hq_sound_found WAV file(s)"
  printf '%s\n' "  - Local untracked LQ sound: $lq_sound_found WAV file(s)"
  printf '%s\n' "  - Local untracked HQ music: $hq_music_found MP3 file(s)"
  printf '%s\n' "  - Local untracked LQ music: $lq_music_found MIDI file(s)"
else
  printf '%s\n' "  (Optional untracked sound/music/fonts absent; run 'sh scripts/prepare-legacy-assets.sh' to prepare them locally)"
fi
