# Jackhammer typed behavior-profile evidence

Status: delivered typed behavior profile for `0.2.208`; spread/falloff,
exact timing/accuracy, controlled legacy runtime comparison, and audiovisual
parity remain `NOT_RUN`.

## Pinned source

Evidence is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` of
`doom-the-roughlike-original`:

- `bin/data/drl/items/uitems.lua:548-602` defines `usjack` and its
  `perk_usjack_altreload` callback.

The callback toggles the selected shot mode between three-shot Burst and
single-shot Single, subtracts one score count, and returns success. The item
definition supplies the surrounding shell, clip, damage, range, spread,
falloff, and knockback scalars; this profile records only the mode and cost
contract.

## DRL-Rust boundary

The immutable `drl_core::behavior::JACKHAMMER_BEHAVIOR` profile records
ordered `AlternateAction::Fire(WeaponFireMode::Single)` and
`AlternateAction::Fire(WeaponFireMode::Burst)` fragments followed by a
`ResourceCost::Score` of one. Dedicated `JackhammerTransition` remains the
execution authority for the Burst/Single toggle and the existing command/event
boundary. This slice adds no command, replay, RNG, or generic callback-dispatch
surface.

Spread/falloff, exact legacy timing/accuracy, controlled legacy runtime
capture, and audiovisual parity remain outside this profile and are not
claimed here.
