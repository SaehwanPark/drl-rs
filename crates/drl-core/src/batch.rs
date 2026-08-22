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

/// Exact terminal-outcome counts retained by a fixed-seed cohort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CohortOutcomeDistribution {
  /// Number of records represented by this distribution.
  pub total_episodes: usize,
  /// Number of episodes that ended in victory.
  pub victories: usize,
  /// Number of episodes that ended in player death.
  pub deaths: usize,
  /// Number of episodes that reached their turn limit.
  pub turn_limit_reached: usize,
  /// Number of episodes whose policy or simulation stalled.
  pub stalled: usize,
  /// Number of records that remain in progress.
  pub in_progress: usize,
}

impl CohortOutcomeDistribution {
  fn from_records(records: &[EpisodeRecord]) -> Self {
    let mut distribution = Self {
      total_episodes: records.len(),
      victories: 0,
      deaths: 0,
      turn_limit_reached: 0,
      stalled: 0,
      in_progress: 0,
    };

    for record in records {
      match &record.metrics.outcome {
        RunOutcome::Victory => distribution.victories += 1,
        RunOutcome::Death { .. } => distribution.deaths += 1,
        RunOutcome::TurnLimitReached => distribution.turn_limit_reached += 1,
        RunOutcome::Stalled => distribution.stalled += 1,
        RunOutcome::InProgress => distribution.in_progress += 1,
      }
    }
    distribution
  }

  fn rate(count: usize, total: usize) -> f64 {
    if total == 0 {
      0.0
    } else {
      count as f64 / total as f64
    }
  }

  /// Returns the sample-size-normalized victory rate.
  #[must_use]
  pub fn victory_rate(self) -> f64 {
    Self::rate(self.victories, self.total_episodes)
  }

  /// Returns the sample-size-normalized death rate.
  #[must_use]
  pub fn death_rate(self) -> f64 {
    Self::rate(self.deaths, self.total_episodes)
  }

  /// Returns the sample-size-normalized turn-limit rate.
  #[must_use]
  pub fn turn_limit_rate(self) -> f64 {
    Self::rate(self.turn_limit_reached, self.total_episodes)
  }

  /// Returns the sample-size-normalized stalled-episode rate.
  #[must_use]
  pub fn stalled_rate(self) -> f64 {
    Self::rate(self.stalled, self.total_episodes)
  }

  /// Returns the sample-size-normalized in-progress rate.
  #[must_use]
  pub fn in_progress_rate(self) -> f64 {
    Self::rate(self.in_progress, self.total_episodes)
  }
}

/// Deterministic combat and economy totals projected from a validated cohort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CohortTelemetryDistribution {
  /// Number of records represented by this distribution.
  pub total_episodes: usize,
  /// Total ranged shots attempted by all episodes.
  pub total_shots_fired: u64,
  /// Total ranged shots that connected.
  pub total_shots_hit: u64,
  /// Total damage dealt by all episodes.
  pub total_damage_dealt: u64,
  /// Total damage sustained by all episodes.
  pub total_damage_taken: u64,
  /// Total hostile enemies slain by all episodes.
  pub total_enemies_killed: u64,
  /// Total ground items collected by all episodes.
  pub total_items_picked_up: u64,
  /// Total consumables used by all episodes.
  pub total_items_used: u64,
}

impl CohortTelemetryDistribution {
  fn from_records(records: &[EpisodeRecord]) -> Self {
    let mut distribution = Self {
      total_episodes: records.len(),
      total_shots_fired: 0,
      total_shots_hit: 0,
      total_damage_dealt: 0,
      total_damage_taken: 0,
      total_enemies_killed: 0,
      total_items_picked_up: 0,
      total_items_used: 0,
    };

    for record in records {
      distribution.total_shots_fired += u64::from(record.metrics.shots_fired);
      distribution.total_shots_hit += u64::from(record.metrics.shots_hit);
      distribution.total_damage_dealt += u64::from(record.metrics.damage_dealt);
      distribution.total_damage_taken += u64::from(record.metrics.damage_taken);
      distribution.total_enemies_killed += u64::from(record.metrics.enemies_killed);
      distribution.total_items_picked_up += u64::from(record.metrics.items_picked_up);
      distribution.total_items_used += u64::from(record.metrics.items_used);
    }
    distribution
  }

