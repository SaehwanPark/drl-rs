# Combat Pistol typed behavior-profile evidence

Status: delivered ordinary-fire profile (`0.2.220`) and Combat Pistol aimed
fire vertical slice (`0.2.245`); exact legacy callback state/timing,
controlled runtime comparison, and audiovisual parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/eitems.lua:171-201` declares `ucpistol` (displayed as
  “combat pistol”) as a ranged weapon using the `ammo` family. It does not
  declare a multi-shot `shots` count or a `shotcost`; its `OnCreate` callback
  attaches `perk_altfire_aimed`.
- `bin/data/drl/perks.lua:128-169` defines the shared aimed mode as a +3
  to-hit bonus and doubled fire time; its callback clears `pp_aimed` after
  firing at `bin/data/drl/perks.lua:151-154`. The Rust slice exposes this as
  the typed `AttackRangedAimed` command for both pistol families without
  claiming callback-state parity.
- `src/dfitem.pas:249-252` defaults absent `shots` and `shotcost` fields to
  zero, and `src/dfbeing.pas:1477-1481` resolves ordinary fire with
  `iShots := Max(aGun.Shots, 1)`, so the ordinary path emits one projectile.
- `src/dfitem.pas:627-634` applies `math.Max(ShotCost, 1)` and multiplies by
  the resolved shot count; with the absent `shotcost` default and one ordinary
  shot, the legacy cost is one round before any callback multiplier.
- The Rust definition maps the stable family to `AmmoType::Ammo9mm`; ordinary
  ranged execution consumes one clip round after complete target, line-of-
  sight, range, and death-drop preflight.

## DRL-Rust boundary

The immutable `drl_core::behavior::COMBAT_PISTOL_BEHAVIOR` profile records
ordered `AttackEffect::ProjectileCount(1)` and
`ResourceCost::Ammo { ammo_type: Ammo9mm, amount: 1 }` fragments plus an
`AlternateAction::AimedFire { accuracy_bonus: 3, fire_cost_multiplier: 2 }`
fragment. Generic ranged execution remains authoritative for legality checks,
damage RNG, event ordering, and transactional clip consumption; direct-core,
replay/MCP JSON/catalog, and `BrowserSession` tests verify the aimed boundary.

A controlled legacy runtime comparison, browser capture, and audiovisual parity
remain deferred and are not inferred from source similarity alone.
