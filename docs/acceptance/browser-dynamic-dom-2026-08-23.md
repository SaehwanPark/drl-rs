# Browser Dynamic DOM Acceptance

Run identifier: `m12-dynamic-dom-2026-08-23-r1`
Run date: 2026-08-23
Owner/role: `/root`, milestone owner
Predecessor: `_workspace/drl/m12-accessibility-contract/04-verification.md`
Predecessor revision: `afdc01f` (dynamic interaction contract)
Input revision: `46d6ec00cbf67c4fe781877a5802d6115aa03283`
Output revision: `46d6ec0` evidence build revision
Overall status: `PARTIAL PASS` — supported-Chromium DOM/runtime observations
pass; keyboard traversal, WCAG, screen-reader, and broad-browser acceptance
remain `NOT_RUN`.

## Environment

- Browser: Codex In-app Browser, Chrome `151.0.0.0`
- Host: macOS `Darwin 25.6.0 arm64` (`MacIntel` platform reported by the page)
- Target: `http://127.0.0.1:8767/index.html`, generated with
  `sh scripts/build-web.sh`
- Secure context: `true` on loopback; WebGPU adapter: available
- GPU backend: `NOT_RUN` (the browser did not expose a backend identity)
- Viewport: `1280x720`, DPR `1`
- Audio: `Audio is gesture-gated`; audible output was not claimed

## Procedure and result

1. The initial DOM exposed the named start/restart/save/load/clear/mute
   controls, volume slider, browser-support disclosure, game HUD, focusable
   canvas, static keyboard help, and inventory region: `PASS`.
2. Selecting **Start game** kept the diagnostic panel hidden, focused the
   canvas, and showed a `3px solid` focus outline. The live status reported
   ready state, including `Textures uploaded: 24`: `PASS`.
3. Pressing `ArrowRight` on the focused canvas advanced the HUD to `Turn: 1`
   and changed the sole `role=status` live region to `Turn 1: Move(East)`;
   keyboard help remained unchanged: `PASS`.
4. Selecting the target and firing produced generated inventory controls with
   item-qualified names `Equip 9mm Ammo`, `Use 9mm Ammo`, `Drop 9mm Ammo`,
   `Equip Small MedPack`, `Use Small MedPack`, and `Drop Small MedPack`.
   Every observed button had an action-qualified `aria-label`, stable action,
   and item ID: `PASS`.
5. Automated Tab traversal was `NOT_RUN`: the browser control did not advance
   focus reliably enough to record a trustworthy order. Direct start-to-canvas
   focus and authored focus styling were observed above.
6. The controlled tab produced no console messages during the run: `PASS`.

## Boundary

This is runtime DOM evidence for the supported Chromium target only. It does
not claim WCAG 2.1 AA conformance, screen-reader or other assistive-technology
compatibility, contrast certification, keyboard traversal completeness,
human-usability results, other browsers/backends, mobile/touch behavior,
audible output, or audiovisual parity.
