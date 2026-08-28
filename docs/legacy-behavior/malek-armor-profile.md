# Malek's Armor typed behavior-profile evidence

Status: delivered typed behavior profile for `0.2.206`; general armor
resistance/degradation, controlled legacy runtime comparison, and audiovisual
parity remain `NOT_RUN`.

## Pinned source

Evidence is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` of
`doom-the-roughlike-original`:

- `bin/data/drl/items/uitems.lua:806-831` defines `umarmor` with recharge
  delay `50`, cadence `5`, amount `1`, and maximum durability `100`.
- `bin/data/drl/perks.lua:388-421` increments the equipped armor timer while
  durability is below maximum, restores one point at timer `55`, and subtracts
  the cadence from the retained timer. The damage callback resets the timer.

The source evidence establishes the descriptive timing policy; it does not by
itself establish controlled runtime or presentation parity.

## DRL-Rust boundary

The immutable `drl_core::behavior::MALEK_ARMOR_BEHAVIOR` profile records one
typed `PeriodicEffect::DurabilityRecharge` fragment with delay `50`, cadence
`5`, and amount `1`. Dedicated `drl-core::malek_armor::MalekRechargeState`
remains the execution authority for accepted-command ticking, full-durability
timer preservation, damage resets, durability clamping, and the
`GameEvent::MalekArmorRecharged` boundary. This slice adds no command, event,
replay, RNG, or generic callback-dispatch surface.

Armor resistance/degradation policy, exact legacy actor-tick cadence,
controlled legacy runtime capture, and audiovisual parity remain outside this
profile and are not claimed here.
