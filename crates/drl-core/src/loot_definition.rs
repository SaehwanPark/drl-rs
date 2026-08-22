//! Immutable definitions for procedural item loot selection.

use drl_protocol::ItemSpawnKind;

/// One ordered generated-loot entry selected by an exclusive roll bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeneratedLootDefinition {
  /// The first roll value not included in this entry.
  pub upper_bound: u32,
  /// The current Rust spawn payload emitted for this entry.
  pub spawn_kind: ItemSpawnKind,
}

/// Current procedural item-loot policy, in ascending roll-bound order.
pub const GENERATED_LOOT_DEFINITIONS: [GeneratedLootDefinition; 6] = [
  GeneratedLootDefinition {
    upper_bound: 35,
    spawn_kind: ItemSpawnKind::Ammo9mm(20),
  },
  GeneratedLootDefinition {
    upper_bound: 55,
    spawn_kind: ItemSpawnKind::SmallMedPack,
  },
  GeneratedLootDefinition {
    upper_bound: 70,
    spawn_kind: ItemSpawnKind::AmmoShells(8),
  },
  GeneratedLootDefinition {
    upper_bound: 85,
    spawn_kind: ItemSpawnKind::Shotgun,
  },
  GeneratedLootDefinition {
    upper_bound: 95,
    spawn_kind: ItemSpawnKind::GreenArmor,
  },
  GeneratedLootDefinition {
    upper_bound: 100,
    spawn_kind: ItemSpawnKind::PhaseDevice,
  },
];

/// Returns the immutable generated-loot definition for a `0..100` roll.
///
/// Values outside the production roll range retain the old final-branch
/// behavior and resolve to the Phase Device entry. The lookup performs no
/// random sampling, allocation, or state mutation.
#[must_use]
pub fn generated_loot_definition_for_roll(roll: u32) -> &'static GeneratedLootDefinition {
  for definition in &GENERATED_LOOT_DEFINITIONS {
    if roll < definition.upper_bound {
      return definition;
    }
  }
  &GENERATED_LOOT_DEFINITIONS[GENERATED_LOOT_DEFINITIONS.len() - 1]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn table_preserves_current_bounds_and_payloads() {
    assert_eq!(
      GENERATED_LOOT_DEFINITIONS,
      [
        GeneratedLootDefinition {
          upper_bound: 35,
          spawn_kind: ItemSpawnKind::Ammo9mm(20),
        },
        GeneratedLootDefinition {
          upper_bound: 55,
          spawn_kind: ItemSpawnKind::SmallMedPack,
        },
        GeneratedLootDefinition {
          upper_bound: 70,
          spawn_kind: ItemSpawnKind::AmmoShells(8),
        },
        GeneratedLootDefinition {
          upper_bound: 85,
          spawn_kind: ItemSpawnKind::Shotgun,
        },
        GeneratedLootDefinition {
          upper_bound: 95,
          spawn_kind: ItemSpawnKind::GreenArmor,
        },
        GeneratedLootDefinition {
          upper_bound: 100,
          spawn_kind: ItemSpawnKind::PhaseDevice,
        },
      ]
    );
  }

  #[test]
  fn every_roll_preserves_threshold_mapping() {
    let expected = |roll: u32| match roll {
      0..35 => ItemSpawnKind::Ammo9mm(20),
      35..55 => ItemSpawnKind::SmallMedPack,
      55..70 => ItemSpawnKind::AmmoShells(8),
      70..85 => ItemSpawnKind::Shotgun,
      85..95 => ItemSpawnKind::GreenArmor,
      _ => ItemSpawnKind::PhaseDevice,
    };

    for roll in 0..100 {
      assert_eq!(
        generated_loot_definition_for_roll(roll).spawn_kind,
        expected(roll),
        "roll {roll}"
      );
    }
    assert_eq!(
      generated_loot_definition_for_roll(100).spawn_kind,
      ItemSpawnKind::PhaseDevice
    );
    assert_eq!(
      generated_loot_definition_for_roll(u32::MAX).spawn_kind,
      ItemSpawnKind::PhaseDevice
    );
  }

  #[test]
  fn table_bounds_are_strictly_increasing_and_cover_the_roll_domain() {
    let mut previous = 0;
    for definition in GENERATED_LOOT_DEFINITIONS {
      assert!(definition.upper_bound > previous);
      previous = definition.upper_bound;
    }
    assert_eq!(previous, 100);
  }
}
