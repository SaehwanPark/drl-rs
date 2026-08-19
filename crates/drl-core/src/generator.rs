//! Procedural level generation creating connected room-and-corridor dungeon layouts.
//!
//! Generates bounded 2D tile maps with non-overlapping rooms, connects them with
//! walkable tunnels, places entry and down-stairs exits, verifies reachability
//! via flood fill, and populates rooms with monsters and floor items.

use drl_protocol::{ItemId, Position};
use std::collections::{HashSet, VecDeque};

use crate::grid::{Map, Tile};
use crate::item::Item;
use crate::rng::GameRng;

/// Bounded rectangular room in grid coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Room {
  pub x: i32,
  pub y: i32,
  pub width: u32,
  pub height: u32,
}

impl Room {
  /// Creates a new room at `(x, y)` with given dimensions.
  #[must_use]
  pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
    Self {
      x,
      y,
      width,
      height,
    }
  }

  /// Calculates the center grid coordinate of the room.
  #[must_use]
  pub const fn center(&self) -> Position {
    Position::new(
      self.x + (self.width as i32 / 2),
      self.y + (self.height as i32 / 2),
    )
  }

  /// Returns true if this room intersects or touches another room (including a 1-tile border buffer).
  #[must_use]
  pub const fn intersects(&self, other: &Self) -> bool {
    let self_right = self.x + self.width as i32;
    let self_bottom = self.y + self.height as i32;
    let other_right = other.x + other.width as i32;
    let other_bottom = other.y + other.height as i32;

    self.x <= other_right
      && self_right >= other.x
      && self.y <= other_bottom
      && self_bottom >= other.y
  }
}

/// Configuration parameters for procedural level generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelGeneratorConfig {
  pub width: u32,
  pub height: u32,
  pub max_rooms: u32,
  pub min_room_size: u32,
  pub max_room_size: u32,
  pub max_monsters_per_room: u32,
  pub max_items_per_room: u32,
}

impl Default for LevelGeneratorConfig {
  fn default() -> Self {
    Self {
      width: 40,
      height: 20,
      max_rooms: 6,
      min_room_size: 4,
      max_room_size: 8,
      max_monsters_per_room: 2,
      max_items_per_room: 2,
    }
  }
}

/// Specifications for spawning a monster in a generated level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterSpawn {
  pub position: Position,
  pub name: String,
  pub hp: u32,
  pub speed: u32,
  pub melee_damage: (u32, u32),
}

/// The result of a procedural level generation step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedLevel {
  pub map: Map,
  pub player_spawn: Position,
  pub stairs_position: Position,
  pub rooms: Vec<Room>,
  pub monster_spawns: Vec<MonsterSpawn>,
  pub item_spawns: Vec<(Position, Item)>,
}

/// Procedural dungeon level generator.
pub struct LevelGenerator;

