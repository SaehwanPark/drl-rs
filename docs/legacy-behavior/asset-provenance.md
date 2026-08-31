# Legacy graphics provenance

The imported files under `assets/legacy/drl/graphics/` come only from Git
revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c` of the legacy checkout,
not from its dirty working tree. The upstream graphics license identifies
Derek Yu and Łukasz Śliwiński and grants CC BY-SA 4.0. The copied `LICENSE`,
`MANIFEST.txt`, and `SHA256SUMS` files are the machine-checkable attribution
and revision record.

The repository MIT license does not relicense these legacy graphics. Sound
effects, music tracks, and bitmap fonts are approved for in-game use when
downloaded separately from official DRL binaries or prepared locally via
`scripts/prepare-legacy-assets.sh`, but must remain untracked by Git and excluded
from static release bundles. Legacy code (GPL) remains excluded.
