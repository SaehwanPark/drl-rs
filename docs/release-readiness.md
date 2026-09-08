# Public release readiness

**Current version:** `0.2.352`
**Status:** **Not release-ready**

This is a compact product-entry summary, not a replacement for the
[roadmap](DRL-RS_Project_Roadmap.md), the [2026-09-07 project
audit](project-audit-2026-09-07.md), or the [release-rights
policy](release-rights.md). A gate is not complete because a nearby test or
build succeeds; the named acceptance evidence is required.

## What can be run today

- **Headless simulation and CLI:** `cargo run -p drl-app --bin drl-rs`
  provides deterministic demos, cohort studies, and bounded replay-file
  verification.
- **MCP agent interface:** `cargo run -p drl-app --bin drl-rs -- --mcp`
  provides the stdio JSON-RPC gameplay, replay, metrics, and resource tools.
  Requests have bounded frame, nesting, and batch limits.
- **Browser playable slice:** `sh scripts/serve-web.sh` serves the WebGPU
  browser slice for a supported desktop Chromium environment with WebGPU.
  Local startup diagnostics and the accessible DOM shell are implemented;
  production hosting and full accessibility acceptance remain open.
- **Native preview:** `cargo run -p drl-desktop -- --validate` validates the
  thin scene/session boundary without claiming a packaged native product.

## Verified evidence at this checkpoint

| Surface | Evidence | Status |
| --- | --- | --- |
| Fair observations | F1 core/MCP tests cover hidden ground-item addition, removal, count changes, two-world equality, JSON non-disclosure, omniscient views, and reveal (`0af1bed`). | PASS |
| MCP safety | F2 MCP library tests, app tests, bounded stdio contracts, and the shipped-binary deep-input subprocess regression (`ae8a451`). | PASS |
| Version policy | `.mjs`, Rust, shell, documentation, settings, exact-bump, and over-bump temporary-Git fixtures (`578702e`). | PASS |
| Browser checks | `sh scripts/check-web.sh` passed in the audit/remediation run; unavailable runtime/capture surfaces stay separate. | PASS / limited scope |
| Full repository check | The local workspace check exceeded the execution allowance during integration tests. | INCONCLUSIVE |
| Hosted CI and branch protection | Not independently queried for this checkpoint. | NOT_RUN |

## Open release gates

The following remain open or `NOT_RUN` and must not be inferred from local
library tests:

- production static HTTPS deployment for desktop Chromium/WebGPU;
- fully functional offline PWA installation in the production deployment;
- production signing-key custody, provisioning, rotation, and trust-root policy;
- dynamic WCAG 2.1 AA and screen-reader acceptance;
- client-side asset-pack detection and diagnostics for optional graphics, audio,
  and font packs;
- approved audiovisual parity against controlled reference captures;
- complete deterministic headless/MCP tooling and external-client compatibility;
- rights-cleared Linux legacy runtime captures and capture-to-game comparison;
- an eligible independent reviewer path before the temporary solo-maintainer
  branch-protection exception can be removed.

See the M3, M12, and M13 sections of the roadmap for authoritative acceptance
criteria and the [browser acceptance records](acceptance/) for environment-
specific evidence. Unsupported environments remain `NOT_RUN`, not passes.

## Release decision rule

Do not announce a public 1.0 release until every applicable open gate has a
recorded `PASS`, or a documented release decision explicitly scopes the product
and names the remaining limitations. Keep gameplay, replay, observation,
platform, rights, accessibility, audiovisual, and performance claims separate.
