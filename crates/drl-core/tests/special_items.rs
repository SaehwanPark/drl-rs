//! Integration tests for special-use consumable items (Phase Device teleportation).

use drl_core::game::Game;
use drl_core::grid::Tile;
use drl_core::item::Item;
use drl_core::replay::ReplayEngine;
use drl_core::{radius_two_blast_positions, roll_explosion_damage};
use drl_protocol::{
  ActionCost, AttackOutcome, Command, CommandError, DamageSource, Direction, EquipmentSlot,
  GameEvent, ItemId, ItemSpawnKind, ItemSpawnSpec, MonsterSpawnSpec, PlayerSpawnConfig, Position,
  ReplayLog, TileKind,
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

fn equipped_bfg10k(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::bfg10k(weapon_id))
    .unwrap();
  (game, weapon_id)
}

fn equipped_double_shotgun(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::double_shotgun(weapon_id))
    .unwrap();
  (game, weapon_id)
}

fn equipped_super_shotgun(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::super_shotgun(weapon_id))
    .unwrap();
  (game, weapon_id)
}

fn equipped_minigun(seed: u64) -> (Game, ItemId) {
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
  (game, weapon_id)
}

fn equipped_chaingun(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::chaingun(weapon_id))
    .unwrap();
  (game, weapon_id)
}

fn equipped_laser_rifle(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::laser_rifle(weapon_id))
    .unwrap();
  (game, weapon_id)
}

fn equipped_frag_shotgun(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::frag_shotgun(weapon_id))
    .unwrap();
  (game, weapon_id)
}

fn equipped_railgun(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::railgun(weapon_id))
    .unwrap();
  (game, weapon_id)
}

fn equipped_null_pointer(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::null_pointer(weapon_id))
    .unwrap();
  (game, weapon_id)
}

fn equipped_tristar_blaster(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::tristar_blaster(weapon_id))
    .unwrap();
  (game, weapon_id)
}

fn equipped_acid_spitter(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::acid_spitter(weapon_id))
    .unwrap();
  (game, weapon_id)
}

fn equipped_mega_buster(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::mega_buster(weapon_id))
    .unwrap();
  (game, weapon_id)
}

fn equipped_plasma_shotgun(seed: u64) -> (Game, ItemId) {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::plasma_shotgun(weapon_id))
    .unwrap();
  (game, weapon_id)
}

fn assert_bfg10k_volley_events(
  events: &[GameEvent],
  attacker_id: drl_protocol::EntityId,
  target_id: drl_protocol::EntityId,
) {
  let mut attacks = Vec::new();
  let mut damages = Vec::new();
  let mut schedules = Vec::new();
  for (index, event) in events.iter().enumerate() {
    match event {
      GameEvent::AttackResolved {
        attacker_id: event_attacker,
        target_id: event_target,
        outcome: drl_protocol::AttackOutcome::Hit { damage, .. },
        is_ranged: true,
      } if *event_attacker == attacker_id && *event_target == target_id => {
        attacks.push((index, *damage));
      }
      GameEvent::DamageApplied {
        target_id: event_target,
        amount,
        source: DamageSource::Actor(_),
        damage_type: None,
        ..
      } if *event_target == target_id => damages.push((index, *amount)),
      GameEvent::Bfg10kExplosionScheduled {
        entity_id,
        target_id: event_target,
        delay,
        radius,
        knockback,
      } if *entity_id == attacker_id && *event_target == target_id => {
        schedules.push((index, *delay, *radius, *knockback));
      }
      _ => {}
    }
  }

  assert_eq!(attacks.len(), 5, "BFG 10K volley must resolve five hits");
  assert_eq!(
    damages.len(),
    5,
    "BFG 10K volley must apply five damage events"
  );
  assert_eq!(
    attacks
      .iter()
      .map(|(_, damage)| *damage)
      .collect::<Vec<_>>(),
    damages
      .iter()
      .map(|(_, amount)| *amount)
      .collect::<Vec<_>>(),
    "each projectile's resolved damage must be applied in order"
  );
  assert_eq!(
    schedules.len(),
    5,
    "BFG 10K volley must schedule five delayed explosions"
  );
  assert!(
    schedules
      .iter()
      .all(|(_, delay, radius, knockback)| (*delay, *radius, *knockback) == (25, 2, 16)),
    "each BFG 10K schedule must preserve delay 25, radius 2, and knockback 16"
  );
  for (((attack_index, _), (damage_index, _)), (schedule_index, _, _, _)) in
    attacks.iter().zip(damages.iter()).zip(schedules.iter())
  {
    assert_eq!(
      *damage_index,
      *attack_index + 1,
      "each BFG 10K attack must be followed immediately by its damage event"
    );
    assert_eq!(
      *schedule_index,
      *damage_index + 1,
      "each BFG 10K damage event must be followed immediately by its schedule"
    );
  }
}

fn assert_standard_bfg_schedule_event(
  events: &[GameEvent],
  attacker_id: drl_protocol::EntityId,
  target_id: drl_protocol::EntityId,
) {
  let attack_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id: event_attacker,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *event_attacker == attacker_id && *event_target == target_id
      )
    })
    .expect("standard BFG shot must resolve a hit");
  let damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied { target_id: event_target, .. }
          if *event_target == target_id
      )
    })
    .expect("standard BFG shot must apply damage");
  let (schedule_index, delay, radius, knockback) = events
    .iter()
    .enumerate()
    .find_map(|(index, event)| match event {
      GameEvent::Bfg9000ExplosionScheduled {
        entity_id,
        target_id: event_target,
        delay,
        radius,
        knockback,
      } if *entity_id == attacker_id && *event_target == target_id => {
        Some((index, *delay, *radius, *knockback))
      }
      _ => None,
    })
    .expect("standard BFG shot must schedule its delayed explosion");
  assert_eq!(damage_index, attack_index + 1);
  assert_eq!(schedule_index, damage_index + 1);
  assert_eq!((delay, radius, knockback), (33, 8, 16));
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg9000ExplosionScheduled { .. }))
      .count(),
    1
  );
}

