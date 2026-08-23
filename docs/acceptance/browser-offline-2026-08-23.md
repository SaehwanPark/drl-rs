# Browser Offline Lifecycle Acceptance

Run identifier: `m10-offline-2026-08-23-r2`
Run date: 2026-08-23
Owner/role: `/root`, milestone owner
Predecessor: `_workspace/drl/m10-offline/02-test-plan.md`
Predecessor revision: `729f5c6` (generated web bundle baseline)
Input revision: `56155fb` (transactional replay baseline)
Output revision: `codex/m10-clear-save-acceptance` at `0.2.61` (working evidence)
Overall status: `PARTIAL PASS` — offline navigation/startup and Save/Load pass;
the Clear Save confirmation/cancel guard passes, while destructive confirmation
remains `NOT_RUN`
Build: `0.2.61`, generated from the current branch
Targets: `http://127.0.0.1:8765/index.html` initial smoke and fresh-origin
`http://127.0.0.1:8766/index.html`, generated with `sh scripts/build-web.sh`;
confirmation guard: fresh origin `http://127.0.0.1:8769/index.html`

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
7. On the fresh `8769` origin, saved the session and opened **Clear save**:
   the explicit dialog was visible, focus moved to **Cancel**, and no storage
   mutation occurred while the dialog was open: `PASS`.
8. Tab and reverse-Tab cycled focus between **Cancel** and **Clear save**;
   Escape remained available as a cancellation path: `PASS`.
9. Clicked **Cancel**, which returned `Saved session kept.`, restored focus to
   the outer Clear save button, and left the dialog hidden: `PASS`.
10. Clicked **Load** after cancellation and received
   `Session loaded from this device.`: the saved session remained available:
   `PASS`.
11. Pressed **Escape** while the dialog was open; it returned
    `Saved session kept.` and closed without mutation: `PASS`.
12. Accepting **Clear save** to delete local browser data remains `NOT_RUN`
    for the new guard; the earlier exploratory origin had already been cleared
    during native-prompt investigation and is not used as this acceptance.
13. Restored the browser network after the check: `PASS`.

## Recorded controls

- Offline control: CDP `Network.emulateNetworkConditions` with
  `offline: true`, zero latency, and disabled throughput; restored with
  `offline: false` and `Network.disable` after the run.
- Console capture: zero messages from the controlled tab during the offline
  reload/start/save/load sequence.

## Boundary

This is real browser evidence for service-worker installation, current-cache
control, offline navigation, offline WASM startup, offline Save/Load after one
online load, and the Clear Save confirmation/cancel guard. It does not claim
destructive confirmation acceptance, an OS-level PWA install prompt, production
HTTPS deployment, other browsers or GPU backends, WCAG/screen-reader
acceptance, audible output, or legacy visual/audio parity.
