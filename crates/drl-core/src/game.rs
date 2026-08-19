//! High-level game execution kernel and turn progression.

use drl_protocol::{
  ActionCost, AttackOutcome, Command, CommandError, DamageSource, DeathCause, Direction, GameEvent,
  LevelId, OmniscientObservation, PlayerObservation, Position, Turn,
};

use crate::combat::CombatResolver;
use crate::grid::Map;
use crate::rng::GameRng;
use crate::scheduler::{ACTION_THRESHOLD, Scheduler};
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
    let player_id = world.spawn_player(player_start, "Marine")?;

    // Give player initial action threshold energy
    if let Some(player) = world.get_actor_mut(player_id) {
      player.set_energy(ACTION_THRESHOLD);
    }

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

    let player_id = self
      .state
      .world
      .player_id()
      .ok_or_else(|| CommandError::InvalidCommand("no player entity in world".to_string()))?;

    let player = self
      .state
      .world
      .get_actor(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?;

    if !player.is_alive() {
      return Err(CommandError::DeadActorCannotAct(player_id));
    }

    let mut events = Vec::new();
    events.push(GameEvent::TurnStarted {
      turn: self.state.turn,
    });

    // 1. Execute player action
    match command {
      Command::Move(dir) => {
        if dir == Direction::None {
          self.execute_player_wait(player_id, &mut events)?;
        } else {
          let p_pos = self
            .state
            .world
            .get_actor(player_id)
            .ok_or(CommandError::EntityNotFound(player_id))?
            .position();
          let target_pos = p_pos + dir;

          if !self.state.world.map().is_in_bounds(target_pos) {
            return Err(CommandError::OutOfBounds(target_pos));
          }

          // If a living monster is at target_pos -> melee bump-attack!
          if let Some(target_monster) = self.state.world.living_actor_at(target_pos) {
            if !target_monster.is_player() {
              let monster_id = target_monster.id();
              self.execute_melee_attack(player_id, monster_id, &mut events)?;
            } else {
              return Err(CommandError::InvalidTarget(target_pos));
            }
          } else if self.state.world.map().is_walkable(target_pos) {
            // Unoccupied walkable tile -> step move
            self.execute_player_move_to(player_id, p_pos, target_pos, &mut events)?;
          } else {
            return Err(CommandError::BlockedByTerrain(target_pos));
          }
        }
      }
      Command::AttackMelee(dir) => {
        if dir == Direction::None {
          return Err(CommandError::InvalidDirection(dir));
        }
        let p_pos = self
          .state
          .world
          .get_actor(player_id)
          .ok_or(CommandError::EntityNotFound(player_id))?
          .position();
        let target_pos = p_pos + dir;

        if let Some(target_monster) = self.state.world.living_actor_at(target_pos) {
          if !target_monster.is_player() {
            let monster_id = target_monster.id();
            self.execute_melee_attack(player_id, monster_id, &mut events)?;
          } else {
            return Err(CommandError::InvalidTarget(target_pos));
          }
        } else {
          return Err(CommandError::InvalidTarget(target_pos));
        }
      }
      Command::AttackRanged(target_pos) => {
        self.execute_player_ranged_attack(player_id, target_pos, &mut events)?;
      }
      Command::Wait => {
        self.execute_player_wait(player_id, &mut events)?;
      }
    }

    // Spend player energy
    if let Some(player) = self.state.world.get_actor_mut(player_id) {
      player.spend_energy(ActionCost::STANDARD);
      events.push(GameEvent::ActionCostPaid {
        entity_id: player_id,
        cost: ActionCost::STANDARD,
      });
    }

    // 2. Execute Monster AI turns until player is ready to act again
    self.run_scheduled_monster_turns(player_id, &mut events)?;

    events.push(GameEvent::TurnEnded {
      turn: self.state.turn,
    });

    self.state.turn = self.state.turn.next();
    Ok(events)
  }

  /// Moves the player to a verified target position.
  fn execute_player_move_to(
    &mut self,
    player_id: drl_protocol::EntityId,
    from: Position,
    to: Position,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let player = self
      .state
      .world
      .get_actor_mut(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?;
    player.set_position(to);

    events.push(GameEvent::EntityMoved {
      entity_id: player_id,
      from,
      to,
    });
    Ok(())
  }

  /// Executes player wait.
  fn execute_player_wait(
    &mut self,
    player_id: drl_protocol::EntityId,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let pos = self
      .state
      .world
      .get_actor(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?
      .position();

    events.push(GameEvent::EntityWaited {
      entity_id: player_id,
      position: pos,
    });
    Ok(())
  }

  /// Executes a melee attack between attacker and defender.
  fn execute_melee_attack(
    &mut self,
    attacker_id: drl_protocol::EntityId,
    target_id: drl_protocol::EntityId,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let (outcome, is_lethal, damage) = {
      let attacker = self
        .state
        .world
        .get_actor(attacker_id)
        .ok_or(CommandError::EntityNotFound(attacker_id))?;
      let defender = self
        .state
        .world
        .get_actor(target_id)
        .ok_or(CommandError::EntityNotFound(target_id))?;

      let outcome = CombatResolver::resolve_melee_attack(attacker, defender, &mut self.state.rng);
      let (is_lethal, damage) = match outcome {
        AttackOutcome::Hit { damage, is_lethal } => (is_lethal, damage),
        _ => (false, 0),
      };
      (outcome, is_lethal, damage)
    };

    events.push(GameEvent::AttackResolved {
      attacker_id,
      target_id,
      outcome,
      is_ranged: false,
    });

    if damage > 0 {
      let (taken, _, death_cause) =
        self
          .state
          .world
          .apply_damage(target_id, damage, DamageSource::Actor(attacker_id))?;

      let remaining = self
        .state
        .world
        .get_actor(target_id)
        .map_or(0, |a| a.hp().current);

      events.push(GameEvent::DamageApplied {
        target_id,
        amount: taken,
        remaining_hp: remaining,
        source: DamageSource::Actor(attacker_id),
      });

      if is_lethal {
        let cause = death_cause.unwrap_or(DeathCause::MeleeAttack { attacker_id });
        events.push(GameEvent::ActorDied {
          entity_id: target_id,
          cause,
        });

        if self.state.world.player_id() == Some(target_id) {
          self.state.is_game_over = true;
        }
      }
    }

    Ok(())
  }

  /// Executes player ranged attack against a target position.
  fn execute_player_ranged_attack(
    &mut self,
    player_id: drl_protocol::EntityId,
    target_pos: Position,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    if !self.state.world.map().is_in_bounds(target_pos) {
      return Err(CommandError::OutOfBounds(target_pos));
    }

    let target_monster_id = self
      .state
      .world
      .living_actor_at(target_pos)
      .filter(|a| !a.is_player())
      .map(|a| a.id())
      .ok_or(CommandError::InvalidTarget(target_pos))?;

    let p_pos = self
      .state
      .world
      .get_actor(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?
      .position();

    let distance = p_pos.distance_chebyshev(target_pos);

    let (outcome, is_lethal, damage) = {
      let player = self
        .state
        .world
        .get_actor(player_id)
        .ok_or(CommandError::EntityNotFound(player_id))?;

      if distance > player.ranged_range() {
        return Err(CommandError::TargetOutOfRange(target_pos));
      }

      let target_monster = self
        .state
        .world
        .get_actor(target_monster_id)
        .ok_or(CommandError::EntityNotFound(target_monster_id))?;

      let outcome = CombatResolver::resolve_ranged_attack(
        player,
        target_monster,
        distance,
        &mut self.state.rng,
      );
      let (is_lethal, damage) = match outcome {
        AttackOutcome::Hit { damage, is_lethal } => (is_lethal, damage),
        _ => (false, 0),
      };
      (outcome, is_lethal, damage)
    };

    events.push(GameEvent::AttackResolved {
      attacker_id: player_id,
      target_id: target_monster_id,
      outcome,
      is_ranged: true,
    });

    if damage > 0 {
      let (taken, _, _) =
        self
          .state
          .world
          .apply_damage(target_monster_id, damage, DamageSource::Actor(player_id))?;

      let remaining = self
        .state
        .world
        .get_actor(target_monster_id)
        .map_or(0, |a| a.hp().current);

      events.push(GameEvent::DamageApplied {
        target_id: target_monster_id,
        amount: taken,
        remaining_hp: remaining,
        source: DamageSource::Actor(player_id),
      });

      if is_lethal {
        events.push(GameEvent::ActorDied {
          entity_id: target_monster_id,
          cause: DeathCause::RangedAttack {
            attacker_id: player_id,
          },
        });
      }
    }

    Ok(())
  }

  /// Runs scheduled monster turns until the player is ready to act again.
  fn run_scheduled_monster_turns(
    &mut self,
    player_id: drl_protocol::EntityId,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    loop {
      if self.state.is_game_over {
        break;
      }

      // 1. Process any monster that is currently ready (energy >= ACTION_THRESHOLD)
      let ready_monster_id = self
        .state
        .world
        .actors()
        .values()
        .filter(|a| !a.is_player() && a.is_alive() && a.energy() >= ACTION_THRESHOLD)
        .max_by(|a, b| {
          a.energy()
            .cmp(&b.energy())
            .then_with(|| b.id().as_u64().cmp(&a.id().as_u64()))
        })
        .map(|a| a.id());

      if let Some(monster_id) = ready_monster_id {
        self.execute_monster_turn(monster_id, player_id, events)?;
        continue;
      }

      // 2. If no monsters are ready and player is ready, finish turn processing
      if self
        .state
        .world
        .get_actor(player_id)
        .is_some_and(|player| player.energy() >= ACTION_THRESHOLD)
      {
        break;
      }

      // 3. Otherwise, advance discrete time ticks until at least one actor is ready
      if Scheduler::advance_until_ready(&mut self.state.world).is_none() {
        break;
      }
    }

    Ok(())
  }

  /// Executes a single monster AI turn step.
  fn execute_monster_turn(
    &mut self,
    monster_id: drl_protocol::EntityId,
    player_id: drl_protocol::EntityId,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let Some(monster) = self.state.world.get_actor(monster_id) else {
      return Ok(());
    };
    if !monster.is_alive() {
      return Ok(());
    }

    let Some(player) = self.state.world.get_actor(player_id) else {
      return Ok(());
    };
    if !player.is_alive() {
      return Ok(());
    }

    let m_pos = monster.position();
    let p_pos = player.position();
    let dist = m_pos.distance_chebyshev(p_pos);

    if dist == 1 {
      // Adjacent to player -> melee attack player!
      self.execute_melee_attack(monster_id, player_id, events)?;
    } else {
      // Step towards player by choosing best walkable direction (preferring straight lines)
      let best_dir = Direction::ALL_8WAY
        .into_iter()
        .filter(|&dir| {
          let target = m_pos + dir;
          self.state.world.map().is_in_bounds(target) && !self.state.world.is_cell_blocked(target)
        })
        .min_by_key(|&dir| {
          let target = m_pos + dir;
          (
            target.distance_chebyshev(p_pos),
            target.distance_squared(p_pos),
          )
        });

      if let Some(dir) = best_dir {
        let new_pos = m_pos + dir;
        if let Some(m) = self.state.world.get_actor_mut(monster_id) {
          m.set_position(new_pos);
        }
        events.push(GameEvent::EntityMoved {
          entity_id: monster_id,
          from: m_pos,
          to: new_pos,
        });
      } else {
        events.push(GameEvent::EntityWaited {
          entity_id: monster_id,
          position: m_pos,
        });
      }
    }

    // Deduct action cost from monster
    if let Some(m) = self.state.world.get_actor_mut(monster_id) {
      m.spend_energy(ActionCost::STANDARD);
      events.push(GameEvent::ActionCostPaid {
        entity_id: monster_id,
        cost: ActionCost::STANDARD,
      });
    }

    Ok(())
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

  #[test]
  fn test_game_step_melee_bump_attack() {
    let mut game = Game::new(100, 10, 10, Position::new(2, 2)).unwrap();
    let m_id = game
      .world_mut()
      .spawn_monster(Position::new(3, 2), "Former Human", 20, 100, (2, 4))
      .unwrap();

    // Step East into monster -> triggers bump melee attack!
    let events = game.step(Command::Move(Direction::East)).unwrap();
    assert!(
      events
        .iter()
        .any(|e| matches!(e, GameEvent::AttackResolved { target_id, .. } if *target_id == m_id))
    );
    // Player remains at (2, 2) during attack
    assert_eq!(
      game.world().player().unwrap().position(),
      Position::new(2, 2)
    );
  }

  #[test]
  fn test_game_step_ranged_attack() {
    let mut game = Game::new(200, 10, 10, Position::new(2, 2)).unwrap();
    let m_id = game
      .world_mut()
      .spawn_monster(Position::new(5, 2), "Imp", 20, 100, (2, 4))
      .unwrap();

    // Ranged attack target at (5, 2)
    let events = game
      .step(Command::AttackRanged(Position::new(5, 2)))
      .unwrap();
    assert!(events.iter().any(|e| matches!(
      e,
      GameEvent::AttackResolved {
        target_id,
        is_ranged: true,
        ..
      } if *target_id == m_id
    )));
  }
}
