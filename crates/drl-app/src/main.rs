//! Application executable entry point, headless demo runner, and MCP server for DRL-Rust.

use drl_core::agent::GreedyCombatBot;
use drl_core::batch::BatchRunner;
use drl_core::item::Item;
use drl_core::scenario::{Scenario, ScenarioRunner};
use drl_core::{Game, ReplayEngine};
use drl_mcp::McpServer;
use drl_mcp::json::JsonValue;
use drl_protocol::{
  Command, Direction, ItemCategory, ItemSpawnKind, ItemSpawnSpec, MonsterSpawnSpec, Position,
  ReplayLog,
};
use std::io;

mod cohort_cli;

use cohort_cli::run_cohort_command;

fn main() {
  let args = std::env::args().skip(1).collect::<Vec<_>>();
  if args
    .first()
    .is_some_and(|arg| arg == "cohort" || arg == "--cohort")
  {
    match run_cohort_command(&args[1..]) {
      Ok(report) => print!("{report}"),
      Err(error) => {
        eprintln!("cohort: {error}");
        std::process::exit(2);
      }
    }
    return;
  }
  // If invoked with `--mcp` or `mcp`, launch the stdio MCP server loop.
  if std::env::args().any(|arg| arg == "--mcp" || arg == "mcp") {
    let mut server = McpServer::new();
    let stdin = io::stdin();
    let stdout = io::stdout();
    if let Err(err) = server.run_stdio(stdin.lock(), stdout.lock()) {
      eprintln!("MCP Server error: {err}");
      std::process::exit(1);
    }
    return;
  }

  println!(
    "DRL-Rust ({}, protocol {}, mcp {}) initialized.",
    drl_core::engine_name(),
    drl_protocol::protocol_version(),
    drl_mcp::mcp_name()
  );

  run_headless_demo();
  run_scenario_bot_demo();
  run_batch_simulation_demo();
  run_mcp_interface_demo();
}

fn run_headless_demo() {
  let seed = 42;
  let width = 20;
  let height = 10;
  let start_pos = Position::new(5, 5);

  println!("\n=== Headless Simulation Arena Demo ({width}x{height}, Seed: {seed}) ===");

  let mut game =
    Game::new(seed, width, height, start_pos).expect("failed to initialize game simulation");

  // Place stairs at (7, 5)
  let stairs_pos = Position::new(7, 5);
  game
    .world_mut()
    .map_mut()
    .set_tile(stairs_pos, drl_core::Tile::StairsDown);

  // Spawn ground loot at (6, 5): Shotgun and Phase Device
  let ground_pos = Position::new(6, 5);
  let shotgun_id = game.world_mut().allocate_item_id();
  let shotgun = Item::shotgun(shotgun_id);
  game
    .world_mut()
    .spawn_ground_item(ground_pos, shotgun)
    .expect("failed to spawn ground shotgun");

  let phase_id = game.world_mut().allocate_item_id();
  let phase_device = Item::phase_device(phase_id);
  game
    .world_mut()
    .spawn_ground_item(ground_pos, phase_device)
    .expect("failed to spawn ground phase device");

  // Spawn representative monster (Former Sergeant) at (8, 5)
  let monster_pos = Position::new(8, 5);
  let sergeant =
    drl_core::Actor::former_sergeant(game.world_mut().allocate_entity_id(), monster_pos);
  game
    .world_mut()
    .actors_mut()
    .insert(sergeant.id(), sergeant);

  println!(
    "Turn {}: Level 1 - Player at ({}, {}), Stairs at ({}, {}), Floor Loot at ({}, {}), Former Sergeant at ({}, {})",
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
    Command::Pickup,                    // 4. Pick up Phase Device
    Command::Equip(shotgun_id),         // 5. Equip Shotgun
    Command::AttackRanged(Position::new(8, 5)), // 6. Blast monster at (8, 5) with Shotgun
    Command::Move(Direction::East),     // 7. Step to (7, 5)
    Command::Use(phase_id),             // 8. Emergency Phase Device teleport across space!
    Command::Wait,                      // 9. Catch breath
  ];

  let mut replay = ReplayLog::new(seed, width, height, start_pos);
  replay.record_stairs(stairs_pos);
  replay.record_item(ItemSpawnSpec::new(ground_pos, ItemSpawnKind::Shotgun));
  replay.record_item(ItemSpawnSpec::new(ground_pos, ItemSpawnKind::PhaseDevice));
  replay.record_monster(
    MonsterSpawnSpec::new(monster_pos, "Former Sergeant", 25, 90, (3, 6)).with_ranged_combat(
      (8, 14),
      5,
      60,
    ),
  );

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
      }
      Err(err) => {
        println!("Command {:?} rejected: {err}", cmd);
      }
    }
  }

  // Demonstrate MedPack healing if available
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

