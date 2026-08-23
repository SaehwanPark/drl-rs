//! Integration tests for automated batch simulation and statistical metrics.

use drl_core::agent::{ExplorerBot, GreedyCombatBot};
use drl_core::batch::{
  BatchRunner, CohortConfig, CohortOutcomeTolerances, CohortReport, CohortReportError,
  CohortTelemetryTolerances, CohortTolerances, EpisodeRecord,
};
use drl_core::generator::LevelGeneratorConfig;
use drl_core::scenario::Scenario;
use drl_protocol::{
  BatchSummary, DeathCause, EpisodeMetrics, LevelId, Position, ReplayLog, RunOutcome,
};

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
  assert!(report.validate().is_ok());
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
  assert!(report.validate().is_ok());
}

#[test]
fn cohort_report_validation_rejects_inconsistent_evidence() {
  let ascii = r#"
#####
#@.>#
#####
"#;
  let scenario = Scenario::from_ascii("IntegrityArena", "Report integrity", ascii).unwrap();
  let config = CohortConfig::new(7_500, 3, 12);
  let report =
    BatchRunner::run_scenario_cohort(&scenario, config, "ExplorerBot", ExplorerBot::new).unwrap();
  assert!(report.validate().is_ok());

  let mut missing_record = report.clone();
  missing_record.records.pop();
  assert!(matches!(
    missing_record.validate(),
    Err(CohortReportError::RecordCount { .. })
  ));

  let mut wrong_seed = report.clone();
  wrong_seed.records[1].seed = 99;
  assert!(matches!(
    wrong_seed.validate(),
    Err(CohortReportError::SeedMismatch { index: 1, .. })
  ));

  let mut wrong_replay_seed = report.clone();
  wrong_replay_seed.records[0].replay.seed = 99;
  assert!(matches!(
    wrong_replay_seed.validate(),
    Err(CohortReportError::ReplaySeedMismatch { index: 0, .. })
  ));

  let mut wrong_summary = report;
  wrong_summary.summary.total_turns += 1;
  assert!(matches!(
    wrong_summary.validate(),
    Err(CohortReportError::SummaryMismatch)
  ));
}

