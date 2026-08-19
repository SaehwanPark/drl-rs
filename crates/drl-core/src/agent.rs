//! Automated test agent policies for headless simulation and bot play.

use crate::rng::GameRng;
use drl_protocol::{
  ActorView, Command, Direction, ItemCategory, PlayerObservation, Position, TileKind, TileView,
};
use std::collections::BTreeSet;

/// Common interface for automated test agents and scripted bots.
pub trait AgentPolicy {
  /// Human-readable name of the bot policy.
  fn name(&self) -> &str;

  /// Decides the next command to execute given the current player observation.
  ///
  /// Returning `None` indicates the agent wishes to halt or has no further actions.
  fn decide_action(&mut self, observation: &PlayerObservation) -> Option<Command>;
}

/// Helper function to find a visible tile in an observation.
fn find_tile(observation: &PlayerObservation, pos: Position) -> Option<&TileView> {
  observation
    .visible_tiles
    .iter()
    .find(|tile| tile.position == pos)
}

/// Helper function to find current player HP.
fn get_player_hp(observation: &PlayerObservation) -> u32 {
  observation
    .visible_actors
    .iter()
    .find(|actor| actor.is_player)
    .and_then(|actor| actor.hp)
    .map_or(50, |hp| hp.current)
}

/// Helper function to determine directional step from `from` towards `to`.
fn direction_towards(from: Position, to: Position) -> Option<Direction> {
  let dx = to.x - from.x;
  let dy = to.y - from.y;
  Direction::from_delta(dx, dy)
}

/// Helper function to check if the line of fire between two positions is clear of walls.
fn has_clear_line_of_fire(observation: &PlayerObservation, from: Position, to: Position) -> bool {
  let mut x0 = from.x;
  let mut y0 = from.y;
  let x1 = to.x;
  let y1 = to.y;

  let dx = (x1 - x0).abs();
  let dy = -(y1 - y0).abs();
  let sx = if x0 < x1 { 1 } else { -1 };
  let sy = if y0 < y1 { 1 } else { -1 };
  let mut err = dx + dy;

  loop {
    if x0 == x1 && y0 == y1 {
      break;
    }
    let cur = Position::new(x0, y0);
    if cur != from
      && cur != to
      && let Some(tile) = find_tile(observation, cur)
      && (tile.kind == TileKind::Wall || tile.kind == TileKind::DoorClosed)
    {
      return false;
    }
    let e2 = 2 * err;
    if e2 >= dy {
      err += dy;
      x0 += sx;
    }
    if e2 <= dx {
      err += dx;
      y0 += sy;
    }
  }

  true
}

/// A bot that picks uniformly at random among legal local actions.
#[derive(Debug)]
pub struct RandomBot {
  rng: GameRng,
}

impl RandomBot {
  /// Creates a new random bot with a deterministic RNG seed.
  #[must_use]
  pub fn new(seed: u64) -> Self {
    Self {
      rng: GameRng::from_seed(seed),
    }
  }
}

impl AgentPolicy for RandomBot {
  fn name(&self) -> &str {
    "RandomBot"
  }

  fn decide_action(&mut self, observation: &PlayerObservation) -> Option<Command> {
    let mut candidates = Vec::with_capacity(16);
    candidates.push(Command::Wait);

    // If standing on floor item, consider pickup
    for item in &observation.ground_items {
      if item.position == observation.player_position {
        candidates.push(Command::Pickup);
      }
    }

    // Check weapon reload if equipped
    if let Some(weapon) = &observation.equipped_weapon
      && let Some((cur, max)) = weapon.clip
      && cur < max
    {
      candidates.push(Command::Reload);
    }

    // Consider moving to any visible walkable neighbor tile
    for &dir in &[
      Direction::North,
      Direction::South,
      Direction::East,
      Direction::West,
      Direction::NorthEast,
      Direction::NorthWest,
      Direction::SouthEast,
      Direction::SouthWest,
    ] {
      let target_pos = observation.player_position + dir;
      if let Some(tile) = find_tile(observation, target_pos)
        && tile.kind != TileKind::Wall
      {
        candidates.push(Command::Move(dir));
      }
    }

    let idx = self.rng.gen_range(0..candidates.len() as u32) as usize;
    Some(candidates[idx])
  }
}

/// Tactical combat bot prioritizing health preservation, ranged and melee engagement,
/// ammo replenishment, looting, and stairs descent.
#[derive(Debug, Default)]
pub struct GreedyCombatBot {
  visited: BTreeSet<Position>,
}

impl GreedyCombatBot {
  /// Creates a new greedy combat bot.
  #[must_use]
  pub fn new() -> Self {
    Self {
      visited: BTreeSet::new(),
    }
  }
}

impl AgentPolicy for GreedyCombatBot {
  fn name(&self) -> &str {
    "GreedyCombatBot"
  }

