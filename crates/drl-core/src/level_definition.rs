//! Immutable typed definitions for current procedural level policies.

/// Immutable metadata for one current Rust procedural-level profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelDefinition {
  pub key: &'static str,
  pub width: u32,
  pub height: u32,
  pub max_rooms: u32,
  pub min_room_size: u32,
  pub max_room_size: u32,
  pub max_monsters_per_room: u32,
  pub max_items_per_room: u32,
}

/// Current Rust-owned procedural-level definitions.
pub const LEVEL_DEFINITIONS: [LevelDefinition; 1] = [LevelDefinition {
  key: "standard-procedural",
  width: 40,
  height: 20,
  max_rooms: 6,
  min_room_size: 4,
  max_room_size: 8,
  max_monsters_per_room: 2,
  max_items_per_room: 2,
}];

/// Returns the current standard procedural-level definition.
#[must_use]
pub const fn standard_procedural() -> LevelDefinition {
  LEVEL_DEFINITIONS[0]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn standard_definition_preserves_current_policy() {
    assert_eq!(LEVEL_DEFINITIONS.len(), 1);
    assert_eq!(
      standard_procedural(),
      LevelDefinition {
        key: "standard-procedural",
        width: 40,
        height: 20,
        max_rooms: 6,
        min_room_size: 4,
        max_room_size: 8,
        max_monsters_per_room: 2,
        max_items_per_room: 2,
      }
    );
  }
}
