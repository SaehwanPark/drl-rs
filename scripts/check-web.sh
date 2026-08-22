#!/bin/sh

set -eu

cd "$(dirname "$0")/.."

sh scripts/check-service-worker.sh

if ! rustup target list --installed | awk '$1 == "wasm32-unknown-unknown" { found = 1 } END { exit found ? 0 : 1 }'; then
  printf '%s\n' 'wasm32-unknown-unknown target is required; install it with rustup target add wasm32-unknown-unknown.' >&2
  exit 1
fi

cargo check --locked -p drl-web --target wasm32-unknown-unknown
cargo test --locked -p drl-assets -p drl-render -p drl-audio -p drl-web

if command -v wasm-pack >/dev/null 2>&1 \
  && { command -v google-chrome >/dev/null 2>&1 \
    || command -v google-chrome-stable >/dev/null 2>&1 \
    || command -v chrome >/dev/null 2>&1 \
    || command -v chromium >/dev/null 2>&1 \
    || command -v chromium-browser >/dev/null 2>&1; }; then
  wasm-pack test --headless --chrome crates/drl-web
else
  printf '%s\n' 'WASM browser runner unavailable; native contract tests passed, browser tests NOT_RUN.'
fi
