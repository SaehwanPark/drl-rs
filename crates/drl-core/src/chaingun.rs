//! Typed first-level Chaingun alternate chainfire behavior.

use crate::item::WeaponProperties;

/// Number of projectiles emitted by the first chainfire burst.
pub const CHAINGUN_CHAINFIRE_PROJECTILE_COUNT: u32 = 3;
/// Number of 9mm rounds consumed by the first chainfire burst.
pub const CHAINGUN_CHAINFIRE_SHOT_COST: u32 = 3;

/// Pure state transition for the bounded Chaingun chainfire slice.
pub struct ChaingunTransition;

impl ChaingunTransition {
  /// Returns whether the first chainfire burst is currently available.
  #[must_use]
  pub const fn can_chainfire(properties: &WeaponProperties) -> bool {
    properties.chainfire_level == 0
  }

  /// Advances the typed warm-up state after an accepted chainfire burst.
  pub const fn advance(properties: &mut WeaponProperties) {
    properties.chainfire_level = properties.chainfire_level.saturating_add(1);
  }

  /// Resets chainfire continuation after an ordinary fire action.
  pub const fn reset(properties: &mut WeaponProperties) {
    properties.chainfire_level = 0;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use drl_protocol::{ActionCost, AmmoType, WeaponFireMode};

  fn weapon() -> WeaponProperties {
    WeaponProperties {
      is_ranged: true,
      ammo_type: Some(AmmoType::Ammo9mm),
      clip_capacity: 40,
      current_clip: 40,
      damage: (1, 6),
      range: 8,
      accuracy: 70,
      knockback: 0,
      fire_cost: ActionCost::RANGED_ATTACK,
      reload_cost: ActionCost::STANDARD,
      exact_hit: false,
      fire_mode: WeaponFireMode::Single,
      chainfire_level: 0,
    }
  }

  #[test]
  fn advances_only_after_an_accepted_burst_and_can_reset() {
    let mut properties = weapon();
    assert!(ChaingunTransition::can_chainfire(&properties));
    ChaingunTransition::advance(&mut properties);
    assert_eq!(properties.chainfire_level, 1);
    assert!(!ChaingunTransition::can_chainfire(&properties));
    ChaingunTransition::reset(&mut properties);
    assert_eq!(properties.chainfire_level, 0);
    assert!(ChaingunTransition::can_chainfire(&properties));
  }
}
