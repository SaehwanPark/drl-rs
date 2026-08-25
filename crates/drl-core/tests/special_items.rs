//! Integration tests for special-use consumable items (Phase Device teleportation).

use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::replay::ReplayEngine;
use drl_protocol::{
  Command, Direction, EquipmentSlot, GameEvent, ItemId, ItemSpawnKind, ItemSpawnSpec,
  MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog,
};

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
