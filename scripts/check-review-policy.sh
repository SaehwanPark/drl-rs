#!/bin/sh

set -eu

policy_not_run() {
  printf 'Review policy: NOT_RUN (%s)\n' "$1"
  exit 0
}

policy_failure() {
  printf 'Review policy: FAIL (%s)\n' "$1" >&2
  exit 1
}

if ! command -v jq >/dev/null 2>&1; then
  if [ "${GITHUB_ACTIONS:-false}" = true ]; then
    policy_failure 'jq is unavailable on the hosted runner'
  fi
  policy_not_run 'jq is unavailable locally'
fi

pull_request_number=${DRL_REVIEW_POLICY_PR:-}
if [ -z "$pull_request_number" ] && [ -n "${GITHUB_EVENT_PATH:-}" ] &&
  [ -f "$GITHUB_EVENT_PATH" ]; then
  pull_request_number=$(jq -r '.pull_request.number // empty' "$GITHUB_EVENT_PATH")
fi

if [ -z "$pull_request_number" ]; then
  policy_not_run 'pull request number is unavailable'
fi

repository=${DRL_REVIEW_POLICY_REPO:-${GITHUB_REPOSITORY:-}}
if [ -z "$repository" ]; then
  policy_not_run 'repository is unavailable'
fi

fetch_api() {
  if [ -n "${GH_TOKEN:-}" ]; then
    GH_TOKEN="$GH_TOKEN" gh api "$@"
  elif [ -n "${GITHUB_TOKEN:-}" ]; then
    GH_TOKEN="$GITHUB_TOKEN" gh api "$@"
  else
    gh api "$@"
  fi
}

if [ "${DRL_REVIEW_POLICY_FILES+x}" = x ]; then
  changed_files=$DRL_REVIEW_POLICY_FILES
else
  if ! command -v gh >/dev/null 2>&1; then
    if [ "${GITHUB_ACTIONS:-false}" = true ]; then
      policy_failure 'gh is unavailable on the hosted runner'
    fi
    policy_not_run 'gh is unavailable locally'
  fi

  authenticated=0
  if [ -n "${GH_TOKEN:-}" ] || [ -n "${GITHUB_TOKEN:-}" ]; then
    authenticated=1
  elif gh auth status >/dev/null 2>&1; then
    authenticated=1
  fi
  if [ "$authenticated" -ne 1 ]; then
    if [ "${GITHUB_ACTIONS:-false}" = true ]; then
      policy_failure 'GitHub credentials are unavailable on the hosted runner'
    fi
    policy_not_run 'GitHub credentials are unavailable locally'
  fi

  if ! changed_files=$(fetch_api --paginate \
    "repos/$repository/pulls/$pull_request_number/files" --jq '.[].filename'); then
    policy_failure 'pull-request file metadata could not be inspected'
  fi
fi

protected_files=$(printf '%s\n' "$changed_files" | awk '
  function protected(path) {
    return path ~ /^crates\/(drl-core|drl-protocol|drl-mcp|drl-app|drl-web|drl-script)\// ||
      path ~ /^docs\/legacy-behavior\//
  }
  protected($0) { print; found = 1 }
  END { if (!found) exit 1 }
' || :)

if [ -z "$protected_files" ]; then
  printf '%s\n' 'Review policy: PASS (pull request has no protected paths)'
  exit 0
fi

if [ "${DRL_REVIEW_POLICY_AUTHOR+x}" = x ]; then
  author=$DRL_REVIEW_POLICY_AUTHOR
else
  if ! command -v gh >/dev/null 2>&1; then
    if [ "${GITHUB_ACTIONS:-false}" = true ]; then
      policy_failure 'gh is unavailable while protected paths are changed'
    fi
    policy_not_run 'gh is unavailable while protected paths are changed'
  fi
  if ! author=$(fetch_api "repos/$repository/pulls/$pull_request_number" \
    --jq '.user.login // empty'); then
    policy_failure 'pull-request author metadata could not be inspected'
  fi
fi

if [ -z "$author" ]; then
  policy_failure 'pull-request author is missing'
fi

if [ "${DRL_REVIEW_POLICY_REVIEWS+x}" = x ]; then
  reviews=$DRL_REVIEW_POLICY_REVIEWS
else
  if ! reviews=$(fetch_api --paginate \
    "repos/$repository/pulls/$pull_request_number/reviews"); then
    policy_failure 'pull-request review metadata could not be inspected'
  fi
fi

if ! review_receipt=$(printf '%s\n' "$reviews" | jq -r --arg author "$author" '
  def has_receipt:
    ((.body // "") | contains("drl-determinism-review: PASS"));
  [ .[]
    | select((.user.login // "") != "" and (.submitted_at // "") != "")
  ]
  | sort_by(.submitted_at)
  | reduce .[] as $review ({}; .[$review.user.login] = $review)
  | any(.[];
      (.user.login != $author and
       .state == "APPROVED" and
       has_receipt)
    )
'); then
  policy_failure 'pull-request review metadata is not valid JSON'
fi

if [ "$review_receipt" != true ]; then
  protected_list=$(printf '%s\n' "$protected_files" | paste -sd ', ' -)
  policy_failure "no current independent approval with drl-determinism-review: PASS for $protected_list"
fi

printf '%s\n' 'Review policy: PASS (current independent determinism-review receipt found)'
