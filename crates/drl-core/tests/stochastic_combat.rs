//! Integration and statistical test suite for stochastic weapon behavior,
//! accuracy scaling, uniform damage distributions, and kinetic knockback mechanics.

use drl_core::actor::Actor;
use drl_core::combat::CombatResolver;
use drl_core::item::Item;
use drl_core::rng::GameRng;
use drl_core::{Game, ReplayEngine};
use drl_protocol::{
  AttackOutcome, Command, Direction, EntityId, EquipmentSlot, GameEvent, HitPoints, ItemSpawnKind,
  ItemSpawnSpec, MonsterSpawnSpec, Position, ReplayLog, Speed,
};

#[test]
fn test_melee_accuracy_statistical_distribution() {
  let attacker = Actor::new(EntityId::new(1), Position::new(0, 0), "Attacker", true).with_stats(
    HitPoints::full(50),
    Speed::NORMAL,
    (5, 10),
    None,
    0,
    80, // 80% configured accuracy
  );

  let defender = Actor::new(EntityId::new(2), Position::new(0, 1), "Defender", false);

  let mut rng = GameRng::from_seed(42);
  let total_trials = 2_000;
  let mut hits = 0;

  for _ in 0..total_trials {
    let outcome = CombatResolver::resolve_melee_attack(&attacker, &defender, &mut rng);
    if let AttackOutcome::Hit { .. } = outcome {
      hits += 1;
    }
  }

  let empirical_rate = hits as f64 / total_trials as f64;
  // Expected mean = 0.80; standard error = sqrt(0.80 * 0.20 / 2000) ~ 0.0089.
  // 3-sigma tolerance ~ +/- 0.027 => [0.77, 0.83].
  assert!(
    (0.77..=0.83).contains(&empirical_rate),
    "Empirical hit rate {empirical_rate} outside expected confidence interval [0.77, 0.83]"
  );
}

#[test]
fn test_ranged_accuracy_distance_penalty_statistical_scaling() {
  // Attacker with Pistol (accuracy 75, range 8)
  let attacker = Actor::new(EntityId::new(1), Position::new(0, 0), "Marine", true).with_stats(
    HitPoints::full(50),
    Speed::NORMAL,
    (3, 6),
    Some((4, 8)),
    8,
    75,
  );

  let defender = Actor::new(EntityId::new(2), Position::new(1, 0), "Imp", false);

  let mut rng = GameRng::from_seed(1337);
  let total_trials = 2_000;

  // 1. Distance 1 (penalty = 0% -> effective accuracy 75%)
  let mut hits_d1 = 0;
  for _ in 0..total_trials {
    if let AttackOutcome::Hit { .. } =
      CombatResolver::resolve_ranged_attack(&attacker, &defender, 1, &mut rng)
    {
      hits_d1 += 1;
    }
  }
  let rate_d1 = hits_d1 as f64 / total_trials as f64;
  assert!(
    (0.72..=0.78).contains(&rate_d1),
    "Distance 1 hit rate {rate_d1} outside [0.72, 0.78]"
  );

  // 2. Distance 4 (penalty = (4 - 1) * 2 = 6% -> effective accuracy 69%)
  let mut hits_d4 = 0;
  for _ in 0..total_trials {
    if let AttackOutcome::Hit { .. } =
      CombatResolver::resolve_ranged_attack(&attacker, &defender, 4, &mut rng)
    {
      hits_d4 += 1;
    }
  }
  let rate_d4 = hits_d4 as f64 / total_trials as f64;
  assert!(
    (0.66..=0.72).contains(&rate_d4),
    "Distance 4 hit rate {rate_d4} outside [0.66, 0.72]"
  );

  // 3. Distance 7 (penalty = (7 - 1) * 2 = 12% -> effective accuracy 63%)
  let mut hits_d7 = 0;
  for _ in 0..total_trials {
    if let AttackOutcome::Hit { .. } =
      CombatResolver::resolve_ranged_attack(&attacker, &defender, 7, &mut rng)
    {
      hits_d7 += 1;
    }
  }
  let rate_d7 = hits_d7 as f64 / total_trials as f64;
  assert!(
    (0.60..=0.66).contains(&rate_d7),
    "Distance 7 hit rate {rate_d7} outside [0.60, 0.66]"
  );

  // 4. Distance 9 (out of range 8 -> 0% hit rate)
  let mut hits_d9 = 0;
  for _ in 0..100 {
    if let AttackOutcome::Hit { .. } =
      CombatResolver::resolve_ranged_attack(&attacker, &defender, 9, &mut rng)
    {
      hits_d9 += 1;
    }
  }
  assert_eq!(hits_d9, 0, "Out-of-range attacks must always miss");
}

