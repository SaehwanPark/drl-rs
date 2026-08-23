# Release-rights inventory

This inventory records the redistribution boundary for the current DRL-Rust
source tree and static bundle. It is evidence tracking, not a legal opinion or
a grant of rights. `scripts/check-release-rights.sh` validates the machine-
checkable fields and rejects excluded material from an available bundle.

For the current provenance review priorities, also see
[`docs/steering/current-priorities.md`](steering/current-priorities.md).

## Machine inventory

```text
category: project-authored
status: INCLUDED
license: MIT
source_path: Rust and web sources in this repository

category: bundled-legacy-graphics
status: INCLUDED
license: CC BY-SA 4.0
source_revision: 17d9be1204751899b2d69d8d3a2dde247bd0cc5c
source_path: assets/legacy/drl/graphics
license_path: assets/legacy/drl/graphics/LICENSE
manifest_path: assets/legacy/drl/graphics/MANIFEST.txt
checksum_path: assets/legacy/drl/graphics/SHA256SUMS

category: legacy-code
status: EXCLUDED
license: NOT_CLEARED
source_path: legacy Pascal/Lua references and assets/legacy-drlhq/**

category: legacy-audio-music-fonts
status: EXCLUDED
license: NOT_CLEARED
source_path: assets/legacy-drlhq/sound, assets/legacy-drlhq/music, legacy fonts

category: captures-and-media
status: NOT_RUN
license: INCONCLUSIVE
source_path: controlled reference captures and replacement media

category: third-party-dependencies
status: NOTICE-ONLY
license: dependency metadata scope; no blanket redistribution clearance
source_path: Cargo.lock and package metadata
```

## Review-only provenance gap: legacy-derived creative text

The machine inventory above remains unchanged so the existing automated rights
check keeps its current contract. Separately, legacy-derived names,
descriptions, flavor text, and other creative expression copied into Rust or
content definitions require an explicit provenance/redistribution review.
Treat this question as `INCONCLUSIVE` until the relevant source, license, and
intended redistribution basis are recorded. Numeric mechanics, identifiers,
and creative text should not be assumed to share the same rights analysis.

This section records a review requirement only; it is not a legal conclusion
and does not change the status of the machine-checkable categories above.

## Boundary decisions

- The bundled graphics are copied from the pinned legacy revision and retain
  their upstream CC BY-SA 4.0 license and attribution. The repository MIT
  license does not relicense them.
- Legacy Pascal/Lua code, legacy sound and music, fonts, WADs, and reference
  capture media are excluded from the static bundle until separate evidence
  supports redistribution. Their presence as reference material in the
  source tree is not a clearance claim.
- Legacy-derived creative text embedded in project-authored data or Rust
  definitions is reviewed separately from code mechanics before a release
  claims its redistribution basis is complete.
- Generated release manifests continue to declare exactly
  `assets/legacy/drl/graphics/LICENSE` as the bundle rights file. Full artifact
  hashes, service-worker coverage, and optional signatures remain governed by
  `scripts/check-release-manifest.sh`.
- Unavailable bundle builds are reported as `NOT_RUN`; source provenance does
  not imply production deployment, audiovisual parity, or legal clearance.