fn assert_nuclear_bfg_schedule_event(
  events: &[GameEvent],
  attacker_id: drl_protocol::EntityId,
  target_id: drl_protocol::EntityId,
) {
  let attack_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id: event_attacker,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *event_attacker == attacker_id && *event_target == target_id
      )
    })
    .expect("Nuclear BFG shot must resolve a hit");
  let damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied { target_id: event_target, .. }
          if *event_target == target_id
      )
    })
    .expect("Nuclear BFG shot must apply damage");
  let (schedule_index, delay, radius, knockback) = events
    .iter()
    .enumerate()
    .find_map(|(index, event)| match event {
      GameEvent::NuclearBfg9000ExplosionScheduled {
        entity_id,
        target_id: event_target,
        delay,
        radius,
        knockback,
      } if *entity_id == attacker_id && *event_target == target_id => {
        Some((index, *delay, *radius, *knockback))
      }
      _ => None,
    })
    .expect("Nuclear BFG shot must schedule its delayed explosion");
  assert_eq!(damage_index, attack_index + 1);
  assert_eq!(schedule_index, damage_index + 1);
  assert_eq!((delay, radius, knockback), (33, 8, 16));
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::NuclearBfg9000ExplosionScheduled { .. }))
      .count(),
    1
  );
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
  let direct_damage = events
    .iter()
    .find_map(|event| match event {
      GameEvent::DamageApplied {
        target_id: event_target,
        amount,
        source: DamageSource::Actor(attacker_id),
        damage_type: None,
        ..
      } if *event_target == target_id && *attacker_id == player_id => Some(*amount),
      _ => None,
    })
    .expect("BFG hit should apply direct damage");
  let total_damage: u32 = events
    .iter()
    .filter_map(|event| match event {
      GameEvent::DamageApplied {
        target_id: event_target,
        amount,
        ..
      } if *event_target == target_id => Some(*amount),
      _ => None,
    })
    .sum();
  assert!(direct_damage > 0);
  assert_eq!(
    game.world().get_actor(target_id).unwrap().hp().current,
    500 - total_damage
  );
  assert_standard_bfg_schedule_event(&events, player_id, target_id);
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
  assert_standard_bfg_schedule_event(&events, player_id, target_id);
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
fn double_shotgun_below_dual_shot_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_double_shotgun(1);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 1, (9, 27))
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
    .current_clip = 1;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn super_shotgun_consumes_two_shells_for_two_projectiles() {
  let (mut game, _weapon_id) = equipped_super_shotgun(2_228);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (8, 32))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  let events = game
    .step(Command::AttackRanged(target))
    .expect("two shells are sufficient for one Super Shotgun volley");
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
    2,
    "Super Shotgun ordinary fire must resolve two ordered projectiles"
  );
}

#[test]
fn super_shotgun_below_dual_shot_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_super_shotgun(2_229);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (8, 32))
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
    .current_clip = 1;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn super_shotgun_replay_preserves_two_projectile_volley_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_230, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::SuperShotgun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 500, 100, (8, 32)));
  replay.record_command(Command::AttackRanged(target));

  let (game, events) = ReplayEngine::run(&replay).expect("Super Shotgun replay should run");
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
    0
  );
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
    2
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn minigun_consumes_eight_9mm_rounds_for_eight_projectiles() {
  let (mut game, _weapon_id) = equipped_minigun(2_233);
  let target = Position::new(5, 2);
  let target_id = game
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
    .current_clip = 8;
  let events = game
    .step(Command::AttackRanged(target))
    .expect("eight rounds are sufficient for one Minigun volley");
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
    8,
    "Minigun ordinary fire must resolve eight ordered projectiles"
  );
}

#[test]
fn minigun_below_eight_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_minigun(2_234);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 0, (1, 6))
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
    .current_clip = 7;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn minigun_replay_preserves_eight_projectile_volley_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_235, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
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
  replay.record_command(Command::AttackRanged(target));

  let (game, events) = ReplayEngine::run(&replay).expect("Minigun replay should run");
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
    192
  );
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
    8
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn chaingun_consumes_four_9mm_rounds_for_four_projectiles() {
  let (mut game, _weapon_id) = equipped_chaingun(2_236);
  let target = Position::new(5, 2);
  let target_id = game
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
    .current_clip = 4;
  let events = game
    .step(Command::AttackRanged(target))
    .expect("four rounds are sufficient for one Chaingun volley");
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
    4,
    "Chaingun ordinary fire must resolve four ordered projectiles"
  );
}

#[test]
fn chaingun_below_four_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_chaingun(2_237);
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
    .current_clip = 3;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn chaingun_first_chainfire_emits_three_projectiles_and_advances_state() {
  let (mut game, _weapon_id) = equipped_chaingun(2_240);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("first chainfire burst should be accepted");

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
    37
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
      .chainfire_level,
    1
  );
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
    3
  );
}

#[test]
fn chaingun_second_chainfire_emits_four_projectiles_and_advances_state() {
  let (mut game, _weapon_id) = equipped_chaingun(2_240);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  let first_events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("first chainfire burst should be accepted");
  assert_eq!(
    first_events
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
    3
  );

  let second_events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("second chainfire burst should be accepted");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 33);
  assert_eq!(weapon_properties.chainfire_level, 2);
  assert_eq!(
    second_events
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
    4,
    "Chaingun second chainfire must resolve four ordered projectiles"
  );
}

#[test]
fn chaingun_third_chainfire_emits_six_projectiles_and_advances_state() {
  let (mut game, _weapon_id) = equipped_chaingun(2_240);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second chainfire burst should be accepted");

  let third_events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("third chainfire burst should be accepted");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 27);
  assert_eq!(weapon_properties.chainfire_level, 3);
  assert_eq!(
    third_events
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
    6,
    "Chaingun third chainfire must resolve six ordered projectiles"
  );
}

#[test]
fn chaingun_fourth_chainfire_emits_six_projectiles_and_advances_state() {
  let (mut game, _weapon_id) = equipped_chaingun(2_240);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third chainfire burst should be accepted");

  let fourth_events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("fourth chainfire burst should be accepted");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 21);
  assert_eq!(weapon_properties.chainfire_level, 4);
  assert_eq!(
    fourth_events
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
    6,
    "Chaingun fourth chainfire must resolve six ordered projectiles"
  );
}

#[test]
fn chaingun_fifth_chainfire_emits_six_projectiles_and_advances_state() {
  let (mut game, _weapon_id) = equipped_chaingun(2_249);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("fourth chainfire burst should be accepted");

  let fifth_events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("fifth chainfire burst should be accepted");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 15);
  assert_eq!(weapon_properties.chainfire_level, 5);
  assert_eq!(
    fifth_events
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
    6,
    "Chaingun fifth chainfire must resolve six ordered projectiles"
  );
}

#[test]
fn chaingun_sixth_chainfire_emits_six_projectiles_and_advances_state() {
  let (mut game, _weapon_id) = equipped_chaingun(2_251);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  for _ in 0..5 {
    game
      .step(Command::AttackRangedChainfire(target))
      .expect("preceding chainfire burst should be accepted");
  }

  let sixth_events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("sixth chainfire burst should be accepted");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 9);
  assert_eq!(weapon_properties.chainfire_level, 6);
  assert_eq!(
    sixth_events
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
    6,
    "Chaingun sixth chainfire must resolve six ordered projectiles"
  );
}

#[test]
fn chaingun_chainfire_keeps_three_outcomes_after_lethal_target() {
  let mut lethal_case = None;

  // Search a fixed, deterministic seed window for a first-projectile lethal
  // hit so this regression exercises the dead-target continuation path.
  for seed in 2_245..2_300 {
    let (mut game, _weapon_id) = equipped_chaingun(seed);
    let target = Position::new(5, 2);
    let target_id = game
      .world_mut()
      .spawn_monster(target, "Fragile Target", 1, 100, (1, 6))
      .unwrap();
    let player_id = game.world().player_id().unwrap();
    let events = game
      .step(Command::AttackRangedChainfire(target))
      .expect("chainfire against a visible target should be accepted");
    let resolved = events
      .iter()
      .filter(|event| {
        matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        )
      })
      .count();

    if events.iter().any(
      |event| matches!(event, GameEvent::ActorDied { entity_id, .. } if *entity_id == target_id),
    ) {
      assert_eq!(resolved, 3);
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
    "fixed seed window should include a lethal first-projectile chainfire"
  );
}

