use drl_core::ReplayEngine;
use drl_core::game::Game;
use drl_core::item::Item;
use drl_protocol::{
  AttackOutcome, CURRENT_GAMEPLAY_SEMANTICS_VERSION, Command, DamageType, EquipmentSlot, GameEvent,
  ItemSpawnKind, MegaBusterMorphMode, MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog,
};

fn equipped_mega_buster(seed: u64) -> Game {
  let mut game = Game::new(seed, 16, 12, Position::new(2, 6)).expect("arena");
  let player_id = game.world().player_id().expect("player");
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .expect("player actor")
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::mega_buster(weapon_id))
    .expect("equip Mega Buster");
  game
}

fn spawn_target_with_weapon(
  game: &mut Game,
  position: Position,
  weapon_kind: Option<ItemSpawnKind>,
) -> drl_protocol::EntityId {
  let target_id = game
    .world_mut()
    .spawn_monster(position, "Equipped Target", 1, 0, (0, 0))
    .expect("target");
  if let Some(kind) = weapon_kind {
    let item_id = game.world_mut().allocate_item_id();
    game
      .world_mut()
      .get_actor_mut(target_id)
      .expect("target actor")
      .equipment_mut()
      .equip(EquipmentSlot::Weapon, Item::from_spawn_kind(item_id, kind))
      .expect("target weapon");
  }
  let player_id = game.world().player_id().expect("player");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .expect("player actor")
    .equipment_mut()
    .weapon_mut()
    .expect("Mega Buster")
    .weapon_properties_mut()
    .expect("Mega Buster properties")
    .accuracy = 100;
  target_id
}

#[test]
fn lethal_direct_hit_morphs_after_death_before_drop() {
  let mut game = equipped_mega_buster(46_301);
  let target = spawn_target_with_weapon(
    &mut game,
    Position::new(7, 6),
    Some(ItemSpawnKind::RocketLauncher),
  );
  game
    .world_mut()
    .get_actor_mut(target)
    .expect("target actor")
    .set_death_drop(Some(ItemSpawnKind::SmallMedPack));

  let events = game
    .step(Command::AttackRanged(Position::new(7, 6)))
    .expect("direct Mega Buster shot");
  let death_index = events
    .iter()
    .position(
      |event| matches!(event, GameEvent::ActorDied { entity_id, .. } if *entity_id == target),
    )
    .expect("target death");
  let morph_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::MegaBusterMorphed { target_id, current: MegaBusterMorphMode::Fire, .. } if *target_id == target))
    .expect("Mega Buster morph");
  assert!(death_index < morph_index, "morph follows ActorDied");
  let drop_index = events
    .iter()
    .position(
      |event| matches!(event, GameEvent::ItemDropped { entity_id, .. } if *entity_id == target),
    )
    .expect("target death drop");
  assert!(morph_index < drop_index, "morph precedes ItemDropped");

  let (previous, current) = events
    .iter()
    .find_map(|event| match event {
      GameEvent::MegaBusterMorphed {
        previous, current, ..
      } => Some((*previous, *current)),
      _ => None,
    })
    .expect("morph event");
  assert_eq!(previous, MegaBusterMorphMode::Bullet);
  assert_eq!(current, MegaBusterMorphMode::Fire);

  let player = game.world().player().expect("player");
  let weapon = player.equipment().weapon().expect("Mega Buster");
  assert_eq!(weapon.mega_buster_morph(), Some(MegaBusterMorphMode::Fire));
  assert_eq!(
    weapon.weapon_properties().expect("properties").damage,
    (4, 8)
  );
  assert_eq!(weapon.weapon_damage_type(), Some(DamageType::Fire));
  assert_eq!(
    weapon.weapon_properties().expect("properties").current_clip,
    51
  );
}

#[test]
fn target_weapon_family_selects_acid_and_plasma_profiles() {
  for (weapon_kind, expected_mode, expected_type, expected_damage) in [
    (
      ItemSpawnKind::AcidSpitter,
      MegaBusterMorphMode::Acid,
      DamageType::Acid,
      (4, 8),
    ),
    (
      ItemSpawnKind::PlasmaRifle,
      MegaBusterMorphMode::Plasma,
      DamageType::Plasma,
      (1, 10),
    ),
    (
      ItemSpawnKind::NullPointer,
      MegaBusterMorphMode::Plasma,
      DamageType::Plasma,
      (1, 10),
    ),
  ] {
    let mut game = equipped_mega_buster(46_302);
    let _target = spawn_target_with_weapon(&mut game, Position::new(7, 6), Some(weapon_kind));
    game
      .step(Command::AttackRanged(Position::new(7, 6)))
      .expect("direct Mega Buster shot");
    let weapon = game
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Mega Buster");
    assert_eq!(weapon.mega_buster_morph(), Some(expected_mode));
    assert_eq!(weapon.weapon_damage_type(), Some(expected_type));
    assert_eq!(
      weapon.weapon_properties().expect("properties").damage,
      expected_damage
    );
  }
}

#[test]
fn null_pointer_target_replay_selects_plasma_profile() {
  let target_position = Position::new(3, 6);
  let mut replay =
    ReplayLog::new(46_300, 16, 12, Position::new(2, 6)).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::MegaBuster),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Null Pointer Target", 1, 0, (0, 0))
      .with_equipped_weapon(Some(ItemSpawnKind::NullPointer)),
  );
  replay.record_command(Command::AttackRanged(target_position));

  let (game, events) = ReplayEngine::run(&replay).expect("Null Pointer morph replay");
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::MegaBusterMorphed {
      current: MegaBusterMorphMode::Plasma,
      ..
    }
  )));
  assert_eq!(
    game
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Mega Buster")
      .mega_buster_morph(),
    Some(MegaBusterMorphMode::Plasma)
  );
  assert!(ReplayEngine::verify_determinism(&replay).expect("Null Pointer replay determinism"));
}

