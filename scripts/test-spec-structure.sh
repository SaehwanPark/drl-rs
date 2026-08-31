#!/bin/sh

set -eu

cd "$(dirname "$0")/.."

temp_dir=${TMPDIR:-/tmp}/drl-spec-structure.$$
if ! (umask 077 && mkdir "$temp_dir"); then
  printf '%s\n' "Unable to create temporary SPEC fixture directory: $temp_dir" >&2
  exit 1
fi
trap 'rm -rf "$temp_dir"' 0 1 2 15

failures=0

expect_pass() {
  label=$1
  path=$2

  if ! sh scripts/check-spec-structure.sh "$path" >/dev/null; then
    printf '%s\n' "SPEC structure fixture failed unexpectedly: $label" >&2
    failures=$((failures + 1))
  fi
}

expect_reject() {
  label=$1
  path=$2

  if sh scripts/check-spec-structure.sh "$path" >/dev/null 2>&1; then
    printf '%s\n' "SPEC structure fixture was accepted unexpectedly: $label" >&2
    failures=$((failures + 1))
  fi
}

canonical="$temp_dir/canonical.md"
cp SPEC.md "$canonical"
expect_pass "canonical SPEC" "$canonical"

extra_top_level="$temp_dir/extra-top-level.md"
awk '
  $0 == "## 3. Enduring invariants" {
    print "## 4. Delivered slice history"
    print "Historical content must not be appended to SPEC.md."
  }
  { print }
' "$canonical" > "$extra_top_level"
expect_reject "extra top-level history section" "$extra_top_level"

indented_top_level="$temp_dir/indented-top-level.md"
awk '
  $0 == "## 3. Enduring invariants" {
    print "  ## 4. Indented delivered slice history"
  }
  { print }
' "$canonical" > "$indented_top_level"
expect_reject "indented extra top-level history section" "$indented_top_level"

duplicate_active="$temp_dir/duplicate-active.md"
awk '
  $0 == "## 3. Enduring invariants" {
    print "## 4. Active implementation slice: historical copy"
  }
  { print }
' "$canonical" > "$duplicate_active"
expect_reject "duplicate active slice" "$duplicate_active"

nested_active="$temp_dir/nested-active.md"
awk '
  $0 == "### 2.1 Objective" {
    print "### Active implementation slice: nested historical copy"
  }
  { print }
' "$canonical" > "$nested_active"
expect_reject "nested active slice marker" "$nested_active"

if [ "$failures" -ne 0 ]; then
  printf '%s\n' "SPEC structure contract failed with $failures fixture issue(s)." >&2
  exit 1
fi

printf '%s\n' 'SPEC structure contract: PASS (canonical and four rejection fixtures)'
