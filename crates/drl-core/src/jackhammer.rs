//! Typed Jackhammer burst/single fire-mode transition.

use drl_protocol::WeaponFireMode;

use crate::item::WeaponProperties;

/// Score-count cost of one Jackhammer fire-mode toggle.
pub const JACKHAMMER_MODE_SCORE_COST: i32 = 1;

/// Pure state transition for the Jackhammer's two fire modes.
pub struct JackhammerTransition;

impl JackhammerTransition {
  /// Applies the next Jackhammer mode and returns it.
  pub fn cycle(properties: &mut WeaponProperties) -> WeaponFireMode {
    let mode = match properties.fire_mode {
      WeaponFireMode::Burst => WeaponFireMode::Single,
      WeaponFireMode::Single | WeaponFireMode::Auto => WeaponFireMode::Burst,
    };
    properties.fire_mode = mode;
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
      ammo_type: Some(AmmoType::Shells),
      clip_capacity: 10,
      current_clip: 10,
      damage: (8, 24),
      range: 15,
      accuracy: 65,
      knockback: 1,
      fire_cost: ActionCost::RANGED_ATTACK,
      reload_cost: ActionCost::STANDARD,
      exact_hit: false,
      fire_mode: WeaponFireMode::Burst,
      chainfire_level: 0,
    }
  }

  #[test]
  fn toggles_burst_and_single_modes() {
    let mut properties = weapon();
    assert_eq!(
      JackhammerTransition::cycle(&mut properties),
      WeaponFireMode::Single
    );
    assert_eq!(properties.shot_count(), 1);
    assert_eq!(
      JackhammerTransition::cycle(&mut properties),
      WeaponFireMode::Burst
    );
    assert_eq!(properties.shot_count(), 3);
  }
}
