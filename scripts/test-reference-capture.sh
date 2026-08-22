#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
checker="$repo_root/scripts/check-reference-capture.sh"
base="$repo_root/_workspace/reference-captures/manifest.txt"
fixture_dir=$(mktemp -d)
trap 'rm -rf "$fixture_dir"' EXIT
valid_hash=sha256:0000000000000000000000000000000000000000000000000000000000000000

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

if ! grep -q '^legacy_dirty_state=' "$valid"; then
  printf '%s\n' 'generated manifest is missing legacy_dirty_state' >&2
  exit 1
fi
if ! grep -q '^evidence_classification=observed$' "$valid"; then
  printf '%s\n' 'generated manifest is missing observed evidence classification' >&2
  exit 1
fi

missing_key="$fixture_dir/missing-key.txt"
sed '/^scenes=/d' "$base" > "$missing_key"
expect_fail "$missing_key"

missing_dirty_state="$fixture_dir/missing-dirty-state.txt"
sed '/^legacy_dirty_state=/d' "$base" > "$missing_dirty_state"
expect_fail "$missing_dirty_state"

missing_classification="$fixture_dir/missing-classification.txt"
sed '/^evidence_classification=/d' "$base" > "$missing_classification"
expect_fail "$missing_classification"

missing_rights="$fixture_dir/missing-rights.txt"
sed '/^rights_status=/d' "$base" > "$missing_rights"
expect_fail "$missing_rights"

invalid_rights="$fixture_dir/invalid-rights.txt"
sed 's/^rights_status=.*/rights_status=unknown/' "$base" > "$invalid_rights"
expect_fail "$invalid_rights"

invalid_classification="$fixture_dir/invalid-classification.txt"
sed 's/^evidence_classification=.*/evidence_classification=unknown/' "$base" > "$invalid_classification"
expect_fail "$invalid_classification"

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

ready_dirty="$fixture_dir/ready-dirty.txt"
sed -e 's/^status=.*/status=READY_FOR_CONTROLLED_CAPTURE/' \
  -e 's/^capture_host=.*/capture_host=Linux-x86_64/' \
  -e "s/^media_hashes=.*/media_hashes=$valid_hash/" \
  -e 's/^viewport=.*/viewport=1280x720/' \
  -e 's/^dpr=.*/dpr=1/' \
  -e 's/^scenario=.*/scenario=fixed/' \
  -e 's/^actions=.*/actions=smoke/' \
  -e 's/^capture_tool=.*/capture_tool=tool/' \
  -e 's/^capture_tool_version=.*/capture_tool_version=1/' "$base" > "$ready_dirty"
expect_fail "$ready_dirty"

ready_inferred="$fixture_dir/ready-inferred.txt"
sed -e 's/^status=.*/status=READY_FOR_CONTROLLED_CAPTURE/' \
  -e 's/^capture_host=.*/capture_host=Linux-x86_64/' \
  -e 's/^legacy_dirty_state=.*/legacy_dirty_state=clean/' \
  -e 's/^evidence_classification=.*/evidence_classification=inferred/' \
  -e 's/^viewport=.*/viewport=1280x720/' \
  -e 's/^dpr=.*/dpr=1/' \
  -e 's/^scenario=.*/scenario=fixed/' \
  -e 's/^actions=.*/actions=smoke/' \
  -e 's/^capture_tool=.*/capture_tool=tool/' \
  -e 's/^capture_tool_version=.*/capture_tool_version=1/' \
  -e "s/^media_hashes=.*/media_hashes=$valid_hash/" "$base" > "$ready_inferred"
expect_fail "$ready_inferred"

