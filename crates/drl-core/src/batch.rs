//! Automated batch simulation runner for statistical studies and regression sweeps.

use crate::agent::AgentPolicy;
use crate::game::Game;
use crate::generator::LevelGeneratorConfig;
use crate::scenario::{Scenario, ScenarioRunner};
use drl_protocol::{BatchSummary, CommandError, EpisodeMetrics, ReplayLog, RunOutcome};

/// Fixed-seed sample definition for a reproducible evaluation cohort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CohortConfig {
  /// First seed in the contiguous cohort range.
  pub start_seed: u64,
  /// Number of episodes to execute.
  pub episode_count: usize,
  /// Maximum commands attempted per episode.
  pub max_turns: u64,
}

impl CohortConfig {
  /// Creates a fixed-seed cohort definition.
  #[must_use]
  pub const fn new(start_seed: u64, episode_count: usize, max_turns: u64) -> Self {
    Self {
      start_seed,
      episode_count,
      max_turns,
    }
  }

  /// Returns the deterministic, wrapping seed sequence for this cohort.
  pub fn seeds(self) -> impl Iterator<Item = u64> {
    (0..self.episode_count).map(move |index| self.start_seed.wrapping_add(index as u64))
  }
}

/// Record of a single completed simulation episode within a batch sweep.
#[derive(Debug, Clone, PartialEq)]
pub struct EpisodeRecord {
  /// Seed used for this episode.
  pub seed: u64,
  /// Telemetry metrics collected during the episode.
  pub metrics: EpisodeMetrics,
  /// Recorded replay log reproducible bit-for-bit.
  pub replay: ReplayLog,
}

/// Reproducible report for one named policy over a fixed seed cohort.
#[derive(Debug, Clone, PartialEq)]
pub struct CohortReport {
  /// Human-readable policy identity supplied by the caller.
  pub policy_name: String,
  /// Sample definition used to produce this report.
  pub config: CohortConfig,
  /// Aggregate outcome and telemetry metrics.
  pub summary: BatchSummary,
  /// Per-seed metrics and replay evidence, in cohort order.
  pub records: Vec<EpisodeRecord>,
}

/// Caller-declared absolute tolerances for a cohort regression comparison.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CohortTolerances {
  /// Maximum permitted absolute win-rate delta.
  pub max_win_rate_delta: f64,
  /// Maximum permitted absolute average-turn delta.
  pub max_average_turns_delta: f64,
}

impl CohortTolerances {
  /// Creates a tolerance pair for a regression gate.
  #[must_use]
  pub const fn new(max_win_rate_delta: f64, max_average_turns_delta: f64) -> Self {
    Self {
      max_win_rate_delta,
      max_average_turns_delta,
    }
  }

  fn is_valid(self) -> bool {
    self.max_win_rate_delta.is_finite()
      && self.max_win_rate_delta >= 0.0
      && self.max_average_turns_delta.is_finite()
      && self.max_average_turns_delta >= 0.0
  }
}

/// Deterministic metric deltas from one compatible cohort comparison.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CohortComparison {
  /// Absolute difference in aggregate win rate.
  pub win_rate_delta: f64,
  /// Absolute difference in aggregate average turns.
  pub average_turns_delta: f64,
  /// Whether both deltas are within the declared tolerances.
  pub within_tolerance: bool,
}

impl CohortReport {
  /// Compares this report against a baseline with caller-declared tolerances.
  ///
  /// Reports are compatible only when their policy identity and complete
  /// sample definitions match. Invalid tolerances and incompatible reports
  /// return `None`; neither input is mutated.
  #[must_use]
  pub fn compare_with(
    &self,
    baseline: &Self,
    tolerances: CohortTolerances,
  ) -> Option<CohortComparison> {
    if self.policy_name != baseline.policy_name
      || self.config != baseline.config
      || !tolerances.is_valid()
    {
      return None;
    }

    let win_rate_delta = (self.summary.win_rate - baseline.summary.win_rate).abs();
    let average_turns_delta = (self.summary.average_turns - baseline.summary.average_turns).abs();
    Some(CohortComparison {
      win_rate_delta,
      average_turns_delta,
      within_tolerance: win_rate_delta <= tolerances.max_win_rate_delta
        && average_turns_delta <= tolerances.max_average_turns_delta,
    })
  }
}

/// Batch simulation runner executing hundreds or thousands of episodes headlessly.
pub struct BatchRunner;

impl BatchRunner {
  /// Runs a procedural fixed-seed cohort and retains per-seed replay evidence.
  pub fn run_procedural_cohort<F, P>(
    config: CohortConfig,
    generator_config: &LevelGeneratorConfig,
    policy_name: impl Into<String>,
    policy_factory: F,
  ) -> Result<CohortReport, CommandError>
  where
    F: FnMut() -> P,
    P: AgentPolicy,
  {
    let (summary, records) = Self::run_procedural_batch(
      config.episode_count,
      config.start_seed,
      config.max_turns,
      generator_config,
      policy_factory,
    )?;
    Ok(CohortReport {
      policy_name: policy_name.into(),
      config,
      summary,
      records,
    })
  }

