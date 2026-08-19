//! Declarative scenario fixture specifications for testing and benchmarks.

use crate::replay::{ItemSpawnSpec, MonsterSpawnSpec, PlayerSpawnConfig};
use crate::types::Position;

/// Map layout specification for a scenario fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioMap {
  /// Open rectangular arena with outer walls.
  Dimensions {
    /// Map grid width.
    width: u32,
    /// Map grid height.
    height: u32,
  },
  /// Full ASCII grid layout specification.
  Ascii(String),
}

/// Declarative fixture describing a reproducible scenario for headless testing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioFixture {
  /// Descriptive name of the scenario.
  pub name: String,
  /// Overview of the scenario objectives or invariants under test.
  pub description: String,
  /// Map layout configuration.
  pub map: ScenarioMap,
  /// Starting coordinate for the player character.
  pub player_start: Position,
  /// Optional custom player stats, starting equipment, or inventory items.
  pub player_config: Option<PlayerSpawnConfig>,
  /// Optional exit down-stairs coordinate.
  pub stairs: Option<Position>,
  /// Hostile monster spawns present at start of scenario.
  pub monsters: Vec<MonsterSpawnSpec>,
  /// Ground items placed in the scenario before first turn.
  pub items: Vec<ItemSpawnSpec>,
  /// Initial seed for deterministic RNG stream.
  pub seed: u64,
  /// Optional maximum turn ceiling for automated agents.
  pub max_turns: Option<u64>,
}

impl ScenarioFixture {
  /// Creates a new scenario fixture with open rectangular bounds.
  #[must_use]
  pub fn new(
    name: impl Into<String>,
    description: impl Into<String>,
    width: u32,
    height: u32,
    player_start: Position,
  ) -> Self {
    Self {
      name: name.into(),
      description: description.into(),
      map: ScenarioMap::Dimensions { width, height },
      player_start,
      player_config: None,
      stairs: None,
      monsters: Vec::new(),
      items: Vec::new(),
      seed: 42,
      max_turns: None,
    }
  }

  /// Creates a scenario fixture with an ASCII map representation.
  #[must_use]
  pub fn with_ascii_map(
    name: impl Into<String>,
    description: impl Into<String>,
    ascii: impl Into<String>,
  ) -> Self {
    Self {
      name: name.into(),
      description: description.into(),
      map: ScenarioMap::Ascii(ascii.into()),
      player_start: Position::new(1, 1),
      player_config: None,
      stairs: None,
      monsters: Vec::new(),
      items: Vec::new(),
      seed: 42,
      max_turns: None,
    }
  }

  /// Sets the player start position.
  #[must_use]
  pub const fn with_player_start(mut self, pos: Position) -> Self {
    self.player_start = pos;
    self
  }

  /// Sets custom player spawn configuration.
  #[must_use]
  pub fn with_player_config(mut self, config: PlayerSpawnConfig) -> Self {
    self.player_config = Some(config);
    self
  }

  /// Sets exit stairs coordinate.
  #[must_use]
  pub const fn with_stairs(mut self, pos: Position) -> Self {
    self.stairs = Some(pos);
    self
  }

  /// Adds a monster spawn to the scenario.
  #[must_use]
  pub fn with_monster(mut self, monster: MonsterSpawnSpec) -> Self {
    self.monsters.push(monster);
    self
  }

  /// Adds a ground item spawn to the scenario.
  #[must_use]
  pub fn with_item(mut self, item: ItemSpawnSpec) -> Self {
    self.items.push(item);
    self
  }

  /// Configures the deterministic RNG seed.
  #[must_use]
  pub const fn with_seed(mut self, seed: u64) -> Self {
    self.seed = seed;
    self
  }

  /// Configures the maximum turn limit.
  #[must_use]
  pub const fn with_max_turns(mut self, max_turns: u64) -> Self {
    self.max_turns = Some(max_turns);
    self
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_scenario_fixture_builder() {
    let fixture = ScenarioFixture::new("Duel", "1v1 melee test", 10, 10, Position::new(2, 2))
      .with_seed(1234)
      .with_max_turns(50)
      .with_stairs(Position::new(8, 8));

    assert_eq!(fixture.name, "Duel");
    assert_eq!(fixture.seed, 1234);
    assert_eq!(fixture.max_turns, Some(50));
    assert_eq!(fixture.stairs, Some(Position::new(8, 8)));
  }
}
