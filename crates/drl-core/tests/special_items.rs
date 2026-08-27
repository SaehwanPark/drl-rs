//! Integration tests for special-use consumable items (Phase Device teleportation).

use drl_core::game::Game;
use drl_core::grid::Tile;
use drl_core::item::Item;
use drl_core::replay::ReplayEngine;
use drl_protocol::{
  ActionCost, Command, CommandError, Direction, EquipmentSlot, GameEvent, ItemId, ItemSpawnKind,
  ItemSpawnSpec, MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog,
};

fn equipped_nuclear_bfg(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new_arena(seed, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::nuclear_bfg9000(weapon_id))
    .unwrap();
  (game, weapon_id)
}

fn equipped_standard_bfg(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new(seed, 10, 6, drl_protocol::Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::bfg9000(weapon_id))
    .unwrap();
  (game, weapon_id)
}

fn equipped_revenants_launcher(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::revenants_launcher(weapon_id))
    .unwrap();
  (game, weapon_id)
}

fn equipped_nuclear_bfg_wide(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new_arena(seed, 24, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::nuclear_bfg9000(weapon_id))
    .unwrap();
  (game, weapon_id)
}

#[test]
fn standard_bfg_exact_hit_resolves_even_at_zero_accuracy() {
  let (mut game, _weapon_id) = equipped_standard_bfg(1);
  let target = drl_protocol::Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 1, (2, 4))
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
    .accuracy = 0;

  let events = game
    .step(Command::AttackRanged(target))
    .expect("standard BFG shot should resolve");
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::AttackResolved {
        attacker_id,
        target_id: event_target,
        outcome: drl_protocol::AttackOutcome::Hit { .. },
        is_ranged: true,
      } if *attacker_id == player_id && *event_target == target_id
    )
  }));
  assert_eq!(
    game
      .world()
      .get_actor(player_id)
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    60
  );
  assert_eq!(
    game.world().get_actor(target_id).unwrap().hp().current,
    500
      - events
        .iter()
        .find_map(|event| match event {
          GameEvent::DamageApplied {
            target_id: event_target,
            amount,
            ..
          } if *event_target == target_id => Some(*amount),
          _ => None,
        })
        .expect("BFG hit should apply damage")
  );
}

#[test]
fn standard_bfg_empty_clip_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_standard_bfg(2);
  let target = drl_protocol::Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 1, (2, 4))
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
    .current_clip = 0;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn standard_bfg_shot_cost_accepts_forty_cells_and_consumes_them_once() {
  let (mut game, _weapon_id) = equipped_standard_bfg(4);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 1, (2, 4))
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
    .current_clip = 40;

  let events = game
    .step(Command::AttackRanged(target))
    .expect("40 cells are sufficient for one BFG shot");
  assert_eq!(
    game
      .world()
      .get_actor(player_id)
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    0
  );
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::AttackResolved {
        attacker_id,
        target_id: event_target,
        outcome: drl_protocol::AttackOutcome::Hit { .. },
        is_ranged: true,
      } if *attacker_id == player_id && *event_target == target_id
    )
  }));
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::ActionCostPaid {
        entity_id,
        cost: ActionCost::RANGED_ATTACK,
      } if *entity_id == player_id
    )
  }));
}

#[test]
fn standard_bfg_below_shot_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_standard_bfg(5);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 1, (2, 4))
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
    .current_clip = 39;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn revenants_launcher_exact_hit_resolves_even_at_zero_accuracy() {
  let (mut game, _weapon_id) = equipped_revenants_launcher(6);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 1, (2, 4))
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
    .accuracy = 0;

  let events = game
    .step(Command::AttackRanged(target))
    .expect("Revenant's Launcher shot should resolve");
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::AttackResolved {
        attacker_id,
        target_id: event_target,
        outcome: drl_protocol::AttackOutcome::Hit { .. },
        is_ranged: true,
      } if *attacker_id == player_id && *event_target == target_id
    )
  }));
  assert_eq!(
    game
      .world()
      .get_actor(player_id)
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    0
  );
}

#[test]
fn revenants_launcher_exact_hit_rejections_are_atomic() {
  let target = Position::new(5, 2);
  let (mut empty_clip, _) = equipped_revenants_launcher(7);
  empty_clip
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 1, (2, 4))
    .unwrap();
  let player_id = empty_clip.world().player_id().unwrap();
  empty_clip
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 0;
  let before_empty = empty_clip.clone();
  assert_eq!(
    empty_clip.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(empty_clip, before_empty);

  let (mut invalid_target, _) = equipped_revenants_launcher(8);
  let before_invalid_target = invalid_target.clone();
  assert_eq!(
    invalid_target
      .step(Command::AttackRanged(Position::new(5, 2)))
      .unwrap_err(),
    CommandError::InvalidTarget(Position::new(5, 2))
  );
  assert_eq!(invalid_target, before_invalid_target);

  let (mut blocked, _) = equipped_revenants_launcher(9);
  blocked
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 1, (2, 4))
    .unwrap();
  blocked
    .world_mut()
    .map_mut()
    .set_tile(Position::new(3, 2), Tile::Wall);
  let before_blocked = blocked.clone();
  assert_eq!(
    blocked.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::LineOfSightBlocked(target)
  );
  assert_eq!(blocked, before_blocked);

  let mut out_of_range = Game::new_arena(10, 24, 12).unwrap();
  let out_target = Position::new(2, 2);
  out_of_range
    .world_mut()
    .spawn_monster(out_target, "Static Target", 500, 1, (2, 4))
    .unwrap();
  let out_player_id = out_of_range.world().player_id().unwrap();
  let out_weapon_id = out_of_range.world_mut().allocate_item_id();
  out_of_range
    .world_mut()
    .get_actor_mut(out_player_id)
    .unwrap()
    .equipment_mut()
    .equip(
      EquipmentSlot::Weapon,
      Item::revenants_launcher(out_weapon_id),
    )
    .unwrap();
  let before_out_of_range = out_of_range.clone();
  assert_eq!(
    out_of_range
      .step(Command::AttackRanged(out_target))
      .unwrap_err(),
    CommandError::TargetOutOfRange(out_target)
  );
  assert_eq!(out_of_range, before_out_of_range);
}

#[test]
fn nuclear_bfg_exact_hit_resolves_even_at_zero_accuracy() {
  let (mut game, _weapon_id) = equipped_nuclear_bfg(3);
  let target = drl_protocol::Position::new(9, 6);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 1, (2, 4))
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
    .accuracy = 0;

  let events = game
    .step(Command::AttackRanged(target))
    .expect("Nuclear BFG shot should resolve");
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::AttackResolved {
        attacker_id,
        target_id: event_target,
        outcome: drl_protocol::AttackOutcome::Hit { .. },
        is_ranged: true,
      } if *attacker_id == player_id && *event_target == target_id
    )
  }));
  assert_eq!(
    game
      .world()
      .get_actor(player_id)
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    0
  );
}

#[test]
fn nuclear_bfg_empty_clip_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_nuclear_bfg(4);
  let target = drl_protocol::Position::new(9, 6);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 1, (2, 4))
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
    .current_clip = 0;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn nuclear_bfg_below_shot_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_nuclear_bfg(41);
  let target = drl_protocol::Position::new(9, 6);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 1, (2, 4))
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
    .current_clip = 39;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn nuclear_bfg_exact_hit_rejections_are_atomic() {
  let (mut invalid_target, _) = equipped_nuclear_bfg(5);
  let invalid_position = drl_protocol::Position::new(8, 6);
  let before_invalid = invalid_target.clone();
  assert_eq!(
    invalid_target
      .step(Command::AttackRanged(invalid_position))
      .unwrap_err(),
    CommandError::InvalidTarget(invalid_position)
  );
  assert_eq!(invalid_target, before_invalid);

  let (mut blocked, _) = equipped_nuclear_bfg(6);
  let blocked_position = drl_protocol::Position::new(9, 6);
  blocked
    .world_mut()
    .spawn_monster(blocked_position, "Static Target", 500, 1, (2, 4))
    .unwrap();
  blocked
    .world_mut()
    .map_mut()
    .set_tile(drl_protocol::Position::new(8, 6), Tile::Wall);
  let before_blocked = blocked.clone();
  assert_eq!(
    blocked
      .step(Command::AttackRanged(blocked_position))
      .unwrap_err(),
    CommandError::LineOfSightBlocked(blocked_position)
  );
  assert_eq!(blocked, before_blocked);

  let (mut distant, _) = equipped_nuclear_bfg_wide(7);
  let distant_position = drl_protocol::Position::new(21, 6);
  distant
    .world_mut()
    .spawn_monster(distant_position, "Static Target", 500, 1, (2, 4))
    .unwrap();
  let before_distant = distant.clone();
  assert_eq!(
    distant
      .step(Command::AttackRanged(distant_position))
      .unwrap_err(),
    CommandError::TargetOutOfRange(distant_position)
  );
  assert_eq!(distant, before_distant);
}