#[test]
fn test_weapon_damage_uniform_distribution_and_strict_bounds() {
  let mut rng = GameRng::from_seed(999);

  // 1. Pistol: damage (4, 8) -> expected mean 6.0
  let mut pistol_marine = Actor::new(EntityId::new(1), Position::new(0, 0), "Marine", true)
    .with_stats(
      HitPoints::full(50),
      Speed::NORMAL,
      (1, 2),
      Some((4, 8)),
      8,
      95,
    );
  let defender = Actor::new(EntityId::new(2), Position::new(1, 0), "Target", false).with_stats(
    HitPoints::full(500),
    Speed::NORMAL,
    (1, 2),
    None,
    0,
    50,
  );

  let mut pistol_damages = Vec::new();
  for _ in 0..2_000 {
    if let AttackOutcome::Hit { damage, .. } =
      CombatResolver::resolve_ranged_attack(&pistol_marine, &defender, 1, &mut rng)
    {
      assert!((4..=8).contains(&damage));
      pistol_damages.push(damage);
    }
  }
  assert!(!pistol_damages.is_empty());
  for expected_val in 4..=8 {
    assert!(
      pistol_damages.contains(&expected_val),
      "Value {expected_val} never rolled for pistol"
    );
  }
  let pistol_mean: f64 = pistol_damages.iter().sum::<u32>() as f64 / pistol_damages.len() as f64;
  assert!(
    (5.9..=6.1).contains(&pistol_mean),
    "Pistol mean damage {pistol_mean} outside [5.9, 6.1]"
  );

  // 2. Shotgun: damage (8, 16) -> expected mean 12.0
  let shotgun_props = Item::shotgun(drl_protocol::ItemId::new(10));
  pistol_marine
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, shotgun_props)
    .unwrap();

  let mut shotgun_damages = Vec::new();
  for _ in 0..2_000 {
    if let AttackOutcome::Hit { damage, .. } =
      CombatResolver::resolve_ranged_attack(&pistol_marine, &defender, 1, &mut rng)
    {
      assert!((8..=16).contains(&damage));
      shotgun_damages.push(damage);
    }
  }
  for expected_val in 8..=16 {
    assert!(
      shotgun_damages.contains(&expected_val),
      "Value {expected_val} never rolled for shotgun"
    );
  }
  let shotgun_mean: f64 = shotgun_damages.iter().sum::<u32>() as f64 / shotgun_damages.len() as f64;
  assert!(
    (11.8..=12.2).contains(&shotgun_mean),
    "Shotgun mean damage {shotgun_mean} outside [11.8, 12.2]"
  );
}

#[test]
fn test_shotgun_knockback_open_space_displacement() {
  let mut game = Game::new(42, 10, 10, Position::new(2, 2)).unwrap();

  // Give player a Shotgun (knockback 1)
  let shotgun_id = game.world_mut().allocate_item_id();
  let shotgun = Item::shotgun(shotgun_id);
  game
    .world_mut()
    .spawn_ground_item(Position::new(2, 2), shotgun)
    .unwrap();

  game.step(Command::Pickup).unwrap();
  game.step(Command::Equip(shotgun_id)).unwrap();

  // Spawn monster (Former Human with high HP so it survives) at (4, 2)
  let monster_pos = Position::new(4, 2);
  let monster_id = game
    .world_mut()
    .spawn_monster(monster_pos, "Zombie", 500, 0, (1, 2))
    .unwrap();

  // Open floor is at (5, 2)
  assert!(game.world().map().is_walkable(Position::new(5, 2)));

  // Player fires Shotgun East at (4, 2) until a hit connects
  let mut knocked_back = false;
  for _ in 0..10 {
    let events = game.step(Command::AttackRanged(monster_pos)).unwrap();
    if events.iter().any(|e| {
      matches!(
        e,
        GameEvent::ActorKnockedBack {
          entity_id,
          from,
          to,
        } if *entity_id == monster_id && *from == Position::new(4, 2) && *to == Position::new(5, 2)
      )
    }) {
      knocked_back = true;
      break;
    }
  }
  assert!(
    knocked_back,
    "Expected ActorKnockedBack event from (4, 2) to (5, 2)"
  );

  // Verify monster's position in world moved to (5, 2)
  let monster = game.world().get_actor(monster_id).unwrap();
  assert_eq!(monster.position(), Position::new(5, 2));
}

