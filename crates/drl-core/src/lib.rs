//! Deterministic headless simulation core for DRL-Rust.
//!
//! This crate contains pure game simulation logic, world models, grid representation,
//! deterministic RNG, and command execution.
//! It remains strictly independent of rendering, audio, OS APIs, filesystem IO,
//! and MCP transports.

pub mod actor;
pub mod game;
pub mod grid;
pub mod replay;
pub mod rng;
pub mod world;

pub use actor::Actor;
pub use game::{Game, GameState};
pub use grid::{Map, Tile};
pub use replay::ReplayEngine;
pub use rng::GameRng;
pub use world::World;

/// Returns the core simulation engine name.
#[must_use]
pub fn engine_name() -> &'static str {
  "drl-core"
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_engine_name() {
    assert_eq!(engine_name(), "drl-core");
  }
}
