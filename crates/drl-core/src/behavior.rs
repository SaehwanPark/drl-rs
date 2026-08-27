//! Typed, deterministic item behaviors owned by the simulation core.
//!
//! This module intentionally models behavior as explicit state transitions.
//! It is not a callback registry and does not execute legacy Lua.

use drl_protocol::HitPoints;

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

/// Weapon-owned state for the Blaster's periodic cell recharge behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WeaponRechargeState {
  timer: u32,
}

impl WeaponRechargeState {
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

  /// Advances one accepted-command tick while the weapon is below capacity.
  pub fn tick(&mut self, current_clip: &mut u32, max_clip: u32) -> WeaponRechargeOutcome {
    if *current_clip >= max_clip {
      return WeaponRechargeOutcome::Full { timer: self.timer };
    }

    self.timer = self.timer.saturating_add(1);
    if self.timer < BLASTER_RECHARGE_DELAY + BLASTER_RECHARGE_TICK {
      return WeaponRechargeOutcome::Waiting { timer: self.timer };
    }

    self.timer = self.timer.saturating_sub(BLASTER_RECHARGE_TICK);
    let restored = BLASTER_RECHARGE_AMOUNT.min(max_clip.saturating_sub(*current_clip));
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
    let mut state = WeaponRechargeState { timer: 12 };
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
}