#[test]
fn chaingun_chainfire_below_three_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_chaingun(2_241);
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
    .current_clip = 2;
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
fn chaingun_second_chainfire_below_four_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_chaingun(2_243);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 0, (1, 6))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first chainfire burst");
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
    .current_clip = 3;
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
fn chaingun_third_chainfire_below_six_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_chaingun(2_245);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second chainfire burst");
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
fn chaingun_ordinary_fire_resets_chainfire_warmup() {
  let (mut game, _weapon_id) = equipped_chaingun(2_242);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 0, (1, 6))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third chainfire burst");
  game
    .step(Command::AttackRanged(target))
    .expect("ordinary fire after chainfire");
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
fn chaingun_thirteenth_chainfire_level_is_rejected_without_mutation() {
  let (mut game, _weapon_id) = equipped_chaingun(2_244);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 0, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  let ammo_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::ammo_9mm(ammo_id, 40))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("fourth chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("fifth chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("sixth chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("seventh chainfire burst");
  game
    .step(Command::Reload)
    .expect("reload before eighth burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("eighth chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("ninth chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("tenth chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("eleventh chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("twelfth chainfire burst");
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::InvalidCommand("higher Chaingun chainfire levels are deferred".to_string())
  );
  assert_eq!(game, before);
}

#[test]
fn chaingun_eighth_chainfire_after_reload_emits_six_projectiles_and_advances_state() {
  let (mut game, _weapon_id) = equipped_chaingun(2_258);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  let ammo_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::ammo_9mm(ammo_id, 40))
    .unwrap();
  for _ in 0..7 {
    game
      .step(Command::AttackRangedChainfire(target))
      .expect("preceding chainfire burst");
  }
  let reload_events = game
    .step(Command::Reload)
    .expect("reload before eighth burst");
  assert!(reload_events.iter().any(|event| matches!(
    event,
    GameEvent::WeaponReloaded {
      ammo_loaded: 37,
      current_clip: 40,
      max_clip: 40,
      ..
    }
  )));

  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("eighth chainfire burst");
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
        } if *event_target == target_id && *attacker_id == player_id
      ))
      .count(),
    6
  );
  let properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(properties.current_clip, 34);
  assert_eq!(properties.chainfire_level, 8);
}

#[test]
fn chaingun_eighth_chainfire_below_six_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_chaingun(2_260);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  for _ in 0..7 {
    game
      .step(Command::AttackRangedChainfire(target))
      .expect("preceding chainfire burst");
  }
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
fn chaingun_ninth_chainfire_after_eighth_emits_six_projectiles_and_advances_state() {
  let (mut game, _weapon_id) = equipped_chaingun(2_262);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  let ammo_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::ammo_9mm(ammo_id, 40))
    .unwrap();
  for _ in 0..7 {
    game
      .step(Command::AttackRangedChainfire(target))
      .expect("preceding chainfire burst");
  }
  game
    .step(Command::Reload)
    .expect("reload before eighth burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("eighth chainfire burst");

  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("ninth chainfire burst");
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
        } if *event_target == target_id && *attacker_id == player_id
      ))
      .count(),
    6
  );
  let properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(properties.current_clip, 28);
  assert_eq!(properties.chainfire_level, 9);
}

#[test]
fn chaingun_ninth_chainfire_below_six_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_chaingun(2_264);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  let ammo_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::ammo_9mm(ammo_id, 40))
    .unwrap();
  for _ in 0..7 {
    game
      .step(Command::AttackRangedChainfire(target))
      .expect("preceding chainfire burst");
  }
  game
    .step(Command::Reload)
    .expect("reload before eighth burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("eighth chainfire burst");
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
fn chaingun_tenth_chainfire_after_ninth_emits_six_projectiles_and_advances_state() {
  let (mut game, _weapon_id) = equipped_chaingun(2_266);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  let ammo_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::ammo_9mm(ammo_id, 40))
    .unwrap();
  for _ in 0..7 {
    game
      .step(Command::AttackRangedChainfire(target))
      .expect("preceding chainfire burst");
  }
  game
    .step(Command::Reload)
    .expect("reload before eighth burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("eighth chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("ninth chainfire burst");

  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("tenth chainfire burst");
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
        } if *event_target == target_id && *attacker_id == player_id
      ))
      .count(),
    6
  );
  let properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(properties.current_clip, 22);
  assert_eq!(properties.chainfire_level, 10);
}

#[test]
fn chaingun_tenth_chainfire_below_six_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_chaingun(2_268);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  let ammo_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::ammo_9mm(ammo_id, 40))
    .unwrap();
  for _ in 0..7 {
    game
      .step(Command::AttackRangedChainfire(target))
      .expect("preceding chainfire burst");
  }
  game
    .step(Command::Reload)
    .expect("reload before eighth burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("eighth chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("ninth chainfire burst");
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
fn chaingun_eleventh_chainfire_after_tenth_emits_six_projectiles_and_advances_state() {
  let (mut game, _weapon_id) = equipped_chaingun(2_270);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  let ammo_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::ammo_9mm(ammo_id, 40))
    .unwrap();
  for _ in 0..7 {
    game
      .step(Command::AttackRangedChainfire(target))
      .expect("preceding chainfire burst");
  }
  game
    .step(Command::Reload)
    .expect("reload before eighth burst");
  for level in [8, 9, 10] {
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|_| panic!("chainfire burst at level {level}"));
  }

  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("eleventh chainfire burst");
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
        } if *event_target == target_id && *attacker_id == player_id
      ))
      .count(),
    6
  );
  let properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(properties.current_clip, 16);
  assert_eq!(properties.chainfire_level, 11);
}

#[test]
fn chaingun_eleventh_chainfire_below_six_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_chaingun(2_272);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  let ammo_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::ammo_9mm(ammo_id, 40))
    .unwrap();
  for _ in 0..7 {
    game
      .step(Command::AttackRangedChainfire(target))
      .expect("preceding chainfire burst");
  }
  game
    .step(Command::Reload)
    .expect("reload before eighth burst");
  for level in [8, 9, 10] {
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|_| panic!("chainfire burst at level {level}"));
  }
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
fn chaingun_twelfth_chainfire_after_eleventh_emits_six_projectiles_and_advances_state() {
  let (mut game, _weapon_id) = equipped_chaingun(2_274);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  let ammo_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::ammo_9mm(ammo_id, 40))
    .unwrap();
  for _ in 0..7 {
    game
      .step(Command::AttackRangedChainfire(target))
      .expect("preceding chainfire burst");
  }
  game
    .step(Command::Reload)
    .expect("reload before eighth burst");
  for level in [8, 9, 10, 11] {
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|_| panic!("chainfire burst at level {level}"));
  }

  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("twelfth chainfire burst");
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
        } if *event_target == target_id && *attacker_id == player_id
      ))
      .count(),
    6
  );
  let properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(properties.current_clip, 10);
  assert_eq!(properties.chainfire_level, 12);
}

