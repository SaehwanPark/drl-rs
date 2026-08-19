//! Integration tests for declarative scenario fixtures and ASCII map parsing.

use drl_core::scenario::{Scenario, ScenarioRunner};
use drl_protocol::{
  Command, Direction, ItemSpawnKind, PlayerSpawnConfig, Position, RunOutcome, ScenarioFixture,
};

#[test]
fn test_ascii_scenario_complex_arena() {
  let ascii = r#"
############
#@..p...m..#
#.####.###.#
#...h...s..#
#.####.###.#
#...i...d..#
#........>.#
############
"#;

  let scenario = Scenario::from_ascii(
    "Gauntlet",
    "Multi-chamber gauntlet testing weapon pickups and multi-archetype combat",
    ascii,
  )
  .unwrap();

  assert_eq!(scenario.width, 12);
  assert_eq!(scenario.height, 8);
  assert_eq!(scenario.player_start, Position::new(1, 1));
  assert_eq!(scenario.stairs, Some(Position::new(9, 6)));
  assert_eq!(scenario.monsters.len(), 4);
  assert_eq!(scenario.items.len(), 2);

  let game = scenario.instantiate().unwrap();
  assert_eq!(
    game.world().player().unwrap().position(),
    Position::new(1, 1)
  );
  assert!(game.world().player().unwrap().is_alive());

  let monster_names: Vec<&str> = game
    .world()
    .actors()
    .values()
    .filter(|a| !a.is_player())
    .map(|a| a.name())
    .collect();
  assert!(monster_names.contains(&"Former Human"));
  assert!(monster_names.contains(&"Former Sergeant"));
  assert!(monster_names.contains(&"Imp"));
  assert!(monster_names.contains(&"Demon"));
}

#[test]
fn test_scenario_custom_player_spawn_config() {
  let ascii = r#"
#######
#@...h#
#######
"#;

  let mut scenario = Scenario::from_ascii("CustomHero", "Shotgun specialist", ascii).unwrap();
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 100,
    max_hp: 100,
    speed: 120,
    initial_items: vec![ItemSpawnKind::AmmoShells(25), ItemSpawnKind::LargeMedPack],
    equipped_weapon: Some(ItemSpawnKind::Shotgun),
    equipped_armor: Some(ItemSpawnKind::GreenArmor),
  });

  let game = scenario.instantiate().unwrap();
  let player = game.world().player().unwrap();
  assert_eq!(player.hp().current, 100);
  assert_eq!(player.hp().max, 100);
  assert_eq!(player.speed().as_u32(), 120);

  let weapon = player.equipment().weapon().unwrap();
  assert_eq!(weapon.name(), "Shotgun");
  let armor = player.equipment().armor().unwrap();
  assert_eq!(armor.name(), "Green Armor");

  assert_eq!(player.inventory().items().len(), 2);
}

#[test]
fn test_scenario_scripted_run_and_metrics_accumulation() {
  let ascii = r#"
#######
#@.p..>
#######
"#;

  let scenario = Scenario::from_ascii("LootAndExit", "Pickup pistol and descend", ascii).unwrap();
  let commands = vec![
    Command::Move(Direction::East),
    Command::Move(Direction::East),
    Command::Pickup,
    Command::Move(Direction::East),
    Command::Move(Direction::East),
    Command::Move(Direction::East),
    Command::Descend,
  ];

  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();

  assert_eq!(metrics.outcome, RunOutcome::Victory);
  assert_eq!(metrics.items_picked_up, 1);
  assert_eq!(metrics.level_reached.0, 2);
  assert_eq!(replay.commands.len(), 7);
  assert!(game.world().level_id().0 > 1);
  assert!(!events.is_empty());
}

#[test]
fn test_scenario_from_protocol_fixture() {
  let fixture = ScenarioFixture::new(
    "ProtocolFixture",
    "Fixture constructed via protocol domain types",
    8,
    8,
    Position::new(2, 2),
  )
  .with_stairs(Position::new(5, 5))
  .with_seed(9999);

  let scenario = Scenario::from_fixture(&fixture).unwrap();
  assert_eq!(scenario.name, "ProtocolFixture");
  assert_eq!(scenario.width, 8);
  assert_eq!(scenario.height, 8);
  assert_eq!(scenario.player_start, Position::new(2, 2));
  assert_eq!(scenario.stairs, Some(Position::new(5, 5)));
  assert_eq!(scenario.seed, 9999);

  let game = scenario.instantiate().unwrap();
  assert_eq!(
    game.world().player().unwrap().position(),
    Position::new(2, 2)
  );
}