#[test]
fn test_phase_device_use_teleports_player_and_updates_visibility() {
  let mut game = Game::new(9999, 20, 20, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();

  // Add Phase Device to player inventory
  let device_id = game.world_mut().allocate_item_id();
  let device = Item::phase_device(device_id);
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(device)
    .unwrap();

  let initial_pos = game.world().player().unwrap().position();
  assert_eq!(initial_pos, Position::new(2, 2));

  // Use Phase Device
  let events = game.step(Command::Use(device_id)).unwrap();

  // Verify PlayerTeleported event was emitted
  let teleport_event = events.iter().find_map(|e| match e {
    GameEvent::PlayerTeleported { from, to } => Some((*from, *to)),
    _ => None,
  });

  assert!(
    teleport_event.is_some(),
    "PlayerTeleported event must be emitted"
  );
  let (from, to) = teleport_event.unwrap();
  assert_eq!(from, initial_pos);
  assert_ne!(to, from);

  // Verify player position was updated and is within walkable map bounds
  let current_pos = game.world().player().unwrap().position();
  assert_eq!(current_pos, to);
  assert!(game.world().map().is_in_bounds(current_pos));
  assert!(game.world().map().is_walkable(current_pos));

  // Verify item was consumed from inventory
  assert!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(device_id)
      .is_none(),
    "Phase device should be consumed on use"
  );

  // Verify player exploration memory includes new position
  assert!(
    game.world().is_explored(current_pos),
    "New position must be explored in fog of war"
  );
}

#[test]
fn test_phase_device_pickup_and_replay_determinism() {
  let mut replay = ReplayLog::new(5555, 15, 15, Position::new(2, 2));
  replay.record_item(ItemSpawnSpec::new(
    Position::new(3, 2),
    ItemSpawnKind::PhaseDevice,
  ));

  // 1. Move East onto Phase Device
  replay.record_command(Command::Move(Direction::East));
  // 2. Pick up Phase Device
  replay.record_command(Command::Pickup);
  // 3. Move North
  replay.record_command(Command::Move(Direction::North));
  // 4. Wait
  replay.record_command(Command::Wait);

  let is_det = ReplayEngine::verify_determinism(&replay).unwrap();
  assert!(
    is_det,
    "Replay with Phase Device pickup must be deterministic"
  );
}

#[test]
fn null_pointer_hit_applies_target_score_branch_and_schedules_explosion() {
  let mut game = Game::new_arena(901, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let player_position = game.world().player().unwrap().position();
  let target_position = player_position + Direction::East;
  let target_id = game
    .world_mut()
    .spawn_monster(target_position, "Target", 30, 100, (1, 2))
    .unwrap();
  game
    .world_mut()
    .get_actor_mut(target_id)
    .unwrap()
    .set_score_count(3500);

  let weapon_id = game.world_mut().allocate_item_id();
  let mut weapon = Item::null_pointer(weapon_id);
  weapon.weapon_properties_mut().unwrap().accuracy = 100;
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, weapon)
    .unwrap();

  let events = game.step(Command::AttackRanged(target_position)).unwrap();
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::AttackResolved {
      target_id: resolved_target,
      outcome: drl_protocol::AttackOutcome::Hit { damage: 0, .. },
      is_ranged: true,
      ..
    } if *resolved_target == target_id
  )));
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::NullPointerHit {
      entity_id,
      item_id,
      target_id: resolved_target,
      target_is_boss: false,
      score_count_remaining: 1500,
    } if *entity_id == player_id && *item_id == weapon_id && *resolved_target == target_id
  )));
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::NullPointerExplosionScheduled {
      entity_id,
      target_id: resolved_target,
      delay: 50,
      radius: 1,
      damage: 10,
    } if *entity_id == player_id && *resolved_target == target_id
  )));
  assert_eq!(
    game.world().get_actor(target_id).unwrap().score_count(),
    1500
  );
}

#[test]
fn null_pointer_hit_applies_boss_score_branch_and_preserves_event_order() {
  let mut game = Game::new_arena(901, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let player_position = game.world().player().unwrap().position();
  let target_position = player_position + Direction::East;
  let target_id = game
    .world_mut()
    .spawn_monster(target_position, "Boss Target", 30, 100, (1, 2))
    .unwrap();
  let target = game.world_mut().get_actor_mut(target_id).unwrap();
  target.set_boss(true);
  target.set_score_count(3500);

  let weapon_id = game.world_mut().allocate_item_id();
  let mut weapon = Item::null_pointer(weapon_id);
  weapon.weapon_properties_mut().unwrap().accuracy = 100;
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, weapon)
    .unwrap();

  let events = game.step(Command::AttackRanged(target_position)).unwrap();
  let hit_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::NullPointerHit {
          entity_id,
          item_id,
          target_id: resolved_target,
          target_is_boss: true,
          score_count_remaining: 2500,
        } if *entity_id == player_id && *item_id == weapon_id && *resolved_target == target_id
      )
    })
    .expect("boss Null Pointer hit event must be emitted");
  let explosion_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::NullPointerExplosionScheduled {
          entity_id,
          target_id: resolved_target,
          delay: 50,
          radius: 1,
          damage: 10,
        } if *entity_id == player_id && *resolved_target == target_id
      )
    })
    .expect("boss Null Pointer explosion schedule must be emitted");
  assert!(hit_index < explosion_index);
  assert_eq!(
    game.world().get_actor(target_id).unwrap().score_count(),
    2500
  );
}

#[test]
fn null_pointer_replay_is_deterministic() {
  let mut replay =
    ReplayLog::new(902, 12, 12, Position::new(5, 5)).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::NullPointer),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(
    MonsterSpawnSpec::new(Position::new(6, 5), "Target", 30, 100, (1, 2)).with_boss(true),
  );
  replay.record_command(Command::AttackRanged(Position::new(6, 5)));
  let (game, _) = ReplayEngine::run(&replay).unwrap();
  assert!(
    game
      .world()
      .actors()
      .values()
      .find(|actor| actor.name() == "Target")
      .is_some_and(|actor| actor.is_boss())
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn acid_spitter_reload_converts_acid_and_spends_score() {
  let mut game = Game::new_arena(904, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let player_position = game.world().player().unwrap().position();
  game
    .world_mut()
    .map_mut()
    .set_tile(player_position, Tile::Acid);
  let weapon_id = game.world_mut().allocate_item_id();
  let weapon = Item::acid_spitter(weapon_id);
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player.set_score_count(1_500);
  player
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, weapon)
    .unwrap();

  let events = game.step(Command::Reload).unwrap();
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::AcidSpitterReloaded {
      entity_id,
      item_id,
      position,
      ammo_loaded: 1,
      current_clip: 1,
      max_clip: 10,
      score_count_remaining: 500,
    } if *entity_id == player_id && *item_id == weapon_id && *position == player_position
  )));
  assert_eq!(
    game.world().map().get_tile(player_position),
    Some(Tile::Water)
  );
  let player = game.world().player().unwrap();
  assert_eq!(player.score_count(), 500);
  assert_eq!(
    player
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    1
  );
}