impl LevelGenerator {
  /// Generates a complete procedural level deterministically using the provided RNG.
  #[must_use]
  pub fn generate(
    config: &LevelGeneratorConfig,
    rng: &mut GameRng,
    item_id_counter: &mut u64,
  ) -> GeneratedLevel {
    let mut map = Map::new(config.width, config.height, Tile::Wall);
    let mut rooms: Vec<Room> = Vec::new();

    let max_attempts = config.max_rooms * 10;
    for _ in 0..max_attempts {
      if rooms.len() >= config.max_rooms as usize {
        break;
      }

      let w = rng.gen_range(config.min_room_size..config.max_room_size + 1);
      let h = rng.gen_range(config.min_room_size..config.max_room_size + 1);

      if w + 2 >= config.width || h + 2 >= config.height {
        continue;
      }

      let x = rng.gen_range(1..(config.width - w - 1)) as i32;
      let y = rng.gen_range(1..(config.height - h - 1)) as i32;

      let candidate = Room::new(x, y, w, h);
      let overlaps = rooms.iter().any(|r| r.intersects(&candidate));

      if !overlaps {
        Self::carve_room(&mut map, &candidate);

        if let Some(prev_room) = rooms.last() {
          let prev_center = prev_room.center();
          let new_center = candidate.center();

          if rng.gen_bool(0.5) {
            Self::carve_h_tunnel(&mut map, prev_center.x, new_center.x, prev_center.y);
            Self::carve_v_tunnel(&mut map, prev_center.y, new_center.y, new_center.x);
          } else {
            Self::carve_v_tunnel(&mut map, prev_center.y, new_center.y, prev_center.x);
            Self::carve_h_tunnel(&mut map, prev_center.x, new_center.x, new_center.y);
          }
        }

        rooms.push(candidate);
      }
    }

    // Ensure at least 2 connected rooms exist even on small configs
    if rooms.len() < 2 {
      let r1 = Room::new(2, 2, 6, 6);
      let r2 = Room::new(
        (config.width.saturating_sub(10)) as i32,
        (config.height.saturating_sub(10)) as i32,
        6,
        6,
      );
      Self::carve_room(&mut map, &r1);
      Self::carve_room(&mut map, &r2);
      Self::carve_h_tunnel(&mut map, r1.center().x, r2.center().x, r1.center().y);
      Self::carve_v_tunnel(&mut map, r1.center().y, r2.center().y, r2.center().x);
      rooms.clear();
      rooms.push(r1);
      rooms.push(r2);
    }

    let player_spawn = rooms[0].center();
    let stairs_position = rooms.last().unwrap().center();

    // Place down stairs
    map.set_tile(stairs_position, Tile::StairsDown);

    // Verify reachability; if corridor failed, carve direct fallback path
    if !Self::verify_connectivity(&map, player_spawn, stairs_position) {
      Self::carve_h_tunnel(&mut map, player_spawn.x, stairs_position.x, player_spawn.y);
      Self::carve_v_tunnel(
        &mut map,
        player_spawn.y,
        stairs_position.y,
        stairs_position.x,
      );
      map.set_tile(stairs_position, Tile::StairsDown);
    }

    // Populate monsters and items in rooms (excluding spawn room 0)
    let mut monster_spawns = Vec::new();
    let mut item_spawns = Vec::new();

    for room in rooms.iter().skip(1) {
      let rx_min = (room.x + 1).max(1) as u32;
      let rx_max = (room.x + room.width as i32 - 1).max(rx_min as i32 + 1) as u32;
      let ry_min = (room.y + 1).max(1) as u32;
      let ry_max = (room.y + room.height as i32 - 1).max(ry_min as i32 + 1) as u32;

      let num_monsters = rng.gen_range(0..config.max_monsters_per_room + 1);
      for _ in 0..num_monsters {
        let mx = rng.gen_range(rx_min..rx_max) as i32;
        let my = rng.gen_range(ry_min..ry_max) as i32;
        let pos = Position::new(mx, my);

        if pos != stairs_position && map.is_walkable(pos) {
          let is_imp = rng.gen_bool(0.35);
          let monster = if is_imp {
            MonsterSpawn {
              position: pos,
              name: "Imp".to_string(),
              hp: 20,
              speed: 100,
              melee_damage: (3, 6),
            }
          } else {
            MonsterSpawn {
              position: pos,
              name: "Former Human".to_string(),
              hp: 15,
              speed: 100,
              melee_damage: (2, 4),
            }
          };
          monster_spawns.push(monster);
        }
      }

      let num_items = rng.gen_range(0..config.max_items_per_room + 1);
      for _ in 0..num_items {
        let ix = rng.gen_range(rx_min..rx_max) as i32;
        let iy = rng.gen_range(ry_min..ry_max) as i32;
        let pos = Position::new(ix, iy);

        if pos != stairs_position && map.is_walkable(pos) {
          *item_id_counter += 1;
          let item_id = ItemId::new(*item_id_counter);

          let roll = rng.gen_range(0..100);
          let item = if roll < 40 {
            Item::ammo_9mm(item_id, 20)
          } else if roll < 65 {
            Item::small_medpack(item_id)
          } else if roll < 85 {
            Item::ammo_shells(item_id, 8)
          } else if roll < 95 {
            Item::shotgun(item_id)
          } else {
            Item::green_armor(item_id)
          };
          item_spawns.push((pos, item));
        }
      }
    }

    GeneratedLevel {
      map,
      player_spawn,
      stairs_position,
      rooms,
      monster_spawns,
      item_spawns,
    }
  }

  /// Carves out the interior floor of a room.
  fn carve_room(map: &mut Map, room: &Room) {
    for x in room.x..(room.x + room.width as i32) {
      for y in room.y..(room.y + room.height as i32) {
        map.set_tile(Position::new(x, y), Tile::Floor);
      }
    }
  }

