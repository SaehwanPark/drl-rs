//! Simulation world state managing map terrain and actor entities.

use drl_protocol::{
  ActorView, CommandError, EntityId, LevelId, OmniscientObservation, PlayerObservation, Position,
  TileView, Turn,
};
use std::collections::BTreeMap;

use crate::actor::Actor;
use crate::grid::Map;

/// Physical world model for a single simulation level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World {
  level_id: LevelId,
  map: Map,
  actors: BTreeMap<EntityId, Actor>,
  player_id: Option<EntityId>,
  next_entity_id: u64,
}

impl World {
  /// Creates a new world with the given map and level ID.
  #[must_use]
  pub fn new(level_id: LevelId, map: Map) -> Self {
    Self {
      level_id,
      map,
      actors: BTreeMap::new(),
      player_id: None,
      next_entity_id: 1,
    }
  }

  /// Allocates a new unique `EntityId`.
  fn allocate_entity_id(&mut self) -> EntityId {
    let id = EntityId::new(self.next_entity_id);
    self.next_entity_id += 1;
    id
  }

  /// Reference to the current level map.
  #[must_use]
  pub const fn map(&self) -> &Map {
    &self.map
  }

  /// Mutable reference to the current level map.
  pub fn map_mut(&mut self) -> &mut Map {
    &mut self.map
  }

  /// Level identifier.
  #[must_use]
  pub const fn level_id(&self) -> LevelId {
    self.level_id
  }

  /// Spawns the player character at the given position.
  pub fn spawn_player(&mut self, pos: Position, name: &str) -> Result<EntityId, CommandError> {
    if !self.map.is_in_bounds(pos) {
      return Err(CommandError::OutOfBounds(pos));
    }
    if !self.map.is_walkable(pos) {
      return Err(CommandError::BlockedByTerrain(pos));
    }
    if let Some(existing) = self.actor_at(pos) {
      return Err(CommandError::BlockedByEntity {
        position: pos,
        entity_id: existing.id(),
      });
    }

    let id = self.allocate_entity_id();
    let actor = Actor::new(id, pos, name, true);
    self.actors.insert(id, actor);
    self.player_id = Some(id);
    Ok(id)
  }

  /// Spawns a non-player actor at the given position.
  pub fn spawn_actor(
    &mut self,
    pos: Position,
    name: &str,
    is_player: bool,
  ) -> Result<EntityId, CommandError> {
    if !self.map.is_in_bounds(pos) {
      return Err(CommandError::OutOfBounds(pos));
    }
    if !self.map.is_walkable(pos) {
      return Err(CommandError::BlockedByTerrain(pos));
    }
    if let Some(existing) = self.actor_at(pos) {
      return Err(CommandError::BlockedByEntity {
        position: pos,
        entity_id: existing.id(),
      });
    }

    let id = self.allocate_entity_id();
    let actor = Actor::new(id, pos, name, is_player);
    self.actors.insert(id, actor);
    if is_player {
      self.player_id = Some(id);
    }
    Ok(id)
  }

  /// Spawns a monster actor with custom combat stats at the given position.
  pub fn spawn_monster(
    &mut self,
    pos: Position,
    name: &str,
    hp: u32,
    speed: u32,
    melee_damage: (u32, u32),
  ) -> Result<EntityId, CommandError> {
    if !self.map.is_in_bounds(pos) {
      return Err(CommandError::OutOfBounds(pos));
    }
    if !self.map.is_walkable(pos) {
      return Err(CommandError::BlockedByTerrain(pos));
    }
    if let Some(existing) = self.actor_at(pos) {
      return Err(CommandError::BlockedByEntity {
        position: pos,
        entity_id: existing.id(),
      });
    }

    let id = self.allocate_entity_id();
    let actor = Actor::new(id, pos, name, false).with_stats(
      drl_protocol::HitPoints::full(hp),
      drl_protocol::Speed::new(speed),
      melee_damage,
      None,
      0,
      65,
    );
    self.actors.insert(id, actor);
    Ok(id)
  }

  /// Returns the player entity ID if spawned.
  #[must_use]
  pub const fn player_id(&self) -> Option<EntityId> {
    self.player_id
  }

  /// Retrieves a reference to the player actor if present.
  #[must_use]
  pub fn player(&self) -> Option<&Actor> {
    self.player_id.and_then(|id| self.actors.get(&id))
  }

  /// Retrieves a mutable reference to the player actor if present.
  pub fn player_mut(&mut self) -> Option<&mut Actor> {
    self.player_id.and_then(|id| self.actors.get_mut(&id))
  }

  /// Retrieves an actor by EntityId.
  #[must_use]
  pub fn get_actor(&self, id: EntityId) -> Option<&Actor> {
    self.actors.get(&id)
  }

  /// Retrieves a mutable reference to an actor by EntityId.
  pub fn get_actor_mut(&mut self, id: EntityId) -> Option<&mut Actor> {
    self.actors.get_mut(&id)
  }

  /// Finds the first blocking actor at a given position.
  #[must_use]
  pub fn actor_at(&self, pos: Position) -> Option<&Actor> {
    self
      .actors
      .values()
      .find(|actor| actor.position() == pos && actor.blocks_movement())
  }

