//! Telemetry and metrics collection for headless simulation runs.

use crate::event::GameEvent;
use crate::types::{AttackOutcome, DeathCause, EntityId, LevelId};

/// Terminal outcome of an automated simulation episode or scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
  /// The run is still active.
  InProgress,
  /// Player successfully descended stairs or completed scenario objectives.
  Victory,
  /// Player was killed during combat.
  Death {
    /// Specific cause of player death.
    cause: DeathCause,
  },
  /// Episode reached configured turn limit without victory or death.
  TurnLimitReached,
  /// Agent could not produce legal actions or simulation halted.
  Stalled,
}

/// Detailed telemetry and aggregate metrics for a single simulation episode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeMetrics {
  /// Number of game turns survived by the player.
  pub turns_survived: u64,
  /// Terminal outcome of the episode.
  pub outcome: RunOutcome,
  /// Total damage dealt to enemy monsters.
  pub damage_dealt: u32,
  /// Total damage sustained by the player.
  pub damage_taken: u32,
  /// Total number of hostile enemies slain.
  pub enemies_killed: u32,
  /// Total ranged shots attempted by player.
  pub shots_fired: u32,
  /// Total ranged shots that connected.
  pub shots_hit: u32,
  /// Total items collected from ground.
  pub items_picked_up: u32,
  /// Total consumable items used (e.g. medpacks, phase devices).
  pub items_used: u32,
  /// Deepest dungeon level reached during the run.
  pub level_reached: LevelId,
}

impl Default for EpisodeMetrics {
  fn default() -> Self {
    Self {
      turns_survived: 0,
      outcome: RunOutcome::InProgress,
      damage_dealt: 0,
      damage_taken: 0,
      enemies_killed: 0,
      shots_fired: 0,
      shots_hit: 0,
      items_picked_up: 0,
      items_used: 0,
      level_reached: LevelId(1),
    }
  }
}

impl EpisodeMetrics {
  /// Creates a new initial metrics record.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Ingests a game event and updates cumulative metrics.
  pub fn record_event(&mut self, player_id: EntityId, event: &GameEvent) {
    match event {
      GameEvent::TurnStarted { turn } => {
        self.turns_survived = turn.count;
      }
      GameEvent::AttackResolved {
        attacker_id,
        outcome,
        is_ranged,
        ..
      } => {
        if *attacker_id == player_id {
          if *is_ranged {
            self.shots_fired += 1;
          }
          if let AttackOutcome::Hit { damage, .. } = outcome {
            if *is_ranged {
              self.shots_hit += 1;
            }
            self.damage_dealt += damage;
          }
        }
      }
      GameEvent::DamageApplied {
        target_id, amount, ..
      } => {
        if *target_id == player_id {
          self.damage_taken += amount;
        }
      }
      GameEvent::ActorDied { entity_id, cause } => {
        if *entity_id == player_id {
          self.outcome = RunOutcome::Death { cause: *cause };
        } else {
          self.enemies_killed += 1;
        }
      }
      GameEvent::ItemPickedUp { entity_id, .. } => {
        if *entity_id == player_id {
          self.items_picked_up += 1;
        }
      }
      GameEvent::ItemUsed { entity_id, .. } => {
        if *entity_id == player_id {
          self.items_used += 1;
        }
      }
      GameEvent::PlayerTeleported { .. } => {
        self.items_used += 1;
      }
      GameEvent::LevelTransitioned { to_level, .. } => {
        if to_level.0 > self.level_reached.0 {
          self.level_reached = *to_level;
        }
      }
      GameEvent::EntityMoved { .. }
      | GameEvent::EntityWaited { .. }
      | GameEvent::ActionCostPaid { .. }
      | GameEvent::ItemDropped { .. }
      | GameEvent::ItemEquipped { .. }
      | GameEvent::ItemUnequipped { .. }
      | GameEvent::WeaponReloaded { .. }
      | GameEvent::MedicalPowerarmorRepaired { .. }
      | GameEvent::SubtleKnifeInvoked { .. }
      | GameEvent::TrigunAltReloaded { .. }
      | GameEvent::GrammatonFireModeChanged { .. }
      | GameEvent::NukeActivated { .. }
      | GameEvent::LevelNuked { .. }
      | GameEvent::ActorKnockedBack { .. }
      | GameEvent::TurnEnded { .. } => {}
    }
  }
}

