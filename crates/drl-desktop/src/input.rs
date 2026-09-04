//! Native physical-key translation into the shared semantic command protocol.

use drl_protocol::{Command, Direction, PlayerObservation};
use winit::keyboard::KeyCode;

/// Maps a physical desktop key to a semantic command using only fair input.
#[must_use]
pub fn command_for_key(key: KeyCode, observation: &PlayerObservation) -> Option<Command> {
  let direction = match key {
    KeyCode::ArrowUp | KeyCode::KeyW | KeyCode::Numpad8 => Some(Direction::North),
    KeyCode::ArrowRight | KeyCode::KeyD | KeyCode::Numpad6 => Some(Direction::East),
    KeyCode::ArrowDown | KeyCode::KeyS | KeyCode::Numpad2 => Some(Direction::South),
    KeyCode::ArrowLeft | KeyCode::KeyA | KeyCode::Numpad4 => Some(Direction::West),
    KeyCode::Numpad7 => Some(Direction::NorthWest),
    KeyCode::Numpad9 => Some(Direction::NorthEast),
    KeyCode::Numpad1 => Some(Direction::SouthWest),
    KeyCode::Numpad3 => Some(Direction::SouthEast),
    _ => None,
  };
  if let Some(direction) = direction {
    return Some(Command::Move(direction));
  }

  match key {
    KeyCode::Period | KeyCode::Numpad5 | KeyCode::Space => Some(Command::Wait),
    KeyCode::KeyG => Some(Command::Pickup),
    KeyCode::KeyR => Some(Command::Reload),
    KeyCode::BracketRight => Some(Command::Descend),
    KeyCode::KeyF => first_visible_target(observation).map(Command::AttackRanged),
    KeyCode::KeyC => first_visible_target(observation).map(Command::AttackRangedChainfire),
    _ => None,
  }
}

fn first_visible_target(observation: &PlayerObservation) -> Option<drl_protocol::Position> {
  observation
    .visible_actors
    .iter()
    .find(|actor| !actor.is_player)
    .map(|actor| actor.position)
}

#[cfg(test)]
mod tests {
  use super::*;
  use drl_protocol::{ActorView, EntityId, HitPoints, Position, Speed, TileKind, TileView, Turn};

  fn observation() -> PlayerObservation {
    PlayerObservation {
      map_width: 3,
      map_height: 3,
      player_position: Position::new(1, 1),
      visible_tiles: vec![TileView {
        position: Position::new(1, 1),
        kind: TileKind::Floor,
        is_walkable: true,
        is_transparent: true,
        is_visible: true,
      }],
      visible_actors: vec![
        ActorView {
          id: EntityId(1),
          position: Position::new(1, 1),
          is_player: true,
          name: "Player".to_string(),
          hp: Some(HitPoints::full(10)),
          is_alive: true,
          speed: Speed::new(100),
          monster_kind: None,
        },
        ActorView {
          id: EntityId(2),
          position: Position::new(2, 1),
          is_player: false,
          name: "Target".to_string(),
          hp: Some(HitPoints::full(10)),
          is_alive: true,
          speed: Speed::new(100),
          monster_kind: None,
        },
      ],
      ground_items: Vec::new(),
      inventory: Vec::new(),
      equipped_weapon: None,
      equipped_armor: None,
      player_hp: Some(HitPoints::full(10)),
      turn: Turn::zero(),
    }
  }

  #[test]
  fn physical_keys_map_to_protocol_commands() {
    let observation = observation();
    assert_eq!(
      command_for_key(KeyCode::ArrowUp, &observation),
      Some(Command::Move(Direction::North))
    );
    assert_eq!(
      command_for_key(KeyCode::Space, &observation),
      Some(Command::Wait)
    );
    assert_eq!(
      command_for_key(KeyCode::KeyF, &observation),
      Some(Command::AttackRanged(Position::new(2, 1)))
    );
    assert_eq!(
      command_for_key(KeyCode::KeyC, &observation),
      Some(Command::AttackRangedChainfire(Position::new(2, 1)))
    );
  }

  #[test]
  fn target_keys_do_not_invent_hidden_targets() {
    let mut observation = observation();
    observation.visible_actors.pop();
    assert_eq!(command_for_key(KeyCode::KeyF, &observation), None);
    assert_eq!(command_for_key(KeyCode::KeyC, &observation), None);
  }
}
