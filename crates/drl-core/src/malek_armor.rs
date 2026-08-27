//! Typed Malek's Armor periodic durability-recharge transition.

/// Malek's Armor's pinned delay before its first durability point is restored.
pub const MALEK_ARMOR_RECHARGE_DELAY: u32 = 50;
/// Malek's Armor's retained timer cadence after each restored point.
pub const MALEK_ARMOR_RECHARGE_TICK: u32 = 5;
/// Durability restored by one Malek's Armor recharge.
pub const MALEK_ARMOR_RECHARGE_AMOUNT: u32 = 1;

/// Armor-owned state for Malek's Armor's periodic durability behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MalekRechargeState {
  timer: u32,
}

impl MalekRechargeState {
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

  /// Resets the timer after the armor receives damage.
  pub const fn reset(&mut self) {
    self.timer = 0;
  }

  /// Advances one accepted-command Malek's Armor tick.
  pub fn tick(&mut self, durability: &mut u32, max_durability: u32) -> MalekRechargeOutcome {
    if *durability >= max_durability {
      return MalekRechargeOutcome::Full { timer: self.timer };
    }

    self.timer = self.timer.saturating_add(1);
    if self.timer < MALEK_ARMOR_RECHARGE_DELAY + MALEK_ARMOR_RECHARGE_TICK {
      return MalekRechargeOutcome::Waiting { timer: self.timer };
    }

    let restored = MALEK_ARMOR_RECHARGE_AMOUNT.min(max_durability.saturating_sub(*durability));
    *durability = durability.saturating_add(restored).min(max_durability);
    self.timer = self.timer.saturating_sub(MALEK_ARMOR_RECHARGE_TICK);
    MalekRechargeOutcome::Recharged {
      durability_restored: restored,
      timer: self.timer,
    }
  }
}

/// Observable result of a typed Malek's Armor transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MalekRechargeOutcome {
  /// Armor is already full; source behavior leaves the timer untouched.
  Full { timer: u32 },
  /// The initial delay or retained cadence has not yet elapsed.
  Waiting { timer: u32 },
  /// One durability point was restored.
  Recharged {
    durability_restored: u32,
    timer: u32,
  },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn waits_for_delay_then_recharges_every_five_ticks() {
    let mut state = MalekRechargeState::new();
    let mut durability = 98;
    for _ in 0..54 {
      assert!(matches!(
        state.tick(&mut durability, 100),
        MalekRechargeOutcome::Waiting { .. }
      ));
    }
    assert_eq!(
      state.tick(&mut durability, 100),
      MalekRechargeOutcome::Recharged {
        durability_restored: 1,
        timer: 50,
      }
    );
    assert_eq!(durability, 99);
    for _ in 0..4 {
      assert!(matches!(
        state.tick(&mut durability, 100),
        MalekRechargeOutcome::Waiting { .. }
      ));
    }
    assert_eq!(
      state.tick(&mut durability, 100),
      MalekRechargeOutcome::Recharged {
        durability_restored: 1,
        timer: 50,
      }
    );
    assert_eq!(durability, 100);
  }

  #[test]
  fn full_armor_preserves_timer_and_damage_resets_it() {
    let mut state = MalekRechargeState::new();
    let mut durability = 100;
    for _ in 0..7 {
      state.tick(&mut durability, 100);
    }
    assert_eq!(state.timer(), 0);
    durability = 99;
    assert!(matches!(
      state.tick(&mut durability, 100),
      MalekRechargeOutcome::Waiting { timer: 1 }
    ));
    state.reset();
    assert_eq!(state.timer(), 0);
    durability = 100;
    assert_eq!(
      state.tick(&mut durability, 100),
      MalekRechargeOutcome::Full { timer: 0 }
    );
  }
}
