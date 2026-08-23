# Browser Offline Lifecycle Acceptance

Run identifier: `m10-offline-2026-08-23-r2`
Run date: 2026-08-23
Owner/role: `/root`, milestone owner
Predecessor: `_workspace/drl/m10-offline/02-test-plan.md`
Input revision: `729f5c69217539a46da9715b64bb5567aa856406`
Output revision: `1a07482` evidence publication; review corrections are the
follow-up commit on this branch
Build: `0.2.41`, generated from the input revision
Targets: `http://127.0.0.1:8765/index.html` initial smoke and fresh-origin
`http://127.0.0.1:8766/index.html`, generated with `sh scripts/build-web.sh`

## Environment

- Browser: Codex In-app Browser, Chrome `151.0.0.0`
- Host: macOS `Darwin 25.6.0 arm64` (`MacIntel` platform reported by the page)
- Secure context: `true` on loopback; WebGPU adapter: available
- GPU backend: `NOT_RUN` (the browser did not expose a backend identity)
- Viewport: `1280x720`, DPR `1`
- Audio: `Audio is gesture-gated`; audible output was not claimed
- Fresh profile: `NOT_RUN`; the second pass used a fresh loopback origin
  (`http://127.0.0.1:8766/`) in the existing Codex browser profile.
- Durable runtime state: service worker `activated` and controlling the page;
  Cache Storage contained exactly `drl-rust-m10-v1-0.2.41-729f5c692175`;
  `localStorage` availability was `true`.

## Procedure and result

1. Served the generated `dist/` directory over loopback HTTP and opened the
   shell on a fresh loopback origin: `PASS`.
2. Selected **Start game** while online. The shell reported `Textures uploaded:
   24` and `Offline cache installation started for the next reload`: `PASS`.
3. Reloaded online and started again. The shell reported `Offline cache ready
   for the next reload` and `HP: 50/50`, `Turn: 0`: `PASS`.
4. Confirmed `service-worker.js` was `activated` and controlling the page,
   with the expected generated cache key above: `PASS`.
5. Disabled network requests through the browser development controls and
   reloaded. The shell loaded from the current release cache and started with
   `HP: 50/50`, `Turn: 0`, `Weapon: Pistol`: `PASS`.
6. While offline, **Save** returned `Session saved on this device.` and
   **Load** returned `Session loaded from this device.`: `PASS`. The move
   between those actions was intentionally auto-persisted by the browser
   session, so this run does not claim rollback-to-an-earlier-turn behavior.
7. **Clear save** is `NOT_RUN` pending action-time confirmation because it
   deletes local browser data; no console errors were observed.
8. Restored the browser network after the check: `PASS`.

## Recorded controls

- Offline control: CDP `Network.emulateNetworkConditions` with
  `offline: true`, zero latency, and disabled throughput; restored with
  `offline: false` and `Network.disable` after the run.
- Console capture: zero messages from the controlled tab during the offline
  reload/start/save/load sequence.

## Boundary

This is real browser evidence for service-worker installation, current-cache
control, offline navigation, offline WASM startup, and offline Save/Load after
one online load. It does not claim the test-plan-required Clear Save action
until confirmed, an OS-level PWA install prompt, production HTTPS deployment,
other browsers or GPU backends, WCAG/screen-reader acceptance, audible output,
or legacy visual/audio parity.
