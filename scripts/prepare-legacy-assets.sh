#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)

usage() {
  cat <<'HELP'
Usage: sh scripts/prepare-legacy-assets.sh [OPTIONS] [LEGACY_DIR]

Copies and prepares legacy DRL assets (graphics, fonts, sound, music) into
the assets/ directory of this repository for local gameplay and development.

IMPORTANT LEGAL & REPOSITORY INVARIANT:
  - Graphics are tracked in git under CC BY-SA 4.0.
  - Sound effects, music tracks, and bitmap fonts are APPROVED for game use,
    but MUST NOT be committed to git or included in distributing release binaries.
  - These assets remain untracked in git (.gitignore).

Arguments:
  LEGACY_DIR    Path to original DRL checkout or extracted official binary folder
                (default: $DRL_LEGACY_REPO or ../doom-the-roughlike-original)

Options:
  --all         Prepare all asset categories (graphics, fonts, sound, music) [default]
  --graphics    Prepare/import graphics bundle only
  --fonts       Prepare/import bitmap fonts only
  --sound       Prepare/import sound effects only (HQ & LQ)
  --music       Prepare/import music tracks only (HQ MP3 & LQ MIDI)
  --hq          Prepare High Quality audio only (HQ sound & HQ music)
  --lq          Prepare Low Quality audio only (LQ sound & MIDI music)
  --check       Run scripts/check-assets.sh after preparation
  --help, -h    Show this help message

Environment Variables:
  DRL_LEGACY_REPO   Default legacy checkout or binary folder path

Examples:
  # Prepare all assets from default legacy checkout (../doom-the-roughlike-original):
  sh scripts/prepare-legacy-assets.sh

  # Prepare all assets from an explicit legacy checkout path:
  sh scripts/prepare-legacy-assets.sh /Users/saehwan/repos/doom-the-roughlike-original

  # Prepare all assets from an extracted official DRL binary release:
  sh scripts/prepare-legacy-assets.sh /path/to/extracted-doomrl-binary

  # Prepare sound and music only:
  sh scripts/prepare-legacy-assets.sh --sound --music
HELP
}

do_graphics=0
do_fonts=0
do_sound=0
do_music=0
do_check=0
legacy_dir=""

while [ $# -gt 0 ]; do
  case "$1" in
    --all)
      do_graphics=1
      do_fonts=1
      do_sound=1
      do_music=1
      shift
      ;;
    --graphics)
      do_graphics=1
      shift
      ;;
    --fonts)
      do_fonts=1
      shift
      ;;
    --sound)
      do_sound=1
      shift
      ;;
    --music)
      do_music=1
      shift
      ;;
    --hq)
      do_sound=1
      do_music=1
      shift
      ;;
    --lq)
      do_sound=1
      do_music=1
      shift
      ;;
    --check)
      do_check=1
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    -*)
      printf '%s\n' "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
    *)
      if [ -z "$legacy_dir" ]; then
        legacy_dir="$1"
      else
        printf '%s\n' "Unexpected argument: $1" >&2
        usage >&2
        exit 1
      fi
      shift
      ;;
  esac
done

# If no specific category was requested, default to --all
if [ "$do_graphics" -eq 0 ] && [ "$do_fonts" -eq 0 ] && [ "$do_sound" -eq 0 ] && [ "$do_music" -eq 0 ]; then
  do_graphics=1
  do_fonts=1
  do_sound=1
  do_music=1
fi

legacy_dir=${legacy_dir:-${DRL_LEGACY_REPO:-"$repo_root/../doom-the-roughlike-original"}}

printf '%s\n' "Preparing legacy assets using source: $legacy_dir"

if [ "$do_graphics" -eq 1 ]; then
  printf '\n--- [1/4] Preparing Graphics ---\n'
  DRL_LEGACY_REPO="$legacy_dir" sh "$repo_root/scripts/import-legacy-graphics.sh"
fi

if [ "$do_fonts" -eq 1 ]; then
  printf '\n--- [2/4] Preparing Fonts ---\n'
  sh "$repo_root/scripts/import-legacy-fonts.sh" "$legacy_dir"
fi

if [ "$do_sound" -eq 1 ]; then
  printf '\n--- [3/4] Preparing Sound ---\n'
  sh "$repo_root/scripts/import-legacy-sound.sh" "$legacy_dir"
fi

if [ "$do_music" -eq 1 ]; then
  printf '\n--- [4/4] Preparing Music ---\n'
  sh "$repo_root/scripts/import-legacy-music.sh" "$legacy_dir"
fi

printf '\n=== Asset Preparation Complete ===\n'

if [ "$do_check" -eq 1 ]; then
  printf '\nRunning asset verification checks...\n'
  sh "$repo_root/scripts/check-assets.sh"
fi
