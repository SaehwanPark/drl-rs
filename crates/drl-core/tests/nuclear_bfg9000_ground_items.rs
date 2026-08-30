use drl_core::ReplayEngine;
use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::nuclear_bfg9000::{radius_eight_blast_positions, roll_explosion_damage};
use drl_protocol::{
  Command, DamageSource, DamageType, EquipmentSlot, GameEvent, ItemId, ItemSpawnKind,
  ItemSpawnSpec, MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog,
};

fn equipped_nuclear_bfg(seed: u64) -> Game {
  let mut game = Game::new(seed, 24, 24, Position::new(12, 12)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::nuclear_bfg9000(weapon_id))
    .unwrap();
  game
}

fn first_destructive_cell(game: &Game, center: Position) -> Position {
  let mut expected_rng = game.rng().clone();
  expected_rng.gen_range(8..49);
  radius_eight_blast_positions(game.world().map(), center)
    .into_iter()
    .skip(1)
    .find(|_| roll_explosion_damage(&mut expected_rng) > 10)
    .expect("a radius-8 blast must contain a destructive 8d6 roll")
}

fn plasma_damage_index(events: &[GameEvent], target_id: drl_protocol::EntityId) -> usize {
  events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          target_id: event_target,
          source: DamageSource::Environment,
          damage_type: Some(DamageType::Plasma),
          ..
        } if *event_target == target_id
      )
    })
    .expect("blast actor should receive environmental plasma damage")
}

#[test]
fn nuclear_bfg_destroys_lowest_id_ordinary_ground_item_above_threshold() {
  let center = Position::new(14, 12);
  let mut game = equipped_nuclear_bfg(31_000);
  let center_id = game
    .world_mut()
    .spawn_monster(center, "Center Target", 500, 0, (0, 0))
    .unwrap();
  let hot_position = first_destructive_cell(&game, center);
  let hot_actor_id = game
    .world_mut()
    .spawn_monster(hot_position, "Hot Target", 500, 0, (0, 0))
    .unwrap();
  let low_item_id = ItemId::new(100);
  let high_item_id = ItemId::new(101);
  game
    .world_mut()
    .spawn_ground_item(hot_position, Item::shotgun(low_item_id))
    .unwrap();
  game
    .world_mut()
    .spawn_ground_item(hot_position, Item::small_medpack(high_item_id))
    .unwrap();

  let mut expected_rng = game.rng().clone();
  expected_rng.gen_range(8..49);
  for _ in radius_eight_blast_positions(game.world().map(), center) {
    roll_explosion_damage(&mut expected_rng);
  }
  let events = game
    .step(Command::AttackRanged(center))
    .expect("Nuclear BFG 9000 direct fire should resolve");

  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::GroundItemDestroyed { item_id, position }
        if *item_id == low_item_id && *position == hot_position
    )
  }));
  let damage_index = plasma_damage_index(&events, hot_actor_id);
  let destroyed_index = events
    .iter()
    .position(|event| {
      matches!(event, GameEvent::GroundItemDestroyed { item_id, .. } if *item_id == low_item_id)
    })
    .unwrap();
  assert!(damage_index < destroyed_index);
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::DamageApplied {
        target_id,
        source: DamageSource::Environment,
        damage_type: Some(DamageType::Plasma),
        ..
      } if *target_id == center_id
    )
  }));
  assert!(!game.world().ground_items().contains_key(&low_item_id));
  assert!(game.world().ground_items().contains_key(&high_item_id));
  assert_eq!(game.rng(), &expected_rng);
}

#[test]
fn nuclear_bfg_ground_item_effect_replays_with_identical_events_and_state() {
  let player_position = Position::new(4, 4);
  let target_position = Position::new(6, 4);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::NuclearBfg9000),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let mut setup = ReplayLog::new(31_000, 12, 12, player_position).with_player_config(player_config);
  setup.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Replay Target",
    500,
    0,
    (0, 0),
  ));
  setup.record_item(ItemSpawnSpec::new(target_position, ItemSpawnKind::Shotgun));
  setup.record_item(ItemSpawnSpec::new(
    target_position,
    ItemSpawnKind::SmallMedPack,
  ));

  let (initial, setup_events) = ReplayEngine::run(&setup).expect("replay setup should load");
  assert!(setup_events.is_empty());
  let item_ids: Vec<_> = initial.world().ground_items().keys().copied().collect();
  assert_eq!(item_ids.len(), 2);

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let expected_events = direct
    .step(command)
    .expect("direct Nuclear BFG fire should resolve");
  let destroyed_id = expected_events
    .iter()
    .find_map(|event| match event {
      GameEvent::GroundItemDestroyed { item_id, .. } => Some(*item_id),
      _ => None,
    })
    .expect("the replay seed should produce a destructive center roll");
  assert_eq!(destroyed_id, item_ids[0]);

  let mut replay = setup;
  replay.record_command(command);
  let (replayed, replay_events) =
    ReplayEngine::run(&replay).expect("Nuclear BFG replay should resolve");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, expected_events);
  assert!(
    ReplayEngine::verify_determinism(&replay).expect("Nuclear BFG replay should be deterministic")
  );
}