#[test]
fn chaingun_twelfth_chainfire_below_six_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_chaingun(2_276);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  let ammo_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::ammo_9mm(ammo_id, 40))
    .unwrap();
  for _ in 0..7 {
    game
      .step(Command::AttackRangedChainfire(target))
      .expect("preceding chainfire burst");
  }
  game
    .step(Command::Reload)
    .expect("reload before eighth burst");
  for level in [8, 9, 10, 11] {
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|_| panic!("chainfire burst at level {level}"));
  }
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
fn chaingun_seventh_chainfire_emits_six_projectiles_and_advances_state() {
  let (mut game, _weapon_id) = equipped_chaingun(2_254);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  for _ in 0..6 {
    game
      .step(Command::AttackRangedChainfire(target))
      .expect("preceding chainfire burst");
  }

  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("seventh chainfire burst");
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
        } if *event_target == target_id && *attacker_id == game.world().player_id().unwrap()
      ))
      .count(),
    6
  );
  let properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(properties.current_clip, 3);
  assert_eq!(properties.chainfire_level, 7);
}

#[test]
fn chaingun_seventh_chainfire_below_six_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_chaingun(2_256);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  for _ in 0..6 {
    game
      .step(Command::AttackRangedChainfire(target))
      .expect("preceding chainfire burst");
  }
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
fn chaingun_fourth_chainfire_below_six_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_chaingun(2_246);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third chainfire burst");
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
fn chaingun_fifth_chainfire_below_six_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_chaingun(2_250);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  for _ in 0..4 {
    game
      .step(Command::AttackRangedChainfire(target))
      .expect("preceding chainfire burst");
  }
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
fn chaingun_sixth_chainfire_below_six_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_chaingun(2_252);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 6))
    .unwrap();
  for _ in 0..5 {
    game
      .step(Command::AttackRangedChainfire(target))
      .expect("preceding chainfire burst");
  }
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
fn chaingun_replay_preserves_four_projectile_volley_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_238, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Chaingun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 500, 100, (1, 6)));
  replay.record_command(Command::AttackRanged(target));

  let (game, events) = ReplayEngine::run(&replay).expect("Chaingun replay should run");
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
    36
  );
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
    4
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn chaingun_chainfire_replay_preserves_three_projectile_burst_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_243, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Chaingun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 500, 100, (1, 6)));
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) = ReplayEngine::run(&replay).expect("Chaingun chainfire replay should run");
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
    37
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
      .chainfire_level,
    1
  );
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
    3
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn chaingun_chainfire_replay_preserves_three_then_four_projectile_bursts_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_246, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Chaingun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 6)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) = ReplayEngine::run(&replay).expect("Chaingun chainfire replay should run");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 33);
  assert_eq!(weapon_properties.chainfire_level, 2);
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
    7
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn chaingun_chainfire_replay_preserves_three_then_four_then_six_projectile_bursts_deterministically()
 {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_247, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Chaingun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 6)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) = ReplayEngine::run(&replay).expect("Chaingun chainfire replay should run");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 27);
  assert_eq!(weapon_properties.chainfire_level, 3);
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
    13
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn chaingun_chainfire_replay_preserves_fourth_level_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_248, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Chaingun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 6)));
  for _ in 0..4 {
    replay.record_command(Command::AttackRangedChainfire(target));
  }

  let (game, events) =
    ReplayEngine::run(&replay).expect("Chaingun fourth chainfire replay should run");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 21);
  assert_eq!(weapon_properties.chainfire_level, 4);
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
    19
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn chaingun_chainfire_replay_preserves_fifth_level_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_250, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Chaingun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 6)));
  for _ in 0..5 {
    replay.record_command(Command::AttackRangedChainfire(target));
  }

  let (game, events) =
    ReplayEngine::run(&replay).expect("Chaingun fifth chainfire replay should run");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 15);
  assert_eq!(weapon_properties.chainfire_level, 5);
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
    25
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn chaingun_chainfire_replay_preserves_sixth_level_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_252, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Chaingun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 6)));
  for _ in 0..6 {
    replay.record_command(Command::AttackRangedChainfire(target));
  }

  let (game, events) =
    ReplayEngine::run(&replay).expect("Chaingun sixth chainfire replay should run");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 9);
  assert_eq!(weapon_properties.chainfire_level, 6);
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
    31
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn chaingun_chainfire_replay_preserves_seventh_level_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_254, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Chaingun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 6)));
  for _ in 0..7 {
    replay.record_command(Command::AttackRangedChainfire(target));
  }

  let (game, events) =
    ReplayEngine::run(&replay).expect("Chaingun seventh chainfire replay should run");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 3);
  assert_eq!(weapon_properties.chainfire_level, 7);
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
    37
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn chaingun_chainfire_replay_preserves_eighth_level_after_reload_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_258, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(40)],
      equipped_weapon: Some(ItemSpawnKind::Chaingun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 6)));
  for _ in 0..7 {
    replay.record_command(Command::AttackRangedChainfire(target));
  }
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) =
    ReplayEngine::run(&replay).expect("Chaingun eighth chainfire replay should run");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 34);
  assert_eq!(weapon_properties.chainfire_level, 8);
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
    43
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn chaingun_chainfire_replay_preserves_ninth_level_after_eighth_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_262, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(40)],
      equipped_weapon: Some(ItemSpawnKind::Chaingun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 6)));
  for _ in 0..7 {
    replay.record_command(Command::AttackRangedChainfire(target));
  }
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) =
    ReplayEngine::run(&replay).expect("Chaingun ninth chainfire replay should run");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 28);
  assert_eq!(weapon_properties.chainfire_level, 9);
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
    49
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn chaingun_chainfire_replay_preserves_tenth_level_after_ninth_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_266, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(40)],
      equipped_weapon: Some(ItemSpawnKind::Chaingun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 6)));
  for _ in 0..7 {
    replay.record_command(Command::AttackRangedChainfire(target));
  }
  replay.record_command(Command::Reload);
  for _ in 0..3 {
    replay.record_command(Command::AttackRangedChainfire(target));
  }

  let (game, events) =
    ReplayEngine::run(&replay).expect("Chaingun tenth chainfire replay should run");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 22);
  assert_eq!(weapon_properties.chainfire_level, 10);
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
    55
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn chaingun_chainfire_replay_preserves_eleventh_level_after_tenth_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_270, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(40)],
      equipped_weapon: Some(ItemSpawnKind::Chaingun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 6)));
  for _ in 0..7 {
    replay.record_command(Command::AttackRangedChainfire(target));
  }
  replay.record_command(Command::Reload);
  for _ in 0..4 {
    replay.record_command(Command::AttackRangedChainfire(target));
  }

  let (game, events) =
    ReplayEngine::run(&replay).expect("Chaingun eleventh chainfire replay should run");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 16);
  assert_eq!(weapon_properties.chainfire_level, 11);
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
    61
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn chaingun_chainfire_replay_preserves_twelfth_level_after_eleventh_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_274, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(40)],
      equipped_weapon: Some(ItemSpawnKind::Chaingun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 6)));
  for _ in 0..7 {
    replay.record_command(Command::AttackRangedChainfire(target));
  }
  replay.record_command(Command::Reload);
  for _ in 0..5 {
    replay.record_command(Command::AttackRangedChainfire(target));
  }

  let (game, events) =
    ReplayEngine::run(&replay).expect("Chaingun twelfth chainfire replay should run");
  let weapon_properties = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon_properties.current_clip, 10);
  assert_eq!(weapon_properties.chainfire_level, 12);
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
    67
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn laser_rifle_consumes_five_cells_for_five_projectiles() {
  let (mut game, _weapon_id) = equipped_laser_rifle(2_239);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (1, 7))
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
  let events = game
    .step(Command::AttackRanged(target))
    .expect("five cells are sufficient for one Laser Rifle volley");
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
    5,
    "Laser Rifle ordinary fire must resolve five ordered projectiles"
  );
}