ready="$fixture_dir/ready.txt"
sed -e 's/^status=.*/status=READY_FOR_CONTROLLED_CAPTURE/' \
  -e 's/^capture_host=.*/capture_host=Linux-x86_64/' \
  -e 's/^legacy_dirty_state=.*/legacy_dirty_state=clean/' \
  -e "s/^media_hashes=.*/media_hashes=$valid_hash/" \
  -e 's/^viewport=.*/viewport=1280x720/' \
  -e 's/^dpr=.*/dpr=1/' \
  -e 's/^scenario=.*/scenario=fixed/' \
  -e 's/^actions=.*/actions=smoke/' \
  -e 's/^capture_tool=.*/capture_tool=tool/' \
  -e 's/^capture_tool_version=.*/capture_tool_version=1/' "$base" > "$ready"
expect_pass "$ready"

malformed_hash="$fixture_dir/malformed-hash.txt"
sed -e 's/^status=.*/status=READY_FOR_CONTROLLED_CAPTURE/' \
  -e 's/^capture_host=.*/capture_host=Linux-x86_64/' \
  -e 's/^legacy_dirty_state=.*/legacy_dirty_state=clean/' \
  -e 's/^media_hashes=.*/media_hashes=sha256:not-a-hash/' \
  -e 's/^viewport=.*/viewport=1280x720/' \
  -e 's/^dpr=.*/dpr=1/' \
  -e 's/^scenario=.*/scenario=fixed/' \
  -e 's/^actions=.*/actions=smoke/' \
  -e 's/^capture_tool=.*/capture_tool=tool/' \
  -e 's/^capture_tool_version=.*/capture_tool_version=1/' "$base" > "$malformed_hash"
expect_fail "$malformed_hash"

inconclusive_dirty="$fixture_dir/inconclusive-dirty.txt"
sed 's/^status=.*/status=INCONCLUSIVE/' "$base" > "$inconclusive_dirty"
expect_fail "$inconclusive_dirty"

pass_unclear="$fixture_dir/pass-unclear.txt"
sed -e 's/^status=.*/status=PASS/' \
  -e 's/^capture_host=.*/capture_host=Linux-x86_64/' \
  -e 's/^legacy_dirty_state=.*/legacy_dirty_state=clean/' \
  -e 's/^rights_status=.*/rights_status=unclear/' \
  -e "s/^media_hashes=.*/media_hashes=$valid_hash/" \
  -e 's/^viewport=.*/viewport=1280x720/' \
  -e 's/^dpr=.*/dpr=1/' \
  -e 's/^scenario=.*/scenario=fixed/' \
  -e 's/^actions=.*/actions=smoke/' \
  -e 's/^capture_tool=.*/capture_tool=tool/' \
  -e 's/^capture_tool_version=.*/capture_tool_version=1/' \
  "$base" > "$pass_unclear"
expect_fail "$pass_unclear"

pass="$fixture_dir/pass.txt"
sed -e 's/^status=.*/status=PASS/' \
  -e 's/^capture_host=.*/capture_host=Linux-x86_64/' \
  -e 's/^legacy_dirty_state=.*/legacy_dirty_state=clean/' \
  -e 's/^rights_status=.*/rights_status=cleared/' \
  -e "s/^media_hashes=.*/media_hashes=$valid_hash/" \
  -e 's/^viewport=.*/viewport=1280x720/' \
  -e 's/^dpr=.*/dpr=1/' \
  -e 's/^scenario=.*/scenario=fixed/' \
  -e 's/^actions=.*/actions=smoke/' \
  -e 's/^capture_tool=.*/capture_tool=tool/' \
  -e 's/^capture_tool_version=.*/capture_tool_version=1/' \
  "$base" > "$pass"
expect_pass "$pass"

bad_hash="$fixture_dir/bad-hash.txt"
sed 's/^executable_sha256=.*/executable_sha256=deadbeef/' "$base" > "$bad_hash"
expect_fail "$bad_hash"

duplicate="$fixture_dir/duplicate.txt"
cp "$base" "$duplicate"
printf '%s\n' 'reason=duplicate' >> "$duplicate"
expect_fail "$duplicate"

printf '%s\n' 'Reference capture preflight fixtures: PASS'
