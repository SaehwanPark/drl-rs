# Combat Shotgun typed behavior-profile evidence

Status: delivered typed behavior profile for `0.2.212`; exact legacy timing,
controlled runtime comparison, chamber presentation, and audiovisual parity
remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/items.lua` declares `ashotgun` with
  `IF_SINGLERELOAD`, a five-shell capacity, and the pump-action callback.
- `bin/data/drl/perks.lua` defines the alternate full-reload callback, which
  fills the complete deficit and caps cumulative shell reload cost at `2,500`.
- `src/dfbeing.pas` routes ordinary flagged reload through the one-shell
  `TBeing.Reload` path before reserve mutation.

## DRL-Rust boundary

The immutable `drl_core::behavior::COMBAT_SHOTGUN_BEHAVIOR` profile records
ordered `AlternateAction::Reload` and
`AlternateAction::FullReload { cost_cap: 2500 }` fragments. Dedicated normal
reload, `CombatShotgunTransition`, and pump-action state remain execution
authority for one-shell loading, full-deficit reserve checks, chamber
transitions, capped cost, and transactional rejection behavior. No command,
replay, RNG, or generic callback-dispatch surface is introduced by the profile.

Exact legacy timing, partial-reserve policy, controlled runtime comparison,
chamber presentation, and audiovisual parity remain deferred and are not
inferred from source similarity alone.
