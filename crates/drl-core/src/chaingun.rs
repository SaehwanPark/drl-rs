//! Compatibility transition names for the core chainfire state model.

use crate::chainfire::{advance_level, reset_level};
use crate::item::WeaponProperties;

/// Number of projectiles emitted by the first chainfire burst.
pub const CHAINGUN_CHAINFIRE_PROJECTILE_COUNT: u32 = 3;
/// Number of 9mm rounds consumed by the first chainfire burst.
pub const CHAINGUN_CHAINFIRE_SHOT_COST: u32 = 3;

/// Backwards-compatible state transition wrapper.
///
/// Burst derivation belongs to [`crate::chainfire`]; these names remain for
/// callers that only need to advance or reset a weapon's persisted byte.
pub struct ChainfireTransition;

impl ChainfireTransition {
  /// Returns whether the persisted state can continue through the model.
  ///
  /// Weapon-family support and ammunition are validated by `Game`; this
  /// compatibility wrapper must not reintroduce a level-one ceiling.
  #[must_use]
  pub const fn can_chainfire(_properties: &WeaponProperties) -> bool {
    true
  }

  /// Advances the typed warm-up state after an accepted chainfire burst.
  pub const fn advance(properties: &mut WeaponProperties) {
    properties.chainfire_level = advance_level(properties.chainfire_level);
  }

  /// Resets chainfire continuation after an ordinary fire action.
  pub const fn reset(properties: &mut WeaponProperties) {
    properties.chainfire_level = reset_level();
  }
}

/// Backwards-compatible name for callers that use the original Chaingun slice.
pub type ChaingunTransition = ChainfireTransition;
/// Explicit name for Minigun's shared first-level chainfire state transition.
pub type MinigunTransition = ChainfireTransition;

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
    assert!(ChainfireTransition::can_chainfire(&properties));
    ChainfireTransition::advance(&mut properties);
    assert_eq!(properties.chainfire_level, 1);
    assert!(ChainfireTransition::can_chainfire(&properties));
    ChainfireTransition::reset(&mut properties);
    assert_eq!(properties.chainfire_level, 0);
    assert!(ChaingunTransition::can_chainfire(&properties));
  }
}
