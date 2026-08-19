//! Action cost and energy-based actor turn scheduling.
//!
//! Implements discrete energy accumulation where actors act when reaching
//! an energy threshold (`ACTION_THRESHOLD = 1000`), breaking ties deterministically
//! by `EntityId`.

use drl_protocol::EntityId;

use crate::world::World;

/// Energy threshold required to execute a standard action.
pub const ACTION_THRESHOLD: i32 = 1000;

/// Action scheduler for managing actor turns according to relative speeds.
pub struct Scheduler;

impl Scheduler {
  /// Finds the currently ready living actor with the highest energy above the action threshold.
  ///
  /// Breaks ties deterministically by lowest `EntityId`.
  #[must_use]
  pub fn find_ready_actor(world: &World) -> Option<EntityId> {
    world
      .actors()
      .values()
      .filter(|actor| actor.is_alive() && actor.energy() >= ACTION_THRESHOLD)
      .max_by(|a, b| {
        a.energy()
          .cmp(&b.energy())
          .then_with(|| b.id().as_u64().cmp(&a.id().as_u64()))
      })
      .map(|actor| actor.id())
  }

  /// Advances energy ticks across all living actors until at least one actor reaches the
  /// action threshold.
  ///
  /// Returns the `EntityId` of the ready actor, or `None` if no living actors exist.
  pub fn advance_until_ready(world: &mut World) -> Option<EntityId> {
    // Check if an actor is already ready
    if let Some(id) = Self::find_ready_actor(world) {
      return Some(id);
    }

    // Accumulate energy in discrete ticks
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 100_000;

    while iterations < MAX_ITERATIONS {
      iterations += 1;
      let mut any_alive = false;

      for actor in world.actors_mut().values_mut() {
        if actor.is_alive() {
          any_alive = true;
          actor.add_energy(actor.speed().as_u32() as i32);
        }
      }

      if !any_alive {
        return None;
      }

      if let Some(ready_id) = Self::find_ready_actor(world) {
        return Some(ready_id);
      }
    }

    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::grid::Map;
  use drl_protocol::{LevelId, Position};

  #[test]
  fn test_scheduler_speed_differences() {
    let map = Map::simple_arena(10, 10);
    let mut world = World::new(LevelId::new(1), map);

    // Player (speed 100)
    let p_id = world.spawn_player(Position::new(1, 1), "Marine").unwrap();
    // Fast monster (speed 200)
    let m_id = world
      .spawn_monster(Position::new(2, 2), "Fast Demon", 30, 200, (3, 5))
      .unwrap();

    // Initial energy is 0 for both. Fast monster (speed 200) reaches 1000 in 5 ticks (5 * 200 = 1000).
    // Player (speed 100) will be at 500.
    let first = Scheduler::advance_until_ready(&mut world).unwrap();
    assert_eq!(first, m_id);

    let fast_actor = world.get_actor_mut(m_id).unwrap();
    assert_eq!(fast_actor.energy(), 1000);
    fast_actor.spend_energy(drl_protocol::ActionCost::STANDARD);
    assert_eq!(fast_actor.energy(), 0);

    let player = world.get_actor(p_id).unwrap();
    assert_eq!(player.energy(), 500);

    // Advance again: fast monster reaches 1000 in 5 more ticks (5 * 200 = 1000).
    // Player will be at 500 + 500 = 1000.
    // Both are at 1000! Tie breaker chooses smallest EntityId (player p_id = 1).
    let second = Scheduler::advance_until_ready(&mut world).unwrap();
    assert_eq!(second, p_id);
  }

  #[test]
  fn test_scheduler_ignores_dead_actors() {
    let map = Map::simple_arena(10, 10);
    let mut world = World::new(LevelId::new(1), map);

    let p_id = world.spawn_player(Position::new(1, 1), "Marine").unwrap();
    let m_id = world
      .spawn_monster(Position::new(2, 2), "Zombie", 10, 100, (1, 2))
      .unwrap();

    // Kill monster
    world.get_actor_mut(m_id).unwrap().take_damage(100);

    let ready = Scheduler::advance_until_ready(&mut world).unwrap();
    assert_eq!(ready, p_id);
  }
}
