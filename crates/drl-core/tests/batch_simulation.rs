//! Integration tests for automated batch simulation and statistical metrics.

use drl_core::agent::{ExplorerBot, GreedyCombatBot};
use drl_core::batch::BatchRunner;
use drl_core::generator::LevelGeneratorConfig;
use drl_core::scenario::Scenario;

#[test]
fn test_procedural_batch_runner_greedy_combat_bot() {
  let config = LevelGeneratorConfig {
    width: 25,
    height: 15,
    min_room_size: 4,
    max_room_size: 7,
    max_rooms: 4,
    max_monsters_per_room: 1,
    max_items_per_room: 1,
  };

  let num_episodes = 20;
  let (summary, records) =
    BatchRunner::run_procedural_batch(num_episodes, 5000, 60, &config, GreedyCombatBot::new)
      .unwrap();

  assert_eq!(summary.total_episodes, num_episodes);
  assert_eq!(records.len(), num_episodes);
  assert!(summary.total_turns > 0);
  assert!(summary.average_turns > 0.0);
  assert!(summary.win_rate >= 0.0 && summary.win_rate <= 1.0);
  assert_eq!(
    summary.victories + summary.deaths + summary.timeouts,
    num_episodes
  );
}

#[test]
fn test_procedural_batch_runner_explorer_bot() {
  let config = LevelGeneratorConfig {
    width: 20,
    height: 15,
    min_room_size: 4,
    max_room_size: 6,
    max_rooms: 3,
    max_monsters_per_room: 1,
    max_items_per_room: 1,
  };

  let num_episodes = 15;
  let (summary, records) =
    BatchRunner::run_procedural_batch(num_episodes, 10000, 50, &config, ExplorerBot::new).unwrap();

  assert_eq!(summary.total_episodes, num_episodes);
  assert_eq!(records.len(), num_episodes);
  assert!(summary.total_turns > 0);
}

#[test]
fn test_scenario_batch_sweep_determinism() {
  let ascii = r#"
###########
#@...h...m#
#....d...A#
#........>#
###########
"#;

  let scenario = Scenario::from_ascii("BatchArena", "Sweep over multiple seeds", ascii).unwrap();
  let seeds = vec![111, 222, 333, 444, 555];

  let (summary1, records1) =
    BatchRunner::run_scenario_batch(&scenario, &seeds, 40, GreedyCombatBot::new).unwrap();
  let (summary2, records2) =
    BatchRunner::run_scenario_batch(&scenario, &seeds, 40, GreedyCombatBot::new).unwrap();

  assert_eq!(summary1, summary2);
  assert_eq!(records1.len(), records2.len());

  for (r1, r2) in records1.iter().zip(records2.iter()) {
    assert_eq!(r1.seed, r2.seed);
    assert_eq!(r1.metrics, r2.metrics);
    assert_eq!(r1.replay, r2.replay);
  }
}
