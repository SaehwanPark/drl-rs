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
}