#[test]
fn laser_rifle_below_five_cell_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_laser_rifle(2_240);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (1, 7))
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
    .current_clip = 4;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn laser_rifle_replay_preserves_five_projectile_volley_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_241, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::LaserRifle),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 500, 100, (1, 7)));
  replay.record_command(Command::AttackRanged(target));

  let (game, events) = ReplayEngine::run(&replay).expect("Laser Rifle replay should run");
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
    35
  );
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
    5
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn frag_shotgun_consumes_two_9mm_rounds_per_ordinary_shot() {
  let (mut game, _weapon_id) = equipped_frag_shotgun(2_220);
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
    .current_clip = 2;

  game
    .step(Command::AttackRanged(target))
    .expect("two-round Frag Shotgun shot should resolve");
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
    0
  );
}

#[test]
fn frag_shotgun_below_two_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_frag_shotgun(2_221);
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
    .current_clip = 1;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn railgun_consumes_five_cells_per_ordinary_shot() {
  let (mut game, _weapon_id) = equipped_railgun(2_222);
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
    .current_clip = 5;

  game
    .step(Command::AttackRanged(target))
    .expect("five-cell Railgun shot should resolve");
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
    0
  );
}

#[test]
fn railgun_below_five_cell_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_railgun(2_223);
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
    .current_clip = 4;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn railgun_piercing_hits_ordered_targets_with_shared_damage() {
  let target_positions = [Position::new(4, 2), Position::new(6, 2)];
  let candidate = (0..256)
    .find_map(|seed| {
      let (mut game, _weapon_id) = equipped_railgun(seed);
      for (index, position) in target_positions.into_iter().enumerate() {
        game
          .world_mut()
          .spawn_monster(
            position,
            if index == 0 {
              "Near Rail Target"
            } else {
              "Far Rail Target"
            },
            500,
            0,
            (2, 4),
          )
          .unwrap();
      }
      let player_id = game.world().player_id().unwrap();
      let events = game.step(Command::AttackRanged(target_positions[1])).ok()?;
      let hits: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
          GameEvent::AttackResolved {
            attacker_id,
            target_id,
            outcome: AttackOutcome::Hit { damage, .. },
            is_ranged: true,
          } if *attacker_id == player_id => Some((*target_id, *damage)),
          _ => None,
        })
        .collect();
      (hits.len() == 2).then_some((game, events, hits))
    })
    .expect("a bounded seed search should produce two Railgun hits");
  let (game, events, hits) = candidate;
  assert!(
    hits[0].0 < hits[1].0,
    "ray targets must be source-to-target ordered"
  );
  assert_eq!(hits[0].1, hits[1].1, "piercing hits share one damage roll");
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::DamageApplied {
          source: DamageSource::Actor(_),
          ..
        }
      ))
      .count(),
    2
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
    35
  );
}

#[test]
fn railgun_piercing_continues_after_lethal_intermediate_hit() {
  let target_positions = [Position::new(4, 2), Position::new(6, 2)];
  let seed = (0..256)
    .find(|seed| {
      let (mut game, _weapon_id) = equipped_railgun(*seed);
      game
        .world_mut()
        .spawn_monster(target_positions[0], "Lethal Rail Target", 1, 0, (2, 4))
        .unwrap();
      game
        .world_mut()
        .spawn_monster(target_positions[1], "Far Rail Target", 500, 0, (2, 4))
        .unwrap();
      let player_id = game.world().player_id().unwrap();
      game
        .step(Command::AttackRanged(target_positions[1]))
        .ok()
        .is_some_and(|events| {
          events.iter().filter(|event| matches!(event,
            GameEvent::AttackResolved { attacker_id, outcome: AttackOutcome::Hit { .. }, is_ranged: true, .. }
              if *attacker_id == player_id
          )).count() == 2
        })
    })
    .expect("a bounded seed search should produce two Railgun hits");
  let (mut game, _weapon_id) = equipped_railgun(seed);
  game
    .world_mut()
    .spawn_monster(target_positions[0], "Lethal Rail Target", 1, 0, (2, 4))
    .unwrap();
  let far_id = game
    .world_mut()
    .spawn_monster(target_positions[1], "Far Rail Target", 500, 0, (2, 4))
    .unwrap();
  let events = game
    .step(Command::AttackRanged(target_positions[1]))
    .expect("Railgun should continue through a lethal hit");
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::ActorDied { entity_id, .. } if *entity_id != far_id
  )));
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::DamageApplied { target_id, source: DamageSource::Actor(_), .. } if *target_id == far_id
  )));
}

#[test]
fn railgun_piercing_blocked_ray_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_railgun(2_225);
  let target = Position::new(6, 2);
  game
    .world_mut()
    .spawn_monster(target, "Blocked Rail Target", 500, 0, (2, 4))
    .unwrap();
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(4, 2), drl_core::Tile::Wall);
  let before = game.clone();
  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::LineOfSightBlocked(target)
  );
  assert_eq!(game, before);
}

#[test]
fn null_pointer_consumes_ten_cells_per_ordinary_shot() {
  let (mut game, _weapon_id) = equipped_null_pointer(2_224);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (2, 4))
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
    .current_clip = 10;

  game
    .step(Command::AttackRanged(target))
    .expect("ten-cell Null Pointer shot should resolve");
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
    0
  );
}

#[test]
fn null_pointer_below_ten_cell_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_null_pointer(2_225);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (2, 4))
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
    .current_clip = 9;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn tristar_blaster_consumes_fifteen_cells_for_three_projectiles() {
  let (mut game, _weapon_id) = equipped_tristar_blaster(2_226);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (2, 4))
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
    .current_clip = 15;
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .accuracy = 100;

  let events = game
    .step(Command::AttackRanged(target))
    .expect("fifteen cells are sufficient for one Tristar Blaster volley");
  let attacks = events
    .iter()
    .filter(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == player_id && *event_target == target_id
      )
    })
    .count();
  assert_eq!(attacks, 3);
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
    0
  );
}

#[test]
fn tristar_blaster_below_fifteen_cell_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_tristar_blaster(2_227);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (2, 4))
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
    .current_clip = 14;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn acid_spitter_consumes_ten_rockets_per_ordinary_shot() {
  let (mut game, _weapon_id) = equipped_acid_spitter(2_228);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (2, 4))
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
    .current_clip = 10;
  let events = game
    .step(Command::AttackRanged(target))
    .expect("ten rockets are sufficient for one Acid Spitter shot");
  assert_eq!(
    events
      .iter()
      .filter(|event| {
        matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        )
      })
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
    0
  );
}