fn run_scenario_bot_demo() {
  println!("\n=== Declarative Scenario & Automated Bot Demo ===");
  let ascii = r#"
############
#@...h...m.#
#.####.###.#
#...S...d..#
#........>.#
############
"#;

  let scenario = Scenario::from_ascii(
    "MiniChamber",
    "Automated bot clearing enemies and reaching stairs",
    ascii,
  )
  .expect("failed to parse ASCII scenario");

  println!(
    "Loaded Scenario '{}' ({}x{}) with {} monsters and {} items",
    scenario.name,
    scenario.width,
    scenario.height,
    scenario.monsters.len(),
    scenario.items.len()
  );

  let mut bot = GreedyCombatBot::new();
  let (game, events, metrics, replay) =
    ScenarioRunner::run_policy(&scenario, &mut bot, 40).expect("failed to run bot policy");

  println!(
    "Bot completed episode in {} turns. Outcome: {:?}, Damage Dealt: {}, Enemies Killed: {}, Level: {}",
    metrics.turns_survived,
    metrics.outcome,
    metrics.damage_dealt,
    metrics.enemies_killed,
    metrics.level_reached.0
  );

  let (replayed_game, replayed_events) =
    ReplayEngine::run(&replay).expect("replay verification failed");
  assert_eq!(game, replayed_game);
  assert_eq!(events, replayed_events);
  println!("Bot Replay Determinism: PASSED (bit-for-bit identical)");
}

fn run_batch_simulation_demo() {
  println!("\n=== Headless Batch Simulation & Metrics Summary ===");
  let ascii = r#"
##########
#@..h..m.#
#...d....#
#.......>#
##########
"#;

  let scenario = Scenario::from_ascii("BatchArena", "Sweep scenario across seeds", ascii).unwrap();
  let seeds = [101, 202, 303, 404, 505, 606, 707, 808, 909, 1001];

  let (summary, _) = BatchRunner::run_scenario_batch(&scenario, &seeds, 35, GreedyCombatBot::new)
    .expect("batch simulation failed");

  println!(
    "Batch Sweep Results ({} episodes across seeds {:?}):",
    summary.total_episodes, seeds
  );
  println!("  Victories:         {}", summary.victories);
  println!("  Deaths:            {}", summary.deaths);
  println!("  Timeouts:          {}", summary.timeouts);
  println!("  Win Rate:          {:.1}%", summary.win_rate * 100.0);
  println!("  Total Turns:       {}", summary.total_turns);
  println!("  Average Turns/Run: {:.1}", summary.average_turns);
  println!("  Total Kills:       {}", summary.total_enemies_killed);
  println!("  Total Damage:      {}", summary.total_damage_dealt);
}

fn run_mcp_interface_demo() {
  println!("\n=== Model Context Protocol (MCP) Interface Demo ===");

  let mut server = McpServer::new();

  // 1. Initialize
  let init_req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
  let init_resp = server.handle_request(init_req);
  println!("MCP Initialize Response: {init_resp}");

  // 2. Tools List
  let list_req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
  let list_resp = server.handle_request(list_req);
  let parsed = JsonValue::parse(&list_resp).unwrap();
  let count = parsed
    .get("result")
    .unwrap()
    .get("tools")
    .unwrap()
    .as_array()
    .unwrap()
    .len();
  println!("Registered MCP Tools: {count} tools available");

  // 3. Start Game via MCP
  let start_req = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":888,"width":20,"height":10}}}"#;
  let start_resp = server.handle_request(start_req);
  let parsed_start = JsonValue::parse(&start_resp).unwrap();
  let obs = parsed_start
    .get("result")
    .unwrap()
    .get("data")
    .unwrap()
    .get("observation")
    .unwrap();
  let p_pos = obs.get("player_position").unwrap();
  println!(
    "Started MCP Game Session (Seed 888) -> Player at ({}, {})",
    p_pos.get("x").unwrap(),
    p_pos.get("y").unwrap()
  );

  // 4. Submit Semantic Action via MCP
  let step_req = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}"#;
  let step_resp = server.handle_request(step_req);
  let parsed_step = JsonValue::parse(&step_resp).unwrap();
  let events_count = parsed_step
    .get("result")
    .unwrap()
    .get("data")
    .unwrap()
    .get("events")
    .unwrap()
    .as_array()
    .unwrap()
    .len();
  println!("Stepped semantic Wait action over MCP -> Emitted {events_count} game event(s)");

  // 5. Query Metrics via MCP
  let metrics_req = r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}"#;
  let metrics_resp = server.handle_request(metrics_req);
  let parsed_metrics = JsonValue::parse(&metrics_resp).unwrap();
  let metrics_data = parsed_metrics.get("result").unwrap().get("data").unwrap();
  println!(
    "MCP Session Telemetry: Turns Survived: {}, Level: {}",
    metrics_data.get("turns_survived").unwrap(),
    metrics_data.get("level_reached").unwrap()
  );
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_app_initialization() {
    assert_eq!(drl_core::engine_name(), "drl-core");
    assert_eq!(drl_protocol::protocol_version(), "0.1.0");
    assert_eq!(drl_mcp::mcp_name(), "drl-mcp");
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
