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
transition for this weapon. The alternate callback performs a complete deficit
reload when enough loose shells are available and caps the resulting speed
cost at `2500` units.

## Rust boundary

The typed normal-reload transition applies an explicit Assault Shotgun
single-shell load limit without adding a generic callback registry or changing
the replay wire schema, while the dedicated alternate-reload transition
preflights and fills the complete clip deficit atomically. A successful
alternate reload emits one aggregate `WeaponReloaded` event and pays
`min(deficit * reload_cost, 2500)`; under-supplied and full clips reject before
mutation. Gameplay semantics advance from `19` to `20`; ammo-pack behavior,
partial-reserve policy, exact legacy timing, runtime comparison, and
presentation parity remain `NOT_RUN`.
