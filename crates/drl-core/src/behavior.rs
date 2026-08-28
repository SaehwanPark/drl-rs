//! Typed, deterministic item behaviors owned by the simulation core.
//!
//! This module intentionally models behavior as explicit state transitions.
//! It is not a callback registry and does not execute legacy Lua.

use drl_protocol::{
  AmmoType, DamageType, EquipmentSlot, HitPoints, ItemArchetype, TileKind, WeaponFireMode,
};

use crate::acid_spitter::{ACID_SPITTER_RELOAD_AMOUNT, ACID_SPITTER_RELOAD_SCORE_COST};
use crate::combat_shotgun::COMBAT_SHOTGUN_ALT_RELOAD_CAP;
use crate::grammaton::GRAMMATON_MODE_SCORE_COST;
use crate::jackhammer::JACKHAMMER_MODE_SCORE_COST;
use crate::malek_armor::{
  MALEK_ARMOR_RECHARGE_AMOUNT, MALEK_ARMOR_RECHARGE_DELAY, MALEK_ARMOR_RECHARGE_TICK,
};
use crate::missile_launcher::MISSILE_LAUNCHER_ALT_RELOAD_CAP;
use crate::null_pointer::{
  NULL_POINTER_BOSS_SCORE_COST, NULL_POINTER_EXPLOSION_DELAY, NULL_POINTER_EXPLOSION_RADIUS,
  NULL_POINTER_MIN_SCORE_COUNT, NULL_POINTER_TARGET_SCORE_COST,
};
use crate::subtle_knife::{
  SUBTLE_KNIFE_HP_COST, SUBTLE_KNIFE_SCORE_COST, SUBTLE_KNIFE_TARGET_DAMAGE,
};
use crate::trigun::{
  TRIGUN_HP_COST, TRIGUN_MAX_HP_COST, TRIGUN_MIN_HP, TRIGUN_MIN_MAX_HP, TRIGUN_NUKE_TIMER,
  TRIGUN_SCORE_COST,
};

/// A compiler-checked behavior fragment. Each variant names one explicit
/// trigger or effect category; there is no string key or runtime callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BehaviorSpec {
  /// A persistent stat or resistance adjustment.
  Passive(PassiveModifier),
  /// An effect applied when an item is equipped.
  Equip(EquipEffect),
  /// The inverse effect applied when an item is unequipped.
  Unequip(EquipEffect),
  /// An effect emitted when an attack is prepared or fired.
  Attack(AttackEffect),
  /// An effect emitted when an attack connects.
  Hit(HitEffect),
  /// An effect emitted when an attack kills its target.
  Kill(KillEffect),
  /// An explicitly typed alternate fire/reload/use action.
  Alternate(AlternateAction),
  /// A deterministic periodic or recharge transition.
  Periodic(PeriodicEffect),
  /// A resource or status cost paid by an accepted action.
  Cost(ResourceCost),
  /// A deterministic target-selection policy.
  Targeting(TargetSelectionPolicy),
}

/// Immutable collection of behavior fragments for one item or actor profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BehaviorProfile {
  specs: &'static [BehaviorSpec],
}

impl BehaviorProfile {
  /// Creates a profile from an immutable compile-time fragment list.
  #[must_use]
  pub const fn new(specs: &'static [BehaviorSpec]) -> Self {
    Self { specs }
  }

  /// Returns the profile's immutable behavior fragments in declaration order.
  #[must_use]
  pub const fn specs(self) -> &'static [BehaviorSpec] {
    self.specs
  }
}

/// A stat or resistance adjustment that can be applied and reversed explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PassiveModifier {
  /// Stat or resistance being adjusted.
  pub stat: PassiveStat,
  /// Signed adjustment amount.
  pub amount: i32,
}

/// Supported passive stat and resistance dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassiveStat {
  /// Accuracy percentage points.
  Accuracy,
  /// Flat armor protection.
  Protection,
  /// Scheduler speed.
  Speed,
  /// Maximum hit points.
  MaxHealth,
  /// Knockback resistance/strength.
  Knockback,
  /// Damage-type resistance percentage points.
  Resistance(DamageType),
}

/// Explicit equip/unequip effects, including typed item-set membership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipEffect {
  /// Apply one passive modifier.
  GrantPassive(PassiveModifier),
  /// Add or remove membership in one typed item set.
  SetMembership(ItemSetId),
  /// Identify the equipment slot affected by the transition.
  Slot(EquipmentSlot),
}

/// Opaque, non-string item-set identity owned by the behavior vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemSetId(u16);

impl ItemSetId {
  /// The set used by the Medical Powerarmor profile.
  pub const MEDICAL_POWERARMOR: Self = Self(1);
  /// The set used by the Trigun profile's explicit weapon membership.
  pub const TRIGUN: Self = Self(2);

  /// Creates a stable numeric set identity for a build-time catalog entry.
  #[must_use]
  pub const fn new(raw: u16) -> Self {
    Self(raw)
  }

  /// Returns the stable numeric identity.
  #[must_use]
  pub const fn as_u16(self) -> u16 {
    self.0
  }
}

/// Effects that can be attached to attack preparation or firing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttackEffect {
  /// Add or subtract an accuracy amount before resolution.
  AccuracyModifier(i32),
  /// Emit a typed number of projectiles.
  ProjectileCount(u32),
  /// Declare an explicit exact-hit policy.
  ExactHit,
}

/// Effects that can be attached to a successful hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HitEffect {
  /// Apply typed damage with an explicit armor policy.
  Damage {
    amount: u32,
    damage_type: DamageType,
    bypass_armor: bool,
  },
  /// Apply deterministic displacement.
  Knockback { distance: u32 },
  /// Schedule a typed delayed explosion with optional knockback metadata.
  ScheduleExplosion {
    delay: u32,
    radius: u32,
    knockback: Option<u32>,
  },
  /// Apply a target-dependent score cost.
  ScoreCost { amount: i32 },
  /// Apply a score branch selected by one explicit target property.
  TargetScoreCost {
    property: TargetProperty,
    matching_amount: i32,
    other_amount: i32,
    minimum: i32,
  },
}

/// Effects that can be attached to a lethal hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KillEffect {
  /// Award or remove score count.
  ScoreDelta { amount: i32 },
  /// Spawn a typed drop archetype.
  Drop(ItemArchetype),
  /// Trigger a typed terminal countdown.
  TriggerNuke { countdown: u32 },
}

/// Explicit alternate action families; behavior remains in dedicated handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlternateAction {
  /// Select a typed alternate fire mode.
  Fire(WeaponFireMode),
  /// Select an alternate reload transition.
  Reload,
  /// Select a full-deficit reload with an explicit action-cost cap.
  FullReload { cost_cap: u32 },
  /// Select Trigun's alternate reload and arm its terminal countdown.
  ReloadAndTriggerNuke { countdown: u32 },
  /// Select an alternate use transition.
  Use,
  /// Select an alternate invoke transition.
  Invoke,
  /// Select a confirmed destructive overload transition.
  Overload,
  /// Reload ammunition while replacing one required terrain with another.
  TerrainReload {
    required_terrain: TileKind,
    resulting_terrain: TileKind,
    amount: u32,
  },
}