/// Aggregate summary metrics across a batch of simulation episodes.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchSummary {
  /// Total number of episodes executed.
  pub total_episodes: usize,
  /// Number of episodes ending in victory/objective completion.
  pub victories: usize,
  /// Number of episodes ending in player death.
  pub deaths: usize,
  /// Number of episodes ending in turn limit timeout.
  pub timeouts: usize,
  /// Total turns survived across all episodes.
  pub total_turns: u64,
  /// Total damage dealt across all episodes.
  pub total_damage_dealt: u32,
  /// Total damage taken across all episodes.
  pub total_damage_taken: u32,
  /// Total enemies killed across all episodes.
  pub total_enemies_killed: u32,
  /// Proportion of runs that achieved victory (0.0 to 1.0).
  pub win_rate: f64,
  /// Average turns survived per episode.
  pub average_turns: f64,
}

impl BatchSummary {
  /// Aggregates a slice of episode metrics into a batch summary.
  #[must_use]
  pub fn from_episodes(episodes: &[EpisodeMetrics]) -> Self {
    let total_episodes = episodes.len();
    if total_episodes == 0 {
      return Self {
        total_episodes: 0,
        victories: 0,
        deaths: 0,
        timeouts: 0,
        total_turns: 0,
        total_damage_dealt: 0,
        total_damage_taken: 0,
        total_enemies_killed: 0,
        win_rate: 0.0,
        average_turns: 0.0,
      };
    }

    let mut victories = 0;
    let mut deaths = 0;
    let mut timeouts = 0;
    let mut total_turns = 0;
    let mut total_damage_dealt = 0;
    let mut total_damage_taken = 0;
    let mut total_enemies_killed = 0;

    for ep in episodes {
      total_turns += ep.turns_survived;
      total_damage_dealt += ep.damage_dealt;
      total_damage_taken += ep.damage_taken;
      total_enemies_killed += ep.enemies_killed;

      match ep.outcome {
        RunOutcome::Victory => victories += 1,
        RunOutcome::Death { .. } => deaths += 1,
        RunOutcome::TurnLimitReached | RunOutcome::Stalled => timeouts += 1,
        RunOutcome::InProgress => {}
      }
    }

    let win_rate = victories as f64 / total_episodes as f64;
    let average_turns = total_turns as f64 / total_episodes as f64;

    Self {
      total_episodes,
      victories,
      deaths,
      timeouts,
      total_turns,
      total_damage_dealt,
      total_damage_taken,
      total_enemies_killed,
      win_rate,
      average_turns,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::types::{EntityId, Turn};

  #[test]
  fn test_episode_metrics_event_aggregation() {
    let player_id = EntityId(1);
    let enemy_id = EntityId(2);
    let mut metrics = EpisodeMetrics::new();

    metrics.record_event(player_id, &GameEvent::TurnStarted { turn: Turn::new(5) });
    metrics.record_event(
      player_id,
      &GameEvent::AttackResolved {
        attacker_id: player_id,
        target_id: enemy_id,
        outcome: AttackOutcome::Hit {
          damage: 12,
          is_lethal: true,
        },
        is_ranged: true,
      },
    );
    metrics.record_event(
      player_id,
      &GameEvent::ActorDied {
        entity_id: enemy_id,
        cause: DeathCause::RangedAttack {
          attacker_id: player_id,
        },
      },
    );

    assert_eq!(metrics.turns_survived, 5);
    assert_eq!(metrics.damage_dealt, 12);
    assert_eq!(metrics.enemies_killed, 1);
    assert_eq!(metrics.shots_fired, 1);
    assert_eq!(metrics.shots_hit, 1);
  }

  #[test]
  fn test_batch_summary_aggregation() {
    let mut ep1 = EpisodeMetrics::new();
    ep1.turns_survived = 10;
    ep1.outcome = RunOutcome::Victory;
    ep1.damage_dealt = 20;
    ep1.enemies_killed = 2;

    let mut ep2 = EpisodeMetrics::new();
    ep2.turns_survived = 6;
    ep2.outcome = RunOutcome::Death {
      cause: DeathCause::Environment,
    };
    ep2.damage_dealt = 5;
    ep2.enemies_killed = 0;

    let summary = BatchSummary::from_episodes(&[ep1, ep2]);
    assert_eq!(summary.total_episodes, 2);
    assert_eq!(summary.victories, 1);
    assert_eq!(summary.deaths, 1);
    assert_eq!(summary.total_turns, 16);
    assert_eq!(summary.total_damage_dealt, 25);
    assert_eq!(summary.total_enemies_killed, 2);
    assert!((summary.win_rate - 0.5).abs() < f64::EPSILON);
    assert!((summary.average_turns - 8.0).abs() < f64::EPSILON);
  }
}