  fn decide_action(&mut self, observation: &PlayerObservation) -> Option<Command> {
    let pos = observation.player_position;
    self.visited.insert(pos);

    // 1. Survival: If HP is low (< 25) and have MedPack in inventory, use it
    let current_hp = get_player_hp(observation);
    if current_hp < 25
      && let Some(medpack) = observation
        .inventory
        .iter()
        .find(|item| item.category == ItemCategory::MedPack)
    {
      return Some(Command::Use(medpack.id));
    }

    // 2. Weapon reload: If equipped weapon clip is empty and we have ammo, reload
    if let Some(weapon) = &observation.equipped_weapon
      && let Some((cur, _)) = weapon.clip
      && cur == 0
    {
      return Some(Command::Reload);
    }

    // 3. Combat: Engage visible hostile monsters
    let mut visible_monsters: Vec<&ActorView> = observation
      .visible_actors
      .iter()
      .filter(|a| !a.is_player && a.is_alive)
      .collect();

    if !visible_monsters.is_empty() {
      visible_monsters.sort_by_key(|m| pos.distance_chebyshev(m.position));
      let nearest = visible_monsters[0];
      let dist = pos.distance_chebyshev(nearest.position);

      // If adjacent, melee bump attack
      if dist == 1
        && let Some(dir) = direction_towards(pos, nearest.position)
      {
        return Some(Command::Move(dir));
      }

      // If weapon equipped with ammo in clip and clear shot, fire at nearest monster
      if let Some(weapon) = &observation.equipped_weapon {
        let has_ammo = weapon.clip.is_none_or(|(cur, _)| cur > 0);
        if has_ammo && dist <= 8 && has_clear_line_of_fire(observation, pos, nearest.position) {
          return Some(Command::AttackRanged(nearest.position));
        }
      }

      // If out of range, no ammo, or blocked line of fire, step toward monster
      if let Some(dir) = direction_towards(pos, nearest.position) {
        let next_pos = pos + dir;
        if let Some(tile) = find_tile(observation, next_pos)
          && tile.kind != TileKind::Wall
        {
          return Some(Command::Move(dir));
        }
      }
    }

    // 4. Pickup items on current tile
    for ground_item in &observation.ground_items {
      if ground_item.position == pos {
        return Some(Command::Pickup);
      }
    }

    // 5. Stairs descent if on stairs
    if let Some(tile) = find_tile(observation, pos)
      && tile.kind == TileKind::StairsDown
    {
      return Some(Command::Descend);
    }

    // 6. Move towards visible stairs if seen
    for tile in &observation.visible_tiles {
      if tile.kind == TileKind::StairsDown
        && let Some(dir) = direction_towards(pos, tile.position)
      {
        let step_pos = pos + dir;
        if let Some(step_tile) = find_tile(observation, step_pos)
          && step_tile.kind != TileKind::Wall
        {
          return Some(Command::Move(dir));
        }
      }
    }

    // 7. Exploration: step towards unvisited visible walkable tiles
    for &dir in &[
      Direction::East,
      Direction::South,
      Direction::North,
      Direction::West,
      Direction::SouthEast,
      Direction::NorthEast,
      Direction::SouthWest,
      Direction::NorthWest,
    ] {
      let next_pos = pos + dir;
      if !self.visited.contains(&next_pos)
        && let Some(tile) = find_tile(observation, next_pos)
        && tile.kind != TileKind::Wall
      {
        return Some(Command::Move(dir));
      }
    }

    // Fallback: move to any walkable neighbor
    for &dir in &[
      Direction::North,
      Direction::East,
      Direction::South,
      Direction::West,
    ] {
      let next_pos = pos + dir;
      if let Some(tile) = find_tile(observation, next_pos)
        && tile.kind != TileKind::Wall
      {
        return Some(Command::Move(dir));
      }
    }

    Some(Command::Wait)
  }
}

/// Exploration bot prioritizing discovering unseen tiles, finding stairs, and descending.
#[derive(Debug, Default)]
pub struct ExplorerBot {
  visited: BTreeSet<Position>,
}

impl ExplorerBot {
  /// Creates a new explorer bot.
  #[must_use]
  pub fn new() -> Self {
    Self {
      visited: BTreeSet::new(),
    }
  }
}

impl AgentPolicy for ExplorerBot {
  fn name(&self) -> &str {
    "ExplorerBot"
  }

  fn decide_action(&mut self, observation: &PlayerObservation) -> Option<Command> {
    let pos = observation.player_position;
    self.visited.insert(pos);

    // If on stairs, descend
    if let Some(tile) = find_tile(observation, pos)
      && tile.kind == TileKind::StairsDown
    {
      return Some(Command::Descend);
    }

    // If stairs are visible, navigate towards stairs
    for tile in &observation.visible_tiles {
      if tile.kind == TileKind::StairsDown
        && let Some(dir) = direction_towards(pos, tile.position)
      {
        let step_pos = pos + dir;
        if let Some(step_tile) = find_tile(observation, step_pos)
          && step_tile.kind != TileKind::Wall
        {
          return Some(Command::Move(dir));
        }
      }
    }

    // Pick up items on floor
    for item in &observation.ground_items {
      if item.position == pos {
        return Some(Command::Pickup);
      }
    }

    // Attack adjacent monsters if blocking way
    for monster in &observation.visible_actors {
      if !monster.is_player
        && monster.is_alive
        && pos.distance_chebyshev(monster.position) == 1
        && let Some(dir) = direction_towards(pos, monster.position)
      {
        return Some(Command::Move(dir));
      }
    }

    // Move to unvisited walkable neighbors
    for &dir in &[
      Direction::North,
      Direction::East,
      Direction::South,
      Direction::West,
      Direction::NorthEast,
      Direction::SouthEast,
      Direction::NorthWest,
      Direction::SouthWest,
    ] {
      let next_pos = pos + dir;
      if !self.visited.contains(&next_pos)
        && let Some(tile) = find_tile(observation, next_pos)
        && tile.kind != TileKind::Wall
      {
        return Some(Command::Move(dir));
      }
    }

    // Fallback: move to any walkable neighbor
    for &dir in &[
      Direction::South,
      Direction::North,
      Direction::East,
      Direction::West,
    ] {
      let next_pos = pos + dir;
      if let Some(tile) = find_tile(observation, next_pos)
        && tile.kind != TileKind::Wall
      {
        return Some(Command::Move(dir));
      }
    }

    Some(Command::Wait)
  }
}
