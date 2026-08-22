//! Integration tests for automated batch simulation and statistical metrics.

use drl_core::agent::{ExplorerBot, GreedyCombatBot};
use drl_core::batch::{BatchRunner, CohortConfig, CohortTolerances};
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

#[test]
fn fixed_seed_cohort_records_sample_definition_and_policy_identity() {
  let ascii = r#"
#######
#@...>#
#.....#
#######
"#;
  let scenario = Scenario::from_ascii("CohortArena", "Fixed seed cohort", ascii).unwrap();
  let config = CohortConfig::new(900, 3, 20);

  let report = BatchRunner::run_scenario_cohort(
    &scenario,
    config,
    "ExplorerBot",
    drl_core::agent::ExplorerBot::new,
  )
  .unwrap();

  assert_eq!(report.policy_name, "ExplorerBot");
  assert_eq!(report.config, config);
  assert_eq!(report.records.len(), 3);
  assert_eq!(
    report
      .records
      .iter()
      .map(|record| record.seed)
      .collect::<Vec<_>>(),
    vec![900, 901, 902]
  );
  assert_eq!(report.summary.total_episodes, config.episode_count);
}

#[test]
fn fixed_seed_cohort_seed_sequence_wraps_deterministically() {
  let config = CohortConfig::new(u64::MAX - 1, 4, 1);
  assert_eq!(
    config.seeds().collect::<Vec<_>>(),
    vec![u64::MAX - 1, u64::MAX, 0, 1]
  );
}

#[test]
fn fixed_seed_cohort_repeats_bit_exactly() {
  let ascii = r#"
#####
#@.>#
#####
"#;
  let scenario = Scenario::from_ascii("CohortRepeat", "Repeatable cohort", ascii).unwrap();
  let config = CohortConfig::new(1200, 4, 12);

  let first = BatchRunner::run_scenario_cohort(
    &scenario,
    config,
    "ExplorerBot",
    drl_core::agent::ExplorerBot::new,
  )
  .unwrap();
  let second = BatchRunner::run_scenario_cohort(
    &scenario,
    config,
    "ExplorerBot",
    drl_core::agent::ExplorerBot::new,
  )
  .unwrap();

  assert_eq!(first, second);
}

#[test]
fn procedural_fixed_seed_cohort_reuses_batch_replay_evidence() {
  let generator_config = LevelGeneratorConfig {
    width: 20,
    height: 15,
    min_room_size: 4,
    max_room_size: 6,
    max_rooms: 3,
    max_monsters_per_room: 1,
    max_items_per_room: 1,
  };
  let config = CohortConfig::new(7_000, 2, 25);

  let report =
    BatchRunner::run_procedural_cohort(config, &generator_config, "ExplorerBot", ExplorerBot::new)
      .unwrap();

  assert_eq!(report.config, config);
  assert_eq!(report.policy_name, "ExplorerBot");
  assert_eq!(report.records.len(), config.episode_count);
  assert!(
    report
      .records
      .iter()
      .all(|record| !record.replay.commands.is_empty())
  );
}

#[test]
fn cohort_comparison_reports_deltas_and_respects_inclusive_tolerances() {
  let ascii = r#"
#####
#@.>#
#####
"#;
  let scenario = Scenario::from_ascii("RegressionArena", "Tolerance gate", ascii).unwrap();
  let config = CohortConfig::new(8_000, 2, 12);
  let baseline =
    BatchRunner::run_scenario_cohort(&scenario, config, "ExplorerBot", ExplorerBot::new).unwrap();
  let mut candidate = baseline.clone();
  candidate.summary.win_rate += 0.1;
  candidate.summary.average_turns += 2.0;

  let comparison = candidate
    .compare_with(&baseline, CohortTolerances::new(0.11, 2.0))
    .unwrap();
  assert!((comparison.win_rate_delta - 0.1).abs() < f64::EPSILON * 2.0);
  assert!((comparison.average_turns_delta - 2.0).abs() < f64::EPSILON);
  assert!(comparison.within_tolerance);
  assert!(
    !candidate
      .compare_with(&baseline, CohortTolerances::new(0.099, 2.0))
      .unwrap()
      .within_tolerance
  );
}

#[test]
fn cohort_comparison_rejects_mismatches_and_invalid_tolerances() {
  let ascii = r#"
#####
#@.>#
#####
"#;
  let scenario = Scenario::from_ascii("RegressionArena", "Tolerance gate", ascii).unwrap();
  let config = CohortConfig::new(8_100, 1, 8);
  let baseline =
    BatchRunner::run_scenario_cohort(&scenario, config, "ExplorerBot", ExplorerBot::new).unwrap();
  let different_policy = BatchRunner::run_scenario_cohort(&scenario, config, "RandomBot", || {
    drl_core::agent::RandomBot::new(11)
  })
  .unwrap();
  assert!(
    different_policy
      .compare_with(&baseline, CohortTolerances::new(1.0, 1.0))
      .is_none()
  );
  assert!(
    baseline
      .compare_with(&baseline, CohortTolerances::new(-0.1, 1.0))
      .is_none()
  );
  assert!(
    baseline
      .compare_with(&baseline, CohortTolerances::new(f64::NAN, 1.0))
      .is_none()
  );
}