#[test]
fn acid_spitter_reload_rejects_non_acid_atomically() {
  let mut game = Game::new_arena(905, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  let weapon = Item::acid_spitter(weapon_id);
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, weapon)
    .unwrap();
  let before = game.clone();

  assert_eq!(
    game.step(Command::Reload),
    Err(CommandError::NoMatchingAmmo)
  );
  assert_eq!(game, before);
}

#[test]
fn acid_spitter_reload_rejects_full_clip_atomically() {
  let mut game = Game::new_arena(9051, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let player_position = game.world().player().unwrap().position();
  game
    .world_mut()
    .map_mut()
    .set_tile(player_position, Tile::Acid);
  let weapon_id = game.world_mut().allocate_item_id();
  let mut weapon = Item::acid_spitter(weapon_id);
  weapon.weapon_properties_mut().unwrap().current_clip = 10;
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, weapon)
    .unwrap();
  let before = game.clone();

  assert_eq!(
    game.step(Command::Reload),
    Err(CommandError::ClipAlreadyFull)
  );
  assert_eq!(game, before);
}

#[test]
fn acid_spitter_replay_preserves_custom_terrain_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(906, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::AcidSpitter),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_tile(player_start, drl_protocol::TileKind::Acid);
  replay.record_command(Command::Reload);

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
  let (game, events) = ReplayEngine::run(&replay).unwrap();
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::AcidSpitterReloaded {
      ammo_loaded: 1,
      current_clip: 1,
      score_count_remaining: -1000,
      ..
    }
  )));
  assert_eq!(game.world().map().get_tile(player_start), Some(Tile::Water));
}

#[test]
fn medical_powerarmor_repairs_on_the_thirtieth_accepted_command() {
  let mut game = Game::new_arena(777, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let armor_id = game.world_mut().allocate_item_id();
  let armor = Item::medical_powerarmor(armor_id);
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player.hp_mut().take_damage(30);
  player
    .equipment_mut()
    .equip(EquipmentSlot::Armor, armor)
    .unwrap();

  for _ in 0..29 {
    let events = game.step(Command::Wait).unwrap();
    assert!(
      !events
        .iter()
        .any(|event| matches!(event, GameEvent::MedicalPowerarmorRepaired { .. }))
    );
  }
  assert_eq!(game.world().player().unwrap().hp().current, 20);
  assert_eq!(game.world().player().unwrap().medical_repair_timer(), 29);

  let events = game.step(Command::Wait).unwrap();
  let repair_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::MedicalPowerarmorRepaired {
          entity_id,
          item_id,
          healed: 1,
          remaining_hp: 21,
          durability_remaining: 99,
          timer: 20,
        } if *entity_id == player_id && *item_id == armor_id
      )
    })
    .expect("repair event must be emitted");
  assert_eq!(repair_index, 2);
  assert!(matches!(events[0], GameEvent::TurnStarted { .. }));
  assert!(matches!(events[1], GameEvent::EntityWaited { .. }));
  assert!(matches!(events[3], GameEvent::ActionCostPaid { .. }));
  assert!(matches!(events[4], GameEvent::TurnEnded { .. }));
  assert_eq!(game.world().player().unwrap().hp().current, 21);
  assert_eq!(game.world().player().unwrap().medical_repair_timer(), 20);
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .equipment()
      .armor()
      .unwrap()
      .armor_properties()
      .unwrap()
      .durability,
    99
  );
}

#[test]
fn medical_powerarmor_replay_events_are_deterministic() {
  let mut replay =
    ReplayLog::new(778, 12, 12, Position::new(1, 1)).with_player_config(PlayerSpawnConfig {
      hp: 20,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Pistol),
      equipped_armor: Some(ItemSpawnKind::MedicalPowerarmor),
      equipped_armor_durability: None,
    });
  for _ in 0..30 {
    replay.record_command(Command::Wait);
  }

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
  let (_game, events) = ReplayEngine::run(&replay).unwrap();
  let repair_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::MedicalPowerarmorRepaired { .. }))
    .expect("replay must include the accepted-turn repair event");
  assert!(repair_index >= 2);
  assert!(matches!(
    events[repair_index - 2],
    GameEvent::TurnStarted { .. }
  ));
  assert!(matches!(
    events[repair_index - 1],
    GameEvent::EntityWaited { .. }
  ));
  assert!(matches!(
    events[repair_index + 1],
    GameEvent::ActionCostPaid { .. }
  ));
  assert!(matches!(
    events[repair_index + 2],
    GameEvent::TurnEnded { .. }
  ));
  assert!(matches!(
    events[repair_index],
    GameEvent::MedicalPowerarmorRepaired {
      healed: 1,
      remaining_hp: 21,
      durability_remaining: 99,
      timer: 20,
      ..
    }
  ));
}

#[test]
fn lava_armor_recharges_on_lava_after_five_accepted_commands() {
  let mut game = Game::new_arena(779, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let player_position = game.world().player().unwrap().position();
  game
    .world_mut()
    .map_mut()
    .set_tile(player_position, drl_core::Tile::Lava);

  let armor_id = game.world_mut().allocate_item_id();
  let armor = Item::lava_armor(armor_id);
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player
    .equipment_mut()
    .equip(EquipmentSlot::Armor, armor)
    .unwrap();
  player
    .equipment_mut()
    .armor_mut()
    .unwrap()
    .armor_properties_mut()
    .unwrap()
    .durability = 10;

  for _ in 0..4 {
    let events = game.step(Command::Wait).unwrap();
    assert!(
      !events
        .iter()
        .any(|event| matches!(event, GameEvent::LavaArmorRecharged { .. }))
    );
  }
  assert_eq!(game.world().player().unwrap().lava_recharge_timer(), 4);

  let events = game.step(Command::Wait).unwrap();
  let recharge_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::LavaArmorRecharged {
          entity_id,
          item_id,
          durability_restored: 3,
          durability_remaining: 13,
          timer: 0,
        } if *entity_id == player_id && *item_id == armor_id
      )
    })
    .expect("lava recharge event must be emitted");
  assert_eq!(recharge_index, 2);
  assert_eq!(game.world().player().unwrap().lava_recharge_timer(), 0);
}

#[test]
fn lava_armor_non_lava_interval_resets_without_recharge() {
  let mut game = Game::new_arena(780, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let armor_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player
    .equipment_mut()
    .equip(EquipmentSlot::Armor, Item::lava_armor(armor_id))
    .unwrap();
  player
    .equipment_mut()
    .armor_mut()
    .unwrap()
    .armor_properties_mut()
    .unwrap()
    .durability = 10;

  for _ in 0..5 {
    let events = game.step(Command::Wait).unwrap();
    assert!(
      !events
        .iter()
        .any(|event| matches!(event, GameEvent::LavaArmorRecharged { .. }))
    );
  }
  let player = game.world().player().unwrap();
  assert_eq!(player.lava_recharge_timer(), 0);
  assert_eq!(
    player
      .equipment()
      .armor()
      .unwrap()
      .armor_properties()
      .unwrap()
      .durability,
    10
  );
}

#[test]
fn lava_armor_replay_with_custom_lava_tile_is_deterministic() {
  let player_start = Position::new(2, 2);
  let mut replay = ReplayLog::new(781, 8, 8, player_start).with_player_config(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Pistol),
    equipped_armor: Some(ItemSpawnKind::LavaArmor),
    equipped_armor_durability: Some(97),
  });
  replay.record_tile(player_start, drl_protocol::TileKind::Lava);
  for _ in 0..5 {
    replay.record_command(Command::Wait);
  }

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
  let (_game, events) = ReplayEngine::run(&replay).unwrap();
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::LavaArmorRecharged {
      durability_restored: 3,
      durability_remaining: 100,
      timer: 0,
      ..
    }
  )));
}

