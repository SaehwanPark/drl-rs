//! Simulation world state managing map terrain, actor entities, and ground items.

use drl_protocol::{
  ActorView, CommandError, EntityId, EquipmentSlot, GroundItemView, ItemId, LevelId,
  OmniscientObservation, PlayerObservation, Position, Turn,
};
use std::collections::{BTreeMap, BTreeSet};

use crate::actor::Actor;
use crate::fov::{DEFAULT_VISION_RADIUS, compute_fov};
use crate::generator::GeneratedLevel;
use crate::grid::Map;
use crate::item::Item;

/// Physical world model for a single simulation level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World {
  level_id: LevelId,
  map: Map,
  actors: BTreeMap<EntityId, Actor>,
  ground_items: BTreeMap<ItemId, (Position, Item)>,
  player_id: Option<EntityId>,
  explored_tiles: BTreeSet<Position>,
  next_entity_id: u64,
  next_item_id: u64,
}

impl World {
  /// Creates a new world with the given map and level ID.
  #[must_use]
  pub fn new(level_id: LevelId, map: Map) -> Self {
    Self {
      level_id,
      map,
      actors: BTreeMap::new(),
      ground_items: BTreeMap::new(),
      player_id: None,
      explored_tiles: BTreeSet::new(),
      next_entity_id: 1,
      next_item_id: 1,
    }
  }

  /// Constructs a new World populated from a `GeneratedLevel`, transferring or spawning the player actor.
  #[must_use]
  pub fn from_generated_level(
    level_id: LevelId,
    level: GeneratedLevel,
    existing_player: Option<Actor>,
  ) -> Self {
    let mut world = Self::new(level_id, level.map);

    if let Some(mut player) = existing_player {
      player.set_position(level.player_spawn);
      let id = player.id();
      world.next_entity_id = id.as_u64() + 1;
      world.actors.insert(id, player);
      world.player_id = Some(id);
      world.update_visibility();
    } else {
      let _ = world.spawn_player(level.player_spawn, "Marine");
    }

    for monster in level.monster_spawns {
      if let Some(kind) = monster.kind {
        let _ = world.spawn_monster_kind(monster.position, kind);
      } else {
        let id = world.allocate_entity_id();
        let mut actor = Actor::new(id, monster.position, &monster.name, false).with_stats(
          drl_protocol::HitPoints::full(monster.hp),
          drl_protocol::Speed::new(monster.speed),
          monster.melee_damage,
          monster.ranged_damage,
          monster.ranged_range,
          monster.accuracy,
        );
        actor = actor.with_knockback(monster.knockback);
        actor.set_death_drop(monster.death_drop);
        world.actors.insert(id, actor);
      }
    }

    for (pos, item) in level.item_spawns {
      let _ = world.spawn_ground_item(pos, item);
    }

    world
  }

  /// Removes and returns the player actor from the world (used during level transitions).
  pub fn take_player(&mut self) -> Option<Actor> {
    let id = self.player_id.take()?;
    self.actors.remove(&id)
  }

  /// Allocates a new unique `EntityId`.
  pub fn allocate_entity_id(&mut self) -> EntityId {
    let id = EntityId::new(self.next_entity_id);
    self.next_entity_id += 1;
    id
  }

