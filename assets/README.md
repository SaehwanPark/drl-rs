# drl-rs Assets

This directory contains presentation assets used by the **drl-rs** game engine.

---

## ⚖️ Asset Policy & Legal Boundaries

Assets in **drl-rs** are organized into two distinct tiers:

1. **Tracked Graphical Assets (`assets/legacy/drl/graphics/`)**:
   - 2D pixel-art sprite sheets imported from upstream *Doom the Roguelike (DRL)*.
   - Licensed under [Creative Commons Attribution-ShareAlike 4.0 (CC BY-SA 4.0)](legacy/drl/graphics/LICENSE).
   - Tracked in Git with provenance manifest (`MANIFEST.txt`) and SHA-256 checksums (`SHA256SUMS`).
   - Cleared for bundling in the static web release.

2. **Untracked Audio & Font Assets (`assets/legacy/drl/fonts/`, `assets/legacy/drlhq/`, `assets/legacy/drllq/`)**:
   - Sound effects (WAV), music tracks (MP3/MIDI), and bitmap fonts from the original game are **approved for use in the game**, provided they are **not included in distributing binaries or visible repository space**.
   - These assets **must remain untracked by Git** (`.gitignore`).
   - Players and contributors download and extract them separately from official DRL binaries or use the pre-downloaded legacy source.

---

## 📁 Asset Directory Layout

| Category | Repository Path | Git Status | Source / Description |
|---|---|---|---|
| **Graphics** | `assets/legacy/drl/graphics/` | Tracked | 32 PNG sprite sheets (`dguy.png`, `enemies.png`, etc.) + `LICENSE` + `MANIFEST.txt` + `SHA256SUMS` |
| **Fonts** | `assets/legacy/drl/fonts/` | Untracked | `font10x19.png` (8058 bytes), `default`, and `font.dat` |
| **HQ Music** | `assets/legacy/drlhq/music/` | Untracked | 21 MP3 tracks (`cde1m1.mp3`..`cde1m9.mp3`, `dark_secrets.mp3`, `rage.mp3`, etc.) |
| **HQ Sound** | `assets/legacy/drlhq/sound/` | Untracked | 92 WAV sound effects (`dsbfg.wav`, `dsshotgn.wav`, `dsplasma.wav`, `dsbarexp.wav`, etc.) |
| **LQ Music** | `assets/legacy/drllq/music/` | Untracked | 31 MIDI tracks (`0  - intro.mid`, `11 - hangar.mid`, `28 - tower of babel.mid`, etc.) |
| **LQ Sound** | `assets/legacy/drllq/sound/` | Untracked | 91 WAV sound effects |
| **Project Media** | `assets/drl-rs-banner.png` | Tracked | Project banner image |

---

## 🛠️ Internal & Future Development Setup

For internal developers and future development workflows, a complete pre-downloaded legacy checkout is available locally:

- **Legacy Checkout Location**: `../doom-the-roughlike-original` (full path: `/Users/saehwan/repos/doom-the-roughlike-original`).

### Automated Import & Preparation

To copy all legacy assets (graphics, fonts, sound, and music) into this repository:

```bash
# Prepares all asset categories from ../doom-the-roughlike-original
sh scripts/prepare-legacy-assets.sh

# Or explicitly pass the path:
sh scripts/prepare-legacy-assets.sh /Users/saehwan/repos/doom-the-roughlike-original
```

### Modular Import Scripts

You can also prepare specific asset categories individually:

```bash
# Graphics (imports CC BY-SA 4.0 graphics from pinned git revision)
sh scripts/import-legacy-graphics.sh

# Bitmap Fonts (copies font10x19.png, default, font.dat to assets/legacy/drl/fonts/)
sh scripts/import-legacy-fonts.sh

# Sound Effects (copies HQ and LQ sound files)
sh scripts/import-legacy-sound.sh

# Music Tracks (copies HQ MP3 and LQ MIDI music files)
sh scripts/import-legacy-music.sh
```

---

## 🌐 Public Player & Contributor Instructions

For public players and external contributors, sound, music, and font assets must be downloaded and extracted directly from the official DRL game releases:

1. **Download Official DRL Binaries**:
   - Visit the official ChaosForge portal: [https://drl.chaosforge.org/](https://drl.chaosforge.org/) or [ChaosForge](https://chaosforge.org/).
   - Download the official full release package (e.g. `doomrl-win-0997.zip`, `doomrl-linux-x64-0.10.0.tar.gz`, or equivalent).

2. **Extract Archive**:
   - Extract the downloaded archive into a folder on your computer.

3. **Import Assets into drl-rs**:
   - Run the automated preparation script pointing to your extracted folder:
     ```bash
     sh scripts/prepare-legacy-assets.sh /path/to/extracted-doomrl-folder
     ```
   - Alternatively, manually copy the asset folders:
     - `data/drl/fonts/*` → `assets/legacy/drl/fonts/`
     - `data/drlhq/music/*` → `assets/legacy/drlhq/music/`
     - `data/drlhq/sound/*` → `assets/legacy/drlhq/sound/`
     - `data/drllq/music/*` → `assets/legacy/drllq/music/`
     - `data/drllq/sound/*` → `assets/legacy/drllq/sound/`

---

## 🛡️ Verification & Integrity Checks

To verify asset integrity, license evidence, and Git tracking boundaries:

```bash
# Standard check: verifies tracked graphics and confirms no untracked assets are committed to git
sh scripts/check-assets.sh

# Complete check: verifies tracked graphics, git safety, AND all optional local assets
sh scripts/check-assets.sh --all
```
