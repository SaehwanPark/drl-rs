# Nuclear BFG 9000 Alternate Overload Evidence

The legacy reference is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` in
`/Users/saehwan/repos/doom-the-roughlike-original`.

- `bin/data/drl/items/eitems.lua:474-518` defines `unbfg9000` with a 40-cell
  clip, `IF_NORELOAD`/`IF_NOUNLOAD`, and the `perk_altreload_nuke` callback.
- `bin/data/drl/perks.lua:223-249` rejects stairs, delegates full-clip and
  confirmation checks to `item:can_overcharge`, arms `being:nuke(1)` on a
  hazard or `being:nuke(100)` elsewhere, marks the item for destruction, and
  subtracts 1,000 score count.
- `bin/data/core/item.lua:115-133` requires a full magazine and confirmation,
  rejects an already fire-destroyed item, and marks the item no-unload after
  arming.
- `src/dfbeing.pas:871-890` removes a successfully destroyed alternate-reload
  weapon from the equipped slot.
- `src/dflevel.pas:1422-1481` resolves the countdown and then performs the
  broader `NukeRun` random explosions/map mutations.

DRL-Rust implements the bounded typed preflight and existing abstract
`NukeState` transition for the BFG archetype. `NukeRun` map-wide effects,
random explosion scheduling, nukecell mutation, controlled legacy runtime, and
audiovisual parity remain explicitly unimplemented and are not inferred from
the source evidence.
