#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/drl-release-rights.XXXXXX")
trap 'rm -rf "$temp_dir"' EXIT HUP INT TERM

bundle="$temp_dir/bundle"
graphics="$bundle/assets/legacy/drl/graphics"
source_graphics="$temp_dir/source-graphics"
mkdir -p "$graphics"
cp -R "$repo_root/assets/legacy/drl/graphics/." "$graphics/"
cp -R "$repo_root/assets/legacy/drl/graphics/." "$source_graphics/"
printf '%s\n' '{"rights":["assets/legacy/drl/graphics/LICENSE"]}' > "$bundle/release-manifest.json"

run_check() {
  DRL_GRAPHICS_DIR="$source_graphics" RELEASE_DIST="$bundle" \
    sh "$repo_root/scripts/check-release-rights.sh" >/dev/null
}

expect_failure() {
  if run_check 2>/dev/null; then
    printf '%s\n' "expected release-rights check to reject $1" >&2
    exit 1
  fi
}

run_check

printf '%s\n' '{"rights":[]}' > "$bundle/release-manifest.json"
expect_failure 'incorrect manifest rights declaration'
printf '%s\n' '{"rights":["assets/legacy/drl/graphics/LICENSE"]}' > "$bundle/release-manifest.json"

printf '%s\n' 'tampered license' >> "$graphics/LICENSE"
expect_failure 'tampered graphics evidence'
cp "$repo_root/assets/legacy/drl/graphics/LICENSE" "$graphics/LICENSE"

rm "$graphics/logo.png"
expect_failure 'missing graphics file'
cp "$repo_root/assets/legacy/drl/graphics/logo.png" "$graphics/logo.png"

mkdir -p "$graphics/nested"
cp "$repo_root/assets/legacy/drl/graphics/logo.png" "$graphics/nested/extra.png"
expect_failure 'nested graphics file'
rm -rf "$graphics/nested"

ln -s "$repo_root/assets/legacy-drlhq/music/cde1m1.mp3" "$bundle/escaped-audio"
expect_failure 'symlink escape'
rm "$bundle/escaped-audio"

mkdir -p "$source_graphics/nested"
cp "$repo_root/assets/legacy/drl/graphics/logo.png" "$source_graphics/nested/extra.png"
expect_failure 'nested source graphics file'
rm -rf "$source_graphics/nested"

rm "$graphics/LICENSE"
expect_failure 'missing graphics license'
cp "$repo_root/assets/legacy/drl/graphics/LICENSE" "$graphics/LICENSE"

mkdir -p "$bundle/assets/legacy-drlhq/music"
touch "$bundle/assets/legacy-drlhq/music/injected.mp3"
expect_failure 'legacy music path'
rm -rf "$bundle/assets/legacy-drlhq"

mkdir -p "$bundle/assets/fonts"
touch "$bundle/assets/fonts/injected.woff2"
expect_failure 'font file'
rm -rf "$bundle/assets/fonts"

mkdir -p "$bundle/reference"
touch "$bundle/reference/injected.lua"
expect_failure 'legacy code file'

printf '%s\n' 'Release rights tests: PASS (positive bundle and exclusion fixtures).'
