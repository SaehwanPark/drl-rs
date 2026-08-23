use drl_core::LevelGeneratorConfig;
use drl_core::agent::{ExplorerBot, GreedyCombatBot, RandomBot};
use drl_core::batch::{BatchRunner, CohortConfig, CohortReport};

const COHORT_USAGE: &str =
  "usage: drl-rust cohort [--seed N] [--episodes N] [--max-turns N] [--bot greedy|random|explorer]";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CohortBot {
  Greedy,
  Random,
  Explorer,
}

impl CohortBot {
  fn parse(value: &str) -> Result<Self, String> {
    match value {
      "greedy" => Ok(Self::Greedy),
      "random" => Ok(Self::Random),
      "explorer" => Ok(Self::Explorer),
      _ => Err(format!(
        "unknown bot {value:?}; expected greedy, random, or explorer"
      )),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CohortCliConfig {
  seed: u64,
  episodes: usize,
  max_turns: u64,
  bot: CohortBot,
}

impl Default for CohortCliConfig {
  fn default() -> Self {
    Self {
      seed: 0,
      episodes: 100,
      max_turns: 200,
      bot: CohortBot::Greedy,
    }
  }
}

fn parse_cohort_args(args: &[String]) -> Result<CohortCliConfig, String> {
  let mut config = CohortCliConfig::default();
  let mut index = 0;
  while index < args.len() {
    let flag = &args[index];
    let value = args
      .get(index + 1)
      .ok_or_else(|| format!("missing value for {flag}"))?;
    match flag.as_str() {
      "--seed" => {
        config.seed = value
          .parse()
          .map_err(|_| format!("invalid --seed value {value:?}"))?;
      }
      "--episodes" => {
        config.episodes = value
          .parse()
          .map_err(|_| format!("invalid --episodes value {value:?}"))?;
        if config.episodes == 0 {
          return Err("--episodes must be greater than zero".to_string());
        }
        if config.episodes > 100_000 {
          return Err("--episodes must not exceed 100000".to_string());
        }
      }
      "--max-turns" => {
        config.max_turns = value
          .parse()
          .map_err(|_| format!("invalid --max-turns value {value:?}"))?;
        if config.max_turns == 0 {
          return Err("--max-turns must be greater than zero".to_string());
        }
        if config.max_turns > 1_000_000 {
          return Err("--max-turns must not exceed 1000000".to_string());
        }
      }
      "--bot" => config.bot = CohortBot::parse(value)?,
      _ => return Err(format!("unknown cohort option {flag:?}; {COHORT_USAGE}")),
    }
    index += 2;
  }
  Ok(config)
}

pub(crate) fn run_cohort_command(args: &[String]) -> Result<String, String> {
  let config = parse_cohort_args(args)?;
  let cohort = CohortConfig::new(config.seed, config.episodes, config.max_turns);
  let generator = LevelGeneratorConfig::default();
  let report = match config.bot {
    CohortBot::Greedy => BatchRunner::run_procedural_cohort(
      cohort,
      &generator,
      "GreedyCombatBot",
      GreedyCombatBot::new,
    ),
    CohortBot::Random => {
      BatchRunner::run_procedural_cohort(cohort, &generator, "RandomBot", move || {
        RandomBot::new(config.seed)
      })
    }
    CohortBot::Explorer => {
      BatchRunner::run_procedural_cohort(cohort, &generator, "ExplorerBot", ExplorerBot::new)
    }
  }
  .map_err(|error| format!("study failed: {error}"))?;
  format_cohort_report(&report)
}

fn format_cohort_report(report: &CohortReport) -> Result<String, String> {
  let outcomes = report
    .outcome_distribution()
    .map_err(|error| format!("invalid cohort report: {error}"))?;
  let telemetry = report
    .telemetry_distribution()
    .map_err(|error| format!("invalid cohort report: {error}"))?;
  let last_seed = report
    .config
    .start_seed
    .wrapping_add(report.config.episode_count.saturating_sub(1) as u64);
  let bot = match report.policy_name.as_str() {
    "GreedyCombatBot" => "greedy",
    "RandomBot" => "random",
    "ExplorerBot" => "explorer",
    _ => "unknown",
  };
  let mut output = String::new();
  use std::fmt::Write;
  writeln!(output, "cohort.schema_version=1").unwrap();
  writeln!(output, "cohort.policy={}", report.policy_name).unwrap();
  writeln!(output, "cohort.bot={bot}").unwrap();
  writeln!(output, "cohort.seed_start={}", report.config.start_seed).unwrap();
  writeln!(output, "cohort.seed_end={last_seed}").unwrap();
  writeln!(output, "cohort.episodes={}", report.config.episode_count).unwrap();
  writeln!(output, "cohort.max_turns={}", report.config.max_turns).unwrap();
  writeln!(output, "outcome.total={}", outcomes.total_episodes).unwrap();
  writeln!(output, "outcome.victories={}", outcomes.victories).unwrap();
  writeln!(output, "outcome.deaths={}", outcomes.deaths).unwrap();
  writeln!(
    output,
    "outcome.turn_limit_reached={}",
    outcomes.turn_limit_reached
  )
  .unwrap();
  writeln!(output, "outcome.stalled={}", outcomes.stalled).unwrap();
  writeln!(output, "outcome.in_progress={}", outcomes.in_progress).unwrap();
  writeln!(
    output,
    "outcome.victory_rate={:.6}",
    outcomes.victory_rate()
  )
  .unwrap();
  writeln!(output, "outcome.death_rate={:.6}", outcomes.death_rate()).unwrap();
  writeln!(
    output,
    "telemetry.total_shots_fired={}",
    telemetry.total_shots_fired
  )
  .unwrap();
  writeln!(
    output,
    "telemetry.total_shots_hit={}",
    telemetry.total_shots_hit
  )
  .unwrap();
  writeln!(
    output,
    "telemetry.total_damage_dealt={}",
    telemetry.total_damage_dealt
  )
  .unwrap();
  writeln!(
    output,
    "telemetry.total_damage_taken={}",
    telemetry.total_damage_taken
  )
  .unwrap();
  writeln!(
    output,
    "telemetry.total_enemies_killed={}",
    telemetry.total_enemies_killed
  )
  .unwrap();
  writeln!(
    output,
    "telemetry.total_items_picked_up={}",
    telemetry.total_items_picked_up
  )
  .unwrap();
  writeln!(
    output,
    "telemetry.total_items_used={}",
    telemetry.total_items_used
  )
  .unwrap();
  writeln!(
    output,
    "telemetry.shot_accuracy_rate={:.6}",
    telemetry.shot_accuracy_rate()
  )
  .unwrap();
  writeln!(
    output,
    "telemetry.average_damage_dealt={:.6}",
    telemetry.average_damage_dealt()
  )
  .unwrap();
  Ok(output)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cohort_cli_defaults_are_bounded_and_deterministic() {
    let config = parse_cohort_args(&[]).expect("default cohort options");
    assert_eq!(config.episodes, 100);
    assert_eq!(config.max_turns, 200);
    assert_eq!(config.bot, CohortBot::Greedy);
  }

  #[test]
  fn cohort_cli_rejects_invalid_options() {
    let invalid_bot = vec!["--bot".to_string(), "omniscient".to_string()];
    assert!(parse_cohort_args(&invalid_bot).is_err());
    let zero_episodes = vec!["--episodes".to_string(), "0".to_string()];
    assert!(parse_cohort_args(&zero_episodes).is_err());
    let too_many_turns = vec!["--max-turns".to_string(), "1000001".to_string()];
    assert!(parse_cohort_args(&too_many_turns).is_err());
  }

  #[test]
  fn cohort_cli_output_is_reproducible_and_machine_readable() {
    let args = [
      "--seed".to_string(),
      "12".to_string(),
      "--episodes".to_string(),
      "2".to_string(),
      "--max-turns".to_string(),
      "20".to_string(),
      "--bot".to_string(),
      "explorer".to_string(),
    ];
    let first = run_cohort_command(&args).expect("cohort run");
    let second = run_cohort_command(&args).expect("repeat cohort run");
    assert_eq!(first, second);
    assert!(first.contains("cohort.schema_version=1\n"));
    assert!(first.contains("cohort.policy=ExplorerBot\n"));
    assert!(first.contains("cohort.seed_start=12\n"));
    assert!(first.contains("cohort.seed_end=13\n"));
    assert!(first.contains("cohort.episodes=2\n"));
    assert!(first.contains("telemetry.total_damage_dealt="));
  }
}
