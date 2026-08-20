# ADR 0002 — No Legacy Backward Compatibility

**Status:** Accepted

**Date:** 2026-08-18

---

## Context

The legacy DRL implementation produces save files, mod interfaces, WAD
packaged resources, and a fixed RNG stream that have accumulated over many
years of development. Maintaining compatibility with any of these formats
would impose ongoing costs and architectural constraints that conflict with
DRL-Rust's goals.

At the same time, the question "should we be compatible with X?" will arise
repeatedly as implementation progresses, particularly for:

- legacy `.wad`-style resource files;
- old save game formats;
- legacy mod scripts and their expectations;
- the exact Pascal-level RNG seed-to-outcome mapping.

A clear policy decision avoids relitigating this at each milestone.

---

## Decision

DRL-Rust intentionally and permanently opts out of backward compatibility
with all legacy implementation artifacts:

- **Save files**: DRL-Rust uses its own versioned save format. Legacy Pascal
  saves are not loaded, migrated, or acknowledged.
- **Mod interface**: DRL-Rust's Lua boundary is designed from scratch for
  Rust's ownership and type system. Legacy mod scripts are not guaranteed to
  work without adaptation.
- **WAD resources**: DRL-Rust may read or reference asset data as a research
  input but does not commit to the legacy WAD binary format as a runtime
  format.
- **RNG stream**: DRL-Rust uses an independent deterministic RNG
  (SplitMix64 + Xoshiro256++). The same seed does not produce the same
  sequence of outcomes as the Pascal implementation. Replays from legacy DRL
  are not expected to be reproducible in DRL-Rust.
- **Network or file protocol compatibility**: not targeted.

The legacy Pascal and Lua implementation remains the canonical **behavioral**
reference — its semantics, probabilities, and design intent are preserved
where understood — but its internal formats are not replicated.

---

## Consequences

- No migration code for legacy saves needs to be written or maintained.
- New DRL-Rust saves, replays, and content formats are designed with
  Rust-idiomatic constraints rather than Pascal legacy constraints.
- When legacy behavior is used as evidence, it is documented as such in
  `docs/legacy-behavior/` with explicit uncertainty notes.
- Players with legacy saves will need to start new games in DRL-Rust.
- Mod authors targeting legacy DRL will need to adapt their scripts to
  DRL-Rust's Lua boundary contract when it is established.
