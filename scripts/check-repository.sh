#!/bin/sh

set -eu

tab=$(printf '\t')

tab_hits=$(
  git ls-files --cached --others --exclude-standard -z \
    | xargs -0 grep -nIH "$tab" -- 2>/dev/null || :
)

if [ -n "$tab_hits" ]; then
  printf '%s\n' 'Literal tab characters are not allowed:' "$tab_hits" >&2
  exit 1
fi

whitespace_hits=$(
  git ls-files --cached --others --exclude-standard -z \
    | xargs -0 grep -nIE '[[:blank:]]+$' -- 2>/dev/null || :
)

if [ -n "$whitespace_hits" ]; then
  printf '%s\n' 'Trailing whitespace is not allowed:' "$whitespace_hits" >&2
  exit 1
fi

cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace
