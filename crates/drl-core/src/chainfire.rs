//! Deterministic whole-rule chainfire state and burst formulas.

use drl_protocol::ItemArchetype;

use crate::behavior::{
  BFG10K_PROJECTILE_COUNT, BFG10K_SHOT_COST, CHAINGUN_PROJECTILE_COUNT,
  LASER_RIFLE_PROJECTILE_COUNT, MINIGUN_PROJECTILE_COUNT, NUCLEAR_PLASMA_PROJECTILE_COUNT,
  PLASMA_RIFLE_PROJECTILE_COUNT,
};

const DEFAULT_CHAINFIRE_PROJECTILE_COST: u32 = 1;

/// Semantic class of a rotary weapon's chainfire continuation level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainfireState {
  /// The first alternate burst, before any warm-up has been established.
  Initial,
  /// The full ordinary volley used by the first continuation burst.
  Warming,
  /// A sustained burst, including every non-saturated level after warming.
  Sustained,
  /// The maximum representable level; its burst formula is sustained.
  Saturated,
}

impl ChainfireState {
  /// Classifies the persisted byte without giving each sustained level a rule.
  #[must_use]
  pub const fn from_level(level: u8) -> Self {
    match level {
      0 => Self::Initial,
      1 => Self::Warming,
      u8::MAX => Self::Saturated,
      _ => Self::Sustained,
    }
  }
}

/// Prepared projectile and resource values for one chainfire burst.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainfireBurst {
  state: ChainfireState,
  projectile_count: u32,
  ammo_cost: u32,
}

impl ChainfireBurst {
  /// Returns the semantic state class used to derive this burst.
  #[must_use]
  pub const fn state(self) -> ChainfireState {
    self.state
  }

  /// Returns the number of projectiles reserved by this burst.
  #[must_use]
  pub const fn projectile_count(self) -> u32 {
    self.projectile_count
  }

  /// Returns the complete aggregate clip cost for this burst.
  #[must_use]
  pub const fn ammo_cost(self) -> u32 {
    self.ammo_cost
  }
}

/// Supported chainfire family and its ordinary-fire formula inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainfireWeapon {
  archetype: ItemArchetype,
  ordinary_projectiles: u32,
  per_projectile_cost: u32,
}

impl ChainfireWeapon {
  /// Resolves one of the six supported chainfire families.
  #[must_use]
  pub const fn from_archetype(archetype: ItemArchetype) -> Option<Self> {
    let (ordinary_projectiles, per_projectile_cost) = match archetype {
      ItemArchetype::Bfg10k => (BFG10K_PROJECTILE_COUNT, BFG10K_SHOT_COST),
      ItemArchetype::Chaingun => (CHAINGUN_PROJECTILE_COUNT, DEFAULT_CHAINFIRE_PROJECTILE_COST),
      ItemArchetype::Minigun => (MINIGUN_PROJECTILE_COUNT, DEFAULT_CHAINFIRE_PROJECTILE_COST),
      ItemArchetype::PlasmaRifle => (
        PLASMA_RIFLE_PROJECTILE_COUNT,
        DEFAULT_CHAINFIRE_PROJECTILE_COST,
      ),
      ItemArchetype::LaserRifle => (
        LASER_RIFLE_PROJECTILE_COUNT,
        DEFAULT_CHAINFIRE_PROJECTILE_COST,
      ),
      ItemArchetype::NuclearPlasmaRifle => (
        NUCLEAR_PLASMA_PROJECTILE_COUNT,
        DEFAULT_CHAINFIRE_PROJECTILE_COST,
      ),
      _ => return None,
    };
    Some(Self {
      archetype,
      ordinary_projectiles,
      per_projectile_cost,
    })
  }

  /// Returns the stable family represented by this formula.
  #[must_use]
  pub const fn archetype(self) -> ItemArchetype {
    self.archetype
  }

  /// Returns the ordinary volley size used by the chainfire formula.
  #[must_use]
  pub const fn ordinary_projectiles(self) -> u32 {
    self.ordinary_projectiles
  }

  /// Returns the ordinary per-projectile clip cost.
  #[must_use]
  pub const fn per_projectile_cost(self) -> u32 {
    self.per_projectile_cost
  }

  /// Derives a burst from the whole-rule state formula.
  #[must_use]
  pub const fn burst(self, level: u8) -> ChainfireBurst {
    chainfire_burst(level, self.ordinary_projectiles, self.per_projectile_cost)
  }
}

