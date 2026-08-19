//! Application executable entry point and headless demo runner for DRL-Rust.

use drl_core::item::Item;
use drl_core::{Game, ReplayEngine};
use drl_protocol::{
  Command, Direction, ItemCategory, ItemSpawnKind, ItemSpawnSpec, MonsterSpawnSpec, Position,
  ReplayLog,
};

fn main() {
  println!(
    "DRL-Rust ({}, protocol {}) initialized.",
    drl_core::engine_name(),
    drl_protocol::protocol_version()
  );

  run_headless_demo();
}

fn run_headless_demo() {
  let seed = 42;
  let width = 20;
  let height = 10;
  let start_pos = Position::new(5, 5);

  println!("Starting headless simulation arena ({width}x{height}) with seed {seed}...");

  let mut game =
    Game::new(seed, width, height, start_pos).expect("failed to initialize game simulation");

  // Place stairs at (7, 5)
  let stairs_pos = Position::new(7, 5);
  game
    .world_mut()
    .map_mut()
    .set_tile(stairs_pos, drl_core::Tile::StairsDown);

  // Spawn ground loot at (6, 5): Shotgun and Shells
  let ground_pos = Position::new(6, 5);
  let shotgun_id = game.world_mut().allocate_item_id();
  let shotgun = Item::shotgun(shotgun_id);
  game
    .world_mut()
    .spawn_ground_item(ground_pos, shotgun)
    .expect("failed to spawn ground shotgun");

  let shells_id = game.world_mut().allocate_item_id();
  let shells = Item::ammo_shells(shells_id, 16);
  game
    .world_mut()
    .spawn_ground_item(ground_pos, shells)
    .expect("failed to spawn ground shells");

  // Spawn representative monster (Former Human) at (8, 5)
  let monster_pos = Position::new(8, 5);
  let _monster_id = game
    .world_mut()
    .spawn_monster(monster_pos, "Former Human", 15, 100, (2, 4))
    .expect("failed to spawn monster");

  println!(
    "Turn {}: Level 1 - Player at ({}, {}), Stairs at ({}, {}), Floor Loot at ({}, {}), Monster at ({}, {})",
    game.turn().count,
    start_pos.x,
    start_pos.y,
    stairs_pos.x,
    stairs_pos.y,
    ground_pos.x,
    ground_pos.y,
    monster_pos.x,
    monster_pos.y
  );

  let commands = [
    Command::AttackRanged(monster_pos), // 1. Fire equipped Pistol at (8, 5)
    Command::Move(Direction::East),     // 2. Step onto (6, 5) with ground items
    Command::Pickup,                    // 3. Pick up Shotgun
    Command::Pickup,                    // 4. Pick up Shells
    Command::Equip(shotgun_id),         // 5. Equip Shotgun
    Command::AttackRanged(Position::new(7, 5)), // 6. Blast monster at (7, 5) with Shotgun
    Command::Move(Direction::East),     // 7. Melee bump-attack to finish monster
    Command::Move(Direction::East),     // 8. Step onto (7, 5) stairs down
    Command::Descend,                   // 9. Descend stairs -> transition to Level 2!
    Command::Move(Direction::East),     // 10. Step East on Level 2
  ];

  let mut replay = ReplayLog::new(seed, width, height, start_pos);
  replay.record_stairs(stairs_pos);
  replay.record_item(ItemSpawnSpec::new(ground_pos, ItemSpawnKind::Shotgun));
  replay.record_item(ItemSpawnSpec::new(
    ground_pos,
    ItemSpawnKind::AmmoShells(16),
  ));
  replay.record_monster(MonsterSpawnSpec::new(
    monster_pos,
    "Former Human",
    15,
    100,
    (2, 4),
  ));

  for cmd in commands {
    match game.step(cmd) {
      Ok(events) => {
        replay.record_command(cmd);
        let p_pos = game
          .world()
          .player()
          .map_or(Position::new(0, 0), |p| p.position());
        let obs = game.observe_player();
        let visible_in_fov = obs.visible_tiles.iter().filter(|t| t.is_visible).count();
        let total_explored = obs.visible_tiles.len();
        let level_id = game.world().level_id().as_u32();
        let weapon_name = obs
          .equipped_weapon
          .as_ref()
          .map_or("None".to_string(), |w| {
            if let Some((cur, max)) = w.clip {
              format!("{} ({}/{})", w.name, cur, max)
            } else {
              w.name.clone()
            }
          });

        println!(
          "Turn {} (Level {}): Executed {:?} -> Player at ({}, {}), Weapon: {}, Inventory: {} item(s), FOV: {}/{} tiles, Events: {}",
          game.turn().count,
          level_id,
          cmd,
          p_pos.x,
          p_pos.y,
          weapon_name,
          obs.inventory.len(),
          visible_in_fov,
          total_explored,
          events.len()
        );
        for event in &events {
          match event {
            drl_protocol::GameEvent::AttackResolved {
              attacker_id,
              target_id,
              outcome,
              is_ranged,
            } => {
              println!(
                "  -> Combat: Actor {} attacked Actor {} (ranged: {}) -> outcome: {:?}",
                attacker_id.as_u64(),
                target_id.as_u64(),
                is_ranged,
                outcome
              );
            }
            drl_protocol::GameEvent::DamageApplied {
              target_id,
              amount,
              remaining_hp,
              ..
            } => {
              println!(
                "  -> Damage: Actor {} took {} damage (remaining HP: {})",
                target_id.as_u64(),
                amount,
                remaining_hp
              );
            }
            drl_protocol::GameEvent::ActorDied { entity_id, cause } => {
              println!(
                "  -> Death: Actor {} died (cause: {:?})",
                entity_id.as_u64(),
                cause
              );
            }
            drl_protocol::GameEvent::ItemPickedUp { item_name, .. } => {
              println!("  -> Loot: Picked up {item_name}");
            }
            drl_protocol::GameEvent::ItemEquipped { slot, .. } => {
              println!("  -> Equip: Equipped item into {slot} slot");
            }
            drl_protocol::GameEvent::WeaponReloaded {
              ammo_loaded,
              current_clip,
              max_clip,
              ..
            } => {
              println!(
                "  -> Reload: Loaded {ammo_loaded} rounds (clip: {current_clip}/{max_clip})"
              );
            }
            drl_protocol::GameEvent::LevelTransitioned {
              from_level,
              to_level,
            } => {
              println!(
                "  -> Level: Descended stairs from Level {} to Level {}!",
                from_level.as_u32(),
                to_level.as_u32()
              );
            }
            _ => {}
          }
        }
      }
      Err(err) => {
        println!("Command {:?} rejected: {err}", cmd);
      }
    }
  }

  // Demonstrate MedPack healing
  if let Some(med_id) = game
    .world()
    .player()
    .and_then(|p| p.inventory().find_first_by_category(ItemCategory::MedPack))
  {
    replay.record_command(Command::Use(med_id));
    if let Ok(events) = game.step(Command::Use(med_id)) {
      let cur_hp = game.world().player().map_or(0, |p| p.hp().current);
      let level_id = game.world().level_id().as_u32();
      println!(
        "Turn {} (Level {}): Used MedPack -> Player HP restored to {}, emitted {} event(s)",
        game.turn().count,
        level_id,
        cur_hp,
        events.len()
      );
    }
  }

  println!("Verifying replay determinism from recorded command log...");
  let is_deterministic =
    ReplayEngine::verify_determinism(&replay).expect("replay verification failed");

  if is_deterministic {
    println!("Simulation determinism check PASSED: Replay yielded bit-for-bit identical state.");
  } else {
    eprintln!("Simulation determinism check FAILED!");
    std::process::exit(1);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_app_initialization() {
    assert_eq!(drl_core::engine_name(), "drl-core");
    assert_eq!(drl_protocol::protocol_version(), "0.1.0");
  }

  #[test]
  fn test_headless_demo_execution() {
    let seed = 123;
    let mut game = Game::new_arena(seed, 10, 10).unwrap();
    let events = game.step(Command::Move(Direction::North)).unwrap();
    assert!(!events.is_empty());
    assert_eq!(game.turn().count, 1);
  }
}
