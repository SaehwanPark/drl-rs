//! Typed Mega Buster post-kill morph profiles.
//!
//! The legacy callback selects a new Mega Buster damage family from the
//! defeated target's weapon.  This module keeps that selection as a pure,
//! total mapping and records the current Rust bounds for each resulting
//! profile.  Runtime state transitions, target equipment lookup, and replay
//! event ordering remain callers' responsibilities.

use drl_protocol::{DamageType, ItemArchetype};

use crate::rng::GameRng;

pub use drl_protocol::MegaBusterMorphMode;

/// Damage dice used by the bounded Bullet Mega Buster profile.
pub const MEGA_BUSTER_BULLET_DAMAGE_DICE: u32 = 1;
/// Sides on each die in the bounded Bullet Mega Buster profile.
pub const MEGA_BUSTER_BULLET_DAMAGE_DIE_SIDES: u32 = 8;
/// Inclusive damage range represented by the bounded Bullet profile.
pub const MEGA_BUSTER_BULLET_DAMAGE_RANGE: (u32, u32) = (1, 8);
/// Radius of the bounded Bullet Mega Buster profile.
pub const MEGA_BUSTER_BULLET_RADIUS: u32 = 0;

/// Damage dice used by the bounded Fire Mega Buster profile.
pub const MEGA_BUSTER_FIRE_DAMAGE_DICE: u32 = 4;
/// Sides on each die in the bounded Fire Mega Buster profile.
pub const MEGA_BUSTER_FIRE_DAMAGE_DIE_SIDES: u32 = 2;
/// Inclusive damage range represented by the bounded Fire profile.
pub const MEGA_BUSTER_FIRE_DAMAGE_RANGE: (u32, u32) = (4, 8);
/// Radius of the bounded Fire Mega Buster profile.
pub const MEGA_BUSTER_FIRE_RADIUS: u32 = 1;

/// Damage dice used by the bounded Acid Mega Buster profile.
pub const MEGA_BUSTER_ACID_DAMAGE_DICE: u32 = 4;
/// Sides on each die in the bounded Acid Mega Buster profile.
pub const MEGA_BUSTER_ACID_DAMAGE_DIE_SIDES: u32 = 2;
/// Inclusive damage range represented by the bounded Acid profile.
pub const MEGA_BUSTER_ACID_DAMAGE_RANGE: (u32, u32) = (4, 8);
/// Radius of the bounded Acid Mega Buster profile.
pub const MEGA_BUSTER_ACID_RADIUS: u32 = 1;

/// Damage dice used by the bounded Plasma Mega Buster profile.
pub const MEGA_BUSTER_PLASMA_DAMAGE_DICE: u32 = 1;
/// Sides on each die in the bounded Plasma Mega Buster profile.
pub const MEGA_BUSTER_PLASMA_DAMAGE_DIE_SIDES: u32 = 10;
/// Inclusive damage range represented by the bounded Plasma profile.
pub const MEGA_BUSTER_PLASMA_DAMAGE_RANGE: (u32, u32) = (1, 10);
/// Radius of the bounded Plasma Mega Buster profile.
pub const MEGA_BUSTER_PLASMA_RADIUS: u32 = 0;

/// Selects a mode from the target's typed weapon damage family.
///
/// A missing target weapon and the current Rust `Physical` family both map
/// to Bullet. This is deliberately total so malformed or unsupported target
/// equipment cannot create an invalid morph state.
#[must_use]
pub const fn mode_for_target_damage_type(
  target_damage_type: Option<DamageType>,
) -> MegaBusterMorphMode {
  match target_damage_type {
    Some(DamageType::Fire) => MegaBusterMorphMode::Fire,
    Some(DamageType::Acid) => MegaBusterMorphMode::Acid,
    Some(DamageType::Plasma) => MegaBusterMorphMode::Plasma,
    Some(DamageType::Physical) | None => MegaBusterMorphMode::Bullet,
  }
}