#[test]
fn rejected_commands_roll_back_lava_recharge_state() {
  let mut game = Game::new(782, 5, 5, Position::new(1, 1)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let player_position = game.world().player().unwrap().position();
  game
    .world_mut()
    .map_mut()
    .set_tile(player_position, drl_core::Tile::Lava);
  let armor_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player
    .equipment_mut()
    .equip(EquipmentSlot::Armor, Item::lava_armor(armor_id))
    .unwrap();
  player
    .equipment_mut()
    .armor_mut()
    .unwrap()
    .armor_properties_mut()
    .unwrap()
    .durability = 10;
  for _ in 0..4 {
    game.step(Command::Wait).unwrap();
  }

  let before = game.clone();
  assert!(
    game
      .step(Command::AttackRanged(Position::new(99, 99)))
      .is_err()
  );
  assert_eq!(game, before);

  let events = game.step(Command::Wait).unwrap();
  assert!(
    events
      .iter()
      .any(|event| matches!(event, GameEvent::LavaArmorRecharged { .. }))
  );
}

#[test]
fn maleks_armor_recharges_after_fifty_five_accepted_commands() {
  let mut game = Game::new_arena(784, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let armor_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player
    .equipment_mut()
    .equip(EquipmentSlot::Armor, Item::maleks_armor(armor_id))
    .unwrap();
  player
    .equipment_mut()
    .armor_mut()
    .unwrap()
    .armor_properties_mut()
    .unwrap()
    .durability = 99;

  for _ in 0..54 {
    let events = game.step(Command::Wait).unwrap();
    assert!(
      !events
        .iter()
        .any(|event| matches!(event, GameEvent::MalekArmorRecharged { .. }))
    );
  }
  assert_eq!(game.world().player().unwrap().malek_recharge_timer(), 54);

  let events = game.step(Command::Wait).unwrap();
  let recharge_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::MalekArmorRecharged {
          entity_id,
          item_id,
          durability_restored: 1,
          durability_remaining: 100,
          timer: 50,
        } if *entity_id == player_id && *item_id == armor_id
      )
    })
    .expect("Malek's Armor recharge event must be emitted");
  assert_eq!(recharge_index, 2);
  assert!(matches!(events[0], GameEvent::TurnStarted { .. }));
  assert!(matches!(events[1], GameEvent::EntityWaited { .. }));
  assert!(matches!(events[3], GameEvent::ActionCostPaid { .. }));
  assert!(matches!(events[4], GameEvent::TurnEnded { .. }));
  assert_eq!(game.world().player().unwrap().malek_recharge_timer(), 50);

  let before_full = game.clone();
  let events = game.step(Command::Wait).unwrap();
  assert!(
    !events
      .iter()
      .any(|event| matches!(event, GameEvent::MalekArmorRecharged { .. }))
  );
  assert_eq!(game.world().player().unwrap().malek_recharge_timer(), 50);
  assert_ne!(
    game, before_full,
    "accepted full-armor wait still advances turn"
  );
}

#[test]
fn maleks_armor_damage_resets_recharge_timer() {
  let mut game = Game::new_arena(785, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let armor_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player
    .equipment_mut()
    .equip(EquipmentSlot::Armor, Item::maleks_armor(armor_id))
    .unwrap();
  player
    .equipment_mut()
    .armor_mut()
    .unwrap()
    .armor_properties_mut()
    .unwrap()
    .durability = 99;

  for _ in 0..12 {
    game.step(Command::Wait).unwrap();
  }
  assert_eq!(game.world().player().unwrap().malek_recharge_timer(), 12);
  game
    .world_mut()
    .apply_damage(player_id, 3, drl_protocol::DamageSource::Environment)
    .unwrap();
  assert_eq!(game.world().player().unwrap().malek_recharge_timer(), 0);
}

#[test]
fn rejected_commands_roll_back_maleks_armor_recharge_state() {
  let mut game = Game::new(786, 5, 5, Position::new(1, 1)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let armor_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player
    .equipment_mut()
    .equip(EquipmentSlot::Armor, Item::maleks_armor(armor_id))
    .unwrap();
  player
    .equipment_mut()
    .armor_mut()
    .unwrap()
    .armor_properties_mut()
    .unwrap()
    .durability = 99;
  for _ in 0..4 {
    game.step(Command::Wait).unwrap();
  }

  let before = game.clone();
  assert!(
    game
      .step(Command::AttackRanged(Position::new(99, 99)))
      .is_err()
  );
  assert_eq!(game, before);

  let events = game.step(Command::Wait).unwrap();
  assert!(
    !events
      .iter()
      .any(|event| matches!(event, GameEvent::MalekArmorRecharged { .. }))
  );
  assert_eq!(game.world().player().unwrap().malek_recharge_timer(), 5);
}

#[test]
fn blaster_recharge_timer_resets_on_fire_and_rejected_commands_are_atomic() {
  let mut game = Game::new(783, 10, 10, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let target_position = Position::new(8, 2);
  game
    .world_mut()
    .spawn_monster(target_position, "Static Target", 500, 0, (2, 4))
    .unwrap();

  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::blaster(weapon_id))
    .unwrap();
  game.step(Command::Equip(weapon_id)).unwrap();

  // A full clip does not advance the timer.
  for _ in 0..5 {
    game.step(Command::Wait).unwrap();
  }
  assert_eq!(game.world().player().unwrap().weapon_recharge_timer(), 0);

  game
    .step(Command::AttackRanged(target_position))
    .expect("first Blaster shot");
  let player = game.world().player().unwrap();
  assert_eq!(player.weapon_recharge_timer(), 1);
  assert_eq!(
    player
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    9
  );

  for _ in 0..3 {
    game.step(Command::Wait).unwrap();
  }
  assert_eq!(game.world().player().unwrap().weapon_recharge_timer(), 4);
  let before_rejection = game.clone();
  assert!(
    game
      .step(Command::AttackRanged(Position::new(99, 99)))
      .is_err()
  );
  assert_eq!(game, before_rejection);

  let events = game.step(Command::Wait).unwrap();
  assert!(
    !events
      .iter()
      .any(|event| matches!(event, GameEvent::WeaponRecharged { .. }))
  );
  assert_eq!(game.world().player().unwrap().weapon_recharge_timer(), 5);
}

#[test]
fn nuclear_plasma_recharge_timer_resets_on_fire() {
  let mut game = Game::new(784, 10, 10, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let target_position = Position::new(8, 2);
  game
    .world_mut()
    .spawn_monster(target_position, "Static Target", 500, 0, (2, 4))
    .unwrap();

  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::nuclear_plasma_rifle(weapon_id))
    .unwrap();
  game.step(Command::Equip(weapon_id)).unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 23;

  for _ in 0..10 {
    game.step(Command::Wait).unwrap();
  }
  assert_eq!(game.world().player().unwrap().weapon_recharge_timer(), 10);

  game
    .step(Command::AttackRanged(target_position))
    .expect("first Nuclear Plasma shot");
  let player = game.world().player().unwrap();
  assert_eq!(player.weapon_recharge_timer(), 1);
  assert_eq!(
    player
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    22
  );
}

#[test]
fn nuclear_plasma_overload_on_floor_removes_weapon_and_arms_nuke() {
  let mut game = Game::new_arena(787, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player.set_score_count(2_000);
  player
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::nuclear_plasma_rifle(weapon_id))
    .unwrap();

  let events = game
    .step(Command::AltReload {
      item_id: weapon_id,
      confirmed: true,
    })
    .unwrap();
  let overload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::NuclearWeaponOverloaded {
          entity_id,
          item_id,
          countdown: 100,
          score_count_remaining: 1_000,
        } if *entity_id == player_id && *item_id == weapon_id
      )
    })
    .expect("floor overload event must be emitted");
  let activate_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::NukeActivated { countdown: 100, .. }))
    .expect("floor overload must arm the nuke");
  let cost_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ActionCostPaid { entity_id, .. } if *entity_id == player_id
      )
    })
    .expect("accepted overload must pay the standard action cost");
  assert!(overload_index < activate_index);
  assert!(activate_index < cost_index);
  assert!(
    game
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .is_none()
  );
  assert_eq!(game.world().player().unwrap().score_count(), 1_000);
  assert_eq!(game.nuke_state().countdown(), Some(99));
  assert!(!game.is_game_over());
}