#[test]
fn acid_spitter_below_ten_rocket_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_acid_spitter(2_229);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (2, 4))
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
    .current_clip = 9;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn mega_buster_consumes_nine_9mm_rounds_for_three_projectiles() {
  let (mut game, _weapon_id) = equipped_mega_buster(2_230);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (1, 8))
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
    .current_clip = 9;
  let events = game
    .step(Command::AttackRanged(target))
    .expect("nine 9mm rounds are sufficient for one Mega Buster volley");
  assert_eq!(
    events
      .iter()
      .filter(|event| {
        matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        )
      })
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
    0
  );
}

#[test]
fn mega_buster_below_nine_round_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_mega_buster(2_231);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (1, 8))
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
    .current_clip = 8;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn mega_buster_replay_preserves_three_projectile_volley_deterministically() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_232, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::MegaBuster),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 500, 100, (1, 8)));
  replay.record_command(Command::AttackRanged(target));

  let (game, events) = ReplayEngine::run(&replay).expect("Mega Buster replay should run");
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
    51
  );
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
    3
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn plasma_shotgun_consumes_three_cells_per_ordinary_shot() {
  let (mut game, _weapon_id) = equipped_plasma_shotgun(2_210);
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
    .current_clip = 3;

  game
    .step(Command::AttackRanged(target))
    .expect("three-cell Plasma Shotgun shot should resolve");
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
    0
  );
}

#[test]
fn plasma_shotgun_below_three_cell_cost_rejection_is_atomic() {
  let (mut game, _weapon_id) = equipped_plasma_shotgun(2_211);
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
    .current_clip = 2;
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn plasma_rifle_below_six_cell_cost_rejection_is_atomic() {
  let mut game = Game::new(2_270, 12, 8, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (1, 7))
    .unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::plasma_rifle(ItemId::new(4)))
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
fn bfg10k_exact_hit_resolves_even_at_zero_accuracy() {
  let (mut game, _weapon_id) = equipped_bfg10k(11);
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
    .expect("BFG 10K shot should resolve");
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
    5
  );
  assert_bfg10k_volley_events(&events, player_id, target_id);
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
    25
  );
}

#[test]
fn bfg10k_volley_consumes_twenty_five_cells_and_resolves_five_hits() {
  let target = Position::new(5, 2);
  let (mut game, _) = equipped_bfg10k(16);
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
    .current_clip = 25;
  let rng_before = game.rng().clone();

  let events = game
    .step(Command::AttackRanged(target))
    .expect("five-projectile BFG 10K volley should resolve");
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::AttackResolved {
      outcome: drl_protocol::AttackOutcome::Hit { .. },
      is_ranged: true,
      ..
    }
  )));
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
    5
  );
  let target_id = game
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("BFG 10K target")
    .id();
  assert_bfg10k_volley_events(&events, player_id, target_id);
  let mut expected_rng = rng_before;
  for _ in 0..5 {
    expected_rng.gen_range(6..25);
    for _ in radius_two_blast_positions(game.world().map(), target) {
      roll_explosion_damage(&mut expected_rng);
    }
  }
  assert_eq!(
    game.rng(),
    &expected_rng,
    "the five exact-hit projectiles must consume five ordered damage draws"
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
    0
  );
}

#[test]
fn bfg10k_exact_hit_rejections_are_atomic() {
  let target = Position::new(5, 2);
  let (mut empty_clip, _) = equipped_bfg10k(12);
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

  let (mut under_cost, _) = equipped_bfg10k(17);
  under_cost
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 1, (2, 4))
    .unwrap();
  let under_cost_player_id = under_cost.world().player_id().unwrap();
  under_cost
    .world_mut()
    .get_actor_mut(under_cost_player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 24;
  let before_under_cost = under_cost.clone();
  assert_eq!(
    under_cost.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(under_cost, before_under_cost);

  let (mut invalid_target, _) = equipped_bfg10k(13);
  let before_invalid_target = invalid_target.clone();
  assert_eq!(
    invalid_target
      .step(Command::AttackRanged(target))
      .unwrap_err(),
    CommandError::InvalidTarget(target)
  );
  assert_eq!(invalid_target, before_invalid_target);

  let (mut blocked, _) = equipped_bfg10k(14);
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

  let mut out_of_range = Game::new_arena(15, 24, 12).unwrap();
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
    .equip(EquipmentSlot::Weapon, Item::bfg10k(out_weapon_id))
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
  assert_nuclear_bfg_schedule_event(&events, player_id, target_id);
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
fn null_pointer_splash_hits_each_actor_once_and_continues_after_lethal() {
  let mut game = Game::new_arena(901, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let player_position = game.world().player().unwrap().position();
  let target_position = player_position + Direction::East;
  let north_position = target_position + Direction::North;
  let east_position = target_position + Direction::East;
  let target_id = game
    .world_mut()
    .spawn_monster(target_position, "Target", 100, 0, (0, 0))
    .unwrap();
  let lethal_id = game
    .world_mut()
    .spawn_monster(north_position, "Lethal", 10, 0, (0, 0))
    .unwrap();
  game
    .world_mut()
    .get_actor_mut(lethal_id)
    .unwrap()
    .set_death_drop(Some(ItemSpawnKind::Ammo9mm(5)));
  let survivor_id = game
    .world_mut()
    .spawn_monster(east_position, "Survivor", 100, 0, (0, 0))
    .unwrap();

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
  let schedule_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::NullPointerExplosionScheduled { .. }))
    .expect("Null Pointer splash must follow its schedule event");
  let damage_events: Vec<_> = events
    .iter()
    .enumerate()
    .filter_map(|(index, event)| match event {
      GameEvent::DamageApplied {
        target_id,
        amount,
        source: DamageSource::Environment,
        damage_type: Some(drl_protocol::DamageType::Plasma),
        ..
      } => Some((index, *target_id, *amount)),
      _ => None,
    })
    .collect();
  let monster_damage: Vec<_> = damage_events
    .iter()
    .copied()
    .filter(|(_, id, _)| *id != player_id)
    .collect();

  assert_eq!(monster_damage.len(), 3);
  assert_eq!(
    monster_damage
      .iter()
      .map(|(_, id, amount)| (*id, *amount))
      .collect::<Vec<_>>(),
    vec![(target_id, 10), (lethal_id, 10), (survivor_id, 10)]
  );
  assert!(
    damage_events
      .iter()
      .all(|(index, _, _)| *index > schedule_index)
  );
  let lethal_damage_index = damage_events
    .iter()
    .find(|(_, id, _)| *id == lethal_id)
    .map(|(index, _, _)| *index)
    .unwrap();
  let death_index = events
    .iter()
    .position(
      |event| matches!(event, GameEvent::ActorDied { entity_id, .. } if *entity_id == lethal_id),
    )
    .expect("lethal splash target must die");
  let drop_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::ItemDropped { entity_id, position, .. } if *entity_id == lethal_id && *position == north_position))
    .expect("lethal splash target must drop its configured item");
  assert!(lethal_damage_index < death_index);
  assert!(death_index < drop_index);
  assert_eq!(game.world().get_actor(target_id).unwrap().hp().current, 90);
  assert!(!game.world().get_actor(lethal_id).unwrap().is_alive());
  assert_eq!(
    game.world().get_actor(survivor_id).unwrap().hp().current,
    90
  );
  assert_eq!(game.world().player().unwrap().hp().current, 40);
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
    50
  );
}

