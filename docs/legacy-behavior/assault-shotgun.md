# Assault Shotgun reload evidence

## Pinned source

- Legacy repository: `/Users/saehwan/repos/doom-the-roughlike-original`
- Revision inspected: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`
- The legacy checkout had unrelated pre-existing dirty files; no legacy files
  were modified for this evidence record.
- Item declaration: `bin/data/drl/items/eitems.lua`, `uashotgun`
- Reload dispatch: `src/dfbeing.pas`, `ActionReload` and `ActionDualReload`
- Alternate callback: `bin/data/drl/perks.lua`, `perk_altreload_full`

## Attributable behavior

The `uashotgun` declaration carries `IF_SINGLERELOAD`, a six-shell capacity,
and the `perk_altreload_full` alternate-reload perk. Pascal reload dispatch
passes that flag into `TBeing.Reload`, so ordinary reload is a single-shell
transition for this weapon. The alternate callback is a separate full-reload
policy with a dynamic score-count cap and remains outside this slice.

## Rust boundary

The typed normal-reload transition applies an explicit Assault Shotgun
single-shell load limit without adding a generic callback registry or changing
the replay wire schema. The accepted transition advances gameplay semantics
from `16` to `17`; alternate reload, ammo-pack behavior, exact legacy timing,
runtime comparison, and presentation parity remain `NOT_RUN`.
