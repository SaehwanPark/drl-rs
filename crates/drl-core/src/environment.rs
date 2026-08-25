//! Pure deterministic environmental hazard transitions.

use drl_protocol::{DamageType, TileKind};

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
}