#[test]
fn nuclear_plasma_overload_on_acid_resolves_typed_nuke() {
  let mut game = Game::new_arena(788, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let player_position = game.world().player().unwrap().position();
  game
    .world_mut()
    .map_mut()
    .set_tile(player_position, Tile::Acid);
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::nuclear_plasma_rifle(weapon_id))
    .unwrap();

  let events = game
    .step(Command::AltReload {
      item_id: weapon_id,
      confirmed: true,
    })
    .unwrap();
  let overload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::NuclearWeaponOverloaded { countdown: 1, .. }
      )
    })
    .expect("hazard overload event must be emitted");
  let activate_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::NukeActivated { countdown: 1, .. }))
    .expect("hazard overload must arm a one-tick nuke");
  let level_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::LevelNuked { .. }))
    .expect("one-tick nuke must resolve");
  let damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied { target_id, .. } if *target_id == player_id
      )
    })
    .expect("resolved nuke must damage the player");
  let death_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ActorDied { entity_id, .. } if *entity_id == player_id
      )
    })
    .expect("resolved nuke must end the player");
  assert!(overload_index < activate_index);
  assert!(activate_index < level_index);
  assert!(level_index < damage_index);
  assert!(damage_index < death_index);
  assert!(game.is_game_over());
  assert!(
    game
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .is_none()
  );
}

#[test]
fn nuclear_plasma_overload_rejections_are_transactional() {
  let mut unconfirmed = Game::new_arena(789, 12, 12).unwrap();
  let player_id = unconfirmed.world().player_id().unwrap();
  let weapon_id = unconfirmed.world_mut().allocate_item_id();
  unconfirmed
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::nuclear_plasma_rifle(weapon_id))
    .unwrap();
  let before_unconfirmed = unconfirmed.clone();
  assert_eq!(
    unconfirmed
      .step(Command::AltReload {
        item_id: weapon_id,
        confirmed: false,
      })
      .unwrap_err(),
    drl_protocol::CommandError::AltReloadNotConfirmed(weapon_id)
  );
  assert_eq!(unconfirmed, before_unconfirmed);

  let mut partial = Game::new_arena(790, 12, 12).unwrap();
  let partial_player = partial.world().player_id().unwrap();
  let partial_id = partial.world_mut().allocate_item_id();
  partial
    .world_mut()
    .get_actor_mut(partial_player)
    .unwrap()
    .equipment_mut()
    .equip(
      EquipmentSlot::Weapon,
      Item::nuclear_plasma_rifle(partial_id),
    )
    .unwrap();
  partial
    .world_mut()
    .get_actor_mut(partial_player)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 23;
  let before_partial = partial.clone();
  assert_eq!(
    partial
      .step(Command::AltReload {
        item_id: partial_id,
        confirmed: true,
      })
      .unwrap_err(),
    drl_protocol::CommandError::CannotAltReload(partial_id)
  );
  assert_eq!(partial, before_partial);

  let mut stairs = Game::new_arena(791, 12, 12).unwrap();
  let stairs_player = stairs.world().player_id().unwrap();
  let stairs_position = stairs.world().player().unwrap().position();
  stairs
    .world_mut()
    .map_mut()
    .set_tile(stairs_position, Tile::StairsDown);
  let stairs_id = stairs.world_mut().allocate_item_id();
  stairs
    .world_mut()
    .get_actor_mut(stairs_player)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::nuclear_plasma_rifle(stairs_id))
    .unwrap();
  let before_stairs = stairs.clone();
  assert_eq!(
    stairs
      .step(Command::AltReload {
        item_id: stairs_id,
        confirmed: true,
      })
      .unwrap_err(),
    drl_protocol::CommandError::CannotAltReload(stairs_id)
  );
  assert_eq!(stairs, before_stairs);

  let mut pending = Game::new_arena(792, 12, 12).unwrap();
  let pending_player = pending.world().player_id().unwrap();
  let first_id = pending.world_mut().allocate_item_id();
  pending
    .world_mut()
    .get_actor_mut(pending_player)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::nuclear_plasma_rifle(first_id))
    .unwrap();
  pending
    .step(Command::AltReload {
      item_id: first_id,
      confirmed: true,
    })
    .unwrap();
  let second_id = pending.world_mut().allocate_item_id();
  pending
    .world_mut()
    .get_actor_mut(pending_player)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::nuclear_plasma_rifle(second_id))
    .unwrap();
  let before_pending = pending.clone();
  assert_eq!(
    pending
      .step(Command::AltReload {
        item_id: second_id,
        confirmed: true,
      })
      .unwrap_err(),
    drl_protocol::CommandError::CannotAltReload(second_id)
  );
  assert_eq!(pending, before_pending);
}

#[test]
fn nuclear_bfg_overload_on_floor_removes_weapon_and_arms_nuke() {
  let (mut game, weapon_id) = equipped_nuclear_bfg(793);
  let player_id = game.world().player_id().unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .set_score_count(2_000);

  let events = game
    .step(Command::AltReload {
      item_id: weapon_id,
      confirmed: true,
    })
    .unwrap();
  let overload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::NuclearWeaponOverloaded {
          entity_id,
          item_id,
          countdown: 100,
          score_count_remaining: 1_000,
        } if *entity_id == player_id && *item_id == weapon_id
      )
    })
    .expect("floor BFG overload event must be emitted");
  let activate_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::NukeActivated { countdown: 100, .. }))
    .expect("floor BFG overload must arm the nuke");
  let cost_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::ActionCostPaid { entity_id, .. } if *entity_id == player_id))
    .expect("accepted BFG overload must pay the action cost");
  assert!(overload_index < activate_index);
  assert!(activate_index < cost_index);
  assert!(
    game
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .is_none()
  );
  assert_eq!(game.world().player().unwrap().score_count(), 1_000);
  assert_eq!(game.nuke_state().countdown(), Some(99));
  assert!(!game.is_game_over());
}

#[test]
fn nuclear_bfg_overload_on_acid_resolves_typed_nuke() {
  let (mut game, weapon_id) = equipped_nuclear_bfg(794);
  let player_id = game.world().player_id().unwrap();
  let position = game.world().player().unwrap().position();
  game.world_mut().map_mut().set_tile(position, Tile::Acid);

  let events = game
    .step(Command::AltReload {
      item_id: weapon_id,
      confirmed: true,
    })
    .unwrap();
  let overload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::NuclearWeaponOverloaded { countdown: 1, .. }
      )
    })
    .expect("hazard BFG overload event must be emitted");
  let activate_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::NukeActivated { countdown: 1, .. }))
    .expect("hazard BFG overload must arm a one-tick nuke");
  let level_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::LevelNuked { .. }))
    .expect("hazard BFG overload must resolve the nuke");
  assert!(overload_index < activate_index);
  assert!(activate_index < level_index);
  assert!(game.is_game_over());
  assert!(
    game
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .is_none()
  );
  assert_eq!(game.world().get_actor(player_id).unwrap().hp().current, 0);
}