  /// Finds any living actor at a given position.
  #[must_use]
  pub fn living_actor_at(&self, pos: Position) -> Option<&Actor> {
    self
      .actors
      .values()
      .find(|actor| actor.position() == pos && actor.is_alive())
  }

  /// Finds mutable reference to any living actor at a given position.
  pub fn living_actor_at_mut(&mut self, pos: Position) -> Option<&mut Actor> {
    self
      .actors
      .values_mut()
      .find(|actor| actor.position() == pos && actor.is_alive())
  }

  /// Checks if a cell is blocked by terrain or any blocking entity.
  #[must_use]
  pub fn is_cell_blocked(&self, pos: Position) -> bool {
    !self.map.is_walkable(pos) || self.actor_at(pos).is_some()
  }

  /// All actors currently in the world.
  #[must_use]
  pub const fn actors(&self) -> &BTreeMap<EntityId, Actor> {
    &self.actors
  }

  /// Mutable map of all actors currently in the world.
  pub fn actors_mut(&mut self) -> &mut BTreeMap<EntityId, Actor> {
    &mut self.actors
  }

  /// Applies damage to an actor in the world.
  ///
  /// Returns `(damage_taken, is_lethal, optional_death_cause)`.
  pub fn apply_damage(
    &mut self,
    target_id: EntityId,
    amount: u32,
    source: drl_protocol::DamageSource,
  ) -> Result<(u32, bool, Option<drl_protocol::DeathCause>), CommandError> {
    let target = self
      .actors
      .get_mut(&target_id)
      .ok_or(CommandError::EntityNotFound(target_id))?;

    let (taken, lethal) = target.take_damage(amount);
    let death_cause = if lethal {
      match source {
        drl_protocol::DamageSource::Actor(attacker_id) => {
          Some(drl_protocol::DeathCause::MeleeAttack { attacker_id })
        }
        drl_protocol::DamageSource::Environment => Some(drl_protocol::DeathCause::Environment),
      }
    } else {
      None
    };

    Ok((taken, lethal, death_cause))
  }

  /// Creates a player observation snapshot.
  #[must_use]
  pub fn create_player_observation(&self, turn: Turn) -> PlayerObservation {
    let player_pos = self.player().map_or(Position::new(0, 0), Actor::position);
    let visible_tiles: Vec<TileView> = self.map.to_tile_views();
    let visible_actors: Vec<ActorView> = self.actors.values().map(Actor::to_view).collect();

    PlayerObservation {
      turn,
      player_position: player_pos,
      visible_tiles,
      visible_actors,
    }
  }

  /// Creates an omniscient debug observation snapshot.
  #[must_use]
  pub fn create_omniscient_observation(&self, turn: Turn) -> OmniscientObservation {
    OmniscientObservation {
      turn,
      width: self.map.width(),
      height: self.map.height(),
      tiles: self.map.to_tile_views(),
      actors: self.actors.values().map(Actor::to_view).collect(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_world_spawning_and_collision() {
    let map = Map::simple_arena(10, 10);
    let mut world = World::new(LevelId::new(1), map);

    // Spawn player at floor (5, 5)
    let player_id = world.spawn_player(Position::new(5, 5), "Marine").unwrap();
    assert_eq!(world.player_id(), Some(player_id));
    assert_eq!(world.player().unwrap().position(), Position::new(5, 5));

    // Collision with wall
    let wall_err = world
      .spawn_player(Position::new(0, 0), "Marine2")
      .unwrap_err();
    assert_eq!(
      wall_err,
      CommandError::BlockedByTerrain(Position::new(0, 0))
    );

    // Collision with player
    let occ_err = world
      .spawn_actor(Position::new(5, 5), "Imp", false)
      .unwrap_err();
    assert_eq!(
      occ_err,
      CommandError::BlockedByEntity {
        position: Position::new(5, 5),
        entity_id: player_id,
      }
    );
  }

  #[test]
  fn test_world_monster_spawning_and_damage() {
    let map = Map::simple_arena(10, 10);
    let mut world = World::new(LevelId::new(1), map);
    let p_id = world.spawn_player(Position::new(1, 1), "Marine").unwrap();
    let m_id = world
      .spawn_monster(Position::new(1, 2), "Former Human", 20, 100, (2, 4))
      .unwrap();

    assert!(world.actor_at(Position::new(1, 2)).is_some());

    // Apply partial damage
    let (taken, lethal, _) = world
      .apply_damage(m_id, 10, drl_protocol::DamageSource::Actor(p_id))
      .unwrap();
    assert_eq!(taken, 10);
    assert!(!lethal);
    assert!(world.actor_at(Position::new(1, 2)).is_some());

    // Apply lethal damage
    let (taken2, lethal2, cause) = world
      .apply_damage(m_id, 15, drl_protocol::DamageSource::Actor(p_id))
      .unwrap();
    assert_eq!(taken2, 10);
    assert!(lethal2);
    assert_eq!(
      cause,
      Some(drl_protocol::DeathCause::MeleeAttack { attacker_id: p_id })
    );

    // Dead actor no longer blocks cell
    assert!(world.actor_at(Position::new(1, 2)).is_none());
  }
}
