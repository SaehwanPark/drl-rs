//! Pure deterministic environmental terrain transitions.

use drl_protocol::{ActionCost, DamageType, TileKind};

/// Integer action cost representing the legacy `move_cost=1.25` fluid ratio.
pub const FLUID_MOVEMENT_COST: ActionCost = ActionCost::new(1_250);

/// Integer action cost representing the legacy `move_cost=1.65` Mud ratio.
pub const MUD_MOVEMENT_COST: ActionCost = ActionCost::new(1_650);

/// Fixed baseline damage produced when a player enters a hazardous tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HazardDamage {
  /// Damage amount before any future resistance policy.
  pub amount: u32,
  /// Legacy damage family retained for future typed resistance work.
  pub damage_type: DamageType,
}

/// Returns the bounded baseline contact damage for an entered tile.
///
/// This deliberately models only Normal-difficulty, non-running, and
/// non-resistant contact. Resistance, avoidance, and movement modifiers belong
/// to a later typed behavior slice rather than an implicit callback.
#[must_use]
pub const fn entered_tile_damage(tile: TileKind) -> Option<HazardDamage> {
  match tile {
    TileKind::Acid => Some(HazardDamage {
      amount: 6,
      damage_type: DamageType::Acid,
    }),
    TileKind::Lava => Some(HazardDamage {
      amount: 12,
      damage_type: DamageType::Fire,
    }),
    _ => None,
  }
}

/// Returns the direct-player movement cost for an entered tile.
#[must_use]
pub const fn movement_cost(tile: TileKind) -> ActionCost {
  match tile {
    TileKind::Acid | TileKind::Lava | TileKind::Water => FLUID_MOVEMENT_COST,
    TileKind::Mud => MUD_MOVEMENT_COST,
    _ => ActionCost::MOVE,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn acid_and_lava_use_pinned_baseline_damage() {
    assert_eq!(
      entered_tile_damage(TileKind::Acid),
      Some(HazardDamage {
        amount: 6,
        damage_type: DamageType::Acid,
      })
    );
    assert_eq!(
      entered_tile_damage(TileKind::Lava),
      Some(HazardDamage {
        amount: 12,
        damage_type: DamageType::Fire,
      })
    );
  }

  #[test]
  fn ordinary_tiles_do_not_apply_contact_damage() {
    assert_eq!(entered_tile_damage(TileKind::Floor), None);
    assert_eq!(entered_tile_damage(TileKind::Water), None);
    assert_eq!(entered_tile_damage(TileKind::StairsDown), None);
  }

  #[test]
  fn fluid_tiles_use_the_pinned_integer_movement_cost() {
    assert_eq!(movement_cost(TileKind::Acid), FLUID_MOVEMENT_COST);
    assert_eq!(movement_cost(TileKind::Lava), FLUID_MOVEMENT_COST);
    assert_eq!(movement_cost(TileKind::Floor), ActionCost::MOVE);
    assert_eq!(movement_cost(TileKind::Water), FLUID_MOVEMENT_COST);
    assert_eq!(movement_cost(TileKind::Mud), MUD_MOVEMENT_COST);
  }
}