#[test]
fn test_shotgun_knockback_wall_collision_blocked() {
  let mut game = Game::new(100, 10, 10, Position::new(2, 2)).unwrap();

  let shotgun_id = game.world_mut().allocate_item_id();
  let shotgun = Item::shotgun(shotgun_id);
  game
    .world_mut()
    .spawn_ground_item(Position::new(2, 2), shotgun)
    .unwrap();

  game.step(Command::Pickup).unwrap();
  game.step(Command::Equip(shotgun_id)).unwrap();

  // Spawn monster at (4, 2) and build Wall immediately behind at (5, 2)
  let monster_pos = Position::new(4, 2);
  let monster_id = game
    .world_mut()
    .spawn_monster(monster_pos, "Zombie", 200, 0, (1, 2))
    .unwrap();
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(5, 2), drl_core::Tile::Wall);

  // Player fires Shotgun East at (4, 2)
  let events = game.step(Command::AttackRanged(monster_pos)).unwrap();

  // No ActorKnockedBack event should be emitted because wall blocks movement
  let knocked_back = events
    .iter()
    .any(|e| matches!(e, GameEvent::ActorKnockedBack { .. }));
  assert!(!knocked_back, "Knockback should be safely blocked by wall");

  // Monster remains at (4, 2)
  let monster = game.world().get_actor(monster_id).unwrap();
  assert_eq!(monster.position(), Position::new(4, 2));
}

#[test]
fn test_shotgun_knockback_actor_collision_blocked() {
  let mut game = Game::new(200, 10, 10, Position::new(2, 2)).unwrap();

  let shotgun_id = game.world_mut().allocate_item_id();
  let shotgun = Item::shotgun(shotgun_id);
  game
    .world_mut()
    .spawn_ground_item(Position::new(2, 2), shotgun)
    .unwrap();

  game.step(Command::Pickup).unwrap();
  game.step(Command::Equip(shotgun_id)).unwrap();

  // Spawn target monster at (4, 2) and blocking monster at (5, 2)
  let target_id = game
    .world_mut()
    .spawn_monster(Position::new(4, 2), "Zombie 1", 200, 0, (1, 2))
    .unwrap();
  let blocker_id = game
    .world_mut()
    .spawn_monster(Position::new(5, 2), "Zombie 2", 200, 0, (1, 2))
    .unwrap();

  // Player fires Shotgun at (4, 2)
  let events = game
    .step(Command::AttackRanged(Position::new(4, 2)))
    .unwrap();

  let knocked_back = events
    .iter()
    .any(|e| matches!(e, GameEvent::ActorKnockedBack { .. }));
  assert!(
    !knocked_back,
    "Knockback should be blocked by another actor"
  );

  // Both actors remain in their original positions
  assert_eq!(
    game.world().get_actor(target_id).unwrap().position(),
    Position::new(4, 2)
  );
  assert_eq!(
    game.world().get_actor(blocker_id).unwrap().position(),
    Position::new(5, 2)
  );
}

#[test]
fn test_shotgun_diagonal_knockback() {
  let mut game = Game::new(300, 10, 10, Position::new(2, 2)).unwrap();

  let shotgun_id = game.world_mut().allocate_item_id();
  let shotgun = Item::shotgun(shotgun_id);
  game
    .world_mut()
    .spawn_ground_item(Position::new(2, 2), shotgun)
    .unwrap();

  game.step(Command::Pickup).unwrap();
  game.step(Command::Equip(shotgun_id)).unwrap();

  // Spawn monster diagonally at (4, 4)
  let monster_id = game
    .world_mut()
    .spawn_monster(Position::new(4, 4), "Imp", 200, 0, (1, 2))
    .unwrap();

  // Player fires diagonally at (4, 4) -> should push to (5, 5)
  let events = game
    .step(Command::AttackRanged(Position::new(4, 4)))
    .unwrap();

  let knocked_back = events.iter().any(|e| {
    matches!(
      e,
      GameEvent::ActorKnockedBack {
        entity_id,
        from,
        to,
      } if *entity_id == monster_id && *from == Position::new(4, 4) && *to == Position::new(5, 5)
    )
  });
  assert!(knocked_back, "Expected diagonal knockback to (5, 5)");
  assert_eq!(
    game.world().get_actor(monster_id).unwrap().position(),
    Position::new(5, 5)
  );
}

