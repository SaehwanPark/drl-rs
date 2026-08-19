//! Pure, deterministic combat calculation routines.
//!
//! Provides independent, fully testable combat formulas for melee and ranged attacks
//! without side effects on the physical game world.

use drl_protocol::AttackOutcome;

use crate::actor::Actor;
use crate::rng::GameRng;

/// Pure combat resolution engine.
pub struct CombatResolver;

impl CombatResolver {
  /// Resolves a melee attack between an attacker and defender.
  ///
  /// Computes hit determination using attacker accuracy and rolls damage within the
  /// attacker's configured melee damage range.
  pub fn resolve_melee_attack(
    attacker: &Actor,
    defender: &Actor,
    rng: &mut GameRng,
  ) -> AttackOutcome {
    let hit_chance = attacker.accuracy().clamp(5, 95) as u32;
    let roll = rng.gen_range(0..100);

    if roll < hit_chance {
      let (min_dam, max_dam) = attacker.melee_damage();
      let damage = if min_dam >= max_dam {
        min_dam
      } else {
        rng.gen_range(min_dam..(max_dam + 1))
      };
      let is_lethal = damage >= defender.hp().current;
      AttackOutcome::Hit { damage, is_lethal }
    } else {
      AttackOutcome::Miss
    }
  }

  /// Resolves a ranged attack between an attacker and defender across a given distance.
  ///
  /// Applies range penalty to accuracy and rolls damage from the attacker's ranged weapon.
  pub fn resolve_ranged_attack(
    attacker: &Actor,
    defender: &Actor,
    distance: u32,
    rng: &mut GameRng,
  ) -> AttackOutcome {
    let Some((min_dam, max_dam)) = attacker.ranged_damage() else {
      return AttackOutcome::Miss;
    };

    if distance > attacker.ranged_range() {
      return AttackOutcome::Miss;
    }

    // Distance penalty: 2% per tile beyond adjacent
    let penalty = distance.saturating_sub(1) * 2;
    let effective_accuracy = (attacker.accuracy() as u32)
      .saturating_sub(penalty)
      .clamp(5, 95);

    let roll = rng.gen_range(0..100);
    if roll < effective_accuracy {
      let damage = if min_dam >= max_dam {
        min_dam
      } else {
        rng.gen_range(min_dam..(max_dam + 1))
      };
      let is_lethal = damage >= defender.hp().current;
      AttackOutcome::Hit { damage, is_lethal }
    } else {
      AttackOutcome::Miss
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use drl_protocol::{EntityId, HitPoints, Position, Speed};

  #[test]
  fn test_melee_attack_deterministic_hit_and_damage_bounds() {
    let attacker = Actor::new(EntityId::new(1), Position::new(0, 0), "Marine", true).with_stats(
      HitPoints::full(50),
      Speed::NORMAL,
      (5, 10),
      None,
      0,
      100, // 100% accuracy clamped to 95%
    );

    let defender = Actor::new(EntityId::new(2), Position::new(0, 1), "Demon", false).with_stats(
      HitPoints::full(30),
      Speed::NORMAL,
      (2, 4),
      None,
      0,
      50,
    );

    let mut rng = GameRng::from_seed(42);
    let mut hits = 0;
    for _ in 0..50 {
      let outcome = CombatResolver::resolve_melee_attack(&attacker, &defender, &mut rng);
      if let AttackOutcome::Hit { damage, is_lethal } = outcome {
        hits += 1;
        assert!((5..=10).contains(&damage));
        assert!(!is_lethal);
      }
    }
    assert!(hits > 40);
  }

  #[test]
  fn test_melee_attack_lethal_outcome() {
    let attacker = Actor::new(EntityId::new(1), Position::new(0, 0), "Marine", true).with_stats(
      HitPoints::full(50),
      Speed::NORMAL,
      (20, 20),
      None,
      0,
      95,
    );

    let defender = Actor::new(EntityId::new(2), Position::new(0, 1), "Zombie", false).with_stats(
      HitPoints::new(10, 20),
      Speed::NORMAL,
      (1, 2),
      None,
      0,
      50,
    );

    let mut rng = GameRng::from_seed(12345);
    let outcome = CombatResolver::resolve_melee_attack(&attacker, &defender, &mut rng);
    assert_eq!(
      outcome,
      AttackOutcome::Hit {
        damage: 20,
        is_lethal: true
      }
    );
  }

  #[test]
  fn test_ranged_attack_range_limit() {
    let attacker = Actor::new(EntityId::new(1), Position::new(0, 0), "Marine", true).with_stats(
      HitPoints::full(50),
      Speed::NORMAL,
      (2, 4),
      Some((5, 10)),
      5, // Range 5
      90,
    );

    let defender = Actor::new(EntityId::new(2), Position::new(10, 0), "Imp", false);

    let mut rng = GameRng::from_seed(1);
    let outcome = CombatResolver::resolve_ranged_attack(&attacker, &defender, 10, &mut rng);
    assert_eq!(outcome, AttackOutcome::Miss);
  }
}