/// Derives chainfire projectile count and aggregate cost from ordinary values.
///
/// The integer formulas are the evidenced legacy rule: initial fire removes
/// one third of the ordinary volley, warming uses the ordinary volley, and
/// sustained/saturated fire adds half of it. Cost is always the complete
/// projectile count multiplied by the ordinary per-projectile cost.
#[must_use]
pub const fn chainfire_burst(
  level: u8,
  ordinary_projectiles: u32,
  per_projectile_cost: u32,
) -> ChainfireBurst {
  let state = ChainfireState::from_level(level);
  let projectile_count = match state {
    ChainfireState::Initial => ordinary_projectiles.saturating_sub(ordinary_projectiles / 3),
    ChainfireState::Warming => ordinary_projectiles,
    ChainfireState::Sustained | ChainfireState::Saturated => {
      ordinary_projectiles.saturating_add(ordinary_projectiles / 2)
    }
  };
  ChainfireBurst {
    state,
    projectile_count,
    ammo_cost: projectile_count.saturating_mul(per_projectile_cost),
  }
}

/// Resolves a supported family and derives its burst at `level`.
#[must_use]
pub const fn chainfire_profile(archetype: ItemArchetype, level: u8) -> Option<ChainfireBurst> {
  match ChainfireWeapon::from_archetype(archetype) {
    Some(weapon) => Some(weapon.burst(level)),
    None => None,
  }
}

/// Converts a unified burst to the tuple shape used by compatibility callers.
pub(crate) const fn chainfire_profile_tuple(
  archetype: ItemArchetype,
  level: u8,
) -> Option<(u32, u32)> {
  match chainfire_profile(archetype, level) {
    Some(burst) => Some((burst.projectile_count(), burst.ammo_cost())),
    None => None,
  }
}

/// Advances after a successfully committed full burst, saturating at `255`.
#[must_use]
pub const fn advance_level(level: u8) -> u8 {
  level.saturating_add(1)
}

/// Returns the ordinary-fire reset level.
#[must_use]
pub const fn reset_level() -> u8 {
  0
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn classifies_initial_warming_sustained_and_saturated_levels() {
    assert_eq!(ChainfireState::from_level(0), ChainfireState::Initial);
    assert_eq!(ChainfireState::from_level(1), ChainfireState::Warming);
    assert_eq!(ChainfireState::from_level(2), ChainfireState::Sustained);
    assert_eq!(ChainfireState::from_level(254), ChainfireState::Sustained);
    assert_eq!(
      ChainfireState::from_level(u8::MAX),
      ChainfireState::Saturated
    );
  }

  #[test]
  fn integer_formula_matches_all_supported_family_inputs() {
    let cases = [
      (ItemArchetype::Bfg10k, 5, 5, 4, 5, 7),
      (ItemArchetype::Chaingun, 4, 1, 3, 4, 6),
      (ItemArchetype::Minigun, 8, 1, 6, 8, 12),
      (ItemArchetype::PlasmaRifle, 6, 1, 4, 6, 9),
      (ItemArchetype::LaserRifle, 5, 1, 4, 5, 7),
      (ItemArchetype::NuclearPlasmaRifle, 6, 1, 4, 6, 9),
    ];
    for (archetype, ordinary, cost, initial, warming, sustained) in cases {
      let weapon = ChainfireWeapon::from_archetype(archetype).expect("supported family");
      assert_eq!(weapon.ordinary_projectiles(), ordinary);
      assert_eq!(weapon.per_projectile_cost(), cost);
      assert_eq!(weapon.burst(0).projectile_count(), initial);
      assert_eq!(weapon.burst(1).projectile_count(), warming);
      assert_eq!(weapon.burst(2).projectile_count(), sustained);
      assert_eq!(weapon.burst(2).ammo_cost(), sustained * cost);
    }
  }

  #[test]
  fn saturation_uses_sustained_formula_and_advance_is_saturating() {
    let families = [
      (ItemArchetype::Bfg10k, 7, 35),
      (ItemArchetype::Chaingun, 6, 6),
      (ItemArchetype::Minigun, 12, 12),
      (ItemArchetype::PlasmaRifle, 9, 9),
      (ItemArchetype::LaserRifle, 7, 7),
      (ItemArchetype::NuclearPlasmaRifle, 9, 9),
    ];
    for (archetype, projectile_count, ammo_cost) in families {
      let burst = chainfire_profile(archetype, u8::MAX).expect("supported family");
      assert_eq!(burst.state(), ChainfireState::Saturated);
      assert_eq!(burst.projectile_count(), projectile_count);
      assert_eq!(burst.ammo_cost(), ammo_cost);
    }
    assert_eq!(advance_level(254), u8::MAX);
    assert_eq!(advance_level(u8::MAX), u8::MAX);
  }

  #[test]
  fn reset_returns_initial_level() {
    assert_eq!(reset_level(), 0);
    assert_eq!(
      ChainfireState::from_level(reset_level()),
      ChainfireState::Initial
    );
  }

  #[test]
  fn unsupported_archetypes_do_not_have_a_chainfire_profile() {
    assert_eq!(chainfire_profile(ItemArchetype::Pistol, 0), None);
  }
}
