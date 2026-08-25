# Jackhammer alternate fire-mode evidence

Status: behavior target for `0.2.129`; controlled runtime and presentation
comparison remain `NOT_RUN`.

## Source identity

- Legacy repository: `doom-the-roughlike-original`
- Revision: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`
- Source: `bin/data/drl/items/uitems.lua:548-602`
- Relevant callback: `perk_usjack_altreload`
- Relevant item: `usjack`

## Attributable behavior

The callback toggles `self.shots` from `3` to `1` and from `1` to `3`,
subtracts one score count, and returns success. The item definition starts
with three shots, uses shells, has a ten-shell clip, `8d3` damage, range 15,
spread 2, falloff 5, and knockback 8.

The callback's UI messages and its comment about nominal delay are retained as
evidence only. The current Rust slice maps the selected shot count into the
existing deterministic ranged command; exact legacy spread, falloff, timing,
and audiovisual effects remain open.

## Rust decisions

- `WeaponFireMode` remains a stable protocol enum; Jackhammer transition and
  shot-count policy are core-owned.
- `Command::AltReload` ignores confirmation for Jackhammer, toggles
  `Burst <-> Single`, spends one score count with saturation, and emits
  `JackhammerFireModeChanged`.
- Ranged validation preflights the selected shell count before clip/RNG
  mutation. Accepted selected shots resolve in order and stop at actual lethal
  damage, with at most one death drop.
- Gameplay-semantics replay identity advances to `8` because the selected mode
  changes future command outcomes.

## Rights and provenance

This note records behavior from the pinned source for implementation evidence;
it does not copy runtime Lua into the shipped Rust product or claim runtime
parity. Legacy code remains excluded from the Rust release bundle.
