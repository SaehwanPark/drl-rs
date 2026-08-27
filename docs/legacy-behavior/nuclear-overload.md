# Nuclear Plasma alternate overload evidence

Legacy evidence is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` of
`doom-the-roughlike-original`.

- `bin/data/drl/items/eitems.lua:436-472` defines `unplasma` with a 24-cell
  clip and attaches `perk_altreload_nuke`.
- `bin/data/drl/perks.lua:223-249` rejects stairs, delegates full-clip and
  confirmation checks to `can_overcharge`, arms a one-tick nuke on hazards or
  a 100-tick nuke elsewhere, marks the item for destruction, and spends 1,000
  score count.
- `bin/data/core/item.lua:115-133` defines the full-clip/confirmation and
  already-armed preconditions; `src/dfbeing.pas:871-890` removes a successful
  destroy-after-action item.
- `src/dflevel.pas:1422-1481` additionally performs legacy `NukeRun` map
  destruction and random explosions. Those effects are outside this bounded
  Rust slice.

DRL-Rust preflights the equipped Nuclear Plasma Rifle, confirmation, clip,
stairs, and pending-nuke state before changing score or equipment. It emits
`GameEvent::NuclearWeaponOverloaded`, arms the existing typed `NukeState`, and
destroys the equipped item. Acid/Lava uses countdown `1`; other tiles use
countdown `100`. Legacy `NukeRun` map mutation, random area effects, exact
runtime timing, and audiovisual parity remain open.
