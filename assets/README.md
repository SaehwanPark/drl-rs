# DRL-Rust assets

The tracked browser bundle currently contains only
`legacy/drl/graphics/`, imported from the pinned legacy Git revision recorded
in its `MANIFEST.txt` and `SHA256SUMS`. The upstream graphics license and
attribution are copied in `legacy/drl/graphics/LICENSE` and documented in
`docs/legacy-behavior/asset-provenance.md`.

Legacy fonts, sound, and music are intentionally not imported: their
redistribution rights need separate evidence. Do not add them to the browser
bundle without a provenance and license record. Use `scripts/check-assets.sh`
to verify the cleared graphics import.
