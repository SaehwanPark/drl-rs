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

The project includes original 2D pixel-art graphics from *Doom the Roguelike (DRL)*:

- **Source**: Imported from the upstream ChaosForge graphics repository under explicit attribution.
- **License**: The graphics attribution and license terms are preserved in `assets/legacy/drl/graphics/LICENSE` ([CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/)).
- **Integrity**: Verified by SHA-256 checksums recorded in `assets/legacy/drl/graphics/SHA256SUMS`.
- **Repository Status**: Tracked in Git and bundled into the browser edition.

---

## 🔊 Sound, Music & Font Assets

Sound effects, music tracks, and bitmap fonts from original DRL are **approved for use in the game**, subject to the following distribution conditions:

- **Exclusion from Binaries & Repository**: Sound, music, and font binaries must **not** be included in distributing release packages or tracked in visible repository space (`.gitignore`).
- **Separate Download**: Players and external contributors must download and extract these assets separately from official DRL game binaries ([https://drl.chaosforge.org/](https://drl.chaosforge.org/)).
- **Local Preparation**: Developers can run `sh scripts/prepare-legacy-assets.sh` pointing to a pre-downloaded legacy checkout or extracted official release folder.

---

## 🛡️ Asset Verification Scripts

To verify that all assets conform to license, integrity, and Git tracking boundaries:

```bash
# Verify tracked graphics integrity and ensure no untracked assets are in git
sh scripts/check-assets.sh

# Verify release bundle boundary
sh scripts/check-release-rights.sh
```
