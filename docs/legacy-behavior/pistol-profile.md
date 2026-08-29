# Pistol typed behavior-profile evidence

Status: delivered typed ordinary profile (`0.2.218`) and a Pistol-only aimed
fire vertical slice (`0.2.244`); exact legacy callback state/timing,
controlled runtime comparison, and audiovisual parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/items.lua:564-566` declares `pistol` as a ranged weapon
  using the `ammo` family and attaches the `perk_altfire_aimed` callback.
- `bin/data/drl/perks.lua:128-169` defines aimed mode as a +3 to-hit bonus,
  doubles fire time, and clears the armed state after firing. The Rust slice
  exposes the accepted result as an explicit typed `AttackRangedAimed` command;
  it does not claim callback-state parity.
- `src/dfbeing.pas:892-960` invokes the alternate-fire callback before
  `FireRanged`, while `src/dfbeing.pas:1493-1515` validates cost/ammunition
  before mutation. The fired hook dispatches at `src/dfbeing.pas:1544-1545`,
  and the aimed perk clears `pp_aimed` in `bin/data/drl/perks.lua:151-154`.
- `src/dfitem.pas:247-252` defaults an absent `shots` field to zero, and
  `src/dfbeing.pas:1477-1480` resolves ordinary ranged fire with
  `iShots := Max(aGun.Shots, 1)`, so this path emits one projectile.
- The Rust definition maps the stable family to `AmmoType::Ammo9mm`; ordinary
  ranged execution consumes one clip round after complete target, line-of-
  sight, range, and death-drop preflight.

## DRL-Rust boundary

The immutable `drl_core::behavior::PISTOL_BEHAVIOR` profile records ordered
`AttackEffect::ProjectileCount(1)` and
`ResourceCost::Ammo { ammo_type: Ammo9mm, amount: 1 }` fragments plus an
`AlternateAction::AimedFire { accuracy_bonus: 3, fire_cost_multiplier: 2 }`
fragment. `Command::AttackRangedAimed` applies those typed values only for the
Pistol, then delegates legality checks, damage RNG, event ordering, and
transactional clip consumption to generic ranged execution. Direct-core,
replay/MCP JSON, and `BrowserSession` tests verify parity and atomic rejection
for non-Pistols and empty clips.

A controlled legacy runtime comparison, browser capture, and audiovisual parity
remain deferred and are not inferred from source similarity alone.