  /// Runs a scenario fixed-seed cohort and retains per-seed replay evidence.
  pub fn run_scenario_cohort<F, P>(
    scenario: &Scenario,
    config: CohortConfig,
    policy_name: impl Into<String>,
    policy_factory: F,
  ) -> Result<CohortReport, CommandError>
  where
    F: FnMut() -> P,
    P: AgentPolicy,
  {
    let seeds = config.seeds().collect::<Vec<_>>();
    let (summary, records) =
      Self::run_scenario_batch(scenario, &seeds, config.max_turns, policy_factory)?;
    Ok(CohortReport {
      policy_name: policy_name.into(),
      config,
      summary,
      records,
    })
  }

  /// Runs a batch of procedurally generated dungeon episodes with a given agent policy.
  pub fn run_procedural_batch<F, P>(
    num_episodes: usize,
    start_seed: u64,
    max_turns: u64,
    generator_config: &LevelGeneratorConfig,
    mut policy_factory: F,
  ) -> Result<(BatchSummary, Vec<EpisodeRecord>), CommandError>
  where
    F: FnMut() -> P,
    P: AgentPolicy,
  {
    let mut records = Vec::with_capacity(num_episodes);
    let mut episode_metrics = Vec::with_capacity(num_episodes);

    for i in 0..num_episodes {
      let seed = start_seed.wrapping_add(i as u64);
      let mut game = Game::new_procedural(seed, generator_config.clone())?;
      let mut policy = policy_factory();

      let player_id = game
        .world()
        .player_id()
        .unwrap_or(drl_protocol::EntityId(1));
      let player_pos = game
        .world()
        .player()
        .map_or(drl_protocol::Position::new(1, 1), |p| p.position());

      let mut replay = ReplayLog::new(
        seed,
        generator_config.width,
        generator_config.height,
        player_pos,
      );

      let mut metrics = EpisodeMetrics::new();

      for _ in 0..max_turns {
        if game.world().level_id().0 > 1 {
          metrics.outcome = RunOutcome::Victory;
          break;
        }
        if let Some(player) = game.world().player() {
          if !player.is_alive() {
            break;
          }
        } else {
          break;
        }

        let obs = game.observe_player();
        let cmd = match policy.decide_action(&obs) {
          Some(c) => c,
          None => {
            metrics.outcome = RunOutcome::Stalled;
            break;
          }
        };

        replay.record_command(cmd);
        let step_events = match game.step(cmd) {
          Ok(events) => events,
          Err(_) => {
            metrics.outcome = RunOutcome::Stalled;
            break;
          }
        };
        for event in &step_events {
          metrics.record_event(player_id, event);
        }
      }

      if metrics.outcome == RunOutcome::InProgress
        && let Some(player) = game.world().player()
        && player.is_alive()
      {
        if game.world().level_id().0 > 1 {
          metrics.outcome = RunOutcome::Victory;
        } else {
          metrics.outcome = RunOutcome::TurnLimitReached;
        }
      }

      episode_metrics.push(metrics.clone());
      records.push(EpisodeRecord {
        seed,
        metrics,
        replay,
      });
    }

    let summary = BatchSummary::from_episodes(&episode_metrics);
    Ok((summary, records))
  }

  /// Runs a scenario across a sequence of explicit seeds with an automated agent policy.
  pub fn run_scenario_batch<F, P>(
    scenario: &Scenario,
    seeds: &[u64],
    max_turns: u64,
    mut policy_factory: F,
  ) -> Result<(BatchSummary, Vec<EpisodeRecord>), CommandError>
  where
    F: FnMut() -> P,
    P: AgentPolicy,
  {
    let mut records = Vec::with_capacity(seeds.len());
    let mut episode_metrics = Vec::with_capacity(seeds.len());

    for &seed in seeds {
      let mut seeded_scenario = scenario.clone();
      seeded_scenario.seed = seed;
      let mut policy = policy_factory();

      let (_, _, metrics, replay) =
        ScenarioRunner::run_policy(&seeded_scenario, &mut policy, max_turns)?;

      episode_metrics.push(metrics.clone());
      records.push(EpisodeRecord {
        seed,
        metrics,
        replay,
      });
    }

    let summary = BatchSummary::from_episodes(&episode_metrics);
    Ok((summary, records))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::agent::RandomBot;
  use crate::scenario::Scenario;

  #[test]
  fn test_batch_scenario_runner() {
    let ascii = r#"
#####
#@..#
#...>
#####
"#;
    let scenario = Scenario::from_ascii("BatchTest", "Simple map", ascii).unwrap();
    let seeds = [101, 102, 103, 104, 105];

    let (summary, records) =
      BatchRunner::run_scenario_batch(&scenario, &seeds, 30, || RandomBot::new(42)).unwrap();

    assert_eq!(summary.total_episodes, 5);
    assert_eq!(records.len(), 5);
    assert!(summary.total_turns > 0);
  }
}
