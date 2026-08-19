//! Automated batch simulation runner for statistical studies and regression sweeps.

use crate::agent::AgentPolicy;
use crate::game::Game;
use crate::generator::LevelGeneratorConfig;
use crate::scenario::{Scenario, ScenarioRunner};
use drl_protocol::{BatchSummary, CommandError, EpisodeMetrics, ReplayLog, RunOutcome};

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

/// Batch simulation runner executing hundreds or thousands of episodes headlessly.
pub struct BatchRunner;

impl BatchRunner {
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