#[test]
fn nuclear_bfg_overload_rejections_are_transactional() {
  let (mut unconfirmed, unconfirmed_id) = equipped_nuclear_bfg(795);
  let before_unconfirmed = unconfirmed.clone();
  assert_eq!(
    unconfirmed
      .step(Command::AltReload {
        item_id: unconfirmed_id,
        confirmed: false,
      })
      .unwrap_err(),
    CommandError::AltReloadNotConfirmed(unconfirmed_id)
  );
  assert_eq!(unconfirmed, before_unconfirmed);

  let (mut partial, partial_id) = equipped_nuclear_bfg(796);
  let partial_player = partial.world().player_id().unwrap();
  partial
    .world_mut()
    .get_actor_mut(partial_player)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 39;
  let before_partial = partial.clone();
  assert_eq!(
    partial
      .step(Command::AltReload {
        item_id: partial_id,
        confirmed: true,
      })
      .unwrap_err(),
    CommandError::CannotAltReload(partial_id)
  );
  assert_eq!(partial, before_partial);

  let (mut stairs, stairs_id) = equipped_nuclear_bfg(797);
  let stairs_position = stairs.world().player().unwrap().position();
  stairs
    .world_mut()
    .map_mut()
    .set_tile(stairs_position, Tile::StairsDown);
  let before_stairs = stairs.clone();
  assert_eq!(
    stairs
      .step(Command::AltReload {
        item_id: stairs_id,
        confirmed: true,
      })
      .unwrap_err(),
    CommandError::CannotAltReload(stairs_id)
  );
  assert_eq!(stairs, before_stairs);

  let (mut pending, first_id) = equipped_nuclear_bfg(798);
  let player_id = pending.world().player_id().unwrap();
  pending
    .step(Command::AltReload {
      item_id: first_id,
      confirmed: true,
    })
    .unwrap();
  let second_id = pending.world_mut().allocate_item_id();
  pending
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::nuclear_bfg9000(second_id))
    .unwrap();
  let before_pending = pending.clone();
  assert_eq!(
    pending
      .step(Command::AltReload {
        item_id: second_id,
        confirmed: true,
      })
      .unwrap_err(),
    CommandError::CannotAltReload(second_id)
  );
  assert_eq!(pending, before_pending);
}

#[test]
fn rejected_commands_roll_back_medical_repair_state() {
  let mut game = Game::new(779, 5, 5, Position::new(1, 1)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let armor_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player.hp_mut().take_damage(30);
  player
    .equipment_mut()
    .equip(EquipmentSlot::Armor, Item::medical_powerarmor(armor_id))
    .unwrap();
  for _ in 0..7 {
    game.step(Command::Wait).unwrap();
  }
  let before = game.clone();

  assert!(game.step(Command::Move(Direction::North)).is_err());
  assert_eq!(game, before);
}

#[test]
fn medical_powerarmor_timer_moves_with_the_equipped_item() {
  let mut game = Game::new_arena(780, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let armor_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player.hp_mut().take_damage(30);
  player
    .equipment_mut()
    .equip(EquipmentSlot::Armor, Item::medical_powerarmor(armor_id))
    .unwrap();
  for _ in 0..7 {
    game.step(Command::Wait).unwrap();
  }
  assert_eq!(game.world().player().unwrap().medical_repair_timer(), 7);

  game.step(Command::Unequip(EquipmentSlot::Armor)).unwrap();
  assert_eq!(game.world().player().unwrap().medical_repair_timer(), 0);
  game.step(Command::Equip(armor_id)).unwrap();
  assert_eq!(game.world().player().unwrap().medical_repair_timer(), 8);
}

#[test]
fn subtle_knife_invoke_hits_visible_targets_in_entity_order() {
  let mut game = Game::new_arena(781, 30, 30).unwrap();
  let player_id = game.world().player_id().unwrap();
  let knife_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player.set_score_count(2_000);
  player
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::subtle_knife(knife_id))
    .unwrap();

  let visible_a = game
    .world_mut()
    .spawn_monster(Position::new(16, 15), "Visible A", 30, 1, (1, 1))
    .unwrap();
  let visible_b = game
    .world_mut()
    .spawn_monster(Position::new(15, 17), "Visible B", 30, 1, (1, 1))
    .unwrap();
  let hidden = game
    .world_mut()
    .spawn_monster(Position::new(25, 25), "Hidden", 30, 1, (1, 1))
    .unwrap();
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(17, 15), drl_core::Tile::Wall);
  let occluded = game
    .world_mut()
    .spawn_monster(Position::new(18, 15), "Occluded", 30, 1, (1, 1))
    .unwrap();

  let events = game.step(Command::Invoke(knife_id)).unwrap();
  let invoke = events
    .iter()
    .find_map(|event| match event {
      GameEvent::SubtleKnifeInvoked {
        entity_id,
        item_id,
        targets,
        remaining_hp,
        score_count_remaining,
      } => Some((
        *entity_id,
        *item_id,
        targets.clone(),
        *remaining_hp,
        *score_count_remaining,
      )),
      _ => None,
    })
    .expect("invoke event must be emitted");
  assert_eq!(invoke.0, player_id);
  assert_eq!(invoke.1, knife_id);
  assert_eq!(invoke.2, vec![visible_a, visible_b]);
  assert_eq!(invoke.3, 45);
  assert_eq!(invoke.4, 1_000);
  assert_eq!(game.world().get_actor(visible_a).unwrap().hp().current, 15);
  assert_eq!(game.world().get_actor(visible_b).unwrap().hp().current, 15);
  assert_eq!(game.world().get_actor(hidden).unwrap().hp().current, 30);
  assert_eq!(game.world().get_actor(occluded).unwrap().hp().current, 30);
  assert!(game.world().player().unwrap().is_tired());
  let damage_targets: Vec<_> = events
    .iter()
    .filter_map(|event| match event {
      GameEvent::DamageApplied {
        target_id,
        amount: 15,
        source: drl_protocol::DamageSource::Actor(attacker_id),
        ..
      } if *attacker_id == player_id => Some(*target_id),
      _ => None,
    })
    .collect();
  assert_eq!(damage_targets, vec![visible_a, visible_b]);
}

#[test]
fn subtle_knife_tired_invoke_rolls_back_without_spending_a_turn() {
  let mut game = Game::new_arena(782, 20, 20).unwrap();
  let player_id = game.world().player_id().unwrap();
  let knife_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player.set_tired(true);
  player
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::subtle_knife(knife_id))
    .unwrap();
  let before = game.clone();

  assert_eq!(
    game.step(Command::Invoke(knife_id)).unwrap_err(),
    drl_protocol::CommandError::CannotInvoke(knife_id)
  );
  assert_eq!(game, before);
}

#[test]
fn subtle_knife_invalid_item_rolls_back_without_spending_a_turn() {
  let mut game = Game::new_arena(786, 20, 20).unwrap();
  let before = game.clone();
  let invalid_item = ItemId::new(999);

  assert_eq!(
    game.step(Command::Invoke(invalid_item)).unwrap_err(),
    drl_protocol::CommandError::CannotInvoke(invalid_item)
  );
  assert_eq!(game, before);
}

#[test]
fn subtle_knife_lethal_target_events_follow_damage_order() {
  let mut game = Game::new_arena(787, 30, 30).unwrap();
  let player_id = game.world().player_id().unwrap();
  let knife_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::subtle_knife(knife_id))
    .unwrap();
  let target_id = game
    .world_mut()
    .spawn_monster(Position::new(16, 15), "Lethal", 10, 1, (1, 1))
    .unwrap();

  let events = game.step(Command::Invoke(knife_id)).unwrap();
  let invoke_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::SubtleKnifeInvoked { .. }))
    .unwrap();
  let damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          target_id: observed,
          amount: 10,
          ..
        } if *observed == target_id
      )
    })
    .unwrap();
  let death_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ActorDied { entity_id, .. } if *entity_id == target_id
      )
    })
    .unwrap();
  assert!(invoke_index < damage_index);
  assert!(damage_index < death_index);
}

#[test]
fn subtle_knife_invoke_pays_cost_without_visible_targets() {
  let mut game = Game::new_arena(784, 20, 20).unwrap();
  let player_id = game.world().player_id().unwrap();
  let knife_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player.hp_mut().current = 3;
  player
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::subtle_knife(knife_id))
    .unwrap();

  let events = game.step(Command::Invoke(knife_id)).unwrap();
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::SubtleKnifeInvoked {
      targets,
      remaining_hp: 1,
      score_count_remaining: -1000,
      ..
    } if targets.is_empty()
  )));
  assert_eq!(game.world().player().unwrap().hp().current, 1);
  assert!(game.world().player().unwrap().is_tired());
}