/// Explicit periodic and recharge policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PeriodicEffect {
  /// Restore ammunition after a delay and cadence.
  Recharge {
    delay: u32,
    cadence: u32,
    amount: u32,
  },
  /// Restore armor durability after a delay and cadence.
  DurabilityRecharge {
    delay: u32,
    cadence: u32,
    amount: u32,
  },
  /// Restore armor durability at an interval while standing on one terrain.
  TerrainRecharge {
    interval: u32,
    amount: u32,
    terrain: TileKind,
  },
  /// Repair health while spending durability at an interval.
  Repair {
    interval: u32,
    amount: u32,
    durability_cost: u32,
  },
}

/// Explicit action resource costs, including status transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceCost {
  /// Current HP cost with a minimum remaining floor.
  HitPoints { amount: u32, minimum: u32 },
  /// Maximum HP cost with a minimum remaining floor.
  MaxHitPoints { amount: u32, minimum: u32 },
  /// Scheduler energy cost.
  Energy { amount: u32 },
  /// Reserve or clip ammunition cost.
  Ammo { ammo_type: AmmoType, amount: u32 },
  /// Signed score-count cost.
  Score { amount: i32 },
  /// Explicit status application/removal cost.
  Status(StatusCost),
}

/// Status transitions that may be represented as an action cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusCost {
  /// Apply a status to the actor.
  Apply(StatusEffect),
  /// Remove a status from the actor.
  Remove(StatusEffect),
}

/// Typed status vocabulary used by current callback-derived cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusEffect {
  /// Subtle Knife's one-use lockout status.
  Tired,
}

/// Fair/current-state source for deterministic target selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetSource {
  /// Use the player's visibility-filtered observation.
  FairObservation,
  /// Use the current simulation state after visibility/legality checks.
  CurrentSimulation,
}

/// Stable ordering for selected targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetOrder {
  /// Sort by stable entity identity.
  EntityIdAscending,
  /// Sort by distance, then stable entity identity.
  DistanceThenEntityId,
}

/// Explicit target property used by target-dependent effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetProperty {
  /// Whether the selected actor is marked as a boss by core state.
  IsBoss,
}

/// Deterministic target-selection policies over an explicit state source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetSelectionPolicy {
  /// Select one eligible target.
  Single {
    source: TargetSource,
    order: TargetOrder,
  },
  /// Select all eligible visible living targets.
  AllVisibleLiving {
    source: TargetSource,
    order: TargetOrder,
  },
  /// Select the actor's current cell for terrain-fed behavior.
  CurrentCell,
}

const MEDICAL_POWERARMOR_BEHAVIOR_SPECS: &[BehaviorSpec] = &[
  BehaviorSpec::Equip(EquipEffect::SetMembership(ItemSetId::MEDICAL_POWERARMOR)),
  BehaviorSpec::Periodic(PeriodicEffect::Repair {
    interval: MEDICAL_REPAIR_INTERVAL,
    amount: 1,
    durability_cost: 1,
  }),
];

/// Declarative profile for the existing Medical Powerarmor transition.
pub const MEDICAL_POWERARMOR_BEHAVIOR: BehaviorProfile =
  BehaviorProfile::new(MEDICAL_POWERARMOR_BEHAVIOR_SPECS);

const SUBTLE_KNIFE_BEHAVIOR_SPECS: &[BehaviorSpec] = &[
  BehaviorSpec::Alternate(AlternateAction::Invoke),
  BehaviorSpec::Cost(ResourceCost::HitPoints {
    amount: SUBTLE_KNIFE_HP_COST,
    minimum: 1,
  }),
  BehaviorSpec::Cost(ResourceCost::Score {
    amount: SUBTLE_KNIFE_SCORE_COST as i32,
  }),
  BehaviorSpec::Cost(ResourceCost::Status(StatusCost::Apply(StatusEffect::Tired))),
  BehaviorSpec::Targeting(TargetSelectionPolicy::AllVisibleLiving {
    source: TargetSource::FairObservation,
    order: TargetOrder::EntityIdAscending,
  }),
  BehaviorSpec::Hit(HitEffect::Damage {
    amount: SUBTLE_KNIFE_TARGET_DAMAGE,
    damage_type: DamageType::Physical,
    bypass_armor: true,
  }),
];

/// Declarative profile for the existing Subtle Knife transition.
pub const SUBTLE_KNIFE_BEHAVIOR: BehaviorProfile =
  BehaviorProfile::new(SUBTLE_KNIFE_BEHAVIOR_SPECS);

const TRIGUN_BEHAVIOR_SPECS: &[BehaviorSpec] = &[
  BehaviorSpec::Equip(EquipEffect::SetMembership(ItemSetId::TRIGUN)),
  BehaviorSpec::Equip(EquipEffect::Slot(EquipmentSlot::Weapon)),
  BehaviorSpec::Alternate(AlternateAction::ReloadAndTriggerNuke {
    countdown: TRIGUN_NUKE_TIMER,
  }),
  BehaviorSpec::Cost(ResourceCost::HitPoints {
    amount: TRIGUN_HP_COST,
    minimum: TRIGUN_MIN_HP,
  }),
  BehaviorSpec::Cost(ResourceCost::MaxHitPoints {
    amount: TRIGUN_MAX_HP_COST,
    minimum: TRIGUN_MIN_MAX_HP,
  }),
  BehaviorSpec::Cost(ResourceCost::Score {
    amount: TRIGUN_SCORE_COST,
  }),
];

/// Declarative profile for the existing Trigun transition.
pub const TRIGUN_BEHAVIOR: BehaviorProfile = BehaviorProfile::new(TRIGUN_BEHAVIOR_SPECS);

const NULL_POINTER_BEHAVIOR_SPECS: &[BehaviorSpec] = &[
  BehaviorSpec::Targeting(TargetSelectionPolicy::Single {
    source: TargetSource::CurrentSimulation,
    order: TargetOrder::EntityIdAscending,
  }),
  BehaviorSpec::Hit(HitEffect::TargetScoreCost {
    property: TargetProperty::IsBoss,
    matching_amount: NULL_POINTER_BOSS_SCORE_COST,
    other_amount: NULL_POINTER_TARGET_SCORE_COST,
    minimum: NULL_POINTER_MIN_SCORE_COUNT,
  }),
  BehaviorSpec::Hit(HitEffect::ScheduleExplosion {
    delay: NULL_POINTER_EXPLOSION_DELAY,
    radius: NULL_POINTER_EXPLOSION_RADIUS,
    knockback: None,
  }),
];

/// Immutable typed profile for Charch's Null Pointer on-hit transition.
pub const NULL_POINTER_BEHAVIOR: BehaviorProfile =
  BehaviorProfile::new(NULL_POINTER_BEHAVIOR_SPECS);

/// Pinned BFG 10K delayed explosion interval.
pub const BFG10K_EXPLOSION_DELAY: u32 = 25;
/// Pinned BFG 10K delayed explosion radius.
pub const BFG10K_EXPLOSION_RADIUS: u32 = 2;
/// Pinned BFG 10K delayed explosion knockback payload.
pub const BFG10K_EXPLOSION_KNOCKBACK: u32 = 16;