#[test]
fn misses_and_nonlethal_hits_do_not_morph() {
  let mut miss_game = equipped_mega_buster(46_303);
  let _target = spawn_target_with_weapon(
    &mut miss_game,
    Position::new(7, 6),
    Some(ItemSpawnKind::RocketLauncher),
  );
  miss_game
    .world_mut()
    .player_mut()
    .expect("player")
    .equipment_mut()
    .weapon_mut()
    .expect("Mega Buster")
    .weapon_properties_mut()
    .expect("properties")
    .accuracy = 0;
  let events = miss_game
    .step(Command::AttackRanged(Position::new(7, 6)))
    .expect("miss");
  assert!(
    events
      .iter()
      .all(|event| !matches!(event, GameEvent::MegaBusterMorphed { .. }))
  );
  assert_eq!(
    miss_game
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Mega Buster")
      .mega_buster_morph(),
    Some(MegaBusterMorphMode::Bullet)
  );

  let mut nonlethal_game = equipped_mega_buster(46_304);
  let target = nonlethal_game
    .world_mut()
    .spawn_monster(Position::new(7, 6), "Tough Target", 100, 0, (0, 0))
    .expect("target");
  nonlethal_game
    .world_mut()
    .player_mut()
    .expect("player")
    .equipment_mut()
    .weapon_mut()
    .expect("Mega Buster")
    .weapon_properties_mut()
    .expect("properties")
    .accuracy = 100;
  let events = nonlethal_game
    .step(Command::AttackRanged(Position::new(7, 6)))
    .expect("nonlethal hit");
  assert!(
    events
      .iter()
      .all(|event| !matches!(event, GameEvent::MegaBusterMorphed { .. }))
  );
  assert!(
    nonlethal_game
      .world()
      .get_actor(target)
      .expect("target")
      .is_alive()
  );
}

#[test]
fn attack_outcome_is_still_direct_typed_damage() {
  let mut game = equipped_mega_buster(46_305);
  let target = spawn_target_with_weapon(&mut game, Position::new(7, 6), None);
  let events = game
    .step(Command::AttackRanged(Position::new(7, 6)))
    .expect("shot");
  assert!(events.iter().any(|event| matches!(event, GameEvent::AttackResolved { target_id, outcome: AttackOutcome::Hit { .. }, .. } if *target_id == target)));
  assert!(events.iter().any(|event| matches!(event, GameEvent::DamageApplied { target_id, damage_type: Some(DamageType::Physical), .. } if *target_id == target)));
}

#[test]
fn morph_replay_is_deterministic_and_rejects_stale_semantics() {
  let target_position = Position::new(3, 6);
  let mut replay =
    ReplayLog::new(46_306, 16, 12, Position::new(2, 6)).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::MegaBuster),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Acid Target", 1, 0, (0, 0))
      .with_equipped_weapon(Some(ItemSpawnKind::AcidSpitter)),
  );
  replay.record_command(Command::AttackRanged(target_position));

  assert_eq!(
    replay.metadata.gameplay_semantics_version,
    CURRENT_GAMEPLAY_SEMANTICS_VERSION
  );
  let (first_game, first_events) = ReplayEngine::run(&replay).expect("first morph replay");
  let (second_game, second_events) = ReplayEngine::run(&replay).expect("second morph replay");
  assert_eq!(first_game, second_game);
  assert_eq!(first_events, second_events);
  assert!(first_events.iter().any(|event| matches!(
    event,
    GameEvent::MegaBusterMorphed {
      current: MegaBusterMorphMode::Acid,
      ..
    }
  )));
  assert!(ReplayEngine::verify_determinism(&replay).expect("morph replay determinism"));

  let mut stale = replay;
  stale.metadata.gameplay_semantics_version = CURRENT_GAMEPLAY_SEMANTICS_VERSION - 1;
  let error = ReplayEngine::validate(&stale).expect_err("stale morph semantics");
  assert!(error.contains("unsupported gameplay semantics version"));
}

#[test]
fn rejected_mega_buster_command_preserves_morph_and_rng() {
  let mut game = equipped_mega_buster(46_307);
  game
    .world_mut()
    .spawn_monster(Position::new(7, 6), "Target", 100, 0, (0, 0))
    .expect("target");
  let player_id = game.world().player_id().expect("player");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .expect("player actor")
    .equipment_mut()
    .weapon_mut()
    .expect("Mega Buster")
    .apply_mega_buster_morph(MegaBusterMorphMode::Fire);
  game
    .world_mut()
    .get_actor_mut(player_id)
    .expect("player actor")
    .equipment_mut()
    .weapon_mut()
    .expect("Mega Buster")
    .weapon_properties_mut()
    .expect("Mega Buster properties")
    .current_clip = 8;
  let before = game.clone();
  let error = game
    .step(Command::AttackRanged(Position::new(7, 6)))
    .expect_err("under-supplied Mega Buster volley");
  assert!(matches!(error, drl_protocol::CommandError::NoAmmoInClip));
  assert_eq!(game, before);
}