/// Returns the immutable profile represented by one mode.
#[must_use]
pub const fn profile_for_mode(mode: MegaBusterMorphMode) -> MegaBusterMorphProfile {
  match mode {
    MegaBusterMorphMode::Bullet => MEGA_BUSTER_BULLET_PROFILE,
    MegaBusterMorphMode::Fire => MEGA_BUSTER_FIRE_PROFILE,
    MegaBusterMorphMode::Acid => MEGA_BUSTER_ACID_PROFILE,
    MegaBusterMorphMode::Plasma => MEGA_BUSTER_PLASMA_PROFILE,
  }
}

/// Immutable typed profile for one Mega Buster morph result.
///
/// `damage_range` is retained alongside the dice shape because existing Rust
/// weapon instances expose a `(min, max)` damage range, while the Fire and
/// Acid profiles also retain their explicit `4d2` provenance for
/// distribution-preserving execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MegaBusterMorphProfile {
  /// Selected morph family.
  pub mode: MegaBusterMorphMode,
  /// Typed damage family used by mitigation and observations.
  pub damage_type: DamageType,
  /// Number of damage dice rolled by the profile.
  pub damage_dice: u32,
  /// Sides on each damage die.
  pub damage_die_sides: u32,
  /// Inclusive `(minimum, maximum)` damage range.
  pub damage_range: (u32, u32),
  /// Current Rust bounded blast radius in cells.
  pub radius: u32,
}

impl MegaBusterMorphProfile {
  /// Selects a profile from optional target weapon damage metadata.
  #[must_use]
  pub const fn from_target_damage_type(target_damage_type: Option<DamageType>) -> Self {
    profile_for_mode(mode_for_target_damage_type(target_damage_type))
  }

  /// Returns the minimum damage in the pinned inclusive range.
  #[must_use]
  pub const fn minimum_damage(self) -> u32 {
    self.damage_range.0
  }

  /// Returns the maximum damage in the pinned inclusive range.
  #[must_use]
  pub const fn maximum_damage(self) -> u32 {
    self.damage_range.1
  }

  /// Rolls this profile's explicit damage dice.
  ///
  /// Each die consumes one [`GameRng`] sample, including the four independent
  /// dice in the Fire and Acid profiles.  Keeping the roll here, next to the
  /// immutable profile, prevents callers from accidentally replacing `4d2`
  /// with a uniform `(4..=8)` range while preserving deterministic replay
  /// consumption.
  pub fn roll_damage(self, rng: &mut GameRng) -> u32 {
    assert!(
      self.damage_dice > 0 && self.damage_die_sides > 0,
      "Mega Buster damage profiles must contain positive dice"
    );

    (0..self.damage_dice)
      .map(|_| rng.gen_range(0..self.damage_die_sides) + 1)
      .sum()
  }
}

/// Immutable Bullet morph profile.
pub const MEGA_BUSTER_BULLET_PROFILE: MegaBusterMorphProfile = MegaBusterMorphProfile {
  mode: MegaBusterMorphMode::Bullet,
  damage_type: DamageType::Physical,
  damage_dice: MEGA_BUSTER_BULLET_DAMAGE_DICE,
  damage_die_sides: MEGA_BUSTER_BULLET_DAMAGE_DIE_SIDES,
  damage_range: MEGA_BUSTER_BULLET_DAMAGE_RANGE,
  radius: MEGA_BUSTER_BULLET_RADIUS,
};

/// Immutable Fire morph profile.
pub const MEGA_BUSTER_FIRE_PROFILE: MegaBusterMorphProfile = MegaBusterMorphProfile {
  mode: MegaBusterMorphMode::Fire,
  damage_type: DamageType::Fire,
  damage_dice: MEGA_BUSTER_FIRE_DAMAGE_DICE,
  damage_die_sides: MEGA_BUSTER_FIRE_DAMAGE_DIE_SIDES,
  damage_range: MEGA_BUSTER_FIRE_DAMAGE_RANGE,
  radius: MEGA_BUSTER_FIRE_RADIUS,
};

