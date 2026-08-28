# Assault Shotgun typed behavior-profile evidence

Status: delivered typed behavior profile for `0.2.214`; exact legacy timing,
partial-reserve policy, controlled runtime comparison, and audiovisual parity
remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/eitems.lua` declares `uashotgun` with
  `IF_SINGLERELOAD`, a six-shell capacity, and the `perk_altreload_full`
  alternate-reload perk.
- `src/dfbeing.pas` routes ordinary flagged reload through the one-shell
  `TBeing.Reload` path before reserve mutation.
- `bin/data/drl/perks.lua` defines the alternate callback as a complete
  deficit reload with a cumulative score-count cost cap of `2,500`.

## DRL-Rust boundary

The immutable `drl_core::behavior::ASSAULT_SHOTGUN_BEHAVIOR` profile records
ordered `AlternateAction::Reload` and
`AlternateAction::FullReload { cost_cap: 2500 }` fragments. Dedicated normal
reload and `AssaultShotgunTransition` planner paths remain execution authority
for one-shell loading, full-deficit reserve checks, capped cost, and
transactional rejection behavior. No command, replay, RNG, or generic
callback-dispatch surface is introduced by the profile.

Exact legacy timing, partial-reserve policy, controlled runtime comparison, and
audiovisual parity remain deferred and are not inferred from source similarity
alone.