#[test]
fn cohort_comparison_reports_deltas_and_respects_inclusive_tolerances() {
  let baseline = synthetic_outcome_report(&vec![RunOutcome::TurnLimitReached; 10]);
  let mut candidate = baseline.clone();
  candidate.records[0].metrics.outcome = RunOutcome::Victory;
  candidate.records[0].metrics.turns_survived = 10;
  candidate.records[1].metrics.turns_survived = 10;
  candidate.summary = BatchSummary::from_episodes(
    &candidate
      .records
      .iter()
      .map(|record| record.metrics.clone())
      .collect::<Vec<_>>(),
  );

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

fn synthetic_outcome_report(outcomes: &[RunOutcome]) -> CohortReport {
  let config = CohortConfig::new(9_000, outcomes.len(), 10);
  let records = outcomes
    .iter()
    .enumerate()
    .map(|(index, outcome)| {
      let seed = config.start_seed + index as u64;
      let mut metrics = EpisodeMetrics::new();
      metrics.outcome = outcome.clone();
      EpisodeRecord {
        seed,
        metrics,
        replay: ReplayLog::new(seed, 3, 3, Position::new(1, 1)),
      }
    })
    .collect::<Vec<_>>();
  let metrics = records
    .iter()
    .map(|record| record.metrics.clone())
    .collect::<Vec<_>>();
  CohortReport {
    policy_name: "synthetic".to_string(),
    config,
    summary: BatchSummary::from_episodes(&metrics),
    records,
  }
}

#[test]
fn cohort_outcome_distribution_preserves_distinct_counts_and_rates() {
  let report = synthetic_outcome_report(&[
    RunOutcome::Victory,
    RunOutcome::Death {
      cause: DeathCause::Environment,
    },
    RunOutcome::TurnLimitReached,
    RunOutcome::Stalled,
    RunOutcome::InProgress,
  ]);

  let distribution = report.outcome_distribution().unwrap();
  assert_eq!(distribution.total_episodes, 5);
  assert_eq!(distribution.victories, 1);
  assert_eq!(distribution.deaths, 1);
  assert_eq!(distribution.turn_limit_reached, 1);
  assert_eq!(distribution.stalled, 1);
  assert_eq!(distribution.in_progress, 1);
  assert!((distribution.victory_rate() - 0.2).abs() < f64::EPSILON);
  assert!((distribution.death_rate() - 0.2).abs() < f64::EPSILON);
  assert!((distribution.turn_limit_rate() - 0.2).abs() < f64::EPSILON);
  assert!((distribution.stalled_rate() - 0.2).abs() < f64::EPSILON);
  assert!((distribution.in_progress_rate() - 0.2).abs() < f64::EPSILON);
}

#[test]
fn cohort_outcome_distribution_requires_integrity_and_handles_empty_samples() {
  let mut invalid = synthetic_outcome_report(&[RunOutcome::Victory]);
  invalid.records.pop();
  assert!(matches!(
    invalid.outcome_distribution(),
    Err(CohortReportError::RecordCount { .. })
  ));

  let empty = synthetic_outcome_report(&[]);
  let distribution = empty.outcome_distribution().unwrap();
  assert_eq!(distribution.total_episodes, 0);
  assert_eq!(distribution.victories, 0);
  assert_eq!(distribution.deaths, 0);
  assert_eq!(distribution.turn_limit_reached, 0);
  assert_eq!(distribution.stalled, 0);
  assert_eq!(distribution.in_progress, 0);
  assert_eq!(distribution.victory_rate(), 0.0);
  assert_eq!(distribution.in_progress_rate(), 0.0);
}

#[test]
fn cohort_validation_rejects_impossible_telemetry_before_projection() {
  let mut report = synthetic_outcome_report(&[RunOutcome::Victory]);
  report.records[0].metrics.shots_fired = 1;
  report.records[0].metrics.shots_hit = 2;
  assert_eq!(
    report.telemetry_distribution(),
    Err(CohortReportError::TelemetryInvariant {
      index: 0,
      field: "shots_hit <= shots_fired",
    })
  );

  let mut report = synthetic_outcome_report(&[RunOutcome::Victory]);
  report.records[0].metrics.level_reached.0 = 0;
  assert_eq!(
    report.outcome_distribution(),
    Err(CohortReportError::TelemetryInvariant {
      index: 0,
      field: "level_reached >= 1",
    })
  );

  let mut report = synthetic_outcome_report(&[RunOutcome::Victory]);
  report.records[0].metrics.turns_survived = report.config.max_turns + 1;
  assert_eq!(
    report.validate(),
    Err(CohortReportError::TelemetryInvariant {
      index: 0,
      field: "turns_survived <= max_turns",
    })
  );
}

#[test]
fn cohort_telemetry_distribution_preserves_totals_and_rates() {
  let mut report = synthetic_outcome_report(&[RunOutcome::Victory, RunOutcome::TurnLimitReached]);
  report.records[0].metrics.shots_fired = 4;
  report.records[0].metrics.shots_hit = 3;
  report.records[0].metrics.damage_dealt = 10;
  report.records[0].metrics.damage_taken = 2;
  report.records[0].metrics.enemies_killed = 1;
  report.records[0].metrics.items_picked_up = 2;
  report.records[0].metrics.items_used = 1;
  report.records[1].metrics.shots_fired = 2;
  report.records[1].metrics.shots_hit = 1;
  report.records[1].metrics.damage_dealt = 6;
  report.records[1].metrics.damage_taken = 4;
  report.records[1].metrics.enemies_killed = 2;
  report.records[1].metrics.items_picked_up = 1;
  report.records[1].metrics.items_used = 0;
  report.summary = BatchSummary::from_episodes(
    &report
      .records
      .iter()
      .map(|record| record.metrics.clone())
      .collect::<Vec<_>>(),
  );

  let telemetry = report.telemetry_distribution().unwrap();
  assert_eq!(telemetry.total_episodes, 2);
  assert_eq!(telemetry.total_shots_fired, 6);
  assert_eq!(telemetry.total_shots_hit, 4);
  assert_eq!(telemetry.total_damage_dealt, 16);
  assert_eq!(telemetry.total_damage_taken, 6);
  assert_eq!(telemetry.total_enemies_killed, 3);
  assert_eq!(telemetry.total_items_picked_up, 3);
  assert_eq!(telemetry.total_items_used, 1);
  assert!((telemetry.shot_accuracy_rate() - (2.0 / 3.0)).abs() < f64::EPSILON);
  assert_eq!(telemetry.average_damage_dealt(), 8.0);
  assert_eq!(telemetry.average_damage_taken(), 3.0);
  assert_eq!(telemetry.average_enemies_killed(), 1.5);
  assert_eq!(telemetry.average_items_picked_up(), 1.5);
  assert_eq!(telemetry.average_items_used(), 0.5);
}

#[test]
fn cohort_telemetry_distribution_requires_integrity_and_handles_empty_samples() {
  let mut invalid = synthetic_outcome_report(&[RunOutcome::Victory]);
  invalid.records.pop();
  assert!(matches!(
    invalid.telemetry_distribution(),
    Err(CohortReportError::RecordCount { .. })
  ));

  let empty = synthetic_outcome_report(&[]);
  let telemetry = empty.telemetry_distribution().unwrap();
  assert_eq!(telemetry.total_episodes, 0);
  assert_eq!(telemetry.total_shots_fired, 0);
  assert_eq!(telemetry.total_damage_dealt, 0);
  assert_eq!(telemetry.shot_accuracy_rate(), 0.0);
  assert_eq!(telemetry.average_damage_dealt(), 0.0);
}

#[test]
fn cohort_depth_distribution_sorts_buckets_and_normalizes_rates() {
  let mut report = synthetic_outcome_report(&[
    RunOutcome::Victory,
    RunOutcome::Victory,
    RunOutcome::TurnLimitReached,
    RunOutcome::Death {
      cause: DeathCause::Environment,
    },
  ]);
  for (record, level) in
    report
      .records
      .iter_mut()
      .zip([LevelId(1), LevelId(3), LevelId(3), LevelId(5)])
  {
    record.metrics.level_reached = level;
  }
  report.summary = BatchSummary::from_episodes(
    &report
      .records
      .iter()
      .map(|record| record.metrics.clone())
      .collect::<Vec<_>>(),
  );

  let distribution = report.depth_distribution().unwrap();
  assert_eq!(distribution.total_episodes, 4);
  assert_eq!(
    distribution.buckets,
    vec![
      drl_core::batch::CohortDepthBucket {
        level: LevelId(1),
        episodes: 1,
      },
      drl_core::batch::CohortDepthBucket {
        level: LevelId(3),
        episodes: 2,
      },
      drl_core::batch::CohortDepthBucket {
        level: LevelId(5),
        episodes: 1,
      },
    ]
  );
  assert_eq!(distribution.rate_at_deepest_level(LevelId(1)), 0.25);
  assert_eq!(distribution.rate_at_deepest_level(LevelId(3)), 0.5);
  assert_eq!(distribution.rate_at_deepest_level(LevelId(5)), 0.25);
  assert_eq!(distribution.rate_at_deepest_level(LevelId(4)), 0.0);
}

#[test]
fn cohort_depth_distribution_requires_integrity_and_handles_empty_samples() {
  let mut invalid = synthetic_outcome_report(&[RunOutcome::Victory]);
  invalid.records.pop();
  assert!(matches!(
    invalid.depth_distribution(),
    Err(CohortReportError::RecordCount { .. })
  ));

  let mut invalid_level = synthetic_outcome_report(&[RunOutcome::Victory]);
  invalid_level.records[0].metrics.level_reached = LevelId(0);
  assert_eq!(
    invalid_level.depth_distribution(),
    Err(CohortReportError::TelemetryInvariant {
      index: 0,
      field: "level_reached >= 1",
    })
  );

  let empty = synthetic_outcome_report(&[]);
  let distribution = empty.depth_distribution().unwrap();
  assert_eq!(distribution.total_episodes, 0);
  assert!(distribution.buckets.is_empty());
  assert_eq!(distribution.rate_at_deepest_level(LevelId(1)), 0.0);
}

fn telemetry_report(
  damage_dealt: [u32; 2],
  items_used: [u32; 2],
  shots_hit: [u32; 2],
) -> CohortReport {
  let mut report = synthetic_outcome_report(&[RunOutcome::Victory, RunOutcome::TurnLimitReached]);
  for (index, record) in report.records.iter_mut().enumerate() {
    record.metrics.shots_fired = 4;
    record.metrics.shots_hit = shots_hit[index];
    record.metrics.damage_dealt = damage_dealt[index];
    record.metrics.damage_taken = 2 + index as u32;
    record.metrics.enemies_killed = 1 + index as u32;
    record.metrics.items_picked_up = 1;
    record.metrics.items_used = items_used[index];
  }
  report.summary = BatchSummary::from_episodes(
    &report
      .records
      .iter()
      .map(|record| record.metrics.clone())
      .collect::<Vec<_>>(),
  );
  report
}

#[test]
fn compatible_telemetry_comparison_reports_absolute_deltas() {
  let baseline = telemetry_report([10, 6], [1, 0], [3, 2]);
  let candidate = telemetry_report([14, 8], [2, 0], [3, 3]);
  let comparison = candidate.compare_telemetry(&baseline).unwrap();

  assert!((comparison.shot_accuracy_rate_delta - 0.125).abs() < f64::EPSILON);
  assert_eq!(comparison.average_damage_dealt_delta, 3.0);
  assert_eq!(comparison.average_damage_taken_delta, 0.0);
  assert_eq!(comparison.average_enemies_killed_delta, 0.0);
  assert_eq!(comparison.average_items_picked_up_delta, 0.0);
  assert_eq!(comparison.average_items_used_delta, 0.5);
}

#[test]
fn telemetry_comparison_rejects_incompatible_or_invalid_reports() {
  let baseline = telemetry_report([10, 6], [1, 0], [3, 2]);
  let mut different_policy = baseline.clone();
  different_policy.policy_name = "other-policy".to_string();
  assert!(different_policy.compare_telemetry(&baseline).is_none());

  let mut invalid = baseline.clone();
  invalid.records.pop();
  assert!(invalid.compare_telemetry(&baseline).is_none());
}

#[test]
fn telemetry_comparison_applies_rate_and_average_tolerances() {
  let baseline = telemetry_report([10, 6], [1, 0], [3, 2]);
  let candidate = telemetry_report([14, 8], [2, 0], [3, 3]);
  let comparison = candidate.compare_telemetry(&baseline).unwrap();

  assert!(comparison.within_tolerance(CohortTelemetryTolerances::new(0.125, 3.0)));
  assert!(!comparison.within_tolerance(CohortTelemetryTolerances::new(0.124, 3.0)));
  assert!(!comparison.within_tolerance(CohortTelemetryTolerances::new(0.125, 2.99)));
  assert!(!comparison.within_tolerance(CohortTelemetryTolerances::new(-0.1, 3.0)));
  assert!(!comparison.within_tolerance(CohortTelemetryTolerances::new(0.125, f64::NAN)));
}

#[test]
fn generic_cohort_comparison_rejects_impossible_telemetry() {
  let baseline = synthetic_outcome_report(&[RunOutcome::Victory]);
  let mut invalid = baseline.clone();
  invalid.records[0].metrics.turns_survived = invalid.config.max_turns + 1;
  assert!(
    invalid
      .compare_with(&baseline, CohortTolerances::new(1.0, 1.0))
      .is_none()
  );
}

#[test]
fn compatible_cohort_outcome_comparison_reports_absolute_rate_deltas() {
  let baseline = synthetic_outcome_report(&[
    RunOutcome::Victory,
    RunOutcome::Death {
      cause: DeathCause::Environment,
    },
    RunOutcome::TurnLimitReached,
    RunOutcome::Stalled,
    RunOutcome::InProgress,
  ]);
  let candidate = synthetic_outcome_report(&[
    RunOutcome::Victory,
    RunOutcome::Victory,
    RunOutcome::TurnLimitReached,
    RunOutcome::Stalled,
    RunOutcome::InProgress,
  ]);

  let comparison = candidate.compare_outcomes(&baseline).unwrap();
  assert!((comparison.victory_rate_delta - 0.2).abs() < f64::EPSILON);
  assert!((comparison.death_rate_delta - 0.2).abs() < f64::EPSILON);
  assert_eq!(comparison.turn_limit_rate_delta, 0.0);
  assert_eq!(comparison.stalled_rate_delta, 0.0);
  assert_eq!(comparison.in_progress_rate_delta, 0.0);
}

#[test]
fn cohort_outcome_comparison_rejects_incompatible_or_invalid_reports() {
  let baseline = synthetic_outcome_report(&[RunOutcome::Victory]);

  let mut different_policy = baseline.clone();
  different_policy.policy_name = "other-policy".to_string();
  assert!(different_policy.compare_outcomes(&baseline).is_none());

  let mut invalid = baseline.clone();
  invalid.records.pop();
  assert!(invalid.compare_outcomes(&baseline).is_none());
}

#[test]
fn cohort_outcome_comparison_applies_finite_non_negative_tolerance() {
  let baseline = synthetic_outcome_report(&[RunOutcome::Victory]);
  let candidate = synthetic_outcome_report(&[RunOutcome::Death {
    cause: DeathCause::Environment,
  }]);
  let comparison = candidate.compare_outcomes(&baseline).unwrap();

  assert!(comparison.within_tolerance(CohortOutcomeTolerances::new(1.0)));
  assert!(!comparison.within_tolerance(CohortOutcomeTolerances::new(0.99)));
  assert!(!comparison.within_tolerance(CohortOutcomeTolerances::new(-0.1)));
  assert!(!comparison.within_tolerance(CohortOutcomeTolerances::new(f64::NAN)));
}
