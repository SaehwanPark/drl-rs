#!/bin/sh

set -eu

cd "$(dirname "$0")/.."

spec_path=${1:-SPEC.md}

if [ "$#" -gt 1 ]; then
  printf '%s\n' "Usage: $0 [SPEC.md path]" >&2
  exit 2
fi

if [ ! -f "$spec_path" ]; then
  printf '%s\n' "SPEC structural check failed: missing file: $spec_path" >&2
  exit 1
fi

if ! awk '
  BEGIN {
    level_two_count = 0
    active_heading_count = 0
    in_fence = 0
    failures = 0
  }

  /^[[:space:]]*(```|~~~)/ {
    in_fence = !in_fence
    next
  }

  !in_fence && $0 ~ /^ {0,3}## [^#]/ {
    level_two_count++
    level_two[level_two_count] = $0
  }

  !in_fence && $0 ~ /^ {0,3}##+[[:space:]]/ && index($0, "Active implementation slice:") {
    active_heading_count++
    active_heading_line = NR
  }

  END {
    if (level_two_count != 3) {
      printf "%s\n", "expected exactly 3 level-two headings, found " level_two_count
      failures++
    }

    if (level_two[1] != "## 1. Status vocabulary") {
      printf "%s\n", "first level-two heading must be ## 1. Status vocabulary"
      failures++
    }

    if (level_two[2] !~ /^## 2\. Active implementation slice: /) {
      printf "%s\n", "second level-two heading must begin ## 2. Active implementation slice:"
      failures++
    }

    if (level_two[3] != "## 3. Enduring invariants") {
      printf "%s\n", "third level-two heading must be ## 3. Enduring invariants"
      failures++
    }

    if (active_heading_count != 1) {
      printf "%s\n", "expected exactly 1 active implementation slice heading, found " active_heading_count
      failures++
    }

    exit failures
  }
' "$spec_path" >&2; then
  printf '%s\n' "SPEC structural check failed: $spec_path" >&2
  exit 1
fi

printf '%s\n' "SPEC structural check: PASS ($spec_path)"