  /// Carves a horizontal tunnel of floor tiles from `x1` to `x2` at row `y`.
  fn carve_h_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    let start_x = x1.min(x2);
    let end_x = x1.max(x2);
    for x in start_x..=end_x {
      map.set_tile(Position::new(x, y), Tile::Floor);
    }
  }

  /// Carves a vertical tunnel of floor tiles from `y1` to `y2` at column `x`.
  fn carve_v_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    let start_y = y1.min(y2);
    let end_y = y1.max(y2);
    for y in start_y..=end_y {
      map.set_tile(Position::new(x, y), Tile::Floor);
    }
  }

  /// Verifies that a walkable path exists between `start` and `target` using breadth-first search.
  #[must_use]
  pub fn verify_connectivity(map: &Map, start: Position, target: Position) -> bool {
    if !map.is_walkable(start) || !map.is_walkable(target) {
      return false;
    }
    if start == target {
      return true;
    }

    let mut visited: HashSet<Position> = HashSet::new();
    let mut queue: VecDeque<Position> = VecDeque::new();

    visited.insert(start);
    queue.push_back(start);

    while let Some(current) = queue.pop_front() {
      if current == target {
        return true;
      }

      for dir in drl_protocol::Direction::ALL_CARDINAL {
        let next = current + dir;
        if map.is_in_bounds(next) && map.is_walkable(next) && visited.insert(next) {
          queue.push_back(next);
        }
      }
    }

    false
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_room_center_and_intersection() {
    let r1 = Room::new(2, 2, 6, 6);
    assert_eq!(r1.center(), Position::new(5, 5));

    let r2 = Room::new(5, 5, 4, 4);
    assert!(r1.intersects(&r2));

    let r3 = Room::new(15, 15, 4, 4);
    assert!(!r1.intersects(&r3));
  }

  #[test]
  fn test_procedural_generation_deterministic() {
    let config = LevelGeneratorConfig::default();
    let mut rng1 = GameRng::from_seed(12345);
    let mut item_id_counter1 = 0;
    let gen1 = LevelGenerator::generate(&config, &mut rng1, &mut item_id_counter1);

    let mut rng2 = GameRng::from_seed(12345);
    let mut item_id_counter2 = 0;
    let gen2 = LevelGenerator::generate(&config, &mut rng2, &mut item_id_counter2);

    assert_eq!(gen1.player_spawn, gen2.player_spawn);
    assert_eq!(gen1.stairs_position, gen2.stairs_position);
    assert_eq!(gen1.rooms, gen2.rooms);
    assert_eq!(gen1.monster_spawns, gen2.monster_spawns);
    assert_eq!(gen1.item_spawns.len(), gen2.item_spawns.len());
  }

  #[test]
  fn test_stairs_and_player_spawn_connectivity() {
    let config = LevelGeneratorConfig::default();
    for seed in [1, 42, 999, 123456] {
      let mut rng = GameRng::from_seed(seed);
      let mut counter = 0;
      let level = LevelGenerator::generate(&config, &mut rng, &mut counter);

      assert!(level.map.is_walkable(level.player_spawn));
      assert!(level.map.is_walkable(level.stairs_position));
      assert_eq!(
        level.map.get_tile(level.stairs_position),
        Some(Tile::StairsDown)
      );
      assert!(
        LevelGenerator::verify_connectivity(&level.map, level.player_spawn, level.stairs_position),
        "Level with seed {seed} must have a walkable path from player spawn to stairs"
      );
    }
  }

  #[test]
  fn test_map_bounds_and_border_walls() {
    let config = LevelGeneratorConfig {
      width: 30,
      height: 20,
      ..Default::default()
    };
    let mut rng = GameRng::from_seed(42);
    let mut counter = 0;
    let level = LevelGenerator::generate(&config, &mut rng, &mut counter);

    // Border rows and columns should be walls
    for x in 0..config.width {
      assert_eq!(
        level.map.get_tile(Position::new(x as i32, 0)),
        Some(Tile::Wall)
      );
      assert_eq!(
        level
          .map
          .get_tile(Position::new(x as i32, (config.height - 1) as i32)),
        Some(Tile::Wall)
      );
    }
    for y in 0..config.height {
      assert_eq!(
        level.map.get_tile(Position::new(0, y as i32)),
        Some(Tile::Wall)
      );
      assert_eq!(
        level
          .map
          .get_tile(Position::new((config.width - 1) as i32, y as i32)),
        Some(Tile::Wall)
      );
    }
  }
}