/// Immutable Acid morph profile.
pub const MEGA_BUSTER_ACID_PROFILE: MegaBusterMorphProfile = MegaBusterMorphProfile {
  mode: MegaBusterMorphMode::Acid,
  damage_type: DamageType::Acid,
  damage_dice: MEGA_BUSTER_ACID_DAMAGE_DICE,
  damage_die_sides: MEGA_BUSTER_ACID_DAMAGE_DIE_SIDES,
  damage_range: MEGA_BUSTER_ACID_DAMAGE_RANGE,
  radius: MEGA_BUSTER_ACID_RADIUS,
};

/// Immutable Plasma morph profile.
pub const MEGA_BUSTER_PLASMA_PROFILE: MegaBusterMorphProfile = MegaBusterMorphProfile {
  mode: MegaBusterMorphMode::Plasma,
  damage_type: DamageType::Plasma,
  damage_dice: MEGA_BUSTER_PLASMA_DAMAGE_DICE,
  damage_die_sides: MEGA_BUSTER_PLASMA_DAMAGE_DIE_SIDES,
  damage_range: MEGA_BUSTER_PLASMA_DAMAGE_RANGE,
  radius: MEGA_BUSTER_PLASMA_RADIUS,
};

/// Selects a total typed profile from optional target weapon metadata.
#[must_use]
pub const fn profile_for_target_damage_type(
  target_damage_type: Option<DamageType>,
) -> MegaBusterMorphProfile {
  MegaBusterMorphProfile::from_target_damage_type(target_damage_type)
}

/// Alias emphasizing that the selector is fed by target equipment metadata.
#[must_use]
pub const fn profile_for_target_type(
  target_damage_type: Option<DamageType>,
) -> MegaBusterMorphProfile {
  profile_for_target_damage_type(target_damage_type)
}

