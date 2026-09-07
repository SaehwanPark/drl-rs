# Plasma Shotgun typed behavior-profile evidence

Status: delivered ordinary-fire profile and `0.2.242` direct-core/
`BrowserSession` boundary target; `0.2.346` adds typed direct Plasma
mitigation. Full spread/falloff/knockback semantics, exact legacy timing/
accuracy, controlled runtime comparison, and audiovisual parity remain
`NOT_RUN`.

## Pinned source

The cited legacy repository is `https://github.com/ChaosForge/doomrl.git` at
revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. The local inspection used
an immutable Git object checkout; no working-tree edits were used as evidence.

- `bin/data/drl/items/eitems.lua:233-258` declares `upshotgun` (displayed as
  “plasma shotgun”) as a ranged weapon using the `cell` ammo family, with
  `shotcost = 3`, `7d3` damage, `DAMAGE_PLASMA`, and no explicit multi-shot
  `shots` count. Its spread, falloff, and knockback fields remain outside this
  bounded direct-target slice.
- `src/dfitem.pas:249-252` defaults absent `shots` to zero and preserves the
  explicit `shotcost`; `src/dfbeing.pas:1477-1481` resolves ordinary fire with
  `iShots := Max(aGun.Shots, 1)`, so the ordinary path emits one projectile.
- `src/dfbeing.pas:1532-1537` routes weapons without shotgun or spread flags
  through `HandleShots`; `src/dfbeing.pas:2629-2644` carries the weapon's
  damage dice and `DamageType` into the hit/explosion path. The Rust slice
  intentionally keeps its existing direct-target geometry rather than
  inferring the unresolved spread behavior.
- `src/dfbeing.pas:2167-2173` selects the Plasma resistance family for
  `DAMAGE_PLASMA` and `DAMAGE_SPLASMA`; current Rust's typed actor path applies
  the catalog-defined family resistance before flat armor protection.

## DRL-Rust boundary

The immutable `drl_core::behavior::PLASMA_SHOTGUN_BEHAVIOR` profile records
ordered `AttackEffect::ProjectileCount(1)` and
`ResourceCost::Ammo { ammo_type: Cell, amount: 3 }` fragments. Generic ranged
execution remains authoritative for preflight, transactional clip
consumption, direct damage RNG, and event ordering. In `0.2.346`, successful
ordinary direct target damage is routed through the existing typed Plasma path,
so Blue Armor's catalog-defined 20% resistance applies before its flat
protection. Raw damage and RNG order remain unchanged.

Focused direct-core/replay/rejection, MCP JSON, and BrowserSession tests cover
the current Rust contract. A controlled legacy runtime comparison, browser
capture, audiovisual parity, exact spread/falloff/knockback behavior, and
human-play acceptance remain deferred and are not inferred from source
similarity alone.
