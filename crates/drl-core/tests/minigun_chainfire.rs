use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::replay::ReplayEngine;
use drl_protocol::{
  Command, CommandError, EquipmentSlot, GameEvent, ItemSpawnKind, MonsterSpawnSpec,
  PlayerSpawnConfig, Position, ReplayLog,
};

fn equipped_minigun(seed: u64) -> Game {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::minigun(weapon_id))
    .unwrap();
  game
}

fn ranged_events(events: &[GameEvent], player_id: drl_protocol::EntityId) -> usize {
  events
    .iter()
    .filter(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          is_ranged: true,
          ..
        } if *attacker_id == player_id
      )
    })
    .count()
}

#[test]
fn minigun_first_chainfire_emits_six_projectiles_and_advances_state() {
  let mut game = equipped_minigun(2_260);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("first Minigun chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  assert_eq!(weapon.weapon_properties().unwrap().current_clip, 194);
  assert_eq!(weapon.weapon_properties().unwrap().chainfire_level, 1);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == player_id && *event_target == target_id
      ))
      .count(),
    6
  );
}

#[test]
fn minigun_chainfire_keeps_six_outcomes_after_lethal_target() {
  let mut lethal_case = None;

  for seed in 2_261..2_340 {
    let mut game = equipped_minigun(seed);
    let target = Position::new(5, 2);
    let target_id = game
      .world_mut()
      .spawn_monster(target, "Fragile Target", 1, 100, (1, 6))
      .unwrap();
    let player_id = game.world().player_id().unwrap();
    let events = game
      .step(Command::AttackRangedChainfire(target))
      .expect("chainfire against a visible target should be accepted");

    if events.iter().any(
      |event| matches!(event, GameEvent::ActorDied { entity_id, .. } if *entity_id == target_id),
    ) {
      assert_eq!(ranged_events(&events, player_id), 6);
      let death_index = events
        .iter()
        .position(|event| matches!(event, GameEvent::ActorDied { entity_id, .. } if *entity_id == target_id))
        .unwrap();
      assert!(!events[death_index + 1..].iter().any(|event| {
        matches!(event, GameEvent::DamageApplied { target_id: event_target, .. } if *event_target == target_id)
      }));
      lethal_case = Some(seed);
      break;
    }
  }

  assert!(
    lethal_case.is_some(),
    "fixed seed window should include a lethal first-projectile Minigun chainfire"
  );
}

#[test]
fn minigun_chainfire_below_six_round_cost_rejection_is_atomic() {
  let mut game = equipped_minigun(2_341);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 5;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn minigun_ordinary_fire_resets_chainfire_warmup() {
  let mut game = equipped_minigun(2_342);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 0, (1, 6))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first Minigun chainfire burst");
  game
    .step(Command::AttackRanged(target))
    .expect("ordinary fire after Minigun chainfire");

  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .chainfire_level,
    0
  );
}

#[test]
fn minigun_higher_chainfire_level_is_rejected_without_mutation() {
  let mut game = equipped_minigun(2_343);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 0, (1, 6))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first Minigun chainfire burst");
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::InvalidCommand("higher Minigun chainfire levels are deferred".to_string())
  );
  assert_eq!(game, before);
}

#[test]
fn minigun_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_344, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Minigun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 500, 100, (1, 6)));
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) = ReplayEngine::run(&replay).expect("Minigun chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  assert_eq!(weapon.weapon_properties().unwrap().current_clip, 194);
  assert_eq!(weapon.weapon_properties().unwrap().chainfire_level, 1);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    6
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn chainfire_rejects_non_rotary_weapon_without_mutation() {
  let mut game = Game::new(2_345, 10, 6, Position::new(2, 2)).unwrap();
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::pistol(weapon_id))
    .unwrap();
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::InvalidCommand(
      "chainfire is only available for the Chaingun, Minigun, Plasma Rifle, Laser Rifle, or Nuclear Plasma Rifle".to_string()
    )
  );
  assert_eq!(game, before);
}