/// Returns the canonical typed damage family for a catalog weapon archetype.
///
/// The mapping is intentionally kept beside the Mega Buster's target-family
/// contract so target equipment cannot be inferred from display names or
/// scalar damage values. Unclassified weapon families remain physical in this
/// bounded slice.
#[must_use]
pub const fn damage_type_for_archetype(archetype: ItemArchetype) -> DamageType {
  match archetype {
    ItemArchetype::RocketLauncher
    | ItemArchetype::MissileLauncher
    | ItemArchetype::RevenantsLauncher
    | ItemArchetype::AntiFreakJackal => DamageType::Fire,
    ItemArchetype::AcidSpitter => DamageType::Acid,
    ItemArchetype::Blaster
    | ItemArchetype::LaserRifle
    | ItemArchetype::NuclearPlasmaRifle
    | ItemArchetype::Bfg10k
    | ItemArchetype::Bfg9000
    | ItemArchetype::NuclearBfg9000
    | ItemArchetype::PlasmaRifle
    | ItemArchetype::PlasmaShotgun
    | ItemArchetype::NullPointer
    | ItemArchetype::TristarBlaster => DamageType::Plasma,
    _ => DamageType::Physical,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn profiles_pin_current_rust_damage_shapes_and_radii() {
    assert_eq!(
      MEGA_BUSTER_BULLET_PROFILE,
      MegaBusterMorphProfile {
        mode: MegaBusterMorphMode::Bullet,
        damage_type: DamageType::Physical,
        damage_dice: 1,
        damage_die_sides: 8,
        damage_range: (1, 8),
        radius: 0,
      }
    );
    assert_eq!(MEGA_BUSTER_FIRE_PROFILE.damage_range, (4, 8));
    assert_eq!(MEGA_BUSTER_FIRE_PROFILE.damage_dice, 4);
    assert_eq!(MEGA_BUSTER_FIRE_PROFILE.damage_die_sides, 2);
    assert_eq!(MEGA_BUSTER_FIRE_PROFILE.damage_type, DamageType::Fire);
    assert_eq!(MEGA_BUSTER_FIRE_PROFILE.radius, 1);
    assert_eq!(MEGA_BUSTER_ACID_PROFILE.damage_range, (4, 8));
    assert_eq!(MEGA_BUSTER_ACID_PROFILE.damage_type, DamageType::Acid);
    assert_eq!(MEGA_BUSTER_ACID_PROFILE.radius, 1);
    assert_eq!(MEGA_BUSTER_PLASMA_PROFILE.damage_range, (1, 10));
    assert_eq!(MEGA_BUSTER_PLASMA_PROFILE.damage_dice, 1);
    assert_eq!(MEGA_BUSTER_PLASMA_PROFILE.damage_die_sides, 10);
    assert_eq!(MEGA_BUSTER_PLASMA_PROFILE.damage_type, DamageType::Plasma);
    assert_eq!(MEGA_BUSTER_PLASMA_PROFILE.radius, 0);
  }

  #[test]
  fn selector_maps_typed_target_damage_and_falls_back_to_bullet() {
    assert_eq!(
      profile_for_target_damage_type(Some(DamageType::Fire)).mode,
      MegaBusterMorphMode::Fire
    );
    assert_eq!(
      profile_for_target_damage_type(Some(DamageType::Acid)).mode,
      MegaBusterMorphMode::Acid
    );
    assert_eq!(
      profile_for_target_damage_type(Some(DamageType::Plasma)).mode,
      MegaBusterMorphMode::Plasma
    );
    assert_eq!(
      profile_for_target_damage_type(Some(DamageType::Physical)).mode,
      MegaBusterMorphMode::Bullet
    );
    assert_eq!(
      profile_for_target_damage_type(None).mode,
      MegaBusterMorphMode::Bullet
    );
    assert_eq!(
      profile_for_target_type(Some(DamageType::Fire)),
      MEGA_BUSTER_FIRE_PROFILE
    );
  }

  #[test]
  fn profile_range_matches_its_pinned_dice_bounds() {
    for profile in [
      MEGA_BUSTER_BULLET_PROFILE,
      MEGA_BUSTER_FIRE_PROFILE,
      MEGA_BUSTER_ACID_PROFILE,
      MEGA_BUSTER_PLASMA_PROFILE,
    ] {
      assert_eq!(profile.minimum_damage(), profile.damage_dice);
      assert_eq!(
        profile.maximum_damage(),
        profile.damage_dice * profile.damage_die_sides
      );
    }
  }

  #[test]
  fn roll_damage_uses_one_rng_draw_per_explicit_die() {
    for profile in [
      MEGA_BUSTER_BULLET_PROFILE,
      MEGA_BUSTER_FIRE_PROFILE,
      MEGA_BUSTER_ACID_PROFILE,
      MEGA_BUSTER_PLASMA_PROFILE,
    ] {
      let mut actual_rng = GameRng::from_seed(0x4d_45_47_41);
      let mut expected_rng = actual_rng.clone();
      let expected_damage = (0..profile.damage_dice)
        .map(|_| expected_rng.gen_range(0..profile.damage_die_sides) + 1)
        .sum();

      assert_eq!(profile.roll_damage(&mut actual_rng), expected_damage);
      assert_eq!(actual_rng, expected_rng);
    }
  }

  #[test]
  fn fire_and_acid_rolls_never_leave_their_four_d_two_bounds() {
    for profile in [MEGA_BUSTER_FIRE_PROFILE, MEGA_BUSTER_ACID_PROFILE] {
      let mut rng = GameRng::from_seed(0x4d_45_47_41);
      for _ in 0..256 {
        assert!(
          (profile.minimum_damage()..=profile.maximum_damage())
            .contains(&profile.roll_damage(&mut rng))
        );
      }
    }
  }
}
