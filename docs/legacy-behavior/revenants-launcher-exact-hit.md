# Revenant’s Launcher Exact-Hit Evidence

Pinned legacy revision: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

## Observed legacy contract

- `bin/data/drl/items/uitems.lua:632-664` defines `urbazooka` (Revenant’s
  Launcher) with `IF_EXACTHIT`, a one-rocket clip, `7d6` damage, and range 8.
- `src/dfbeing.pas:2317-2344` maps `IF_EXACTHIT` to a 100% ranged to-hit
  result.
- The same item also has separate homing and delayed-explosion behavior in
  `uitems.lua:660-663`; projectile path handling is distinct at
  `src/dfbeing.pas:485` and `:2567-2568`.

## DRL-Rust boundary

Gameplay semantics `34` (project version `0.2.187`) extends the existing typed
exact-hit policy to Revenant’s Launcher. Valid visible, in-range shots bypass
only the to-hit RNG while retaining clip/action-cost checks, damage RNG, and
existing attack/damage events; invalid commands remain atomic. Homing,
projectile routing, delayed explosions, mods, controlled legacy runtime, and
audio/visual comparison are `NOT_RUN` or separate future slices.
