#!/bin/sh

set -eu

cd "$(dirname "$0")/.."

temp_root=${TMPDIR:-/tmp}/drl-branch-protection.$$
umask 077
mkdir "$temp_root"
trap 'rm -rf "$temp_root"' 0 1 2 15

script=sh
check=./scripts/check-branch-protection.sh

passing='{
  "required_status_checks": {
    "strict": true,
    "contexts": ["Repository checks", "WASM browser checks", "Review policy"],
    "checks": []
  },
  "required_pull_request_reviews": {
    "required_approving_review_count": 1,
    "dismiss_stale_reviews": true
  },
  "enforce_admins": {"enabled": false}
}'

run_pass() {
  if ! env DRL_BRANCH_PROTECTION_REPO=fixture/example \
    DRL_BRANCH_PROTECTION_BRANCH=main \
    DRL_BRANCH_PROTECTION_JSON="$1" \
    "$script" "$check" >"$temp_root/output" 2>&1; then
    printf 'Expected branch-protection pass, got:\n%s\n' "$(sed -n '1,40p' "$temp_root/output")" >&2
    exit 1
  fi
}

run_fail() {
  if env DRL_BRANCH_PROTECTION_REPO=fixture/example \
    DRL_BRANCH_PROTECTION_BRANCH=main \
    DRL_BRANCH_PROTECTION_JSON="$1" \
    "$script" "$check" >"$temp_root/output" 2>&1; then
    printf 'Expected branch-protection failure, got:\n%s\n' "$(sed -n '1,40p' "$temp_root/output")" >&2
    exit 1
  fi
}

run_pass "$passing"
run_fail "$(printf '%s\n' "$passing" | jq '.required_status_checks.strict = false')"
run_fail "$(printf '%s\n' "$passing" | jq 'del(.required_pull_request_reviews)')"
run_fail "$(printf '%s\n' "$passing" | jq '.required_pull_request_reviews.dismiss_stale_reviews = false')"
run_fail "$(printf '%s\n' "$passing" | jq '.required_status_checks.contexts = ["Repository checks", "WASM browser checks"]')"
run_fail "$(printf '%s\n' "$passing" | jq '.enforce_admins.enabled = true')"
run_fail 'null'

printf '%s\n' 'Branch-protection fixtures: PASS'