/// Pinned standard BFG 9000 delayed explosion interval.
pub const BFG9000_EXPLOSION_DELAY: u32 = 33;
/// Pinned standard BFG 9000 delayed explosion radius.
pub const BFG9000_EXPLOSION_RADIUS: u32 = 8;
/// Pinned standard BFG 9000 delayed explosion knockback payload.
pub const BFG9000_EXPLOSION_KNOCKBACK: u32 = 16;

/// Pinned Nuclear BFG 9000 delayed explosion interval.
pub const NUCLEAR_BFG9000_EXPLOSION_DELAY: u32 = 33;
/// Pinned Nuclear BFG 9000 delayed explosion radius.
pub const NUCLEAR_BFG9000_EXPLOSION_RADIUS: u32 = 8;
/// Pinned Nuclear BFG 9000 delayed explosion knockback payload.
pub const NUCLEAR_BFG9000_EXPLOSION_KNOCKBACK: u32 = 16;

const BFG10K_BEHAVIOR_SPECS: &[BehaviorSpec] = &[
  BehaviorSpec::Attack(AttackEffect::ExactHit),
  // Scatter and projectile routing remain separate from the typed count.
  BehaviorSpec::Attack(AttackEffect::ProjectileCount(5)),
  BehaviorSpec::Hit(HitEffect::ScheduleExplosion {
    delay: BFG10K_EXPLOSION_DELAY,
    radius: BFG10K_EXPLOSION_RADIUS,
    knockback: Some(BFG10K_EXPLOSION_KNOCKBACK),
  }),
  BehaviorSpec::Cost(ResourceCost::Ammo {
    ammo_type: AmmoType::Cell,
    amount: 5,
  }),
];

/// Immutable typed profile for the current BFG 10K five-projectile behavior.
pub const BFG10K_BEHAVIOR: BehaviorProfile = BehaviorProfile::new(BFG10K_BEHAVIOR_SPECS);

const BFG9000_BEHAVIOR_SPECS: &[BehaviorSpec] = &[
  BehaviorSpec::Attack(AttackEffect::ExactHit),
  BehaviorSpec::Attack(AttackEffect::ProjectileCount(1)),
  BehaviorSpec::Hit(HitEffect::ScheduleExplosion {
    delay: BFG9000_EXPLOSION_DELAY,
    radius: BFG9000_EXPLOSION_RADIUS,
    knockback: Some(BFG9000_EXPLOSION_KNOCKBACK),
  }),
  BehaviorSpec::Cost(ResourceCost::Ammo {
    ammo_type: AmmoType::Cell,
    amount: 40,
  }),
];

/// Immutable typed profile for the current standard BFG 9000 one-shot behavior.
pub const BFG9000_BEHAVIOR: BehaviorProfile = BehaviorProfile::new(BFG9000_BEHAVIOR_SPECS);

const NUCLEAR_BFG9000_BEHAVIOR_SPECS: &[BehaviorSpec] = &[
  BehaviorSpec::Attack(AttackEffect::ExactHit),
  BehaviorSpec::Attack(AttackEffect::ProjectileCount(1)),
  BehaviorSpec::Hit(HitEffect::ScheduleExplosion {
    delay: NUCLEAR_BFG9000_EXPLOSION_DELAY,
    radius: NUCLEAR_BFG9000_EXPLOSION_RADIUS,
    knockback: Some(NUCLEAR_BFG9000_EXPLOSION_KNOCKBACK),
  }),
  BehaviorSpec::Cost(ResourceCost::Ammo {
    ammo_type: AmmoType::Cell,
    amount: 40,
  }),
  BehaviorSpec::Alternate(AlternateAction::Overload),
  BehaviorSpec::Periodic(PeriodicEffect::Recharge {
    delay: NUCLEAR_BFG_RECHARGE_DELAY,
    cadence: NUCLEAR_BFG_RECHARGE_TICK,
    amount: NUCLEAR_BFG_RECHARGE_AMOUNT,
  }),
];

/// Immutable typed profile for the current Nuclear BFG 9000 behavior.
pub const NUCLEAR_BFG9000_BEHAVIOR: BehaviorProfile =
  BehaviorProfile::new(NUCLEAR_BFG9000_BEHAVIOR_SPECS);

const NUCLEAR_PLASMA_BEHAVIOR_SPECS: &[BehaviorSpec] = &[
  BehaviorSpec::Alternate(AlternateAction::Overload),
  BehaviorSpec::Periodic(PeriodicEffect::Recharge {
    delay: NUCLEAR_PLASMA_RECHARGE_DELAY,
    cadence: NUCLEAR_PLASMA_RECHARGE_TICK,
    amount: NUCLEAR_PLASMA_RECHARGE_AMOUNT,
  }),
];

/// Immutable typed profile for the current Nuclear Plasma Rifle behavior.
pub const NUCLEAR_PLASMA_BEHAVIOR: BehaviorProfile =
  BehaviorProfile::new(NUCLEAR_PLASMA_BEHAVIOR_SPECS);

const BLASTER_BEHAVIOR_SPECS: &[BehaviorSpec] =
  &[BehaviorSpec::Periodic(PeriodicEffect::Recharge {
    delay: BLASTER_RECHARGE_DELAY,
    cadence: BLASTER_RECHARGE_TICK,
    amount: BLASTER_RECHARGE_AMOUNT,
  })];

/// Immutable typed profile for the current Blaster behavior.
pub const BLASTER_BEHAVIOR: BehaviorProfile = BehaviorProfile::new(BLASTER_BEHAVIOR_SPECS);

const MALEK_ARMOR_BEHAVIOR_SPECS: &[BehaviorSpec] =
  &[BehaviorSpec::Periodic(PeriodicEffect::DurabilityRecharge {
    delay: MALEK_ARMOR_RECHARGE_DELAY,
    cadence: MALEK_ARMOR_RECHARGE_TICK,
    amount: MALEK_ARMOR_RECHARGE_AMOUNT,
  })];

/// Immutable typed profile for the current Malek's Armor behavior.
pub const MALEK_ARMOR_BEHAVIOR: BehaviorProfile = BehaviorProfile::new(MALEK_ARMOR_BEHAVIOR_SPECS);

const LAVA_ARMOR_BEHAVIOR_SPECS: &[BehaviorSpec] =
  &[BehaviorSpec::Periodic(PeriodicEffect::TerrainRecharge {
    interval: LAVA_RECHARGE_INTERVAL,
    amount: LAVA_RECHARGE_AMOUNT,
    terrain: TileKind::Lava,
  })];

/// Immutable typed profile for the current Lava Armor behavior.
pub const LAVA_ARMOR_BEHAVIOR: BehaviorProfile = BehaviorProfile::new(LAVA_ARMOR_BEHAVIOR_SPECS);

const JACKHAMMER_BEHAVIOR_SPECS: &[BehaviorSpec] = &[
  BehaviorSpec::Alternate(AlternateAction::Fire(WeaponFireMode::Single)),
  BehaviorSpec::Alternate(AlternateAction::Fire(WeaponFireMode::Burst)),
  BehaviorSpec::Cost(ResourceCost::Score {
    amount: JACKHAMMER_MODE_SCORE_COST,
  }),
];

