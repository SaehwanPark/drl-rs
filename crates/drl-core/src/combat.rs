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

    let hits = attacker.ranged_exact_hit() || rng.gen_range(0..100) < effective_accuracy;
    if hits {
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
  use crate::item::Item;
  use drl_protocol::{EntityId, EquipmentSlot, HitPoints, ItemId, Position, Speed};

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

  #[test]
  fn standard_bfg_exact_hit_skips_to_hit_rng_but_keeps_damage_rng() {
    let mut attacker = Actor::new(EntityId::new(1), Position::new(0, 0), "Marine", true)
      .with_stats(
        HitPoints::full(50),
        Speed::NORMAL,
        (3, 6),
        Some((4, 8)),
        8,
        5,
      );
    attacker
      .equipment_mut()
      .equip(EquipmentSlot::Weapon, Item::bfg9000(ItemId::new(3)))
      .expect("BFG equips in the weapon slot");
    assert!(attacker.ranged_exact_hit());

    let defender = Actor::new(EntityId::new(2), Position::new(1, 0), "Demon", false);
    let (min_damage, max_damage) = attacker.ranged_damage().expect("BFG damage policy");
    let seed = 0;
    let mut miss_probe = GameRng::from_seed(seed);
    assert!(
      miss_probe.gen_range(0..100) >= 5,
      "seed must miss at minimum accuracy"
    );
    let mut expected_rng = GameRng::from_seed(seed);
    let expected_damage = expected_rng.gen_range(min_damage..(max_damage + 1));
    let mut actual_rng = GameRng::from_seed(seed);
    let outcome = CombatResolver::resolve_ranged_attack(&attacker, &defender, 1, &mut actual_rng);

    assert_eq!(
      outcome,
      AttackOutcome::Hit {
        damage: expected_damage,
        is_lethal: expected_damage >= defender.hp().current,
      }
    );
    assert_eq!(
      actual_rng, expected_rng,
      "exact-hit consumes only damage RNG"
    );
  }

  #[test]
  fn nuclear_bfg_exact_hit_skips_to_hit_rng_but_keeps_damage_rng() {
    let mut attacker = Actor::new(EntityId::new(1), Position::new(0, 0), "Marine", true)
      .with_stats(
        HitPoints::full(50),
        Speed::NORMAL,
        (3, 6),
        Some((4, 8)),
        8,
        5,
      );
    attacker
      .equipment_mut()
      .equip(EquipmentSlot::Weapon, Item::nuclear_bfg9000(ItemId::new(3)))
      .expect("Nuclear BFG equips in the weapon slot");
    assert!(attacker.ranged_exact_hit());

    let defender = Actor::new(EntityId::new(2), Position::new(1, 0), "Demon", false);
    let (min_damage, max_damage) = attacker.ranged_damage().expect("Nuclear BFG damage policy");
    let seed = 0;
    let mut miss_probe = GameRng::from_seed(seed);
    assert!(
      miss_probe.gen_range(0..100) >= 5,
      "seed must miss at minimum accuracy"
    );
    let mut expected_rng = GameRng::from_seed(seed);
    let expected_damage = expected_rng.gen_range(min_damage..(max_damage + 1));
    let mut actual_rng = GameRng::from_seed(seed);
    let outcome = CombatResolver::resolve_ranged_attack(&attacker, &defender, 1, &mut actual_rng);

    assert_eq!(
      outcome,
      AttackOutcome::Hit {
        damage: expected_damage,
        is_lethal: expected_damage >= defender.hp().current,
      }
    );
    assert_eq!(
      actual_rng, expected_rng,
      "exact-hit consumes only damage RNG"
    );
  }

  #[test]
  fn revenants_launcher_exact_hit_skips_to_hit_rng_but_keeps_damage_rng() {
    let mut attacker = Actor::new(EntityId::new(1), Position::new(0, 0), "Marine", true)
      .with_stats(
        HitPoints::full(50),
        Speed::NORMAL,
        (3, 6),
        Some((4, 8)),
        8,
        5,
      );
    attacker
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        Item::revenants_launcher(ItemId::new(3)),
      )
      .expect("Revenant's Launcher equips in the weapon slot");
    assert!(attacker.ranged_exact_hit());

    let defender = Actor::new(EntityId::new(2), Position::new(1, 0), "Demon", false);
    let (min_damage, max_damage) = attacker
      .ranged_damage()
      .expect("Revenant's Launcher damage policy");
    let seed = 0;
    let mut miss_probe = GameRng::from_seed(seed);
    assert!(
      miss_probe.gen_range(0..100) >= 5,
      "seed must miss at minimum accuracy"
    );
    let mut expected_rng = GameRng::from_seed(seed);
    let expected_damage = expected_rng.gen_range(min_damage..(max_damage + 1));
    let mut actual_rng = GameRng::from_seed(seed);
    let outcome = CombatResolver::resolve_ranged_attack(&attacker, &defender, 1, &mut actual_rng);

    assert_eq!(
      outcome,
      AttackOutcome::Hit {
        damage: expected_damage,
        is_lethal: expected_damage >= defender.hp().current,
      }
    );
    assert_eq!(
      actual_rng, expected_rng,
      "exact-hit consumes only damage RNG"
    );
  }

  #[test]
  fn bfg10k_exact_hit_skips_to_hit_rng_but_keeps_damage_rng() {
    let mut attacker = Actor::new(EntityId::new(1), Position::new(0, 0), "Marine", true)
      .with_stats(
        HitPoints::full(50),
        Speed::NORMAL,
        (3, 6),
        Some((4, 8)),
        8,
        5,
      );
    attacker
      .equipment_mut()
      .equip(EquipmentSlot::Weapon, Item::bfg10k(ItemId::new(3)))
      .expect("BFG 10K equips in the weapon slot");
    assert!(attacker.ranged_exact_hit());

    let defender = Actor::new(EntityId::new(2), Position::new(1, 0), "Demon", false);
    let (min_damage, max_damage) = attacker.ranged_damage().expect("BFG 10K damage policy");
    let seed = 0;
    let mut miss_probe = GameRng::from_seed(seed);
    assert!(
      miss_probe.gen_range(0..100) >= 5,
      "seed must miss at minimum accuracy"
    );
    let mut expected_rng = GameRng::from_seed(seed);
    let expected_damage = expected_rng.gen_range(min_damage..(max_damage + 1));
    let mut actual_rng = GameRng::from_seed(seed);
    let outcome = CombatResolver::resolve_ranged_attack(&attacker, &defender, 1, &mut actual_rng);

    assert_eq!(
      outcome,
      AttackOutcome::Hit {
        damage: expected_damage,
        is_lethal: expected_damage >= defender.hp().current,
      }
    );
    assert_eq!(
      actual_rng, expected_rng,
      "exact-hit consumes only damage RNG"
    );
  }

  #[test]
  fn exact_hit_policy_is_limited_to_current_exact_families() {
    let mut standard = Actor::new(EntityId::new(1), Position::new(0, 0), "Marine", true);
    standard
      .equipment_mut()
      .equip(EquipmentSlot::Weapon, Item::bfg9000(ItemId::new(1)))
      .unwrap();
    assert!(standard.ranged_exact_hit());

    let mut nuclear = Actor::new(EntityId::new(2), Position::new(0, 0), "Marine", true);
    nuclear
      .equipment_mut()
      .equip(EquipmentSlot::Weapon, Item::nuclear_bfg9000(ItemId::new(2)))
      .unwrap();
    assert!(nuclear.ranged_exact_hit());

    let mut bfg10k = Actor::new(EntityId::new(3), Position::new(0, 0), "Marine", true);
    bfg10k
      .equipment_mut()
      .equip(EquipmentSlot::Weapon, Item::bfg10k(ItemId::new(3)))
      .unwrap();
    assert!(bfg10k.ranged_exact_hit());

    let mut revenant = Actor::new(EntityId::new(4), Position::new(0, 0), "Marine", true);
    revenant
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        Item::revenants_launcher(ItemId::new(4)),
      )
      .unwrap();
    assert!(revenant.ranged_exact_hit());

    let mut pistol = Actor::new(EntityId::new(5), Position::new(0, 0), "Marine", true);
    pistol
      .equipment_mut()
      .equip(EquipmentSlot::Weapon, Item::pistol(ItemId::new(5)))
      .unwrap();
    assert!(!pistol.ranged_exact_hit());
  }
}
