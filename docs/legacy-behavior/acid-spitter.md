# Acid Spitter reload evidence

## Pinned source

- Legacy checkout: `/Users/saehwan/repos/doom-the-roughlike-original`
- Revision: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`
- Callback: `bin/data/drl/items/uitems.lua:666-685`
- Item definition: `bin/data/drl/items/uitems.lua:687-725`
- Terrain definitions: `bin/data/drl/cells.lua:387-449`

## Supported facts

The `perk_uacid` `OnPreReload` callback rejects a full clip. Otherwise it
checks the actor's current cell: on Acid it adds one round up to the clip cap,
subtracts 1000 score count, and changes the cell to Water; on any other cell it
does not load ammunition. The Acid Spitter definition uses rockets, a ten
round clip, and initializes its ammunition to zero.

The source marks both Acid and Water as walkable liquid cells. Acid also has
hazard damage/resistance and movement behavior, while Water has fluid behavior;
those effects are not part of this slice.

## Rust boundary

The typed transition owns the Acid-to-Water terrain change, one-round clip
increment, and saturating score policy. `AcidSpitterReloaded` makes the
accepted transition replay- and MCP-visible without adding a runtime callback
registry. The existing reload action cost remains Rust policy rather than a
claim of exact legacy timing.

Acid hazard damage, resistance equations, fluid movement cost, runtime Lua,
controlled legacy comparison, and audiovisual parity are `NOT_RUN`.