#[test]
fn null_pointer_splash_death_drop_preflight_is_atomic() {
  let mut game = Game::new_arena(902, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let player_position = game.world().player().unwrap().position();
  let target_position = player_position + Direction::East;
  let blocked_drop_position = target_position + Direction::North;
  game
    .world_mut()
    .spawn_monster(target_position, "Target", 100, 0, (0, 0))
    .unwrap();
  let splash_id = game
    .world_mut()
    .spawn_monster(blocked_drop_position, "Splash Target", 100, 0, (0, 0))
    .unwrap();
  game
    .world_mut()
    .get_actor_mut(splash_id)
    .unwrap()
    .set_death_drop(Some(ItemSpawnKind::Ammo9mm(5)));
  game
    .world_mut()
    .map_mut()
    .set_tile(blocked_drop_position, Tile::Wall);

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

  let before = game.clone();
  let error = game
    .step(Command::AttackRanged(target_position))
    .unwrap_err();
  assert_eq!(error, CommandError::BlockedByTerrain(blocked_drop_position));
  assert_eq!(game, before);
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

  let events = game
    .step(Command::AttackRanged(target_position))
    .expect("first Nuclear Plasma volley");
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
    17
  );
}

#[test]
fn nuclear_plasma_below_six_cell_cost_rejection_is_atomic() {
  let mut game = Game::new(2_271, 12, 8, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (1, 7))
    .unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(
      EquipmentSlot::Weapon,
      Item::nuclear_plasma_rifle(ItemId::new(4)),
    )
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
  let before = game.clone();

  assert_eq!(
    game.step(Command::AttackRanged(target)).unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
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
fn pistol_aimed_fire_doubles_time_and_replays_deterministically() {
  let target_position = Position::new(4, 2);
  let mut replay =
    ReplayLog::new(2_260, 12, 8, Position::new(2, 2)).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(6)],
      equipped_weapon: Some(ItemSpawnKind::Pistol),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    100,
    (1, 7),
  ));
  replay.record_command(Command::AttackRangedAimed(target_position));

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
  let (game, events) = ReplayEngine::run(&replay).unwrap();
  let player = game.world().player().unwrap();
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
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::ActionCostPaid {
        cost: ActionCost(2_000),
        ..
      }
    )
  }));
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
    1
  );
}

#[test]
fn combat_pistol_aimed_fire_doubles_time_and_replays_deterministically() {
  let target_position = Position::new(4, 2);
  let mut replay =
    ReplayLog::new(2_264, 12, 8, Position::new(2, 2)).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(6)],
      equipped_weapon: Some(ItemSpawnKind::CombatPistol),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    100,
    (1, 7),
  ));
  replay.record_command(Command::AttackRangedAimed(target_position));

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
  let (game, events) = ReplayEngine::run(&replay).unwrap();
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
    14
  );
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::ActionCostPaid {
        cost: ActionCost(2_000),
        ..
      }
    )
  }));
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
    1
  );
}

#[test]
fn blaster_aimed_fire_doubles_time_resets_recharge_and_replays_deterministically() {
  let target_position = Position::new(4, 2);
  let mut replay =
    ReplayLog::new(2_266, 12, 8, Position::new(2, 2)).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(6)],
      equipped_weapon: Some(ItemSpawnKind::Blaster),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    100,
    (1, 7),
  ));
  replay.record_command(Command::AttackRangedAimed(target_position));

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
  let (game, events) = ReplayEngine::run(&replay).unwrap();
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
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::ActionCostPaid {
        cost: ActionCost(2_000),
        ..
      }
    )
  }));
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
    1
  );
}

#[test]
fn trigun_aimed_fire_doubles_time_and_replays_deterministically() {
  let target_position = Position::new(4, 2);
  let mut replay =
    ReplayLog::new(2_268, 12, 8, Position::new(2, 2)).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(6)],
      equipped_weapon: Some(ItemSpawnKind::Trigun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    100,
    (1, 7),
  ));
  replay.record_command(Command::AttackRangedAimed(target_position));

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
  let (game, events) = ReplayEngine::run(&replay).unwrap();
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
    5
  );
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::ActionCostPaid {
        cost: ActionCost(2_000),
        ..
      }
    )
  }));
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
    1
  );
}

#[test]
fn anti_freak_jackal_aimed_fire_doubles_time_and_replays_deterministically() {
  let target_position = Position::new(4, 2);
  let mut replay =
    ReplayLog::new(2_269, 12, 8, Position::new(2, 2)).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(6)],
      equipped_weapon: Some(ItemSpawnKind::AntiFreakJackal),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    100,
    (1, 7),
  ));
  replay.record_command(Command::AttackRangedAimed(target_position));

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
  let (game, events) = ReplayEngine::run(&replay).unwrap();
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
    5
  );
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::ActionCostPaid {
        cost: ActionCost(2_000),
        ..
      }
    )
  }));
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
    1
  );
}