/// Immutable typed profile for the current Jackhammer burst/single toggle.
pub const JACKHAMMER_BEHAVIOR: BehaviorProfile = BehaviorProfile::new(JACKHAMMER_BEHAVIOR_SPECS);

const GRAMMATON_BEHAVIOR_SPECS: &[BehaviorSpec] = &[
  BehaviorSpec::Alternate(AlternateAction::Fire(WeaponFireMode::Single)),
  BehaviorSpec::Alternate(AlternateAction::Fire(WeaponFireMode::Burst)),
  BehaviorSpec::Alternate(AlternateAction::Fire(WeaponFireMode::Auto)),
  BehaviorSpec::Cost(ResourceCost::Score {
    amount: GRAMMATON_MODE_SCORE_COST,
  }),
];

/// Immutable typed profile for the current Grammaton Single/Burst/Auto cycle.
pub const GRAMMATON_BEHAVIOR: BehaviorProfile = BehaviorProfile::new(GRAMMATON_BEHAVIOR_SPECS);

const ACID_SPITTER_BEHAVIOR_SPECS: &[BehaviorSpec] = &[
  BehaviorSpec::Alternate(AlternateAction::TerrainReload {
    required_terrain: TileKind::Acid,
    resulting_terrain: TileKind::Water,
    amount: ACID_SPITTER_RELOAD_AMOUNT,
  }),
  BehaviorSpec::Cost(ResourceCost::Score {
    amount: ACID_SPITTER_RELOAD_SCORE_COST,
  }),
];

/// Immutable typed profile for the current Acid Spitter terrain reload.
pub const ACID_SPITTER_BEHAVIOR: BehaviorProfile =
  BehaviorProfile::new(ACID_SPITTER_BEHAVIOR_SPECS);

const MISSILE_LAUNCHER_BEHAVIOR_SPECS: &[BehaviorSpec] = &[
  BehaviorSpec::Alternate(AlternateAction::Reload),
  BehaviorSpec::Alternate(AlternateAction::FullReload {
    cost_cap: MISSILE_LAUNCHER_ALT_RELOAD_CAP,
  }),
];

/// Immutable typed profile for the current Missile Launcher reload policies.
pub const MISSILE_LAUNCHER_BEHAVIOR: BehaviorProfile =
  BehaviorProfile::new(MISSILE_LAUNCHER_BEHAVIOR_SPECS);

const COMBAT_SHOTGUN_BEHAVIOR_SPECS: &[BehaviorSpec] = &[
  BehaviorSpec::Alternate(AlternateAction::Reload),
  BehaviorSpec::Alternate(AlternateAction::FullReload {
    cost_cap: COMBAT_SHOTGUN_ALT_RELOAD_CAP,
  }),
];

/// Immutable typed profile for the current Combat Shotgun reload policies.
pub const COMBAT_SHOTGUN_BEHAVIOR: BehaviorProfile =
  BehaviorProfile::new(COMBAT_SHOTGUN_BEHAVIOR_SPECS);

/// Medical Powerarmor begins repair only while durability is strictly above
/// the legacy callback's `20`-point guard.
pub const MEDICAL_REPAIR_MIN_DURABILITY_EXCLUSIVE: u32 = 20;
/// Number of eligible item ticks before one HP is restored.
pub const MEDICAL_REPAIR_INTERVAL: u32 = 30;
/// Timer value retained after a successful repair.
pub const MEDICAL_REPAIR_TIMER_AFTER_REPAIR: u32 = 20;

/// Number of accepted commands between Lava Armor recharge checks.
pub const LAVA_RECHARGE_INTERVAL: u32 = 5;
/// Maximum durability restored by one Lava Armor recharge.
pub const LAVA_RECHARGE_AMOUNT: u32 = 3;

/// Blaster's pinned recharge delay before the first cell is restored.
pub const BLASTER_RECHARGE_DELAY: u32 = 30;
/// Blaster's timer cadence retained after each restored cell.
pub const BLASTER_RECHARGE_TICK: u32 = 10;
/// Number of cells restored by one Blaster recharge.
pub const BLASTER_RECHARGE_AMOUNT: u32 = 1;

/// Nuclear Plasma Rifle's pinned recharge delay before the first cell is
/// restored.
pub const NUCLEAR_PLASMA_RECHARGE_DELAY: u32 = 40;
/// Nuclear Plasma Rifle's timer cadence retained after each restored cell.
pub const NUCLEAR_PLASMA_RECHARGE_TICK: u32 = 2;
/// Number of cells restored by one Nuclear Plasma Rifle recharge.
pub const NUCLEAR_PLASMA_RECHARGE_AMOUNT: u32 = 1;

/// Nuclear BFG 9000's pinned recharge delay before the first cell is restored.
pub const NUCLEAR_BFG_RECHARGE_DELAY: u32 = 0;
/// Nuclear BFG 9000's timer cadence retained after each restored cell.
pub const NUCLEAR_BFG_RECHARGE_TICK: u32 = 5;
/// Number of cells restored by one Nuclear BFG 9000 recharge.
pub const NUCLEAR_BFG_RECHARGE_AMOUNT: u32 = 1;

/// Armor-owned state for Lava Armor's periodic durability behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LavaRechargeState {
  timer: u32,
}

impl LavaRechargeState {
  /// Creates a fresh recharge timer.
  #[must_use]
  pub const fn new() -> Self {
    Self { timer: 0 }
  }

  /// Returns the current deterministic recharge timer.
  #[must_use]
  pub const fn timer(self) -> u32 {
    self.timer
  }

  /// Advances one accepted-command Lava Armor tick.
  pub fn tick(
    &mut self,
    on_lava: bool,
    durability: &mut u32,
    max_durability: u32,
  ) -> LavaRechargeOutcome {
    if *durability >= max_durability {
      return LavaRechargeOutcome::Full { timer: self.timer };
    }

    self.timer = self.timer.saturating_add(1);
    if self.timer < LAVA_RECHARGE_INTERVAL {
      return LavaRechargeOutcome::Waiting { timer: self.timer };
    }

    self.timer = 0;
    if !on_lava {
      return LavaRechargeOutcome::NotOnLava { timer: self.timer };
    }

    let restored = LAVA_RECHARGE_AMOUNT.min(max_durability.saturating_sub(*durability));
    *durability = durability.saturating_add(restored).min(max_durability);
    LavaRechargeOutcome::Recharged {
      durability_restored: restored,
      timer: self.timer,
    }
  }
}

/// Observable result of a typed Lava Armor transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LavaRechargeOutcome {
  /// Armor is already full; source behavior leaves the timer untouched.
  Full { timer: u32 },
  /// Interval has not yet elapsed.
  Waiting { timer: u32 },
  /// Interval elapsed on Lava and durability was restored.
  Recharged {
    durability_restored: u32,
    timer: u32,
  },
  /// Interval elapsed away from Lava; timer resets without repair.
  NotOnLava { timer: u32 },
}

/// Explicit timing and amount policy for a rechargeable weapon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeaponRechargePolicy {
  /// Accepted-command delay before the first restoration.
  pub delay: u32,
  /// Accepted-command cadence between restorations.
  pub tick: u32,
  /// Cells restored at each cadence boundary.
  pub amount: u32,
}

