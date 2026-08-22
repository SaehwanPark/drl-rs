# DRL reference-capture manifest

Status: `NOT_RUN` on 2026-08-21.

The legacy checkout was dirty at inspection time (local audio/meta changes and
an untracked directory); those changes are excluded. The available ignored
legacy executable is an x86-64 Linux ELF while the acceptance host is arm64
macOS. Capture execution must happen in a controlled
Linux x86-64 VM/container with the exact legacy revision and configuration;
running the dirty checkout or inventing screenshots would invalidate the
comparison. `scripts/record-legacy-reference.sh` records the required metadata
and writes media only below ignored `_workspace/`.

The machine-readable manifest also records `legacy_dirty_state=clean` or
`dirty` (and uses `unavailable` when the checkout cannot be inspected). A
`READY_FOR_CONTROLLED_CAPTURE`, `INCONCLUSIVE`, or `PASS` record must say
`clean`; the current dirty checkout remains `NOT_RUN`.

Each record carries `evidence_classification` using the proposal vocabulary:
`observed`, `inferred`, `implementation-artifact`, `ambiguous`, or
`drl-rust-decision`. Promotable statuses require `observed`; a non-observed
classification cannot become capture evidence by inference.

The manifest also records `rights_status` as `not-assessed`, `cleared`,
`unclear`, or `rejected`, plus comma-separated media hashes in the form
`sha256:<64-hex>`. `PASS` requires `rights_status=cleared` and valid hashes;
`READY_FOR_CONTROLLED_CAPTURE` may retain `not-assessed`, while unresolved
records remain `INCONCLUSIVE` or `NOT_RUN`.

Required scenes:

| Scene | Media | Rights/acceptance |
| --- | --- | --- |
| map lighting and fog | still + short video | still may be committed only when CC-cleared; video stays ignored |
| targeting and ranged combat | still + video + cue timing | audio/video stay ignored until rights clear |
| knockback and death | still + video | event timing recorded |
| low-health treatment | still | compare glow/tint tolerance |
| inventory and HUD | still | font rights recorded before bundling |
| level transition | still + video + music cue | music rights recorded before bundling |

Every record must include legacy revision, dirty state, evidence
classification, rights status, executable SHA-256, build/config flags,
front-end, viewport, actions, capture tool versions, media hashes, and
provenance.
