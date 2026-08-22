#!/bin/sh

set -eu

cd "$(dirname "$0")/.."

html=web/index.html
bootstrap=web/bootstrap.js
wasm=crates/drl-web/src/lib.rs
test -s "$html"
test -s "$bootstrap"
test -s "$wasm"
grep -F 'id="game-diagnostics"' "$html" >/dev/null
grep -F 'role="alert"' "$html" >/dev/null
grep -F 'id="browser-support"' "$html" >/dev/null
grep -F 'Tested target: desktop Chromium with WebGPU enabled.' "$html" >/dev/null
grep -F 'function writeDiagnostic' "$bootstrap" >/dev/null
grep -F 'navigator.gpu' "$bootstrap" >/dev/null
grep -F 'Offline cache unavailable' "$bootstrap" >/dev/null
grep -F 'Audio unavailable' "$bootstrap" >/dev/null
grep -F 'fn set_diagnostic' "$wasm" >/dev/null
grep -F 'WebGPU presentation unavailable' "$wasm" >/dev/null
if grep -F 'fetch(' "$bootstrap" >/dev/null; then
  printf '%s\n' 'Browser diagnostics must not add telemetry fetches.' >&2
  exit 1
fi

printf '%s\n' 'Browser support and diagnostics contracts passed.'