  fn rate(count: u64, total: u64) -> f64 {
    if total == 0 {
      0.0
    } else {
      count as f64 / total as f64
    }
  }

  fn average(total: u64, episodes: usize) -> f64 {
    Self::rate(total, episodes as u64)
  }

  /// Returns the hit rate over episodes that attempted ranged shots.
  #[must_use]
  pub fn shot_accuracy_rate(self) -> f64 {
    Self::rate(self.total_shots_hit, self.total_shots_fired)
  }

  /// Returns average damage dealt per episode.
  #[must_use]
  pub fn average_damage_dealt(self) -> f64 {
    Self::average(self.total_damage_dealt, self.total_episodes)
  }

  /// Returns average damage taken per episode.
  #[must_use]
  pub fn average_damage_taken(self) -> f64 {
    Self::average(self.total_damage_taken, self.total_episodes)
  }

  /// Returns average enemies killed per episode.
  #[must_use]
  pub fn average_enemies_killed(self) -> f64 {
    Self::average(self.total_enemies_killed, self.total_episodes)
  }

  /// Returns average items picked up per episode.
  #[must_use]
  pub fn average_items_picked_up(self) -> f64 {
    Self::average(self.total_items_picked_up, self.total_episodes)
  }

  /// Returns average consumables used per episode.
  #[must_use]
  pub fn average_items_used(self) -> f64 {
    Self::average(self.total_items_used, self.total_episodes)
  }
}

/// Absolute per-outcome rate deltas for two compatible cohort reports.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CohortOutcomeComparison {
  /// Absolute victory-rate delta.
  pub victory_rate_delta: f64,
  /// Absolute death-rate delta.
  pub death_rate_delta: f64,
  /// Absolute turn-limit-rate delta.
  pub turn_limit_rate_delta: f64,
  /// Absolute stalled-episode-rate delta.
  pub stalled_rate_delta: f64,
  /// Absolute in-progress-rate delta.
  pub in_progress_rate_delta: f64,
}

/// Caller-declared absolute tolerance shared by every outcome-rate delta.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CohortOutcomeTolerances {
  /// Maximum permitted absolute delta for any outcome rate.
  pub max_rate_delta: f64,
}

impl CohortOutcomeTolerances {
  /// Creates an outcome-rate regression tolerance.
  #[must_use]
  pub const fn new(max_rate_delta: f64) -> Self {
    Self { max_rate_delta }
  }

  fn is_valid(self) -> bool {
    self.max_rate_delta.is_finite() && self.max_rate_delta >= 0.0
  }
}

impl CohortOutcomeComparison {
  /// Returns whether every outcome-rate delta is within the caller's bound.
  #[must_use]
  pub fn within_tolerance(self, tolerances: CohortOutcomeTolerances) -> bool {
    tolerances.is_valid()
      && self.victory_rate_delta <= tolerances.max_rate_delta
      && self.death_rate_delta <= tolerances.max_rate_delta
      && self.turn_limit_rate_delta <= tolerances.max_rate_delta
      && self.stalled_rate_delta <= tolerances.max_rate_delta
      && self.in_progress_rate_delta <= tolerances.max_rate_delta
  }
}

/// Evidence-integrity failure for a fixed-seed cohort report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CohortReportError {
  /// The report does not contain one record for every configured episode.
  RecordCount { expected: usize, actual: usize },
  /// A record is outside the configured contiguous seed sequence.
  SeedMismatch {
    /// Zero-based record position.
    index: usize,
    /// Seed required by the cohort definition.
    expected: u64,
    /// Seed recorded in the report.
    actual: u64,
  },
  /// A replay record does not identify the episode seed it is evidence for.
  ReplaySeedMismatch {
    /// Zero-based record position.
    index: usize,
    /// Seed recorded on the episode record.
    expected: u64,
    /// Seed recorded in the replay log.
    actual: u64,
  },
  /// Aggregate metrics do not equal the report's per-episode metrics.
  SummaryMismatch,
}

