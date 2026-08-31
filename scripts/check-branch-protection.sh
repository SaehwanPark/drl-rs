#!/bin/sh

set -eu

branch_not_run() {
  printf 'Branch protection: NOT_RUN (%s)\n' "$1"
  exit 0
}

branch_failure() {
  printf 'Branch protection: FAIL (%s)\n' "$1" >&2
  exit 1
}

if ! command -v jq >/dev/null 2>&1; then
  if [ "${GITHUB_ACTIONS:-false}" = true ]; then
    branch_failure 'jq is unavailable on the hosted runner'
  fi
  branch_not_run 'jq is unavailable locally'
fi

repository=${DRL_BRANCH_PROTECTION_REPO:-${GITHUB_REPOSITORY:-}}
branch=${DRL_BRANCH_PROTECTION_BRANCH:-main}

if [ "${DRL_BRANCH_PROTECTION_JSON+x}" = x ]; then
  protection=$DRL_BRANCH_PROTECTION_JSON
else
  if [ -z "$repository" ]; then
    branch_not_run 'repository is unavailable'
  fi
  if ! command -v gh >/dev/null 2>&1; then
    if [ "${GITHUB_ACTIONS:-false}" = true ]; then
      branch_failure 'gh is unavailable on the hosted runner'
    fi
    branch_not_run 'gh is unavailable locally'
  fi

  authenticated=0
  if [ -n "${GH_TOKEN:-}" ] || [ -n "${GITHUB_TOKEN:-}" ]; then
    authenticated=1
  elif gh auth status >/dev/null 2>&1; then
    authenticated=1
  fi
  if [ "$authenticated" -ne 1 ]; then
    if [ "${GITHUB_ACTIONS:-false}" = true ]; then
      branch_failure 'GitHub credentials are unavailable on the hosted runner'
    fi
    branch_not_run 'GitHub credentials are unavailable locally'
  fi

  if [ -n "${GH_TOKEN:-}" ]; then
    if ! protection=$(GH_TOKEN="$GH_TOKEN" gh api \
      "repos/$repository/branches/$branch/protection"); then
      branch_failure "GitHub branch protection could not be inspected for $branch"
    fi
  elif [ -n "${GITHUB_TOKEN:-}" ]; then
    if ! protection=$(GH_TOKEN="$GITHUB_TOKEN" gh api \
      "repos/$repository/branches/$branch/protection"); then
      branch_failure "GitHub branch protection could not be inspected for $branch"
    fi
  else
    if ! protection=$(gh api "repos/$repository/branches/$branch/protection"); then
      branch_failure "GitHub branch protection could not be inspected for $branch"
    fi
  fi
fi

if ! printf '%s\n' "$protection" | jq -e type >/dev/null 2>&1; then
  branch_failure 'branch-protection response is not valid JSON'
fi

approvals=$(printf '%s\n' "$protection" | jq -r \
  '.required_pull_request_reviews.required_approving_review_count // 0')
case "$approvals" in
  ''|*[!0-9]*) branch_failure 'approval count is not numeric' ;;
esac
if [ "$approvals" -lt 1 ]; then
  branch_failure 'at least one approving review is required'
fi

if [ "$(printf '%s\n' "$protection" | jq -r \
  '.required_pull_request_reviews.dismiss_stale_reviews // false')" != true ]; then
  branch_failure 'stale approvals must be dismissed'
fi

if [ "$(printf '%s\n' "$protection" | jq -r \
  '.required_status_checks.strict // false')" != true ]; then
  branch_failure 'required status checks must use strict branch updates'
fi

required_contexts='Repository checks
WASM browser checks
Review policy'
available_contexts=$(printf '%s\n' "$protection" | jq -r '
  [
    (.required_status_checks.contexts // []),
    ((.required_status_checks.checks // []) | map(.context))
  ]
  | add
  | .[]
')
missing_contexts=$(printf '%s\n' "$required_contexts" | while IFS= read -r context
do
  [ -z "$context" ] && continue
  if ! printf '%s\n' "$available_contexts" | grep -F -x "$context" >/dev/null 2>&1; then
    printf '%s\n' "$context"
  fi
done)

if [ -n "$missing_contexts" ]; then
  branch_failure "required contexts are missing: $(printf '%s' "$missing_contexts" | paste -sd ', ' -)"
fi

if [ "$(printf '%s\n' "$protection" | jq -r \
  'if .enforce_admins.enabled == false then "false" elif .enforce_admins.enabled == true then "true" else "" end')" != false ]; then
  branch_failure 'enforce_admins.enabled must be false as the documented solo-maintainer exception'
fi

printf 'Branch protection: PASS (%s has the required review and status policy; enforce_admins=false is the documented solo-maintainer exception)\n' "$branch"