#[test]
fn subtle_knife_internal_damage_bypasses_target_armor() {
  let mut game = Game::new_arena(785, 30, 30).unwrap();
  let player_id = game.world().player_id().unwrap();
  let knife_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::subtle_knife(knife_id))
    .unwrap();
  let target_id = game
    .world_mut()
    .spawn_monster(Position::new(16, 15), "Armored", 20, 1, (1, 1))
    .unwrap();
  let armor_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(target_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Armor, Item::blue_armor(armor_id))
    .unwrap();

  game.step(Command::Invoke(knife_id)).unwrap();
  assert_eq!(game.world().get_actor(target_id).unwrap().hp().current, 5);
}

#[test]
fn subtle_knife_replay_with_player_config_is_deterministic() {
  let mut replay =
    ReplayLog::new(783, 30, 30, Position::new(15, 15)).with_player_config(PlayerSpawnConfig {
      hp: 20,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::SubtleKnife),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(MonsterSpawnSpec::new(
    Position::new(16, 15),
    "Visible A",
    30,
    1,
    (1, 1),
  ));
  replay.record_monster(MonsterSpawnSpec::new(
    Position::new(15, 17),
    "Visible B",
    30,
    1,
    (1, 1),
  ));
  replay.record_command(Command::Invoke(ItemId::new(4)));

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
  let (_game, events) = ReplayEngine::run(&replay).unwrap();
  assert!(
    events
      .iter()
      .any(|event| matches!(event, GameEvent::SubtleKnifeInvoked { .. }))
  );
}

#[test]
fn trigun_alt_reload_applies_costs_without_destroying_weapon() {
  let mut game = Game::new_arena(788, 20, 20).unwrap();
  let player_id = game.world().player_id().unwrap();
  let trigun_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  *player.hp_mut() = drl_protocol::HitPoints::new(12, 20);
  player.set_score_count(2_000);
  player
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::trigun(trigun_id))
    .unwrap();

  let events = game
    .step(Command::AltReload {
      item_id: trigun_id,
      confirmed: true,
    })
    .unwrap();

  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::TrigunAltReloaded {
      entity_id,
      item_id,
      remaining_hp: drl_protocol::HitPoints { current: 7, max: 15 },
      score_count_remaining: 1_000,
    } if *entity_id == player_id && *item_id == trigun_id
  )));
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .id(),
    trigun_id
  );
  assert!(game.is_game_over());
  assert_eq!(game.world().player().unwrap().hp().current, 0);
  assert!(game.nuke_state().level_nuked());
}

#[test]
fn trigun_alt_reload_rejections_are_transactional() {
  let mut declined = Game::new_arena(789, 20, 20).unwrap();
  let player_id = declined.world().player_id().unwrap();
  let trigun_id = declined.world_mut().allocate_item_id();
  declined
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::trigun(trigun_id))
    .unwrap();
  let before_declined = declined.clone();
  assert_eq!(
    declined
      .step(Command::AltReload {
        item_id: trigun_id,
        confirmed: false,
      })
      .unwrap_err(),
    drl_protocol::CommandError::AltReloadNotConfirmed(trigun_id)
  );
  assert_eq!(declined, before_declined);

  let mut low_health = Game::new_arena(790, 20, 20).unwrap();
  let low_player = low_health.world().player_id().unwrap();
  let low_id = low_health.world_mut().allocate_item_id();
  let low_actor = low_health.world_mut().get_actor_mut(low_player).unwrap();
  *low_actor.hp_mut() = drl_protocol::HitPoints::new(10, 10);
  low_actor
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::trigun(low_id))
    .unwrap();
  let before_low = low_health.clone();
  assert_eq!(
    low_health
      .step(Command::AltReload {
        item_id: low_id,
        confirmed: true,
      })
      .unwrap_err(),
    drl_protocol::CommandError::CannotAltReload(low_id)
  );
  assert_eq!(low_health, before_low);

  let mut missing = Game::new_arena(791, 20, 20).unwrap();
  let missing_id = ItemId::new(999);
  let before_missing = missing.clone();
  assert_eq!(
    missing
      .step(Command::AltReload {
        item_id: missing_id,
        confirmed: true,
      })
      .unwrap_err(),
    drl_protocol::CommandError::CannotAltReload(missing_id)
  );
  assert_eq!(missing, before_missing);
}

#[test]
fn trigun_nuke_events_resolve_in_typed_order_and_end_the_game() {
  let mut game = Game::new_arena(792, 20, 20).unwrap();
  let player_id = game.world().player_id().unwrap();
  let trigun_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::trigun(trigun_id))
    .unwrap();

  let events = game
    .step(Command::AltReload {
      item_id: trigun_id,
      confirmed: true,
    })
    .unwrap();
  let reload_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::TrigunAltReloaded { .. }))
    .unwrap();
  let activate_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::NukeActivated { .. }))
    .unwrap();
  let level_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::LevelNuked { .. }))
    .unwrap();
  let damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          target_id,
          amount: 45,
          source: drl_protocol::DamageSource::Environment,
          remaining_hp: 0,
          damage_type: None,
        } if *target_id == player_id
      )
    })
    .unwrap();
  let death_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ActorDied {
          entity_id,
          cause: drl_protocol::DeathCause::Environment,
        } if *entity_id == player_id
      )
    })
    .unwrap();
  assert!(reload_index < activate_index);
  assert!(activate_index < level_index);
  assert!(level_index < damage_index);
  assert!(damage_index < death_index);
  assert!(game.is_game_over());
  assert_eq!(
    game.step(Command::Wait).unwrap_err(),
    drl_protocol::CommandError::InvalidCommand("game is over".to_string())
  );
}

#[test]
fn trigun_alt_reload_replay_is_deterministic() {
  let mut replay =
    ReplayLog::new(793, 20, 20, Position::new(10, 10)).with_player_config(PlayerSpawnConfig {
      hp: 20,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Trigun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_command(Command::AltReload {
    item_id: ItemId::new(4),
    confirmed: true,
  });

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
  let (_game, events) = ReplayEngine::run(&replay).unwrap();
  assert!(
    events
      .iter()
      .any(|event| matches!(event, GameEvent::LevelNuked { .. }))
  );
}

#[test]
fn grammaton_alt_reload_cycles_modes_and_resolves_shot_counts() {
  let mut game = Game::new_arena(794, 20, 20).unwrap();
  let player_id = game.world().player_id().unwrap();
  let grammaton_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player.set_score_count(1_000);
  player
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::grammaton_beretta(grammaton_id))
    .unwrap();
  let target_id = game
    .world_mut()
    .spawn_monster(Position::new(12, 10), "Target", 200, 1, (1, 1))
    .unwrap();

  let events = game
    .step(Command::AltReload {
      item_id: grammaton_id,
      confirmed: false,
    })
    .unwrap();
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::GrammatonFireModeChanged {
      entity_id,
      item_id,
      mode: drl_protocol::WeaponFireMode::Burst,
      score_count_remaining: 800,
    } if *entity_id == player_id && *item_id == grammaton_id
  )));
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  assert_eq!(
    weapon.weapon_properties().unwrap().fire_mode,
    drl_protocol::WeaponFireMode::Burst
  );
  assert_eq!(weapon.weapon_properties().unwrap().damage, (1, 8));

  let events = game
    .step(Command::AttackRanged(Position::new(12, 10)))
    .unwrap();
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          target_id: observed,
          is_ranged: true,
          ..
        } if *attacker_id == player_id && *observed == target_id
      ))
      .count(),
    3
  );
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
      .current_clip,
    15
  );

  game
    .step(Command::AltReload {
      item_id: grammaton_id,
      confirmed: true,
    })
    .unwrap();
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
      .fire_mode,
    drl_protocol::WeaponFireMode::Auto
  );
  let events = game
    .step(Command::AttackRanged(Position::new(12, 10)))
    .unwrap();
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          target_id: observed,
          is_ranged: true,
          ..
        } if *attacker_id == player_id && *observed == target_id
      ))
      .count(),
    6
  );
  assert_eq!(game.world().player().unwrap().score_count(), 600);
}