impl std::fmt::Display for CohortReportError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::RecordCount { expected, actual } => {
        write!(
          formatter,
          "cohort record count mismatch: expected {expected}, got {actual}"
        )
      }
      Self::SeedMismatch {
        index,
        expected,
        actual,
      } => write!(
        formatter,
        "cohort seed mismatch at record {index}: expected {expected}, got {actual}"
      ),
      Self::ReplaySeedMismatch {
        index,
        expected,
        actual,
      } => write!(
        formatter,
        "cohort replay seed mismatch at record {index}: expected {expected}, got {actual}"
      ),
      Self::SummaryMismatch => write!(formatter, "cohort summary does not match episode metrics"),
    }
  }
}

impl std::error::Error for CohortReportError {}

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
  /// Verifies the report's sample definition and retained evidence agree.
  ///
  /// This is a pure integrity gate for caller-owned reports. It does not
  /// rerun simulations or infer statistical significance, balance, or
  /// difficulty conclusions.
  pub fn validate(&self) -> Result<(), CohortReportError> {
    if self.records.len() != self.config.episode_count {
      return Err(CohortReportError::RecordCount {
        expected: self.config.episode_count,
        actual: self.records.len(),
      });
    }

    let mut metrics = Vec::with_capacity(self.records.len());
    for (index, (expected_seed, record)) in self.config.seeds().zip(&self.records).enumerate() {
      if record.seed != expected_seed {
        return Err(CohortReportError::SeedMismatch {
          index,
          expected: expected_seed,
          actual: record.seed,
        });
      }
      if record.replay.seed != record.seed {
        return Err(CohortReportError::ReplaySeedMismatch {
          index,
          expected: record.seed,
          actual: record.replay.seed,
        });
      }
      metrics.push(record.metrics.clone());
    }

    if BatchSummary::from_episodes(&metrics) != self.summary {
      return Err(CohortReportError::SummaryMismatch);
    }
    Ok(())
  }

  /// Projects exact outcome counts and normalized rates from retained records.
  ///
  /// The report is validated before projection so incomplete or tampered
  /// evidence cannot be summarized accidentally. This does not rerun episodes,
  /// mutate the report, or infer balance, difficulty, or significance.
  pub fn outcome_distribution(&self) -> Result<CohortOutcomeDistribution, CohortReportError> {
    self.validate()?;
    Ok(CohortOutcomeDistribution::from_records(&self.records))
  }

  /// Projects combat and item telemetry from retained records.
  ///
  /// The report is validated before projection so incomplete or tampered
  /// evidence cannot be summarized accidentally. The rates are descriptive
  /// sample projections, not balance, difficulty, or statistical claims.
  pub fn telemetry_distribution(&self) -> Result<CohortTelemetryDistribution, CohortReportError> {
    self.validate()?;
    Ok(CohortTelemetryDistribution::from_records(&self.records))
  }

  /// Compares outcome rates for compatible, integrity-checked reports.
  ///
  /// Compatibility requires the same policy identity and complete sample
  /// definition. Invalid evidence returns `None`; no episodes are rerun and
  /// the deltas do not imply a balance result or statistical significance.
  #[must_use]
  pub fn compare_outcomes(&self, baseline: &Self) -> Option<CohortOutcomeComparison> {
    if self.policy_name != baseline.policy_name || self.config != baseline.config {
      return None;
    }
    let candidate = self.outcome_distribution().ok()?;
    let baseline = baseline.outcome_distribution().ok()?;
    Some(CohortOutcomeComparison {
      victory_rate_delta: (candidate.victory_rate() - baseline.victory_rate()).abs(),
      death_rate_delta: (candidate.death_rate() - baseline.death_rate()).abs(),
      turn_limit_rate_delta: (candidate.turn_limit_rate() - baseline.turn_limit_rate()).abs(),
      stalled_rate_delta: (candidate.stalled_rate() - baseline.stalled_rate()).abs(),
      in_progress_rate_delta: (candidate.in_progress_rate() - baseline.in_progress_rate()).abs(),
    })
  }

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