impl Default for WeaponRechargePolicy {
  fn default() -> Self {
    Self::blaster()
  }
}

impl WeaponRechargePolicy {
  /// The pinned Blaster recharge policy.
  #[must_use]
  pub const fn blaster() -> Self {
    Self {
      delay: BLASTER_RECHARGE_DELAY,
      tick: BLASTER_RECHARGE_TICK,
      amount: BLASTER_RECHARGE_AMOUNT,
    }
  }

  /// The pinned Nuclear Plasma Rifle recharge policy.
  #[must_use]
  pub const fn nuclear_plasma() -> Self {
    Self {
      delay: NUCLEAR_PLASMA_RECHARGE_DELAY,
      tick: NUCLEAR_PLASMA_RECHARGE_TICK,
      amount: NUCLEAR_PLASMA_RECHARGE_AMOUNT,
    }
  }

  /// The pinned Nuclear BFG 9000 recharge policy.
  #[must_use]
  pub const fn nuclear_bfg() -> Self {
    Self {
      delay: NUCLEAR_BFG_RECHARGE_DELAY,
      tick: NUCLEAR_BFG_RECHARGE_TICK,
      amount: NUCLEAR_BFG_RECHARGE_AMOUNT,
    }
  }
}

/// Weapon-owned state for a typed periodic cell-recharge behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WeaponRechargeState {
  timer: u32,
  policy: WeaponRechargePolicy,
}

impl WeaponRechargeState {
  /// Creates a fresh recharge timer.
  #[must_use]
  pub const fn new() -> Self {
    Self::with_policy(WeaponRechargePolicy::blaster())
  }

  /// Creates a fresh timer with an explicit typed weapon policy.
  #[must_use]
  pub const fn with_policy(policy: WeaponRechargePolicy) -> Self {
    Self { timer: 0, policy }
  }

  /// Returns the current deterministic recharge timer.
  #[must_use]
  pub const fn timer(self) -> u32 {
    self.timer
  }

  /// Advances one accepted-command tick while the weapon is below capacity.
  pub fn tick(&mut self, current_clip: &mut u32, max_clip: u32) -> WeaponRechargeOutcome {
    if *current_clip >= max_clip {
      return WeaponRechargeOutcome::Full { timer: self.timer };
    }

    self.timer = self.timer.saturating_add(1);
    if self.timer < self.policy.delay.saturating_add(self.policy.tick) {
      return WeaponRechargeOutcome::Waiting { timer: self.timer };
    }

    self.timer = self.timer.saturating_sub(self.policy.tick);
    let restored = self
      .policy
      .amount
      .min(max_clip.saturating_sub(*current_clip));
    *current_clip = current_clip.saturating_add(restored).min(max_clip);
    WeaponRechargeOutcome::Recharged {
      ammo_recharged: restored,
      timer: self.timer,
    }
  }

  /// Resets the timer after an accepted shot.
  pub const fn reset(&mut self) {
    self.timer = 0;
  }
}

/// Observable result of a typed weapon recharge transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponRechargeOutcome {
  /// Weapon is already full; source behavior leaves the timer untouched.
  Full { timer: u32 },
  /// Recharge interval has not yet elapsed.
  Waiting { timer: u32 },
  /// One or more cells were restored at the interval boundary.
  Recharged { ammo_recharged: u32, timer: u32 },
}

/// Armor-owned state for the Medical Powerarmor periodic behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MedicalRepairState {
  timer: u32,
}

impl MedicalRepairState {
  /// Creates a fresh timer state.
  #[must_use]
  pub const fn new() -> Self {
    Self { timer: 0 }
  }

  /// Returns the current deterministic repair timer.
  #[must_use]
  pub const fn timer(self) -> u32 {
    self.timer
  }

  /// Advances one eligible Medical Powerarmor tick.
  ///
  /// The source callback nests both health-threshold branches under the
  /// durability guard. Therefore durability at or below `20` leaves the
  /// timer untouched, while healthy actors reset the timer only when the
  /// guard is satisfied. The half-health comparison intentionally uses the
  /// integer division shape documented by the legacy source.
  pub fn tick(&mut self, hit_points: &mut HitPoints, durability: &mut u32) -> MedicalRepairOutcome {
    if *durability <= MEDICAL_REPAIR_MIN_DURABILITY_EXCLUSIVE {
      return MedicalRepairOutcome::DurabilityGuarded { timer: self.timer };
    }

    if hit_points.current < hit_points.max / 2 {
      self.timer = self.timer.saturating_add(1);
      if self.timer < MEDICAL_REPAIR_INTERVAL {
        return MedicalRepairOutcome::Waiting { timer: self.timer };
      }

      let healed = hit_points.heal(1);
      let durability_spent = if healed > 0 {
        *durability = durability.saturating_sub(1);
        1
      } else {
        0
      };
      self.timer = MEDICAL_REPAIR_TIMER_AFTER_REPAIR;
      return MedicalRepairOutcome::Repaired {
        healed,
        durability_spent,
        timer: self.timer,
      };
    }

    self.timer = 0;
    MedicalRepairOutcome::Reset { timer: self.timer }
  }
}

