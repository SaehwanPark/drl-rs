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

Required scenes:

| Scene | Media | Rights/acceptance |
| --- | --- | --- |
| map lighting and fog | still + short video | still may be committed only when CC-cleared; video stays ignored |
| targeting and ranged combat | still + video + cue timing | audio/video stay ignored until rights clear |
| knockback and death | still + video | event timing recorded |
| low-health treatment | still | compare glow/tint tolerance |
| inventory and HUD | still | font rights recorded before bundling |
| level transition | still + video + music cue | music rights recorded before bundling |

Every record must include legacy revision, executable SHA-256, build/config
flags, front-end, viewport, actions, capture tool versions, hashes, and an
observed/inferred/provenance classification.
