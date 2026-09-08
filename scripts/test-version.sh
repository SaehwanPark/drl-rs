#!/bin/sh

set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/drl-version-test.XXXXXX")
trap 'rm -rf "$temp_dir"' EXIT HUP INT TERM

fixture="$temp_dir/fixture"
mkdir -p "$fixture/scripts" "$fixture/crates"
cp "$root/scripts/check-version.sh" "$fixture/scripts/check-version.sh"
chmod +x "$fixture/scripts/check-version.sh"
cat > "$fixture/Cargo.toml" <<'EOF'
[workspace]
resolver = "3"

[workspace.package]
version = "0.1.0"

[workspace.dependencies]
EOF
printf '%s\n' '0.1.0' > "$fixture/VERSION"
git -C "$fixture" init -q
git -C "$fixture" config user.email version-fixture@example.invalid
git -C "$fixture" config user.name version-fixture
git -C "$fixture" add Cargo.toml VERSION scripts/check-version.sh
git -C "$fixture" commit -q -m base
base=$(git -C "$fixture" rev-parse HEAD)

run_check() {
  (cd "$fixture" && DRL_VERSION_BASE="$base" sh scripts/check-version.sh)
}

expect_pass() {
  label=$1
  if ! output=$(run_check 2>&1); then
    printf '%s\n%s\n' "version fixture failed: $label" "$output" >&2
    exit 1
  fi
}

expect_fail() {
  label=$1
  if output=$(run_check 2>&1); then
    printf '%s\n%s\n' "version fixture unexpectedly passed: $label" "$output" >&2
    exit 1
  fi
}

reset_fixture() {
  git -C "$fixture" reset --hard -q "$base"
  git -C "$fixture" clean -fdq
}

set_version() {
  version=$1
  python3 - "$fixture" "$version" <<'PY'
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
version = sys.argv[2]
(root / "VERSION").write_text(version + "\n")
manifest = root / "Cargo.toml"
text = manifest.read_text()
manifest.write_text(text.replace('version = "0.1.0"', f'version = "{version}"'))
PY
}

# A shipped runtime module is code, even though its suffix is not Rust.
reset_fixture
mkdir -p "$fixture/web"
printf '%s\n' 'export const enabled = false;' > "$fixture/web/module.mjs"
git -C "$fixture" add web/module.mjs
git -C "$fixture" commit -q -m mjs-change
expect_fail 'mjs behavior change without version bump'

reset_fixture
mkdir -p "$fixture/web"
printf '%s\n' 'export const enabled = true;' > "$fixture/web/module.mjs"
set_version 0.1.1
git -C "$fixture" add VERSION Cargo.toml web/module.mjs
git -C "$fixture" commit -q -m mjs-versioned
expect_pass 'mjs behavior change with one patch bump'

reset_fixture
mkdir -p "$fixture/web"
printf '%s\n' 'export const enabled = true;' > "$fixture/web/module.mjs"
set_version 0.3.0
git -C "$fixture" add VERSION Cargo.toml web/module.mjs
git -C "$fixture" commit -q -m mjs-overbumped
expect_fail 'mjs change with more than one allowed transition'

reset_fixture
mkdir -p "$fixture/src"
printf '%s\n' 'fn changed() {}' > "$fixture/src/lib.rs"
git -C "$fixture" add src/lib.rs
git -C "$fixture" commit -q -m rust-change
expect_fail 'Rust change without version bump'

reset_fixture
mkdir -p "$fixture/scripts"
printf '%s\n' '#!/bin/sh' > "$fixture/scripts/runtime.sh"
git -C "$fixture" add scripts/runtime.sh
git -C "$fixture" commit -q -m shell-change
expect_fail 'shell change without version bump'

reset_fixture
mkdir -p "$fixture/docs" "$fixture/settings"
printf '%s\n' 'updated notes' > "$fixture/docs/notes.md"
printf '%s\n' 'enabled=true' > "$fixture/settings/local.conf"
git -C "$fixture" add docs/notes.md settings/local.conf
git -C "$fixture" commit -q -m documentation-and-settings
expect_pass 'documentation and settings changes without bump'

reset_fixture
mkdir -p "$fixture/docs"
printf '%s\n' 'updated notes' > "$fixture/docs/notes.md"
set_version 0.1.1
git -C "$fixture" add VERSION Cargo.toml docs/notes.md
git -C "$fixture" commit -q -m doc-overbumped
expect_fail 'documentation-only change with version bump'

printf '%s\n' 'Version classifier fixtures: PASS (.mjs, Rust, shell, documentation, and settings paths)'
