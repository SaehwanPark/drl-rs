---
title: "Versioning Policy"
description: "Semantic versioning rules and continuous integration release validation in drl-rs."
---

# Versioning Policy

**drl-rs** uses explicit, three-component (`x.y.z`) non-negative integer versioning. The single source of truth for the project version is the top-level `VERSION` file.

---

## 🔢 Version Transition Rules

Versions advance according to the following strict state transition rules:

1. **Major Release (`x.0.0`)**: Increments `x` and resets `y` and `z` to zero for sweeping architectural overhauls.
2. **Significant Milestone Feature (`x.y.0`)**: Increments `y` and resets `z` to zero when a new milestone or major gameplay slice lands.
3. **Codebase Change (`x.y.z+1`)**: Any Pull Request modifying code paths (`.rs`, `.js`, `.py`, `.sh`, `.html`, `.css`, `.wgsl`) must increment `z` by exactly one digit.
4. **Documentation & Setting Diffs**: Changes touching solely markdown (`.md`), configuration (`.toml`, `.yml`, `.json`), or documentation must **not** increment `VERSION`.

---

## 🤖 Continuous Integration Enforcement

In CI, `scripts/check-version.sh` automatically compares the branch diff against the base commit (`DRL_VERSION_BASE`):

- Fails if a code diff fails to bump `VERSION`.
- Fails if a doc-only diff erroneously bumps `VERSION`.
- Validates that root `Cargo.toml`, workspace dependency pins, MCP server metadata, and `release-manifest.json` match `VERSION` exactly.

Run the local version validator:
```bash
sh scripts/check-version.sh
```