/// Observable result of a typed Medical Powerarmor transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MedicalRepairOutcome {
  /// Durability is at or below the strict source guard; state is preserved.
  DurabilityGuarded { timer: u32 },
  /// The owner remains below half health but the interval is not reached.
  Waiting { timer: u32 },
  /// The owner is healthy enough to reset the interval timer.
  Reset { timer: u32 },
  /// One eligible repair transition completed.
  Repaired {
    healed: u32,
    durability_spent: u32,
    timer: u32,
  },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn behavior_vocabulary_covers_explicit_trigger_categories() {
    const SPECS: &[BehaviorSpec] = &[
      BehaviorSpec::Passive(PassiveModifier {
        stat: PassiveStat::Resistance(DamageType::Fire),
        amount: 25,
      }),
      BehaviorSpec::Equip(EquipEffect::SetMembership(ItemSetId::MEDICAL_POWERARMOR)),
      BehaviorSpec::Unequip(EquipEffect::Slot(EquipmentSlot::Armor)),
      BehaviorSpec::Attack(AttackEffect::ExactHit),
      BehaviorSpec::Hit(HitEffect::Knockback { distance: 1 }),
      BehaviorSpec::Kill(KillEffect::Drop(ItemArchetype::SmallMedPack)),
      BehaviorSpec::Alternate(AlternateAction::Fire(WeaponFireMode::Burst)),
      BehaviorSpec::Alternate(AlternateAction::TerrainReload {
        required_terrain: TileKind::Acid,
        resulting_terrain: TileKind::Water,
        amount: 1,
      }),
      BehaviorSpec::Alternate(AlternateAction::FullReload { cost_cap: 2_500 }),
      BehaviorSpec::Periodic(PeriodicEffect::Recharge {
        delay: 30,
        cadence: 10,
        amount: 1,
      }),
      BehaviorSpec::Periodic(PeriodicEffect::TerrainRecharge {
        interval: 5,
        amount: 3,
        terrain: TileKind::Lava,
      }),
      BehaviorSpec::Cost(ResourceCost::Ammo {
        ammo_type: AmmoType::Cell,
        amount: 5,
      }),
      BehaviorSpec::Targeting(TargetSelectionPolicy::AllVisibleLiving {
        source: TargetSource::FairObservation,
        order: TargetOrder::EntityIdAscending,
      }),
    ];
    let profile = BehaviorProfile::new(SPECS);

    assert_eq!(profile.specs().len(), 13);
    assert!(matches!(
      profile.specs()[0],
      BehaviorSpec::Passive(PassiveModifier {
        stat: PassiveStat::Resistance(DamageType::Fire),
        amount: 25,
      })
    ));
    assert_eq!(ItemSetId::new(42).as_u16(), 42);
  }

  #[test]
  fn selected_stress_profiles_are_immutable_and_deterministically_composed() {
    assert_eq!(
      MEDICAL_POWERARMOR_BEHAVIOR.specs(),
      &[
        BehaviorSpec::Equip(EquipEffect::SetMembership(ItemSetId::MEDICAL_POWERARMOR)),
        BehaviorSpec::Periodic(PeriodicEffect::Repair {
          interval: MEDICAL_REPAIR_INTERVAL,
          amount: 1,
          durability_cost: 1,
        }),
      ]
    );
    assert_eq!(
      SUBTLE_KNIFE_BEHAVIOR.specs(),
      &[
        BehaviorSpec::Alternate(AlternateAction::Invoke),
        BehaviorSpec::Cost(ResourceCost::HitPoints {
          amount: SUBTLE_KNIFE_HP_COST,
          minimum: 1,
        }),
        BehaviorSpec::Cost(ResourceCost::Score {
          amount: SUBTLE_KNIFE_SCORE_COST as i32,
        }),
        BehaviorSpec::Cost(ResourceCost::Status(StatusCost::Apply(StatusEffect::Tired))),
        BehaviorSpec::Targeting(TargetSelectionPolicy::AllVisibleLiving {
          source: TargetSource::FairObservation,
          order: TargetOrder::EntityIdAscending,
        }),
        BehaviorSpec::Hit(HitEffect::Damage {
          amount: SUBTLE_KNIFE_TARGET_DAMAGE,
          damage_type: DamageType::Physical,
          bypass_armor: true,
        }),
      ]
    );
    assert_eq!(
      TRIGUN_BEHAVIOR.specs(),
      &[
        BehaviorSpec::Equip(EquipEffect::SetMembership(ItemSetId::TRIGUN)),
        BehaviorSpec::Equip(EquipEffect::Slot(EquipmentSlot::Weapon)),
        BehaviorSpec::Alternate(AlternateAction::ReloadAndTriggerNuke {
          countdown: TRIGUN_NUKE_TIMER,
        }),
        BehaviorSpec::Cost(ResourceCost::HitPoints {
          amount: TRIGUN_HP_COST,
          minimum: TRIGUN_MIN_HP,
        }),
        BehaviorSpec::Cost(ResourceCost::MaxHitPoints {
          amount: TRIGUN_MAX_HP_COST,
          minimum: TRIGUN_MIN_MAX_HP,
        }),
        BehaviorSpec::Cost(ResourceCost::Score {
          amount: TRIGUN_SCORE_COST,
        }),
      ]
    );
    assert_eq!(
      NULL_POINTER_BEHAVIOR.specs(),
      &[
        BehaviorSpec::Targeting(TargetSelectionPolicy::Single {
          source: TargetSource::CurrentSimulation,
          order: TargetOrder::EntityIdAscending,
        }),
        BehaviorSpec::Hit(HitEffect::TargetScoreCost {
          property: TargetProperty::IsBoss,
          matching_amount: NULL_POINTER_BOSS_SCORE_COST,
          other_amount: NULL_POINTER_TARGET_SCORE_COST,
          minimum: NULL_POINTER_MIN_SCORE_COUNT,
        }),
        BehaviorSpec::Hit(HitEffect::ScheduleExplosion {
          delay: NULL_POINTER_EXPLOSION_DELAY,
          radius: NULL_POINTER_EXPLOSION_RADIUS,
          knockback: None,
        }),
      ]
    );
    assert_eq!(
      BFG10K_BEHAVIOR.specs(),
      &[
        BehaviorSpec::Attack(AttackEffect::ExactHit),
        BehaviorSpec::Attack(AttackEffect::ProjectileCount(5)),
        BehaviorSpec::Hit(HitEffect::ScheduleExplosion {
          delay: BFG10K_EXPLOSION_DELAY,
          radius: BFG10K_EXPLOSION_RADIUS,
          knockback: Some(BFG10K_EXPLOSION_KNOCKBACK),
        }),
        BehaviorSpec::Cost(ResourceCost::Ammo {
          ammo_type: AmmoType::Cell,
          amount: 5,
        }),
      ]
    );
    assert_eq!(
      BFG9000_BEHAVIOR.specs(),
      &[
        BehaviorSpec::Attack(AttackEffect::ExactHit),
        BehaviorSpec::Attack(AttackEffect::ProjectileCount(1)),
        BehaviorSpec::Hit(HitEffect::ScheduleExplosion {
          delay: BFG9000_EXPLOSION_DELAY,
          radius: BFG9000_EXPLOSION_RADIUS,
          knockback: Some(BFG9000_EXPLOSION_KNOCKBACK),
        }),
        BehaviorSpec::Cost(ResourceCost::Ammo {
          ammo_type: AmmoType::Cell,
          amount: 40,
        }),
      ]
    );
    assert_eq!(
      NUCLEAR_BFG9000_BEHAVIOR.specs(),
      &[
        BehaviorSpec::Attack(AttackEffect::ExactHit),
        BehaviorSpec::Attack(AttackEffect::ProjectileCount(1)),
        BehaviorSpec::Hit(HitEffect::ScheduleExplosion {
          delay: NUCLEAR_BFG9000_EXPLOSION_DELAY,
          radius: NUCLEAR_BFG9000_EXPLOSION_RADIUS,
          knockback: Some(NUCLEAR_BFG9000_EXPLOSION_KNOCKBACK),
        }),
        BehaviorSpec::Cost(ResourceCost::Ammo {
          ammo_type: AmmoType::Cell,
          amount: 40,
        }),
        BehaviorSpec::Alternate(AlternateAction::Overload),
        BehaviorSpec::Periodic(PeriodicEffect::Recharge {
          delay: NUCLEAR_BFG_RECHARGE_DELAY,
          cadence: NUCLEAR_BFG_RECHARGE_TICK,
          amount: NUCLEAR_BFG_RECHARGE_AMOUNT,
        }),
      ]
    );
    assert_eq!(
      NUCLEAR_PLASMA_BEHAVIOR.specs(),
      &[
        BehaviorSpec::Alternate(AlternateAction::Overload),
        BehaviorSpec::Periodic(PeriodicEffect::Recharge {
          delay: NUCLEAR_PLASMA_RECHARGE_DELAY,
          cadence: NUCLEAR_PLASMA_RECHARGE_TICK,
          amount: NUCLEAR_PLASMA_RECHARGE_AMOUNT,
        }),
      ]
    );
    assert_eq!(
      BLASTER_BEHAVIOR.specs(),
      &[BehaviorSpec::Periodic(PeriodicEffect::Recharge {
        delay: BLASTER_RECHARGE_DELAY,
        cadence: BLASTER_RECHARGE_TICK,
        amount: BLASTER_RECHARGE_AMOUNT,
      })]
    );
    assert_eq!(
      MALEK_ARMOR_BEHAVIOR.specs(),
      &[BehaviorSpec::Periodic(PeriodicEffect::DurabilityRecharge {
        delay: MALEK_ARMOR_RECHARGE_DELAY,
        cadence: MALEK_ARMOR_RECHARGE_TICK,
        amount: MALEK_ARMOR_RECHARGE_AMOUNT,
      })]
    );
    assert_eq!(
      LAVA_ARMOR_BEHAVIOR.specs(),
      &[BehaviorSpec::Periodic(PeriodicEffect::TerrainRecharge {
        interval: LAVA_RECHARGE_INTERVAL,
        amount: LAVA_RECHARGE_AMOUNT,
        terrain: TileKind::Lava,
      })]
    );
    assert_eq!(
      JACKHAMMER_BEHAVIOR.specs(),
      &[
        BehaviorSpec::Alternate(AlternateAction::Fire(WeaponFireMode::Single)),
        BehaviorSpec::Alternate(AlternateAction::Fire(WeaponFireMode::Burst)),
        BehaviorSpec::Cost(ResourceCost::Score {
          amount: JACKHAMMER_MODE_SCORE_COST,
        }),
      ]
    );
    assert_eq!(
      GRAMMATON_BEHAVIOR.specs(),
      &[
        BehaviorSpec::Alternate(AlternateAction::Fire(WeaponFireMode::Single)),
        BehaviorSpec::Alternate(AlternateAction::Fire(WeaponFireMode::Burst)),
        BehaviorSpec::Alternate(AlternateAction::Fire(WeaponFireMode::Auto)),
        BehaviorSpec::Cost(ResourceCost::Score {
          amount: GRAMMATON_MODE_SCORE_COST,
        }),
      ]
    );
    assert_eq!(
      ACID_SPITTER_BEHAVIOR.specs(),
      &[
        BehaviorSpec::Alternate(AlternateAction::TerrainReload {
          required_terrain: TileKind::Acid,
          resulting_terrain: TileKind::Water,
          amount: ACID_SPITTER_RELOAD_AMOUNT,
        }),
        BehaviorSpec::Cost(ResourceCost::Score {
          amount: ACID_SPITTER_RELOAD_SCORE_COST,
        }),
      ]
    );
    assert_eq!(
      MISSILE_LAUNCHER_BEHAVIOR.specs(),
      &[
        BehaviorSpec::Alternate(AlternateAction::Reload),
        BehaviorSpec::Alternate(AlternateAction::FullReload {
          cost_cap: MISSILE_LAUNCHER_ALT_RELOAD_CAP,
        }),
      ]
    );
    assert_eq!(
      COMBAT_SHOTGUN_BEHAVIOR.specs(),
      &[
        BehaviorSpec::Alternate(AlternateAction::Reload),
        BehaviorSpec::Alternate(AlternateAction::FullReload {
          cost_cap: COMBAT_SHOTGUN_ALT_RELOAD_CAP,
        }),
      ]
    );
  }

  #[test]
  fn waits_then_repairs_and_retains_shortened_timer() {
    let mut state = MedicalRepairState::default();
    let mut hp = HitPoints::new(10, 50);
    let mut durability = 100;

    for expected_timer in 1..MEDICAL_REPAIR_INTERVAL {
      assert_eq!(
        state.tick(&mut hp, &mut durability),
        MedicalRepairOutcome::Waiting {
          timer: expected_timer
        }
      );
    }
    assert_eq!(hp.current, 10);

    assert_eq!(
      state.tick(&mut hp, &mut durability),
      MedicalRepairOutcome::Repaired {
        healed: 1,
        durability_spent: 1,
        timer: MEDICAL_REPAIR_TIMER_AFTER_REPAIR,
      }
    );
    assert_eq!(hp.current, 11);
    assert_eq!(durability, 99);
  }

  #[test]
  fn durability_guard_preserves_timer_and_health() {
    let mut state = MedicalRepairState { timer: 7 };
    let mut hp = HitPoints::new(10, 50);
    let mut durability = MEDICAL_REPAIR_MIN_DURABILITY_EXCLUSIVE;

    assert_eq!(
      state.tick(&mut hp, &mut durability),
      MedicalRepairOutcome::DurabilityGuarded { timer: 7 }
    );
    assert_eq!(state.timer(), 7);
    assert_eq!(hp.current, 10);
    assert_eq!(durability, MEDICAL_REPAIR_MIN_DURABILITY_EXCLUSIVE);
  }

  #[test]
  fn repair_at_the_boundary_spends_durability_to_the_guard() {
    let mut state = MedicalRepairState {
      timer: MEDICAL_REPAIR_INTERVAL - 1,
    };
    let mut hp = HitPoints::new(10, 50);
    let mut durability = MEDICAL_REPAIR_MIN_DURABILITY_EXCLUSIVE + 1;

    assert!(matches!(
      state.tick(&mut hp, &mut durability),
      MedicalRepairOutcome::Repaired {
        healed: 1,
        durability_spent: 1,
        timer: MEDICAL_REPAIR_TIMER_AFTER_REPAIR,
      }
    ));
    assert_eq!(durability, MEDICAL_REPAIR_MIN_DURABILITY_EXCLUSIVE);
    assert_eq!(
      state.tick(&mut hp, &mut durability),
      MedicalRepairOutcome::DurabilityGuarded {
        timer: MEDICAL_REPAIR_TIMER_AFTER_REPAIR,
      }
    );
  }

  #[test]
  fn healthy_owner_resets_timer_only_after_guard_passes() {
    let mut state = MedicalRepairState { timer: 7 };
    let mut hp = HitPoints::new(25, 50);
    let mut durability = 100;

    assert_eq!(
      state.tick(&mut hp, &mut durability),
      MedicalRepairOutcome::Reset { timer: 0 }
    );
    assert_eq!(state.timer(), 0);
  }

  #[test]
  fn full_health_still_reaches_repair_boundary_without_negative_mutation() {
    let mut state = MedicalRepairState {
      timer: MEDICAL_REPAIR_INTERVAL - 1,
    };
    let mut hp = HitPoints::full(50);
    let mut durability = 100;

    assert_eq!(
      state.tick(&mut hp, &mut durability),
      MedicalRepairOutcome::Reset { timer: 0 }
    );
    assert_eq!(hp, HitPoints::full(50));
    assert_eq!(durability, 100);
  }

  #[test]
  fn lava_recharge_waits_five_ticks_then_restores_three() {
    let mut state = LavaRechargeState::default();
    let mut durability = 10;

    for expected_timer in 1..LAVA_RECHARGE_INTERVAL {
      assert_eq!(
        state.tick(true, &mut durability, 20),
        LavaRechargeOutcome::Waiting {
          timer: expected_timer,
        }
      );
    }
    assert_eq!(
      state.tick(true, &mut durability, 20),
      LavaRechargeOutcome::Recharged {
        durability_restored: 3,
        timer: 0,
      }
    );
    assert_eq!(durability, 13);
  }

  #[test]
  fn lava_recharge_clamps_and_non_lava_resets_interval() {
    let mut state = LavaRechargeState {
      timer: LAVA_RECHARGE_INTERVAL - 1,
    };
    let mut durability = 19;
    assert_eq!(
      state.tick(true, &mut durability, 20),
      LavaRechargeOutcome::Recharged {
        durability_restored: 1,
        timer: 0,
      }
    );
    assert_eq!(durability, 20);

    let mut state = LavaRechargeState {
      timer: LAVA_RECHARGE_INTERVAL - 1,
    };
    let mut durability = 10;
    assert_eq!(
      state.tick(false, &mut durability, 20),
      LavaRechargeOutcome::NotOnLava { timer: 0 }
    );
    assert_eq!(durability, 10);
  }

  #[test]
  fn full_lava_armor_preserves_timer_without_sampling_tile() {
    let mut state = LavaRechargeState { timer: 4 };
    let mut durability = 20;
    assert_eq!(
      state.tick(true, &mut durability, 20),
      LavaRechargeOutcome::Full { timer: 4 }
    );
    assert_eq!(state.timer(), 4);
  }

  #[test]
  fn weapon_recharge_waits_for_delay_then_recharges_every_tick_cadence() {
    let mut state = WeaponRechargeState::default();
    let mut current_clip = 0;

    for expected_timer in 1..(BLASTER_RECHARGE_DELAY + BLASTER_RECHARGE_TICK) {
      assert_eq!(
        state.tick(&mut current_clip, 10),
        WeaponRechargeOutcome::Waiting {
          timer: expected_timer,
        }
      );
    }
    assert_eq!(
      state.tick(&mut current_clip, 10),
      WeaponRechargeOutcome::Recharged {
        ammo_recharged: BLASTER_RECHARGE_AMOUNT,
        timer: BLASTER_RECHARGE_DELAY,
      }
    );
    assert_eq!(current_clip, 1);

    for _ in 0..(BLASTER_RECHARGE_TICK - 1) {
      assert!(matches!(
        state.tick(&mut current_clip, 10),
        WeaponRechargeOutcome::Waiting { .. }
      ));
    }
    assert!(matches!(
      state.tick(&mut current_clip, 10),
      WeaponRechargeOutcome::Recharged {
        ammo_recharged: BLASTER_RECHARGE_AMOUNT,
        timer: BLASTER_RECHARGE_DELAY,
      }
    ));
    assert_eq!(current_clip, 2);
  }

  #[test]
  fn weapon_recharge_full_clip_preserves_timer_and_reset_clears_it() {
    let mut state = WeaponRechargeState {
      timer: 12,
      policy: WeaponRechargePolicy::blaster(),
    };
    let mut current_clip = 10;

    assert_eq!(
      state.tick(&mut current_clip, 10),
      WeaponRechargeOutcome::Full { timer: 12 }
    );
    state.reset();
    assert_eq!(state.timer(), 0);
    current_clip = 9;
    assert_eq!(
      state.tick(&mut current_clip, 10),
      WeaponRechargeOutcome::Waiting { timer: 1 }
    );
  }

  #[test]
  fn weapon_recharge_clamps_at_capacity() {
    let mut state = WeaponRechargeState {
      timer: BLASTER_RECHARGE_DELAY + BLASTER_RECHARGE_TICK - 1,
      policy: WeaponRechargePolicy::blaster(),
    };
    let mut current_clip = 9;

    assert_eq!(
      state.tick(&mut current_clip, 10),
      WeaponRechargeOutcome::Recharged {
        ammo_recharged: 1,
        timer: BLASTER_RECHARGE_DELAY,
      }
    );
    assert_eq!(current_clip, 10);
    assert_eq!(
      state.tick(&mut current_clip, 10),
      WeaponRechargeOutcome::Full {
        timer: BLASTER_RECHARGE_DELAY
      }
    );
  }

  #[test]
  fn nuclear_plasma_recharge_uses_pinned_delay_and_cadence() {
    let mut state = WeaponRechargeState::with_policy(WeaponRechargePolicy::nuclear_plasma());
    let mut current_clip = 0;

    for expected_timer in 1..(NUCLEAR_PLASMA_RECHARGE_DELAY + NUCLEAR_PLASMA_RECHARGE_TICK) {
      assert_eq!(
        state.tick(&mut current_clip, 24),
        WeaponRechargeOutcome::Waiting {
          timer: expected_timer,
        }
      );
    }
    assert_eq!(
      state.tick(&mut current_clip, 24),
      WeaponRechargeOutcome::Recharged {
        ammo_recharged: NUCLEAR_PLASMA_RECHARGE_AMOUNT,
        timer: NUCLEAR_PLASMA_RECHARGE_DELAY,
      }
    );
    assert_eq!(current_clip, 1);

    assert_eq!(
      state.tick(&mut current_clip, 24),
      WeaponRechargeOutcome::Waiting {
        timer: NUCLEAR_PLASMA_RECHARGE_DELAY + 1,
      }
    );
    assert_eq!(
      state.tick(&mut current_clip, 24),
      WeaponRechargeOutcome::Recharged {
        ammo_recharged: NUCLEAR_PLASMA_RECHARGE_AMOUNT,
        timer: NUCLEAR_PLASMA_RECHARGE_DELAY,
      }
    );
    assert_eq!(current_clip, 2);
  }

  #[test]
  fn nuclear_plasma_full_clip_preserves_timer_and_reset_clears_it() {
    let mut state = WeaponRechargeState {
      timer: 12,
      policy: WeaponRechargePolicy::nuclear_plasma(),
    };
    let mut current_clip = 24;

    assert_eq!(
      state.tick(&mut current_clip, 24),
      WeaponRechargeOutcome::Full { timer: 12 }
    );
    state.reset();
    current_clip = 23;
    assert_eq!(
      state.tick(&mut current_clip, 24),
      WeaponRechargeOutcome::Waiting { timer: 1 }
    );
  }

  #[test]
  fn nuclear_bfg_recharge_uses_immediate_five_tick_cadence() {
    let mut state = WeaponRechargeState::with_policy(WeaponRechargePolicy::nuclear_bfg());
    let mut current_clip = 0;

    for expected_timer in 1..NUCLEAR_BFG_RECHARGE_TICK {
      assert_eq!(
        state.tick(&mut current_clip, 40),
        WeaponRechargeOutcome::Waiting {
          timer: expected_timer,
        }
      );
    }
    assert_eq!(
      state.tick(&mut current_clip, 40),
      WeaponRechargeOutcome::Recharged {
        ammo_recharged: NUCLEAR_BFG_RECHARGE_AMOUNT,
        timer: NUCLEAR_BFG_RECHARGE_DELAY,
      }
    );
    assert_eq!(current_clip, 1);
    state.reset();
    current_clip = 39;
    assert_eq!(
      state.tick(&mut current_clip, 40),
      WeaponRechargeOutcome::Waiting { timer: 1 }
    );
  }
}
