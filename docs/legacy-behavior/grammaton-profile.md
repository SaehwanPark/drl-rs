# Grammaton Cleric Beretta typed behavior-profile evidence

Status: delivered typed behavior profile for `0.2.209`; legacy accuracy
equations, exact timing, controlled legacy runtime comparison, and
audiovisual parity remain `NOT_RUN`.

## Pinned source

Evidence is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` of
`doom-the-roughlike-original`:

- `bin/data/drl/items/uitems.lua:478-508` defines the
  `perk_uberetta_altreload` callback.
- `bin/data/drl/items/uitems.lua:510-541` defines the `uberetta` item and its
  surrounding weapon scalars.

The callback cycles Single, Burst, and Auto modes, applies the pinned
200-point score-count cost, and returns success. The existing typed transition
also carries the mode-specific shot counts and damage/accuracy policy; this
profile records only the mode alternatives and cost contract.

## DRL-Rust boundary

The immutable `drl_core::behavior::GRAMMATON_BEHAVIOR` profile records ordered
`AlternateAction::Fire` fragments for `WeaponFireMode::Single`, `Burst`, and
`Auto`, followed by a `ResourceCost::Score` of `200`. Dedicated
`GrammatonTransition` remains the execution authority for cycling modes,
updating mode-specific weapon properties, and preserving the existing command
and event boundary. This slice adds no command, replay, RNG, or generic
callback-dispatch surface.

Legacy accuracy equations, exact timing, controlled legacy runtime capture, and
audiovisual parity remain outside this profile and are not claimed here.
