//! Native session adapter over the deterministic core.

use drl_core::{Game, Scenario};
use drl_protocol::{Command, CommandError, PlayerObservation};
use drl_render::{PresentationStep, RenderScene};

/// Fixed demo fixture used only by the native shell executable.
pub const DEMO_SCENARIO: &str = "########################
#......................#
#..@..S...i........>...#
#......................#
########################";

/// Builds the small deterministic fixture used by `drl-desktop`.
pub fn demo_scenario() -> Result<Scenario, String> {
  Scenario::from_ascii(
    "Native frontend boundary",
    "A deterministic geometry-rendering and keyboard-input proof fixture.",
    DEMO_SCENARIO,
  )
}

/// Native-facing simulation boundary that exposes no hidden world state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopSession {
  game: Game,
  last_error: Option<String>,
}

impl DesktopSession {
  /// Instantiates a caller-owned scenario through the canonical core path.
  pub fn new(scenario: &Scenario) -> Result<Self, CommandError> {
    Ok(Self {
      game: scenario.instantiate()?,
      last_error: None,
    })
  }

  /// Returns the fair observation consumed by input and presentation code.
  #[must_use]
  pub fn observation(&self) -> PlayerObservation {
    self.game.observe_player()
  }

  /// Builds the shared renderer-neutral scene from the fair observation.
  #[must_use]
  pub fn scene(&self) -> RenderScene {
    RenderScene::from_observation(&self.observation())
  }

  /// Returns the latest command rejection without exposing core state.
  #[must_use]
  pub fn last_error(&self) -> Option<&str> {
    self.last_error.as_deref()
  }

  /// Submits one semantic command and builds the shared presentation step.
  pub fn submit(&mut self, command: Command) -> Result<PresentationStep, String> {
    let before = self.observation();
    match self.game.step(command) {
      Ok(events) => {
        self.last_error = None;
        let after = self.observation();
        Ok(PresentationStep::from_transition(
          before, command, events, after,
        ))
      }
      Err(error) => {
        let message = error.to_string();
        self.last_error = Some(message.clone());
        Err(message)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use drl_protocol::{Command, Direction, Position};

  fn blocked_scenario() -> Scenario {
    Scenario::from_ascii("blocked", "rejection fixture", "#####\n#@..#\n#####")
      .expect("blocked scenario")
  }

  #[test]
  fn accepted_commands_match_canonical_game_events_and_scene() {
    let scenario = demo_scenario().expect("demo scenario");
    let mut expected = scenario.instantiate().expect("expected game");
    let mut session = DesktopSession::new(&scenario).expect("desktop session");

    let expected_events = expected.step(Command::Wait).expect("expected wait");
    let step = session.submit(Command::Wait).expect("desktop wait");

    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, expected.observe_player());
    assert_eq!(step.after, session.observation());
    assert_eq!(session.scene(), RenderScene::from_observation(&step.after));
  }

  #[test]
  fn rejected_commands_preserve_authoritative_game_identity() {
    let mut session = DesktopSession::new(&blocked_scenario()).expect("desktop session");
    let before = session.game.clone();
    let before_observation = session.observation();

    let result = session.submit(Command::Move(Direction::North));

    assert!(result.is_err());
    assert_eq!(session.game, before);
    assert_eq!(session.observation(), before_observation);
    assert!(session.last_error().is_some());
  }

  #[test]
  fn demo_fixture_contains_only_explicit_fair_targets() {
    let scenario = demo_scenario().expect("demo scenario");
    let session = DesktopSession::new(&scenario).expect("desktop session");
    assert_eq!(session.observation().player_position, Position::new(3, 2));
    assert!(
      session
        .observation()
        .visible_actors
        .iter()
        .any(|actor| !actor.is_player)
    );
  }
}
