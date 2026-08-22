//! Immutable definitions for procedural monster-kind selection.

use drl_protocol::MonsterKind;

/// One ordered generated-monster entry selected by an exclusive roll bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeneratedMonsterDefinition {
  /// The first roll value not included in this entry.
  pub upper_bound: u32,
  /// The monster archetype emitted for this entry.
  pub kind: MonsterKind,
}

/// Current procedural monster-kind policy, in ascending roll-bound order.
pub const GENERATED_MONSTER_DEFINITIONS: [GeneratedMonsterDefinition; 4] = [
  GeneratedMonsterDefinition {
    upper_bound: 40,
    kind: MonsterKind::FormerHuman,
  },
  GeneratedMonsterDefinition {
    upper_bound: 65,
    kind: MonsterKind::Imp,
  },
  GeneratedMonsterDefinition {
    upper_bound: 85,
    kind: MonsterKind::FormerSergeant,
  },
  GeneratedMonsterDefinition {
    upper_bound: 100,
    kind: MonsterKind::Demon,
  },
];

/// Returns the immutable generated-monster definition for a `0..100` roll.
///
/// Values outside the production range retain the old final-branch behavior
/// and resolve to the Demon entry. The lookup performs no random sampling,
/// allocation, clock, I/O, or state mutation.
#[must_use]
pub fn generated_monster_definition_for_roll(roll: u32) -> &'static GeneratedMonsterDefinition {
  for definition in &GENERATED_MONSTER_DEFINITIONS {
    if roll < definition.upper_bound {
      return definition;
    }
  }
  &GENERATED_MONSTER_DEFINITIONS[GENERATED_MONSTER_DEFINITIONS.len() - 1]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn table_preserves_current_bounds_and_archetypes() {
    assert_eq!(
      GENERATED_MONSTER_DEFINITIONS,
      [
        GeneratedMonsterDefinition {
          upper_bound: 40,
          kind: MonsterKind::FormerHuman,
        },
        GeneratedMonsterDefinition {
          upper_bound: 65,
          kind: MonsterKind::Imp,
        },
        GeneratedMonsterDefinition {
          upper_bound: 85,
          kind: MonsterKind::FormerSergeant,
        },
        GeneratedMonsterDefinition {
          upper_bound: 100,
          kind: MonsterKind::Demon,
        },
      ]
    );
  }

  #[test]
  fn every_roll_preserves_threshold_mapping() {
    let expected = |roll: u32| match roll {
      0..40 => MonsterKind::FormerHuman,
      40..65 => MonsterKind::Imp,
      65..85 => MonsterKind::FormerSergeant,
      _ => MonsterKind::Demon,
    };

    for roll in 0..100 {
      assert_eq!(
        generated_monster_definition_for_roll(roll).kind,
        expected(roll),
        "roll {roll}"
      );
    }
    assert_eq!(
      generated_monster_definition_for_roll(100).kind,
      MonsterKind::Demon
    );
    assert_eq!(
      generated_monster_definition_for_roll(u32::MAX).kind,
      MonsterKind::Demon
    );
  }

  #[test]
  fn table_bounds_are_strictly_increasing_and_cover_the_roll_domain() {
    let mut previous = 0;
    for definition in GENERATED_MONSTER_DEFINITIONS {
      assert!(definition.upper_bound > previous);
      previous = definition.upper_bound;
    }
    assert_eq!(previous, 100);
  }
}