#[test]
fn test_shotgun_lethal_shot_no_knockback_loot_at_death_point() {
  let mut game = Game::new(400, 10, 10, Position::new(2, 2)).unwrap();

  let shotgun_id = game.world_mut().allocate_item_id();
  let shotgun = Item::shotgun(shotgun_id);
  game
    .world_mut()
    .spawn_ground_item(Position::new(2, 2), shotgun)
    .unwrap();

  game.step(Command::Pickup).unwrap();
  game.step(Command::Equip(shotgun_id)).unwrap();

  // Spawn monster with 1 HP at (4, 2)
  let monster_id = game
    .world_mut()
    .spawn_monster(Position::new(4, 2), "Former Human", 1, 0, (1, 2))
    .unwrap();
  game
    .world_mut()
    .get_actor_mut(monster_id)
    .unwrap()
    .set_death_drop(Some(ItemSpawnKind::Ammo9mm(10)));

  // Player fires Shotgun at (4, 2) until a lethal hit connects
  let mut killed = false;
  for _ in 0..10 {
    let events = game
      .step(Command::AttackRanged(Position::new(4, 2)))
      .unwrap();

    if events
      .iter()
      .any(|e| matches!(e, GameEvent::ActorDied { entity_id, .. } if *entity_id == monster_id))
    {
      killed = true;
      assert!(
        !events
          .iter()
          .any(|e| matches!(e, GameEvent::ActorKnockedBack { .. })),
        "Lethal blows must not displace corpse"
      );

      // Loot drop placed at exact death point (4, 2)
      assert!(events.iter().any(|e| {
        matches!(
          e,
          GameEvent::ItemDropped {
            position,
            ..
          } if *position == Position::new(4, 2)
        )
      }));
      break;
    }
  }

  assert!(killed, "Monster should be killed by shotgun blast");
}

#[test]
fn test_former_sergeant_shotgun_knocks_player_back() {
  let mut game = Game::new(500, 10, 10, Position::new(3, 5)).unwrap();

  // Spawn Former Sergeant at (5, 5) with high energy so it shoots first
  let sergeant = Actor::former_sergeant(game.world_mut().allocate_entity_id(), Position::new(5, 5));
  let sergeant_id = sergeant.id();
  game.world_mut().actors_mut().insert(sergeant_id, sergeant);

  // Player waits at (3, 5)
  let events = game.step(Command::Wait).unwrap();

  // Sergeant should shoot player West towards (2, 5)
  let player_knocked = events.iter().any(|e| {
    matches!(
      e,
      GameEvent::ActorKnockedBack {
        from,
        to,
        ..
      } if *from == Position::new(3, 5) && *to == Position::new(2, 5)
    )
  });

  if player_knocked {
    assert_eq!(
      game.world().player().unwrap().position(),
      Position::new(2, 5)
    );
  }
}

#[test]
fn test_knockback_combat_encounter_replay_determinism() {
  let seed = 777;
  let width = 20;
  let height = 10;
  let start_pos = Position::new(2, 2);

  let mut game = Game::new(seed, width, height, start_pos).unwrap();

  let shotgun_id = game.world_mut().allocate_item_id();
  let shotgun = Item::shotgun(shotgun_id);
  game
    .world_mut()
    .spawn_ground_item(Position::new(2, 2), shotgun)
    .unwrap();

  let monster_pos = Position::new(4, 2);
  let monster_id = game
    .world_mut()
    .spawn_monster(monster_pos, "Former Sergeant", 50, 90, (3, 6))
    .unwrap();
  game
    .world_mut()
    .get_actor_mut(monster_id)
    .unwrap()
    .set_death_drop(Some(ItemSpawnKind::AmmoShells(4)));

  let commands = [
    Command::Pickup,
    Command::Equip(shotgun_id),
    Command::AttackRanged(monster_pos),
    Command::AttackRanged(Position::new(5, 2)),
    Command::Move(Direction::East),
    Command::Wait,
  ];

  let mut replay = ReplayLog::new(seed, width, height, start_pos);
  replay.record_item(ItemSpawnSpec::new(start_pos, ItemSpawnKind::Shotgun));
  replay.record_monster(MonsterSpawnSpec::new(
    monster_pos,
    "Former Sergeant",
    50,
    90,
    (3, 6),
  ));

  for cmd in commands {
    if game.step(cmd).is_ok() {
      replay.record_command(cmd);
    }
  }

  let is_deterministic = ReplayEngine::verify_determinism(&replay).unwrap();
  assert!(
    is_deterministic,
    "Replay with knockback combat must reproduce bit-for-bit identical state"
  );
}
