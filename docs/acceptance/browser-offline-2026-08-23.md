# Browser Offline Lifecycle Acceptance

Run date: 2026-08-23
Build: `0.2.41`, source revision `729f5c69217539a46da9715b64bb5567aa856406`
Target: `http://127.0.0.1:8765/index.html`, generated with `sh scripts/build-web.sh`

## Environment

- Browser: Codex In-app Browser, Chrome `151.0.0.0`
- Host: macOS `Darwin 25.6.0 arm64` (`MacIntel` platform reported by the page)
- Secure context: `true` on loopback; WebGPU adapter: available
- GPU backend: `NOT_RUN` (the browser did not expose a backend identity)
- Viewport: `1280x720`, DPR `1`
- Audio: `Audio is gesture-gated`; audible output was not claimed

## Procedure and result

1. Served the generated `dist/` directory over loopback HTTP and opened the
   shell: `PASS`.
2. Selected **Start game** while online. The shell reported `Textures uploaded:
   24` and `Offline cache installation started for the next reload`: `PASS`.
3. Reloaded online and started again. The shell reported `Offline cache ready
   for the next reload` and `HP: 50/50`, `Turn: 0`: `PASS`.
4. Disabled network requests through the browser's development controls and
   reloaded. The shell loaded from the current release cache: `PASS`.
5. Selected **Start game** while offline. The game reached `HP: 50/50`,
   `Turn: 0`, `Weapon: Pistol`, with no console errors: `PASS`.
6. Restored the browser network after the check: `PASS`.

## Boundary

This is real browser evidence for service-worker installation, current-cache
control, offline navigation, and offline WASM startup after one online load. It
does not claim an OS-level PWA install prompt, production HTTPS deployment,
other browsers or GPU backends, WCAG/screen-reader acceptance, audible output,
or legacy visual/audio parity.
