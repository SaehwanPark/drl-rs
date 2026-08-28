# Combat Shotgun reload and pump-action evidence

Status: delivered typed reload behavior profile through `0.2.212`; controlled
legacy runtime comparison and audiovisual parity remain `NOT_RUN`.

## Pinned source

- Legacy repository: `/Users/saehwan/repos/doom-the-roughlike-original`
- Revision inspected: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`
- The legacy checkout had unrelated pre-existing dirty files; no legacy files
  were modified for this evidence record.
- Item declaration: `bin/data/drl/items/items.lua`, `ashotgun`
- Alternate callback: `bin/data/drl/perks.lua`, `perk_altreload_full`
- Reload dispatch: `src/dfbeing.pas`, `ActionReload` and `TBeing.Reload`

## Attributable behavior

The `ashotgun` declaration carries `IF_SINGLERELOAD`, a five-shell capacity,
and the `perk_pump_action` callback. Pascal's normal reload dispatch passes
that flag into `TBeing.Reload`; with the flag set, one reload action transfers
at most one matching shell from reserve to the clip.

The pinned Lua callback establishes the chamber transitions: firing marks the
chamber empty, firing is rejected while the chamber is empty and clip ammo
remains, accepted equipped movement chambers a round without extra cost, and
an empty-chamber reload pumps for 200 speed units without consuming reserve
ammo. When the clip is empty, normal reload loads one shell and clears the
chamber. These hooks run after successful movement, after shot resolution, and
before normal reload clip/ammo checks respectively.

The same item also carries `perk_altreload_full`. Its alternate callback clears
the empty chamber, loads the complete clip deficit from loose shells, and caps
cumulative shell reload cost at 2,500 units.

## Rust boundary

The typed normal-reload transition applies the single-shell load limit to both
Assault Shotgun and Combat Shotgun definitions. Combat Shotgun instances also
own explicit chamber state: the direct core rejects an empty-chamber shot
atomically, accepted movement or a pump-only reload chambers it, and a regular
one-shell reload restores it after an empty clip.

The dedicated `combat_shotgun` alternate transition preflights the complete
deficit and requires enough loose `AmmoShells` before mutating inventory or
clip. Success consumes exactly the deficit, emits one aggregate
`WeaponReloaded` event, and uses `min(deficit * reload_cost, 2500)`. The
item-owned pump state is reset as part of the successful alternate reload, so
the next shot is accepted without an extra pump command. Full and
under-supplied clips reject atomically. Gameplay semantics advance from `20`
to `21`; ammo-pack behavior, partial-reserve policy, exact legacy timing,
controlled runtime comparison, and chamber presentation/audio remain
`NOT_RUN`.
