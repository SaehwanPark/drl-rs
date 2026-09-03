#!/bin/sh

set -eu

cd "$(dirname "$0")/.."

html=web/index.html
bootstrap=web/bootstrap.js
offline_cache=web/offline-cache.mjs
browser_support=web/browser-support.mjs
# Browser shell surface. The WASM shell is a module map plus focused modules, so
# the contract greps below run across every file that owns browser behavior.
wasm="
  crates/drl-web/src/lib.rs
  crates/drl-web/src/session.rs
  crates/drl-web/src/input.rs
  crates/drl-web/src/dom.rs
  crates/drl-web/src/gpu.rs
  crates/drl-web/src/wasm/mod.rs
  crates/drl-web/src/wasm/storage.rs
  crates/drl-web/src/wasm/renderer.rs
  crates/drl-web/src/wasm/scene.rs
  crates/drl-web/src/wasm/app.rs
  crates/drl-web/src/wasm/shell_dom.rs
  crates/drl-web/src/wasm/animation_loop.rs
  crates/drl-web/src/wasm/exports.rs
"
wasm_exports=crates/drl-web/src/wasm/exports.rs
wasm_shell_dom=crates/drl-web/src/wasm/shell_dom.rs
wasm_animation_loop=crates/drl-web/src/wasm/animation_loop.rs

test -s "$html"
test -s "$bootstrap"
test -s "$offline_cache"
test -s "$browser_support"
for file in $wasm; do
  test -s "$file"
done
grep -F 'id="game-diagnostics"' "$html" >/dev/null
grep -F 'role="alert"' "$html" >/dev/null
grep -F 'id="browser-support"' "$html" >/dev/null
grep -F 'Tested target: desktop Chromium with WebGPU enabled.' "$html" >/dev/null
grep -F 'function writeDiagnostic' "$bootstrap" >/dev/null
grep -F 'navigator.gpu' "$bootstrap" >/dev/null
grep -F 'browserSupportDiagnostic' "$bootstrap" "$browser_support" >/dev/null
grep -F 'window.isSecureContext' "$bootstrap" >/dev/null
grep -F 'Offline cache unavailable' "$offline_cache" >/dev/null
grep -F 'updateViaCache: "none"' "$offline_cache" >/dev/null
grep -F 'Offline cache update is ready' "$offline_cache" >/dev/null
grep -F 'Audio unavailable' "$bootstrap" >/dev/null
# Each contract string is asserted on the module that owns it, not on the aggregate
# module set: losing an owner must fail the check even when a collaborating module
# still mentions the same string.
# The incompatible-save title is a producer/consumer pair. wasm/exports.rs writes the
# diagnostics title and wasm/shell_dom.rs compares against it, so both must keep it.
grep -F 'Saved session incompatible' "$wasm_exports" >/dev/null
grep -F '== Some("Saved session incompatible")' "$wasm_shell_dom" >/dev/null
grep -F 'Use Clear save to remove it, then save a new session from this build.' "$wasm_exports" >/dev/null
grep -F 'persistence_diagnostic_active' "$wasm_shell_dom" >/dev/null
grep -F 'data-diagnostic-source' "$wasm_shell_dom" >/dev/null
grep -F 'data-diagnostic-source' "$bootstrap" >/dev/null
grep -F 'fn set_diagnostic' "$wasm_shell_dom" >/dev/null
grep -F 'WebGPU presentation unavailable' "$wasm_animation_loop" >/dev/null
if grep -F 'fetch(' "$bootstrap" >/dev/null; then
  printf '%s\n' 'Browser diagnostics must not add telemetry fetches.' >&2
  exit 1
fi

printf '%s\n' 'Browser support and diagnostics contracts passed.'
