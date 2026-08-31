#!/bin/sh

set -eu

cd "$(dirname "$0")/.."

sh scripts/check-service-worker.sh
sh scripts/test-service-worker.sh
sh scripts/test-offline-cache.sh
sh scripts/test-browser-controls.sh
node scripts/test-browser-support.mjs
sh scripts/check-browser-diagnostics.sh
sh scripts/check-browser-accessibility.sh

if ! rustup target list --installed | awk '$1 == "wasm32-unknown-unknown" { found = 1 } END { exit found ? 0 : 1 }'; then
  printf '%s\n' 'wasm32-unknown-unknown target is required; install it with rustup target add wasm32-unknown-unknown.' >&2
  exit 1
fi

cargo check --locked -p drl-web --target wasm32-unknown-unknown
cargo test --locked -p drl-assets -p drl-render -p drl-audio -p drl-web

browser_runner=""
for browser_command in google-chrome google-chrome-stable chrome chromium chromium-browser; do
  if command -v "$browser_command" >/dev/null 2>&1; then
    browser_runner="$browser_command"
    break
  fi
done
if [ -z "$browser_runner" ]; then
  for browser_path in \
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
    "/Applications/Chromium.app/Contents/MacOS/Chromium"; do
    if [ -x "$browser_path" ]; then
      browser_runner="$browser_path"
      break
    fi
  done
fi

if command -v wasm-pack >/dev/null 2>&1 && [ -n "$browser_runner" ]; then
  wasm-pack test --headless --chrome crates/drl-web
else
  printf '%s\n' 'WASM browser runner unavailable; native contract tests passed, browser tests NOT_RUN.'
fi
