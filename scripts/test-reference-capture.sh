#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
checker="$repo_root/scripts/check-reference-capture.sh"
base="$repo_root/_workspace/reference-captures/manifest.txt"
fixture_dir=$(mktemp -d)
trap 'rm -rf "$fixture_dir"' EXIT

expect_pass() {
  manifest=$1
  DRL_CAPTURE_MANIFEST="$manifest" "$checker" >/dev/null
}

expect_fail() {
  manifest=$1
  if DRL_CAPTURE_MANIFEST="$manifest" "$checker" >/dev/null 2>&1; then
    printf 'expected preflight failure: %s\n' "$manifest" >&2
    exit 1
  fi
}

missing="$fixture_dir/missing.txt"
DRL_CAPTURE_MANIFEST="$missing" "$checker" >/dev/null

valid="$fixture_dir/valid.txt"
cp "$base" "$valid"
expect_pass "$valid"

missing_key="$fixture_dir/missing-key.txt"
sed '/^scenes=/d' "$base" > "$missing_key"
expect_fail "$missing_key"

bad_revision="$fixture_dir/bad-revision.txt"
sed 's/^legacy_revision=.*/legacy_revision=bad/' "$base" > "$bad_revision"
expect_fail "$bad_revision"

bad_status="$fixture_dir/bad-status.txt"
sed 's/^status=.*/status=UNKNOWN/' "$base" > "$bad_status"
expect_fail "$bad_status"

missing_scene="$fixture_dir/missing-scene.txt"
sed 's/,hud,/,/' "$base" > "$missing_scene"
expect_fail "$missing_scene"

ready_placeholders="$fixture_dir/ready-placeholders.txt"
sed -e 's/^status=.*/status=READY_FOR_CONTROLLED_CAPTURE/' \
  -e 's/^capture_host=.*/capture_host=Linux-x86_64/' "$base" > "$ready_placeholders"
expect_fail "$ready_placeholders"

ready="$fixture_dir/ready.txt"
sed -e 's/^status=.*/status=READY_FOR_CONTROLLED_CAPTURE/' \
  -e 's/^capture_host=.*/capture_host=Linux-x86_64/' \
  -e 's/^viewport=.*/viewport=1280x720/' \
  -e 's/^dpr=.*/dpr=1/' \
  -e 's/^scenario=.*/scenario=fixed/' \
  -e 's/^actions=.*/actions=smoke/' \
  -e 's/^capture_tool=.*/capture_tool=tool/' \
  -e 's/^capture_tool_version=.*/capture_tool_version=1/' \
  -e 's/^media_hashes=.*/media_hashes=recorded/' "$base" > "$ready"
expect_pass "$ready"

bad_hash="$fixture_dir/bad-hash.txt"
sed 's/^executable_sha256=.*/executable_sha256=deadbeef/' "$base" > "$bad_hash"
expect_fail "$bad_hash"

duplicate="$fixture_dir/duplicate.txt"
cp "$base" "$duplicate"
printf '%s\n' 'reason=duplicate' >> "$duplicate"
expect_fail "$duplicate"

printf '%s\n' 'Reference capture preflight fixtures: PASS'
