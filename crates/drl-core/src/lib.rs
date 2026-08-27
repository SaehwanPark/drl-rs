//! Deterministic headless simulation core for DRL-Rust.
//!
//! This crate contains pure game simulation logic, world models, grid representation,
//! deterministic RNG, and command execution.
//! It remains strictly independent of rendering, audio, OS APIs, filesystem IO,
//! and MCP transports.

pub mod acid_spitter;
pub mod actor;
pub mod agent;
pub mod ai;
pub mod assault_shotgun;
pub mod batch;
pub mod behavior;
pub mod combat;
pub mod combat_shotgun;
pub mod content_validation;
pub mod environment;
pub mod fov;
pub mod game;
pub mod generator;
pub mod grammaton;
pub mod grid;
pub mod inventory;
pub mod item;
pub mod item_definition;
pub mod jackhammer;
pub mod level_definition;
pub mod loot_definition;
pub mod missile_launcher;
pub mod monster_roll_definition;
pub mod nuke;
pub mod null_pointer;
pub mod pump_action;
pub mod replay;
pub mod rng;
pub mod scenario;
pub mod scheduler;
pub mod special_level_definition;
pub mod subtle_knife;
pub mod targeting;
pub mod trigun;
pub mod world;

pub use actor::Actor;
pub use agent::{AgentPolicy, ExplorerBot, GreedyCombatBot, RandomBot};
pub use ai::{MonsterAction, MonsterAi};
pub use assault_shotgun::ASSAULT_SHOTGUN_ALT_RELOAD_CAP;
pub use batch::{
  BatchRunner, CohortComparison, CohortConfig, CohortDepthBucket, CohortDepthDistribution,
  CohortOutcomeComparison, CohortOutcomeDistribution, CohortOutcomeTolerances, CohortReport,
  CohortReportError, CohortTolerances, EpisodeRecord,
};
pub use behavior::{
  BLASTER_RECHARGE_AMOUNT, BLASTER_RECHARGE_DELAY, BLASTER_RECHARGE_TICK, MEDICAL_REPAIR_INTERVAL,
  MEDICAL_REPAIR_MIN_DURABILITY_EXCLUSIVE, MEDICAL_REPAIR_TIMER_AFTER_REPAIR, MedicalRepairOutcome,
  MedicalRepairState, NUCLEAR_BFG_RECHARGE_AMOUNT, NUCLEAR_BFG_RECHARGE_DELAY,
  NUCLEAR_BFG_RECHARGE_TICK, NUCLEAR_PLASMA_RECHARGE_AMOUNT, NUCLEAR_PLASMA_RECHARGE_DELAY,
  NUCLEAR_PLASMA_RECHARGE_TICK, WeaponRechargeOutcome, WeaponRechargePolicy, WeaponRechargeState,
};
pub use combat::CombatResolver;
pub use combat_shotgun::COMBAT_SHOTGUN_ALT_RELOAD_CAP;
pub use content_validation::{ContentValidationError, validate_current_content};
pub use environment::{
  FLUID_MOVEMENT_COST, HazardDamage, MUD_MOVEMENT_COST, entered_tile_damage, movement_cost,
};
pub use fov::{DEFAULT_VISION_RADIUS, compute_fov, has_line_of_sight, line_points};
pub use game::{Game, GameState};
pub use generator::{GeneratedLevel, LevelGenerator, LevelGeneratorConfig, MonsterSpawn, Room};
pub use grammaton::{GRAMMATON_MODE_SCORE_COST, GrammatonTransition};
pub use grid::{Map, Tile};
pub use inventory::{DEFAULT_INVENTORY_CAPACITY, Equipment, Inventory};
pub use item::Item;
pub use jackhammer::{JACKHAMMER_MODE_SCORE_COST, JackhammerTransition};
pub use level_definition::{LEVEL_DEFINITIONS, LevelDefinition, standard_procedural};
pub use missile_launcher::MISSILE_LAUNCHER_ALT_RELOAD_CAP;
pub use nuke::{NukeError, NukeState};
pub use pump_action::PUMP_ACTION_COST;
pub use replay::ReplayEngine;
pub use rng::GameRng;
pub use scenario::{Scenario, ScenarioRunner};
pub use scheduler::{ACTION_THRESHOLD, Scheduler};
pub use special_level_definition::{
  SPECIAL_LEVEL_DEFINITIONS, SpecialLevelDefinition, at_legacy_depth, by_id,
};
pub use subtle_knife::{
  SUBTLE_KNIFE_HP_COST, SUBTLE_KNIFE_SCORE_COST, SUBTLE_KNIFE_TARGET_DAMAGE, SubtleKnifeCost,
  SubtleKnifeError, SubtleKnifeTransition, TiredStatus,
};
pub use targeting::TargetingSystem;
pub use trigun::{
  TRIGUN_HP_COST, TRIGUN_MAX_HP_COST, TRIGUN_MIN_HP, TRIGUN_MIN_MAX_HP, TRIGUN_NUKE_TIMER,
  TRIGUN_SCORE_COST, TrigunCost, TrigunError, TrigunTransition,
};
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
