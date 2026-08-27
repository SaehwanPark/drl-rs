# Malek’s Armor recharge evidence

Legacy evidence is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` of
`doom-the-roughlike-original`.

- `bin/data/drl/items/uitems.lua:806-831` defines `umarmor` with recharge
  delay `50`, cadence `5`, amount `1`, and maximum durability `100`.
- `bin/data/drl/perks.lua:388-421` increments the equipped armor timer only
  while durability is below maximum, restores one point at timer `55`, then
  subtracts the cadence from the retained timer. Its damage callback resets
  the timer.

DRL-Rust models accepted player commands as the deterministic scheduler
boundary. `drl-core::malek_armor` owns the timer transition; equipped Malek’s
Armor emits `GameEvent::MalekArmorRecharged` at durability restoration, and
rejected commands roll the complete state back. Full durability leaves the
timer unchanged, and received actor damage resets it. General armor durability
degradation/resistance, exact legacy actor-tick cadence, controlled legacy
runtime capture, and audiovisual parity are not implemented in this slice.