#[test]
fn anti_freak_jackal_hit_records_delayed_explosion_schedule() {
  let target_position = Position::new(3, 2);
  let mut replay =
    ReplayLog::new(0, 10, 6, Position::new(2, 2)).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(6)],
      equipped_weapon: Some(ItemSpawnKind::AntiFreakJackal),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    0,
    (0, 0),
  ));
  replay.record_tile(Position::new(4, 2), TileKind::Wall);
  for _ in 0..6 {
    replay.record_command(Command::AttackRangedAimed(target_position));
  }

  let (game, events) = ReplayEngine::run(&replay).unwrap();
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
    0
  );
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::AntiFreakJackalExplosionScheduled {
        delay: 40,
        radius: 1,
        knockback: 8,
        ..
      }
    )
  }));
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn anti_freak_jackal_splash_fanout_hits_only_radius_one_actors() {
  let center = Position::new(3, 2);
  let blast_positions = [
    Position::new(3, 2),
    Position::new(3, 1),
    Position::new(4, 1),
    Position::new(4, 2),
    Position::new(4, 3),
    Position::new(3, 3),
    Position::new(2, 3),
    Position::new(2, 2),
    Position::new(2, 1),
  ];
  let far = Position::new(5, 2);
  let mut replay =
    ReplayLog::new(0, 8, 8, Position::new(1, 2)).with_player_config(PlayerSpawnConfig {
      hp: 500,
      max_hp: 500,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(6)],
      equipped_weapon: Some(ItemSpawnKind::AntiFreakJackal),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  for (index, position) in blast_positions.iter().enumerate() {
    replay.record_monster(MonsterSpawnSpec::new(
      *position,
      format!("Blast {index}"),
      500,
      1,
      (0, 0),
    ));
    replay.record_item(ItemSpawnSpec::new(*position, ItemSpawnKind::Ammo9mm(20)));
  }
  replay.record_monster(MonsterSpawnSpec::new(far, "Far", 500, 1, (0, 0)));
  replay.record_item(ItemSpawnSpec::new(far, ItemSpawnKind::Ammo9mm(20)));
  replay.record_command(Command::AttackRangedAimed(center));

  let (game, events) = ReplayEngine::run(&replay).unwrap();
  let splash_damage: Vec<_> = events
    .iter()
    .filter_map(|event| match event {
      GameEvent::DamageApplied {
        target_id,
        amount,
        source: drl_protocol::DamageSource::Environment,
        damage_type: Some(drl_protocol::DamageType::Fire),
        ..
      } => Some((*target_id, *amount)),
      _ => None,
    })
    .collect();
  let expected_ids: Vec<_> = game
    .world()
    .actors()
    .values()
    .filter(|actor| actor.name().starts_with("Blast "))
    .map(|actor| actor.id())
    .collect();
  assert_eq!(splash_damage.len(), expected_ids.len());
  assert_eq!(
    splash_damage.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
    expected_ids
  );
  assert!(
    splash_damage
      .iter()
      .all(|(_, amount)| (5..=15).contains(amount))
  );
  let center_id = expected_ids[0];
  for (index, event) in events.iter().enumerate() {
    let GameEvent::DamageApplied {
      target_id,
      amount,
      source: drl_protocol::DamageSource::Environment,
      damage_type: Some(drl_protocol::DamageType::Fire),
      ..
    } = event
    else {
      continue;
    };
    let actual_knockback = events
      .get(index.saturating_sub(1))
      .is_some_and(|event| matches!(event, GameEvent::ActorKnockedBack { entity_id, .. } if entity_id == target_id));
    if actual_knockback {
      assert_ne!(*target_id, center_id);
      assert!(*amount >= 8);
    }
  }
  assert!(
    events
      .iter()
      .any(|event| matches!(event, GameEvent::ActorKnockedBack { .. }))
  );
  let destroyed_positions: Vec<_> = events
    .iter()
    .filter_map(|event| match event {
      GameEvent::GroundItemDestroyed { position, .. } => Some(*position),
      _ => None,
    })
    .collect();
  let expected_destroyed = splash_damage
    .iter()
    .filter(|(_, amount)| *amount > 10)
    .count();
  assert_eq!(destroyed_positions.len(), expected_destroyed);
  assert!(
    destroyed_positions
      .iter()
      .all(|position| blast_positions.contains(position))
  );
  assert!(
    game
      .world()
      .ground_items_at(far)
      .iter()
      .any(|item| item.is_ammo())
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn anti_freak_jackal_splash_can_destroy_ammo_on_empty_blast_cells() {
  let center = Position::new(3, 2);
  let blast_positions = [
    center,
    Position::new(3, 1),
    Position::new(4, 1),
    Position::new(4, 2),
    Position::new(4, 3),
    Position::new(3, 3),
    Position::new(2, 3),
    Position::new(2, 2),
    Position::new(2, 1),
  ];
  let mut replay =
    ReplayLog::new(0, 8, 8, Position::new(1, 2)).with_player_config(PlayerSpawnConfig {
      hp: 500,
      max_hp: 500,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(6)],
      equipped_weapon: Some(ItemSpawnKind::AntiFreakJackal),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(MonsterSpawnSpec::new(
    center,
    "Blast Center",
    500,
    0,
    (0, 0),
  ));
  for position in blast_positions {
    replay.record_item(ItemSpawnSpec::new(position, ItemSpawnKind::Ammo9mm(20)));
  }
  replay.record_command(Command::AttackRangedAimed(center));

  let (_game, events) = ReplayEngine::run(&replay).unwrap();
  let destroyed_positions: Vec<_> = events
    .iter()
    .filter_map(|event| match event {
      GameEvent::GroundItemDestroyed { position, .. } => Some(*position),
      _ => None,
    })
    .collect();
  assert!(!destroyed_positions.is_empty());
  assert!(
    destroyed_positions
      .iter()
      .any(|position| *position != center)
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn aimed_fire_rejection_is_atomic_for_non_pistol_and_empty_clip() {
  let target_position = Position::new(4, 2);
  let mut unsupported = Game::new(2_261, 12, 8, Position::new(2, 2)).unwrap();
  let player_id = unsupported.world().player_id().unwrap();
  unsupported
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::shotgun(ItemId::new(4)))
    .unwrap();
  unsupported
    .world_mut()
    .spawn_monster(target_position, "Target", 500, 100, (1, 7))
    .unwrap();
  let before_unsupported = unsupported.clone();
  assert_eq!(
    unsupported
      .step(Command::AttackRangedAimed(target_position))
      .unwrap_err(),
    CommandError::InvalidCommand(
      "aimed fire is only available for the Pistol, Combat Pistol, Blaster, Trigun, or Anti-Freak Jackal".to_string(),
    )
  );
  assert_eq!(unsupported, before_unsupported);

  let mut empty_clip = Game::new(2_262, 12, 8, Position::new(2, 2)).unwrap();
  let player_id = empty_clip.world().player_id().unwrap();
  empty_clip
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::pistol(ItemId::new(4)))
    .unwrap();
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
  empty_clip
    .world_mut()
    .spawn_monster(target_position, "Target", 500, 100, (1, 7))
    .unwrap();
  let before_empty = empty_clip.clone();
  assert_eq!(
    empty_clip
      .step(Command::AttackRangedAimed(target_position))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(empty_clip, before_empty);

  let mut empty_trigun = Game::new(2_263, 12, 8, Position::new(2, 2)).unwrap();
  let player_id = empty_trigun.world().player_id().unwrap();
  empty_trigun
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::trigun(ItemId::new(4)))
    .unwrap();
  empty_trigun
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 0;
  empty_trigun
    .world_mut()
    .spawn_monster(target_position, "Target", 500, 100, (1, 7))
    .unwrap();
  let before_empty_trigun = empty_trigun.clone();
  assert_eq!(
    empty_trigun
      .step(Command::AttackRangedAimed(target_position))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(empty_trigun, before_empty_trigun);

  let mut empty_jackal = Game::new(2_264, 12, 8, Position::new(2, 2)).unwrap();
  let player_id = empty_jackal.world().player_id().unwrap();
  empty_jackal
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(
      EquipmentSlot::Weapon,
      Item::anti_freak_jackal(ItemId::new(5)),
    )
    .unwrap();
  empty_jackal
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 0;
  empty_jackal
    .world_mut()
    .spawn_monster(target_position, "Target", 500, 100, (1, 7))
    .unwrap();
  let before_empty_jackal = empty_jackal.clone();
  assert_eq!(
    empty_jackal
      .step(Command::AttackRangedAimed(target_position))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(empty_jackal, before_empty_jackal);
}

#[test]
fn blaster_aimed_fire_empty_clip_rejection_is_atomic() {
  let target_position = Position::new(4, 2);
  let mut game = Game::new(2_267, 12, 8, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::blaster(ItemId::new(4)))
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
    .current_clip = 0;
  game
    .world_mut()
    .spawn_monster(target_position, "Target", 500, 100, (1, 7))
    .unwrap();

  let before = game.clone();
  assert_eq!(
    game
      .step(Command::AttackRangedAimed(target_position))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
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
