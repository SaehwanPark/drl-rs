---
title: "Asset Licensing Policy"
description: "Legal boundaries, legacy asset provenance, and open-source licensing compliance for drl-rs."
---

# Asset Licensing Policy

**drl-rs** adheres to clear, transparent legal provenance standards for all code, documentation, and graphical assets.

---

## 📜 Code & Documentation License

- **Source Code & Documentation**: All Rust source code, documentation, specifications, and scripts in this repository are licensed under the [MIT License](https://opensource.org/licenses/MIT).

---

## 🎨 Legacy Graphical Assets

The browser release bundle includes original 2D pixel-art graphics from *Doom the Roguelike (DRL)*:

- **Source**: Imported from the upstream ChaosForge graphics repository under explicit attribution.
- **License**: The graphics attribution and license terms are preserved in `assets/legacy/drl/graphics/LICENSE`.
- **Integrity**: Verified by SHA-256 checksums recorded in `assets/legacy/drl/graphics/SHA256SUMS`.
- **Audio & Music Exclusion**: Legacy fonts, voice lines, sound effects, and music tracks are intentionally excluded from the repository pending individual copyright clearances.

---

## 🛡️ Asset Verification Script

To verify that all bundled assets conform to license and integrity declarations:

```bash
sh scripts/check-assets.sh
sh scripts/check-release-rights.sh
```
