# Combat Shotgun single-shell reload evidence

## Pinned source

- Legacy repository: `/Users/saehwan/repos/doom-the-roughlike-original`
- Revision inspected: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`
- The legacy checkout had unrelated pre-existing dirty files; no legacy files
  were modified for this evidence record.
- Item declaration: `bin/data/drl/items/items.lua`, `ashotgun`
- Reload dispatch: `src/dfbeing.pas`, `ActionReload` and `TBeing.Reload`

## Attributable behavior

The `ashotgun` declaration carries `IF_SINGLERELOAD` and a five-shell
capacity. Pascal's normal reload dispatch passes that flag into
`TBeing.Reload`; with the flag set, one reload action transfers at most one
matching shell from reserve to the clip. The item also has a separate
`perk_pump_action` callback, but pump-action chamber state is outside this
slice.

## Rust boundary

The typed normal-reload transition now applies the single-shell load limit to
both Assault Shotgun and Combat Shotgun definitions. The accepted Combat
Shotgun transition advances gameplay semantics from `17` to `18`; alternate
reload, pump-action state, ammo-pack behavior, exact legacy timing, controlled
runtime comparison, and presentation parity remain `NOT_RUN`.
