//! Actor entities representing creatures and the player character.

use drl_protocol::{ActorView, EntityId, Position};

/// Simulation actor instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
  id: EntityId,
  position: Position,
  name: String,
  is_player: bool,
  blocks_movement: bool,
}

impl Actor {
  /// Creates a new actor.
  #[must_use]
  pub fn new(id: EntityId, position: Position, name: impl Into<String>, is_player: bool) -> Self {
    Self {
      id,
      position,
      name: name.into(),
      is_player,
      blocks_movement: true,
    }
  }

  /// Returns the actor's unique EntityId.
  #[must_use]
  pub const fn id(&self) -> EntityId {
    self.id
  }

  /// Returns the actor's current grid position.
  #[must_use]
  pub const fn position(&self) -> Position {
    self.position
  }

  /// Updates the actor's grid position.
  pub fn set_position(&mut self, pos: Position) {
    self.position = pos;
  }

  /// Returns the actor's display name.
  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Returns true if this actor is the player character.
  #[must_use]
  pub const fn is_player(&self) -> bool {
    self.is_player
  }

  /// Returns true if this actor blocks movement into its tile.
  #[must_use]
  pub const fn blocks_movement(&self) -> bool {
    self.blocks_movement
  }

  /// Sets whether this actor blocks movement.
  pub fn set_blocks_movement(&mut self, blocks: bool) {
    self.blocks_movement = blocks;
  }

  /// Converts this actor to an immutable `ActorView` for observations.
  #[must_use]
  pub fn to_view(&self) -> ActorView {
    ActorView {
      id: self.id,
      position: self.position,
      is_player: self.is_player,
      name: self.name.clone(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_actor_creation_and_view() {
    let actor = Actor::new(EntityId::new(1), Position::new(3, 4), "Marine", true);
    assert_eq!(actor.id(), EntityId::new(1));
    assert_eq!(actor.position(), Position::new(3, 4));
    assert!(actor.is_player());
    assert!(actor.blocks_movement());

    let view = actor.to_view();
    assert_eq!(view.id, EntityId::new(1));
    assert_eq!(view.name, "Marine");
  }
}
