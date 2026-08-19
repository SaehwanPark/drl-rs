//! High-level game execution kernel and turn progression.

use drl_protocol::{
  Command, CommandError, Direction, GameEvent, LevelId, OmniscientObservation, PlayerObservation,
  Position, Turn,
};

use crate::grid::Map;
use crate::rng::GameRng;
use crate::world::World;

/// Complete snapshot of the simulation state at a specific turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
  pub turn: Turn,
  pub world: World,
  pub rng: GameRng,
  pub is_game_over: bool,
}

/// Simulation runner executing turns deterministically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
  state: GameState,
}

impl Game {
  /// Initializes a new game instance with an explicit seed and player starting position.
  pub fn new(
    seed: u64,
    width: u32,
    height: u32,
    player_start: Position,
  ) -> Result<Self, CommandError> {
    let map = Map::simple_arena(width, height);
    let mut world = World::new(LevelId::new(1), map);
    world.spawn_player(player_start, "Marine")?;

    let state = GameState {
      turn: Turn::zero(),
      world,
      rng: GameRng::from_seed(seed),
      is_game_over: false,
    };

    Ok(Self { state })
  }

  /// Initializes a simple arena game with the player at the center.
  pub fn new_arena(seed: u64, width: u32, height: u32) -> Result<Self, CommandError> {
    let start_x = (width / 2) as i32;
    let start_y = (height / 2) as i32;
    Self::new(seed, width, height, Position::new(start_x, start_y))
  }

  /// Current turn.
  #[must_use]
  pub const fn turn(&self) -> Turn {
    self.state.turn
  }

  /// Immutable reference to the world.
  #[must_use]
  pub const fn world(&self) -> &World {
    &self.state.world
  }

  /// Mutable reference to the world.
  pub fn world_mut(&mut self) -> &mut World {
    &mut self.state.world
  }

  /// Immutable reference to the RNG.
  #[must_use]
  pub const fn rng(&self) -> &GameRng {
    &self.state.rng
  }

  /// Mutable reference to the RNG.
  pub fn rng_mut(&mut self) -> &mut GameRng {
    &mut self.state.rng
  }

  /// Returns true if the game has ended.
  #[must_use]
  pub const fn is_game_over(&self) -> bool {
    self.state.is_game_over
  }

  /// Generates a player observation snapshot.
  #[must_use]
  pub fn observe_player(&self) -> PlayerObservation {
    self.state.world.create_player_observation(self.state.turn)
  }

  /// Generates an omniscient observation snapshot.
  #[must_use]
  pub fn observe_omniscient(&self) -> OmniscientObservation {
    self
      .state
      .world
      .create_omniscient_observation(self.state.turn)
  }

  /// Advances the game by one player command step, emitting deterministic events.
  pub fn step(&mut self, command: Command) -> Result<Vec<GameEvent>, CommandError> {
    if self.state.is_game_over {
      return Err(CommandError::InvalidCommand("game is over".to_string()));
    }

    let mut events = Vec::new();
    events.push(GameEvent::TurnStarted {
      turn: self.state.turn,
    });

    match command {
      Command::Move(dir) => {
        let move_events = self.execute_player_move(dir)?;
        events.extend(move_events);
      }
      Command::Wait => {
        let wait_events = self.execute_player_wait()?;
        events.extend(wait_events);
      }
    }

    events.push(GameEvent::TurnEnded {
      turn: self.state.turn,
    });

    self.state.turn = self.state.turn.next();
    Ok(events)
  }

  /// Executes player movement in a given direction.
  fn execute_player_move(&mut self, dir: Direction) -> Result<Vec<GameEvent>, CommandError> {
    if dir == Direction::None {
      return self.execute_player_wait();
    }

    let player_id = self
      .state
      .world
      .player_id()
      .ok_or_else(|| CommandError::InvalidCommand("no player entity in world".to_string()))?;

    let from_pos = self
      .state
      .world
      .get_actor(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?
      .position();

    let target_pos = from_pos + dir;

    // Validate map bounds
    if !self.state.world.map().is_in_bounds(target_pos) {
      return Err(CommandError::OutOfBounds(target_pos));
    }

    // Validate terrain walkability
    if !self.state.world.map().is_walkable(target_pos) {
      return Err(CommandError::BlockedByTerrain(target_pos));
    }

    // Validate entity collisions
    if let Some(blocking_actor) = self.state.world.actor_at(target_pos) {
      return Err(CommandError::BlockedByEntity {
        position: target_pos,
        entity_id: blocking_actor.id(),
      });
    }

    // Apply movement
    let player = self
      .state
      .world
      .get_actor_mut(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?;
    player.set_position(target_pos);

    Ok(vec![GameEvent::EntityMoved {
      entity_id: player_id,
      from: from_pos,
      to: target_pos,
    }])
  }

  /// Executes player wait in place.
  fn execute_player_wait(&mut self) -> Result<Vec<GameEvent>, CommandError> {
    let player_id = self
      .state
      .world
      .player_id()
      .ok_or_else(|| CommandError::InvalidCommand("no player entity in world".to_string()))?;

    let pos = self
      .state
      .world
      .get_actor(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?
      .position();

    Ok(vec![GameEvent::EntityWaited {
      entity_id: player_id,
      position: pos,
    }])
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_game_step_movement_and_wait() {
    let mut game = Game::new_arena(42, 20, 20).unwrap();
    let start_pos = game.world().player().unwrap().position();

    // Step East
    let events = game.step(Command::Move(Direction::East)).unwrap();
    assert_eq!(
      game.world().player().unwrap().position(),
      start_pos + Direction::East
    );
    assert_eq!(game.turn().count, 1);
    assert!(
      events
        .iter()
        .any(|e| matches!(e, GameEvent::EntityMoved { .. }))
    );

    // Step Wait
    let events2 = game.step(Command::Wait).unwrap();
    assert_eq!(game.turn().count, 2);
    assert!(
      events2
        .iter()
        .any(|e| matches!(e, GameEvent::EntityWaited { .. }))
    );
  }

  #[test]
  fn test_game_step_wall_collision_rejected() {
    // Arena 5x5: border at 0 and 4. Center at (2, 2).
    let mut game = Game::new(42, 5, 5, Position::new(1, 1)).unwrap();

    // Move North into Wall at (1, 0)
    let err = game.step(Command::Move(Direction::North)).unwrap_err();
    assert_eq!(err, CommandError::BlockedByTerrain(Position::new(1, 0)));
    // Turn should NOT advance on failed command
    assert_eq!(game.turn().count, 0);
    assert_eq!(
      game.world().player().unwrap().position(),
      Position::new(1, 1)
    );
  }
}
