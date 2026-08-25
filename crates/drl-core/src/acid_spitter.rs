//! Pure Acid Spitter terrain-reload transition.

use drl_protocol::TileKind;

/// Score count spent by one Acid Spitter terrain reload.
pub const ACID_SPITTER_RELOAD_SCORE_COST: i32 = 1_000;

/// Number of rockets loaded by one Acid Spitter terrain reload.
pub const ACID_SPITTER_RELOAD_AMOUNT: u32 = 1;

/// Result of a successful Acid Spitter terrain reload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcidSpitterReloadOutcome {
  pub current_clip: u32,
  pub score_count_remaining: i32,
  pub resulting_tile: TileKind,
}

/// Rejection reason for the pure Acid Spitter reload transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcidSpitterReloadError {
  ClipFull,
  NotOnAcid,
}

/// Applies the source-backed Acid-only one-rocket reload policy.
#[must_use = "the transition result determines whether the reload is accepted"]
pub const fn apply(
  current_clip: u32,
  clip_capacity: u32,
  score_count: i32,
  tile: TileKind,
) -> Result<AcidSpitterReloadOutcome, AcidSpitterReloadError> {
  if current_clip >= clip_capacity {
    return Err(AcidSpitterReloadError::ClipFull);
  }
  if !matches!(tile, TileKind::Acid) {
    return Err(AcidSpitterReloadError::NotOnAcid);
  }
  Ok(AcidSpitterReloadOutcome {
    current_clip: current_clip.saturating_add(ACID_SPITTER_RELOAD_AMOUNT),
    score_count_remaining: score_count.saturating_sub(ACID_SPITTER_RELOAD_SCORE_COST),
    resulting_tile: TileKind::Water,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reloads_one_rocket_and_converts_acid_to_water() {
    assert_eq!(
      apply(0, 10, 1_500, TileKind::Acid),
      Ok(AcidSpitterReloadOutcome {
        current_clip: 1,
        score_count_remaining: 500,
        resulting_tile: TileKind::Water,
      })
    );
  }

  #[test]
  fn score_cost_saturates_and_non_acid_rejects() {
    assert_eq!(
      apply(2, 10, 0, TileKind::Acid)
        .unwrap()
        .score_count_remaining,
      -1_000
    );
    assert_eq!(
      apply(2, 10, 1_500, TileKind::Lava),
      Err(AcidSpitterReloadError::NotOnAcid)
    );
  }

  #[test]
  fn full_clip_rejects_before_terrain_policy() {
    assert_eq!(
      apply(10, 10, 1_500, TileKind::Acid),
      Err(AcidSpitterReloadError::ClipFull)
    );
  }
}
