//! Typed Grammaton Cleric Beretta fire-mode transition.

use drl_protocol::WeaponFireMode;

use crate::item::WeaponProperties;

/// Score-count cost of one Grammaton fire-mode cycle.
pub const GRAMMATON_MODE_SCORE_COST: i32 = 200;

/// Pure state transition for the Grammaton's three fire modes.
pub struct GrammatonTransition;

impl GrammatonTransition {
  /// Returns the number of rounds resolved by one mode-selected command.
  #[must_use]
  pub const fn shot_count(mode: WeaponFireMode) -> u32 {
    match mode {
      WeaponFireMode::Single => 1,
      WeaponFireMode::Burst => 3,
      WeaponFireMode::Auto => 6,
    }
  }

  /// Applies the next mode's profile to a weapon and returns the new mode.
  pub fn cycle(properties: &mut WeaponProperties) -> WeaponFireMode {
    let mode = match properties.fire_mode {
      WeaponFireMode::Single => WeaponFireMode::Burst,
      WeaponFireMode::Burst => WeaponFireMode::Auto,
      WeaponFireMode::Auto => WeaponFireMode::Single,
    };
    let (accuracy, damage) = match mode {
      WeaponFireMode::Single => (80, (2, 6)),
      WeaponFireMode::Burst => (75, (1, 8)),
      WeaponFireMode::Auto => (70, (1, 7)),
    };
    properties.fire_mode = mode;
    properties.accuracy = accuracy;
    properties.damage = damage;
    mode
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use drl_protocol::{ActionCost, AmmoType};

  fn weapon() -> WeaponProperties {
    WeaponProperties {
      is_ranged: true,
      ammo_type: Some(AmmoType::Ammo9mm),
      clip_capacity: 18,
      current_clip: 18,
      damage: (2, 6),
      range: 8,
      accuracy: 80,
      knockback: 0,
      fire_cost: ActionCost::RANGED_ATTACK,
      reload_cost: ActionCost::STANDARD,
      exact_hit: false,
      fire_mode: WeaponFireMode::Single,
    }
  }

  #[test]
  fn cycles_profiles_in_legacy_order() {
    let mut properties = weapon();
    assert_eq!(
      GrammatonTransition::cycle(&mut properties),
      WeaponFireMode::Burst
    );
    assert_eq!(properties.damage, (1, 8));
    assert_eq!(properties.accuracy, 75);
    assert_eq!(GrammatonTransition::shot_count(properties.fire_mode), 3);

    assert_eq!(
      GrammatonTransition::cycle(&mut properties),
      WeaponFireMode::Auto
    );
    assert_eq!(properties.damage, (1, 7));
    assert_eq!(properties.accuracy, 70);
    assert_eq!(GrammatonTransition::shot_count(properties.fire_mode), 6);

    assert_eq!(
      GrammatonTransition::cycle(&mut properties),
      WeaponFireMode::Single
    );
    assert_eq!(properties.damage, (2, 6));
    assert_eq!(properties.accuracy, 80);
    assert_eq!(GrammatonTransition::shot_count(properties.fire_mode), 1);
  }
}