  /// Allocates a new unique `ItemId`.
  pub fn allocate_item_id(&mut self) -> ItemId {
    let id = ItemId::new(self.next_item_id);
    self.next_item_id += 1;
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

  /// Explored tiles remembered by the player.
  #[must_use]
  pub const fn explored_tiles(&self) -> &BTreeSet<Position> {
    &self.explored_tiles
  }

  /// Returns true if the position has been explored by the player.
  #[must_use]
  pub fn is_explored(&self, pos: Position) -> bool {
    self.explored_tiles.contains(&pos)
  }

  /// Updates the player's field of view and adds visible tiles to explored memory.
  pub fn update_visibility(&mut self) -> BTreeSet<Position> {
    let visible = if let Some(player) = self.player() {
      compute_fov(&self.map, player.position(), DEFAULT_VISION_RADIUS)
    } else {
      BTreeSet::new()
    };

    for &pos in &visible {
      self.explored_tiles.insert(pos);
    }
    visible
  }

  /// Spawns the player character at the given position with default starting loadout.
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
    let mut actor = Actor::new(id, pos, name, true);

    // Initial player equipment & inventory loadout
    let pistol_id = self.allocate_item_id();
    let pistol = Item::pistol(pistol_id);
    let _ = actor.equipment_mut().equip(EquipmentSlot::Weapon, pistol);

    let ammo_id = self.allocate_item_id();
    let ammo = Item::ammo_9mm(ammo_id, 30);
    let _ = actor.inventory_mut().add_item(ammo);

    let med_id = self.allocate_item_id();
    let med = Item::small_medpack(med_id);
    let _ = actor.inventory_mut().add_item(med);

    self.actors.insert(id, actor);
    self.player_id = Some(id);
    self.update_visibility();
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

  /// Spawns a monster actor based on a specific `MonsterKind`.
  pub fn spawn_monster_kind(
    &mut self,
    pos: Position,
    kind: drl_protocol::MonsterKind,
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
    let actor = Actor::from_monster_kind(id, pos, kind);
    self.actors.insert(id, actor);
    Ok(id)
  }

  /// Returns true if there is an unblocked line of sight between two positions.
  #[must_use]
  pub fn has_line_of_sight(&self, from: Position, to: Position) -> bool {
    crate::fov::has_line_of_sight(&self.map, from, to)
  }

  /// Finds a random walkable floor cell that is not occupied by any actor.
  #[must_use]
  pub fn find_random_walkable_unoccupied_cell(
    &self,
    rng: &mut crate::rng::GameRng,
  ) -> Option<Position> {
    let mut available = Vec::new();
    for y in 0..self.map.height() {
      for x in 0..self.map.width() {
        let pos = Position::new(x as i32, y as i32);
        if self.map.is_walkable(pos) && self.living_actor_at(pos).is_none() {
          available.push(pos);
        }
      }
    }

    if available.is_empty() {
      None
    } else {
      let idx = rng.gen_range(0..available.len() as u32) as usize;
      Some(available[idx])
    }
  }

  /// Spawns an item on the ground at a given position.
  pub fn spawn_ground_item(&mut self, pos: Position, item: Item) -> Result<ItemId, CommandError> {
    if !self.map.is_in_bounds(pos) {
      return Err(CommandError::OutOfBounds(pos));
    }
    if !self.map.is_walkable(pos) {
      return Err(CommandError::BlockedByTerrain(pos));
    }
    let id = item.id();
    self.ground_items.insert(id, (pos, item));
    Ok(id)
  }

  /// Picks up the first item lying on the ground at the given position.
  pub fn pickup_ground_item(&mut self, pos: Position) -> Result<Item, CommandError> {
    let target_id = self
      .ground_items
      .iter()
      .find(|(_, (p, _))| *p == pos)
      .map(|(id, _)| *id)
      .ok_or(CommandError::NoItemAtPosition(pos))?;

    let (_, item) = self.ground_items.remove(&target_id).unwrap();
    Ok(item)
  }

  /// Drops an item to the ground at the specified position.
  pub fn drop_item_to_ground(&mut self, pos: Position, item: Item) -> Result<ItemId, CommandError> {
    if !self.map.is_in_bounds(pos) {
      return Err(CommandError::OutOfBounds(pos));
    }
    let id = item.id();
    self.ground_items.insert(id, (pos, item));
    Ok(id)
  }

  /// Reference to all ground items in the world.
  #[must_use]
  pub const fn ground_items(&self) -> &BTreeMap<ItemId, (Position, Item)> {
    &self.ground_items
  }

  /// Returns all items located on the ground at a specific position.
  #[must_use]
  pub fn ground_items_at(&self, pos: Position) -> Vec<&Item> {
    self
      .ground_items
      .values()
      .filter(|(p, _)| *p == pos)
      .map(|(_, item)| item)
      .collect()
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
    let (player_pos, player_hp, visible_positions, inventory, equipped_weapon, equipped_armor) =
      if let Some(player) = self.player() {
        let pos = player.position();
        let fov = compute_fov(&self.map, pos, DEFAULT_VISION_RADIUS);
        (
          pos,
          Some(player.hp()),
          fov,
          player.inventory().to_views(),
          player.equipment().weapon_view(),
          player.equipment().armor_view(),
        )
      } else {
        (
          Position::new(0, 0),
          None,
          BTreeSet::new(),
          Vec::new(),
          None,
          None,
        )
      };

    let mut visible_tiles = Vec::with_capacity(self.explored_tiles.len());
    for &pos in &self.explored_tiles {
      if let Some(tile_view) = self.map.to_tile_view(pos, visible_positions.contains(&pos)) {
        visible_tiles.push(tile_view);
      }
    }

    let visible_actors: Vec<ActorView> = self
      .actors
      .values()
      .filter(|actor| {
        actor.is_alive() && (actor.is_player() || visible_positions.contains(&actor.position()))
      })
      .map(Actor::to_view)
      .collect();

    let ground_items: Vec<GroundItemView> = self
      .ground_items
      .values()
      .filter(|(pos, _)| self.explored_tiles.contains(pos))
      .map(|(pos, item)| GroundItemView {
        position: *pos,
        item: item.to_view(),
      })
      .collect();

    PlayerObservation {
      turn,
      map_width: self.map.width(),
      map_height: self.map.height(),
      player_position: player_pos,
      player_hp,
      visible_tiles,
      visible_actors,
      inventory,
      equipped_weapon,
      equipped_armor,
      ground_items,
    }
  }

  /// Creates an omniscient debug observation snapshot.
  #[must_use]
  pub fn create_omniscient_observation(&self, turn: Turn) -> OmniscientObservation {
    OmniscientObservation {
      turn,
      width: self.map.width(),
      height: self.map.height(),
      tiles: self.map.to_tile_views(true),
      actors: self.actors.values().map(Actor::to_view).collect(),
      ground_items: self
        .ground_items
        .values()
        .map(|(pos, item)| GroundItemView {
          position: *pos,
          item: item.to_view(),
        })
        .collect(),
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
  fn test_world_ground_items_and_pickup() {
    let map = Map::simple_arena(10, 10);
    let mut world = World::new(LevelId::new(1), map);
    let _p_id = world.spawn_player(Position::new(1, 1), "Marine").unwrap();

    let shotgun_id = world.allocate_item_id();
    let shotgun = Item::shotgun(shotgun_id);
    world
      .spawn_ground_item(Position::new(2, 2), shotgun)
      .unwrap();

    assert_eq!(world.ground_items_at(Position::new(2, 2)).len(), 1);
    assert_eq!(
      world.ground_items_at(Position::new(2, 2))[0].name(),
      "Shotgun"
    );

    let picked = world.pickup_ground_item(Position::new(2, 2)).unwrap();
    assert_eq!(picked.name(), "Shotgun");
    assert!(world.ground_items_at(Position::new(2, 2)).is_empty());
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

  #[test]
  fn test_world_visibility_and_player_observation_filtering() {
    let mut map = Map::simple_arena(20, 20);
    // Add wall separating room at x = 10
    for y in 0..20 {
      map.set_tile(Position::new(10, y), crate::grid::Tile::Wall);
    }

    let mut world = World::new(LevelId::new(1), map);
    let _p_id = world.spawn_player(Position::new(5, 5), "Marine").unwrap();

    // Visible monster on player's side of wall
    let m1_id = world
      .spawn_monster(Position::new(7, 5), "Imp", 15, 100, (2, 4))
      .unwrap();

    // Hidden monster behind wall
    let m2_id = world
      .spawn_monster(Position::new(15, 5), "Baron", 50, 100, (4, 8))
      .unwrap();

    let obs = world.create_player_observation(Turn::zero());

    // Player position is correct
    assert_eq!(obs.player_position, Position::new(5, 5));
    assert_eq!(obs.map_width, 20);
    assert_eq!(obs.map_height, 20);
    assert_eq!(obs.player_hp.map(|hp| hp.max), Some(50));

    // Visible actors contains player and Imp (m1), but NOT Baron (m2)
    let visible_ids: Vec<EntityId> = obs.visible_actors.iter().map(|a| a.id).collect();
    assert!(visible_ids.contains(&m1_id));
    assert!(!visible_ids.contains(&m2_id));

    // Omniscient observation still contains all entities
    let omni = world.create_omniscient_observation(Turn::zero());
    let omni_ids: Vec<EntityId> = omni.actors.iter().map(|a| a.id).collect();
    assert!(omni_ids.contains(&m1_id));
    assert!(omni_ids.contains(&m2_id));
  }
}
