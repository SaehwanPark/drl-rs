#!/bin/sh

set -eu

cd "$(dirname "$0")/.."

temp_root=${TMPDIR:-/tmp}/drl-review-policy.$$
umask 077
mkdir "$temp_root"
trap 'rm -rf "$temp_root"' 0 1 2 15

script=sh
check=./scripts/check-review-policy.sh
repository=fixture/example
author=author
pr=17

run_pass() {
  if ! env \
    DRL_REVIEW_POLICY_PR="$pr" \
    DRL_REVIEW_POLICY_REPO="$repository" \
    DRL_REVIEW_POLICY_FILES="$1" \
    DRL_REVIEW_POLICY_AUTHOR="$author" \
    DRL_REVIEW_POLICY_REVIEWS="${2:-[]}" \
    "$script" "$check" >"$temp_root/output" 2>&1; then
    printf 'Expected policy pass, got:\n%s\n' "$(sed -n '1,40p' "$temp_root/output")" >&2
    exit 1
  fi
}

run_fail() {
  if env \
    DRL_REVIEW_POLICY_PR="$pr" \
    DRL_REVIEW_POLICY_REPO="$repository" \
    DRL_REVIEW_POLICY_FILES="$1" \
    DRL_REVIEW_POLICY_AUTHOR="$author" \
    DRL_REVIEW_POLICY_REVIEWS="${2:-[]}" \
    "$script" "$check" >"$temp_root/output" 2>&1; then
    printf 'Expected policy failure, got:\n%s\n' "$(sed -n '1,40p' "$temp_root/output")" >&2
    exit 1
  fi
}

run_pass 'README.md'

run_fail 'crates/drl-core/src/game.rs'

run_fail 'crates/drl-protocol/src/lib.rs' '[
  {"user":{"login":"author"},"state":"APPROVED","body":"drl-determinism-review: PASS","submitted_at":"2026-08-31T12:00:00Z"}
]'

run_pass 'docs/legacy-behavior/chainfire.md' '[
  {"user":{"login":"reviewer"},"state":"APPROVED","body":"drl-determinism-review: PASS\nChecked rejection and replay boundaries.","submitted_at":"2026-08-31T12:00:00Z"}
]'

run_fail 'crates/drl-mcp/src/lib.rs' '[
  {"user":{"login":"reviewer"},"state":"APPROVED","body":"drl-determinism-review: PASS","submitted_at":"2026-08-31T12:00:00Z"},
  {"user":{"login":"reviewer"},"state":"CHANGES_REQUESTED","body":"Found an unresolved boundary issue.","submitted_at":"2026-08-31T12:01:00Z"}
]'

run_pass 'crates/drl-web/src/session.rs' '[
  {"user":{"login":"reviewer"},"state":"APPROVED","body":"drl-determinism-review: PASS","submitted_at":"2026-08-31T12:00:00Z"},
  {"user":{"login":"other-reviewer"},"state":"COMMENTED","body":"Reviewed documentation.","submitted_at":"2026-08-31T12:01:00Z"}
]'

printf '%s\n' 'Review-policy fixtures: PASS'