#[test]
fn grammaton_partial_burst_rejection_preserves_game_and_rng() {
  let mut game = Game::new_arena(795, 20, 20).unwrap();
  let player_id = game.world().player_id().unwrap();
  let grammaton_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::grammaton_beretta(grammaton_id))
    .unwrap();
  game
    .step(Command::AltReload {
      item_id: grammaton_id,
      confirmed: true,
    })
    .unwrap();
  game
    .step(Command::AltReload {
      item_id: grammaton_id,
      confirmed: true,
    })
    .unwrap();
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
  let target_id = game
    .world_mut()
    .spawn_monster(Position::new(12, 10), "Target", 200, 1, (1, 1))
    .unwrap();
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRanged(Position::new(12, 10)))
      .unwrap_err(),
    drl_protocol::CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
  assert!(game.world().get_actor(target_id).unwrap().is_alive());
}

#[test]
fn grammaton_burst_stops_on_lethal_hit_and_drops_once() {
  let mut game = Game::new_arena(797, 20, 20).unwrap();
  let player_id = game.world().player_id().unwrap();
  let grammaton_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::grammaton_beretta(grammaton_id))
    .unwrap();
  game
    .step(Command::AltReload {
      item_id: grammaton_id,
      confirmed: true,
    })
    .unwrap();
  let target_position = Position::new(12, 10);
  let target_id = game
    .world_mut()
    .spawn_monster(target_position, "Dropper", 1, 1, (1, 1))
    .unwrap();
  game
    .world_mut()
    .get_actor_mut(target_id)
    .unwrap()
    .set_death_drop(Some(ItemSpawnKind::SmallMedPack));

  let events = game.step(Command::AttackRanged(target_position)).unwrap();

  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          target_id: observed,
          is_ranged: true,
          ..
        } if *observed == target_id
      ))
      .count(),
    1,
    "the burst must stop after the first lethal shot"
  );
  let death_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ActorDied { entity_id, .. } if *entity_id == target_id
      )
    })
    .unwrap();
  let drop_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ItemDropped { entity_id, .. } if *entity_id == target_id
      )
    })
    .unwrap();
  assert!(death_index < drop_index);
  assert_eq!(
    events
      .iter()
      .filter(
        |event| matches!(event, GameEvent::ItemDropped { entity_id, .. } if *entity_id == target_id)
      )
      .count(),
    1
  );
  assert_eq!(game.world().ground_items_at(target_position).len(), 1);
  assert!(!game.world().get_actor(target_id).unwrap().is_alive());
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
      .current_clip,
    15,
    "the selected three-round clip cost is committed even when the first shot kills"
  );
}

#[test]
fn grammaton_mode_replay_is_deterministic() {
  let mut replay =
    ReplayLog::new(796, 20, 20, Position::new(10, 10)).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::GrammatonBeretta),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(MonsterSpawnSpec::new(
    Position::new(12, 10),
    "Target",
    200,
    1,
    (1, 1),
  ));
  replay.record_command(Command::AltReload {
    item_id: ItemId::new(4),
    confirmed: true,
  });
  replay.record_command(Command::AttackRanged(Position::new(12, 10)));

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
  let (_game, events) = ReplayEngine::run(&replay).unwrap();
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::GrammatonFireModeChanged {
      mode: drl_protocol::WeaponFireMode::Burst,
      ..
    }
  )));
}

#[test]
fn jackhammer_alt_reload_toggles_modes_and_resolves_selected_shell_counts() {
  let mut game = Game::new_arena(798, 20, 20).unwrap();
  let player_id = game.world().player_id().unwrap();
  let jackhammer_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player.set_score_count(5);
  player
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::jackhammer(jackhammer_id))
    .unwrap();
  let target_id = game
    .world_mut()
    .spawn_monster(Position::new(12, 10), "Target", 200, 1, (1, 1))
    .unwrap();

  let events = game
    .step(Command::AttackRanged(Position::new(12, 10)))
    .unwrap();
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          target_id: observed,
          is_ranged: true,
          ..
        } if *attacker_id == player_id && *observed == target_id
      ))
      .count(),
    3
  );
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
      .current_clip,
    7
  );
  let target_position = game.world().get_actor(target_id).unwrap().position();

  let events = game
    .step(Command::AltReload {
      item_id: jackhammer_id,
      confirmed: false,
    })
    .unwrap();
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::JackhammerFireModeChanged {
      entity_id,
      item_id,
      mode: drl_protocol::WeaponFireMode::Single,
      score_count_remaining: 4,
    } if *entity_id == player_id && *item_id == jackhammer_id
  )));
  let events = game.step(Command::AttackRanged(target_position)).unwrap();
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::AttackResolved { target_id: observed, .. } if *observed == target_id))
      .count(),
    1
  );
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
      .current_clip,
    6
  );
}

#[test]
fn jackhammer_burst_stops_on_lethal_hit_and_drops_once() {
  let mut game = Game::new_arena(4, 20, 20).unwrap();
  let player_id = game.world().player_id().unwrap();
  let jackhammer_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::jackhammer(jackhammer_id))
    .unwrap();
  let target_position = Position::new(12, 10);
  let target_id = game
    .world_mut()
    .spawn_monster(target_position, "Dropper", 1, 1, (1, 1))
    .unwrap();
  game
    .world_mut()
    .get_actor_mut(target_id)
    .unwrap()
    .set_death_drop(Some(ItemSpawnKind::SmallMedPack));

  let events = game.step(Command::AttackRanged(target_position)).unwrap();
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          target_id: observed,
          is_ranged: true,
          ..
        } if *observed == target_id
      ))
      .count(),
    1,
    "a lethal first shell must stop the burst"
  );
  let attack_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::AttackResolved { target_id: observed, .. } if *observed == target_id
      )
    })
    .unwrap();
  let death_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ActorDied { entity_id, .. } if *entity_id == target_id
      )
    })
    .unwrap();
  let drop_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ItemDropped { entity_id, .. } if *entity_id == target_id
      )
    })
    .unwrap();
  assert!(attack_index < death_index);
  assert!(death_index < drop_index);
  assert_eq!(
    events
      .iter()
      .filter(
        |event| matches!(event, GameEvent::ItemDropped { entity_id, .. } if *entity_id == target_id)
      )
      .count(),
    1
  );
  assert_eq!(game.world().ground_items_at(target_position).len(), 1);
  assert!(!game.world().get_actor(target_id).unwrap().is_alive());
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
      .current_clip,
    7,
    "the selected three-shell cost is committed even when the first shell kills"
  );
}

#[test]
fn jackhammer_partial_burst_rejection_preserves_game_and_rng() {
  let mut game = Game::new_arena(799, 20, 20).unwrap();
  let player_id = game.world().player_id().unwrap();
  let jackhammer_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::jackhammer(jackhammer_id))
    .unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 2;
  let target_position = Position::new(12, 10);
  let target_id = game
    .world_mut()
    .spawn_monster(target_position, "Target", 200, 1, (1, 1))
    .unwrap();
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRanged(target_position))
      .unwrap_err(),
    drl_protocol::CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
  assert!(game.world().get_actor(target_id).unwrap().is_alive());
}

#[test]
fn jackhammer_mode_replay_is_deterministic() {
  let mut replay =
    ReplayLog::new(800, 20, 20, Position::new(10, 10)).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Jackhammer),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(MonsterSpawnSpec::new(
    Position::new(12, 10),
    "Target",
    200,
    1,
    (1, 1),
  ));
  replay.record_command(Command::AltReload {
    item_id: ItemId::new(4),
    confirmed: true,
  });
  replay.record_command(Command::AttackRanged(Position::new(12, 10)));

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}
