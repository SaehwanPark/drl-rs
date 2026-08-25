//! High-level game execution kernel and turn progression.

use drl_protocol::{
  ActionCost, AttackOutcome, Command, CommandError, DamageSource, DeathCause, Direction, GameEvent,
  LevelId, OmniscientObservation, PlayerObservation, Position, TileKind, Turn,
};

use crate::acid_spitter::{ACID_SPITTER_RELOAD_AMOUNT, AcidSpitterReloadError};
use crate::behavior::{LavaRechargeOutcome, MedicalRepairOutcome};
use crate::combat::CombatResolver;
use crate::environment::{entered_tile_damage, movement_cost};
use crate::fov::DEFAULT_VISION_RADIUS;
use crate::generator::{LevelGenerator, LevelGeneratorConfig};
use crate::grammaton::{GRAMMATON_MODE_SCORE_COST, GrammatonTransition};
use crate::grid::{Map, Tile};
use crate::item::Item;
use crate::jackhammer::{JACKHAMMER_MODE_SCORE_COST, JackhammerTransition};
use crate::level_definition::standard_procedural;
use crate::nuke::NukeState;
use crate::null_pointer::{
  NULL_POINTER_EXPLOSION_DAMAGE, NULL_POINTER_EXPLOSION_DELAY, NULL_POINTER_EXPLOSION_RADIUS,
  NullPointerHitTransition,
};
use crate::rng::GameRng;
use crate::scheduler::{ACTION_THRESHOLD, Scheduler};
use crate::subtle_knife::{SUBTLE_KNIFE_TARGET_DAMAGE, SubtleKnifeError};
use crate::targeting::TargetingSystem;
use crate::trigun::{TRIGUN_NUKE_TIMER, TrigunError};
use crate::world::World;

/// Complete snapshot of the simulation state at a specific turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
  pub turn: Turn,
  pub world: World,
  pub rng: GameRng,
  pub is_game_over: bool,
  pub nuke: NukeState,
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
      nuke: NukeState::new(),
    };

    Ok(Self { state })
  }

  /// Initializes a simple arena game with the player at the center.
  pub fn new_arena(seed: u64, width: u32, height: u32) -> Result<Self, CommandError> {
    let start_x = (width / 2) as i32;
    let start_y = (height / 2) as i32;
    Self::new(seed, width, height, Position::new(start_x, start_y))
  }

  /// Initializes a new game instance with a procedurally generated dungeon level.
  pub fn new_procedural(seed: u64, config: LevelGeneratorConfig) -> Result<Self, CommandError> {
    let mut rng = GameRng::from_seed(seed);
    let mut item_counter = 0;
    let generated = LevelGenerator::generate(&config, &mut rng, &mut item_counter);

    let mut world = World::from_generated_level(LevelId::new(1), generated, None);

    let player_id = world
      .player_id()
      .ok_or_else(|| CommandError::InvalidCommand("no player spawned in level".to_string()))?;

    if let Some(player) = world.get_actor_mut(player_id) {
      player.set_energy(ACTION_THRESHOLD);
    }

    let state = GameState {
      turn: Turn::zero(),
      world,
      rng,
      is_game_over: false,
      nuke: NukeState::new(),
    };

    Ok(Self { state })
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

  /// Returns the typed level-nuke state.
  #[must_use]
  pub const fn nuke_state(&self) -> NukeState {
    self.state.nuke
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
  ///
  /// Rejected commands are transactional: the complete pre-command state is
  /// restored, including turn, world, and RNG state. Individual handlers still
  /// validate before commit where practical; this bounded rollback guard is the
  /// interim backstop for later fallible substeps.
  pub fn step(&mut self, command: Command) -> Result<Vec<GameEvent>, CommandError> {
    let before = self.state.clone();
    match self.step_inner(command) {
      Ok(events) => Ok(events),
      Err(error) => {
        self.state = before;
        Err(error)
      }
    }
  }

  fn step_inner(&mut self, command: Command) -> Result<Vec<GameEvent>, CommandError> {
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

    let mut action_cost = ActionCost::STANDARD;

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
            action_cost = self.execute_player_move_to(player_id, p_pos, target_pos, &mut events)?;
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
        action_cost = self.execute_player_ranged_attack(player_id, target_pos, &mut events)?;
      }
      Command::Wait => {
        self.execute_player_wait(player_id, &mut events)?;
      }
      Command::Pickup => {
        self.execute_player_pickup(player_id, &mut events)?;
      }
      Command::Drop(item_id) => {
        self.execute_player_drop(player_id, item_id, &mut events)?;
      }
      Command::Equip(item_id) => {
        self.execute_player_equip(player_id, item_id, &mut events)?;
      }
      Command::Unequip(slot) => {
        self.execute_player_unequip(player_id, slot, &mut events)?;
      }
      Command::Use(item_id) => {
        self.execute_player_use(player_id, item_id, &mut events)?;
      }
      Command::Invoke(item_id) => {
        self.execute_player_invoke(player_id, item_id, &mut events)?;
      }
      Command::AltReload { item_id, confirmed } => {
        self.execute_player_alt_reload(player_id, item_id, confirmed, &mut events)?;
      }
      Command::Reload => {
        action_cost = self.execute_player_reload(player_id, &mut events)?;
      }
      Command::Descend => {
        action_cost = self.execute_player_descend(player_id, &mut events)?;
      }
    }

    if !self.state.is_game_over {
      self.tick_player_medical_powerarmor(player_id, &mut events)?;
      self.tick_player_lava_armor(player_id, &mut events)?;
    }

    // Spend player energy
    if let Some(player) = self.state.world.get_actor_mut(player_id) {
      player.spend_energy(action_cost);
      events.push(GameEvent::ActionCostPaid {
        entity_id: player_id,
        cost: action_cost,
      });
    }

    if !self.state.is_game_over {
      self.tick_nuke(player_id, &mut events)?;
    }

    // 2. Execute Monster AI turns until player is ready to act again
    self.run_scheduled_monster_turns(player_id, &mut events)?;

    events.push(GameEvent::TurnEnded {
      turn: self.state.turn,
    });

    self.state.turn = self.state.turn.next();
    Ok(events)
  }

  /// Executes the typed Subtle Knife alternate invoke action.
  fn execute_player_invoke(
    &mut self,
    player_id: drl_protocol::EntityId,
    item_id: drl_protocol::ItemId,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let player = self
      .state
      .world
      .get_actor(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?;
    let weapon = player
      .equipment()
      .weapon()
      .filter(|item| item.id() == item_id)
      .filter(|item| item.archetype() == drl_protocol::ItemArchetype::SubtleKnife)
      .ok_or(CommandError::CannotInvoke(item_id))?;
    let _ = weapon;
    let player_position = player.position();
    let mut target_ids: Vec<_> = TargetingSystem::find_visible_targets(
      &self.state.world,
      player_position,
      DEFAULT_VISION_RADIUS,
    )
    .into_iter()
    .map(|(target_id, _, _)| target_id)
    .collect();
    target_ids.sort_unstable();

    // Validate every possible death-drop destination before paying the invoke
    // cost or applying damage. The drop is committed only after the target's
    // death, so this preflight keeps a late world error outside the mutation
    // boundary instead of relying solely on the command rollback guard.
    for target_id in &target_ids {
      let Some(target) = self.state.world.get_actor(*target_id) else {
        continue;
      };
      if target.death_drop().is_some() {
        self.validate_death_drop_position(target.position())?;
      }
    }

    let cost = self
      .state
      .world
      .get_actor_mut(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?
      .invoke_subtle_knife()
      .map_err(|SubtleKnifeError::Tired| CommandError::CannotInvoke(item_id))?;

    events.push(GameEvent::SubtleKnifeInvoked {
      entity_id: player_id,
      item_id,
      targets: target_ids.clone(),
      remaining_hp: cost.remaining_hp,
      score_count_remaining: cost.score_count_remaining,
    });

    for target_id in target_ids {
      let (taken, is_lethal, death_cause) = self.state.world.apply_internal_damage(
        target_id,
        SUBTLE_KNIFE_TARGET_DAMAGE,
        DamageSource::Actor(player_id),
      )?;
      let remaining_hp = self
        .state
        .world
        .get_actor(target_id)
        .map_or(0, |actor| actor.hp().current);
      events.push(GameEvent::DamageApplied {
        target_id,
        amount: taken,
        remaining_hp,
        source: DamageSource::Actor(player_id),
        damage_type: None,
      });

      if is_lethal {
        let death_drop = self
          .state
          .world
          .get_actor(target_id)
          .and_then(|actor| actor.death_drop());
        let drop_pos = self
          .state
          .world
          .get_actor(target_id)
          .map(|actor| actor.position());
        events.push(GameEvent::ActorDied {
          entity_id: target_id,
          cause: death_cause.unwrap_or(DeathCause::MeleeAttack {
            attacker_id: player_id,
          }),
        });
        if let (Some(drop_kind), Some(position)) = (death_drop, drop_pos) {
          self.spawn_death_drop(target_id, position, drop_kind, events)?;
        }
      }
    }
    Ok(())
  }

  /// Executes the typed Trigun alternate reload and schedules its level nuke.
  fn execute_player_alt_reload(
    &mut self,
    player_id: drl_protocol::EntityId,
    item_id: drl_protocol::ItemId,
    confirmed: bool,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let player = self
      .state
      .world
      .get_actor(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?;
    let weapon = player
      .equipment()
      .weapon()
      .filter(|item| item.id() == item_id)
      .ok_or(CommandError::CannotAltReload(item_id))?;
    match weapon.archetype() {
      drl_protocol::ItemArchetype::GrammatonBeretta => {
        let mode = self
          .state
          .world
          .get_actor_mut(player_id)
          .and_then(|actor| actor.equipment_mut().weapon_mut())
          .and_then(|item| item.weapon_properties_mut())
          .map(GrammatonTransition::cycle)
          .ok_or(CommandError::CannotAltReload(item_id))?;
        let score_count_remaining = self
          .state
          .world
          .get_actor_mut(player_id)
          .ok_or(CommandError::EntityNotFound(player_id))?
          .spend_score_count(GRAMMATON_MODE_SCORE_COST);
        events.push(GameEvent::GrammatonFireModeChanged {
          entity_id: player_id,
          item_id,
          mode,
          score_count_remaining,
        });
        Ok(())
      }
      drl_protocol::ItemArchetype::Jackhammer => {
        let mode = self
          .state
          .world
          .get_actor_mut(player_id)
          .and_then(|actor| actor.equipment_mut().weapon_mut())
          .and_then(|item| item.weapon_properties_mut())
          .map(JackhammerTransition::cycle)
          .ok_or(CommandError::CannotAltReload(item_id))?;
        let score_count_remaining = self
          .state
          .world
          .get_actor_mut(player_id)
          .ok_or(CommandError::EntityNotFound(player_id))?
          .spend_score_count(JACKHAMMER_MODE_SCORE_COST);
        events.push(GameEvent::JackhammerFireModeChanged {
          entity_id: player_id,
          item_id,
          mode,
          score_count_remaining,
        });
        Ok(())
      }
      drl_protocol::ItemArchetype::Trigun => {
        if !confirmed {
          return Err(CommandError::AltReloadNotConfirmed(item_id));
        }
        let cost = self
          .state
          .world
          .get_actor_mut(player_id)
          .ok_or(CommandError::EntityNotFound(player_id))?
          .alt_reload_trigun()
          .map_err(|error| match error {
            TrigunError::MaximumHealthTooLow | TrigunError::Dead => {
              CommandError::CannotAltReload(item_id)
            }
          })?;

        self
          .state
          .nuke
          .activate(TRIGUN_NUKE_TIMER)
          .map_err(|_| CommandError::CannotAltReload(item_id))?;
        events.push(GameEvent::TrigunAltReloaded {
          entity_id: player_id,
          item_id,
          remaining_hp: cost.remaining_hp,
          score_count_remaining: cost.score_count_remaining,
        });
        events.push(GameEvent::NukeActivated {
          level_id: self.state.world.level_id(),
          countdown: TRIGUN_NUKE_TIMER,
        });
        Ok(())
      }
      _ => Err(CommandError::CannotAltReload(item_id)),
    }
  }

  /// Resolves a pending nuke at the accepted command boundary.
  fn tick_nuke(
    &mut self,
    player_id: drl_protocol::EntityId,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    if !self.state.nuke.tick() {
      return Ok(());
    }

    let level_id = self.state.world.level_id();
    events.push(GameEvent::LevelNuked { level_id });
    let (taken, lethal, _) =
      self
        .state
        .world
        .apply_internal_damage(player_id, 6_000, DamageSource::Environment)?;
    let remaining_hp = self
      .state
      .world
      .get_actor(player_id)
      .map_or(0, |actor| actor.hp().current);
    events.push(GameEvent::DamageApplied {
      target_id: player_id,
      amount: taken,
      remaining_hp,
      source: DamageSource::Environment,
      damage_type: None,
    });
    if lethal {
      events.push(GameEvent::ActorDied {
        entity_id: player_id,
        cause: DeathCause::Environment,
      });
      self.state.is_game_over = true;
    }
    Ok(())
  }

  /// Advances explicit periodic behavior after an accepted player command.
  fn tick_player_medical_powerarmor(
    &mut self,
    player_id: drl_protocol::EntityId,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let player = self
      .state
      .world
      .get_actor_mut(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?;
    let Some((item_id, outcome)) = player.tick_medical_powerarmor() else {
      return Ok(());
    };

    if let MedicalRepairOutcome::Repaired { healed, timer, .. } = outcome {
      let remaining_hp = player.hp().current;
      let durability_remaining = player
        .equipment()
        .armor()
        .and_then(Item::armor_properties)
        .map_or(0, |properties| properties.durability);
      events.push(GameEvent::MedicalPowerarmorRepaired {
        entity_id: player_id,
        item_id,
        healed,
        remaining_hp,
        durability_remaining,
        timer,
      });
    }
    Ok(())
  }

  /// Advances Lava Armor after an accepted command using the owner's tile.
  fn tick_player_lava_armor(
    &mut self,
    player_id: drl_protocol::EntityId,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let position = self
      .state
      .world
      .get_actor(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?
      .position();
    let on_lava = self
      .state
      .world
      .map()
      .get_tile(position)
      .is_some_and(|tile| tile == crate::grid::Tile::Lava);
    let player = self
      .state
      .world
      .get_actor_mut(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?;
    let Some((item_id, outcome)) = player.tick_lava_armor(on_lava) else {
      return Ok(());
    };

    if let LavaRechargeOutcome::Recharged {
      durability_restored,
      timer,
    } = outcome
    {
      let durability_remaining = player
        .equipment()
        .armor()
        .and_then(Item::armor_properties)
        .map_or(0, |properties| properties.durability);
      events.push(GameEvent::LavaArmorRecharged {
        entity_id: player_id,
        item_id,
        durability_restored,
        durability_remaining,
        timer,
      });
    }
    Ok(())
  }

  /// Moves the player to a verified target position.
  fn execute_player_move_to(
    &mut self,
    player_id: drl_protocol::EntityId,
    from: Position,
    to: Position,
    events: &mut Vec<GameEvent>,
  ) -> Result<ActionCost, CommandError> {
    let target_tile = self
      .state
      .world
      .map()
      .get_tile(to)
      .ok_or(CommandError::OutOfBounds(to))?;
    let player = self
      .state
      .world
      .get_actor_mut(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?;
    player.set_position(to);

    self.state.world.update_visibility();

    events.push(GameEvent::EntityMoved {
      entity_id: player_id,
      from,
      to,
    });

    self.apply_player_hazard_contact(player_id, to, events)?;
    Ok(movement_cost(target_tile.to_kind()))
  }

  /// Applies the bounded entered-cell hazard policy after a successful move.
  fn apply_player_hazard_contact(
    &mut self,
    player_id: drl_protocol::EntityId,
    position: Position,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let Some(tile) = self.state.world.map().get_tile(position) else {
      return Err(CommandError::OutOfBounds(position));
    };
    let Some(hazard) = entered_tile_damage(tile.to_kind()) else {
      return Ok(());
    };

    let (taken, lethal, death_cause) = self.state.world.apply_internal_damage(
      player_id,
      hazard.amount,
      DamageSource::Environment,
    )?;
    let remaining_hp = self
      .state
      .world
      .get_actor(player_id)
      .map_or(0, |actor| actor.hp().current);
    events.push(GameEvent::DamageApplied {
      target_id: player_id,
      amount: taken,
      remaining_hp,
      source: DamageSource::Environment,
      damage_type: Some(hazard.damage_type),
    });

    if lethal {
      events.push(GameEvent::ActorDied {
        entity_id: player_id,
        cause: death_cause.unwrap_or(DeathCause::Environment),
      });
      self.state.is_game_over = true;
    }
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

  /// Executes player item pickup from the ground.
  fn execute_player_pickup(
    &mut self,
    player_id: drl_protocol::EntityId,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let p_pos = self
      .state
      .world
      .get_actor(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?
      .position();

    if !self.state.world.map().is_in_bounds(p_pos) {
      return Err(CommandError::OutOfBounds(p_pos));
    }

    let item = self.state.world.pickup_ground_item(p_pos)?;
    let item_id = item.id();
    let item_name = item.name().to_string();

    let player = self
      .state
      .world
      .get_actor_mut(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?;

    if let Err(err) = player.inventory_mut().add_item(item.clone()) {
      let _ = self.state.world.drop_item_to_ground(p_pos, item);
      return Err(err);
    }

    events.push(GameEvent::ItemPickedUp {
      entity_id: player_id,
      item_id,
      item_name,
    });
    Ok(())
  }

  /// Executes player item drop to the ground.
  fn execute_player_drop(
    &mut self,
    player_id: drl_protocol::EntityId,
    item_id: drl_protocol::ItemId,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let p_pos = self
      .state
      .world
      .get_actor(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?
      .position();

    if !self.state.world.map().is_in_bounds(p_pos) {
      return Err(CommandError::OutOfBounds(p_pos));
    }

    let player = self
      .state
      .world
      .get_actor_mut(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?;

    let item = player.inventory_mut().remove_item(item_id)?;
    let item_name = item.name().to_string();

    self.state.world.drop_item_to_ground(p_pos, item)?;

    events.push(GameEvent::ItemDropped {
      entity_id: player_id,
      item_id,
      item_name,
      position: p_pos,
    });
    Ok(())
  }

  /// Executes equipping an item from inventory into its slot.
  fn execute_player_equip(
    &mut self,
    player_id: drl_protocol::EntityId,
    item_id: drl_protocol::ItemId,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let slot = self
      .state
      .world
      .get_actor(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?
      .inventory()
      .get_item(item_id)
      .ok_or(CommandError::ItemNotFound(item_id))?
      .equipment_slot()
      .ok_or(CommandError::CannotEquip(item_id))?;

    let player = self
      .state
      .world
      .get_actor_mut(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?;

    let item = player.inventory_mut().remove_item(item_id)?;
    let prev = player
      .equipment_mut()
      .equip(slot, item)
      .expect("equipment slot was validated before inventory mutation");

    if let Some(old_item) = prev {
      player
        .inventory_mut()
        .add_item(old_item)
        .expect("equipping frees an inventory slot for the replaced item");
    }

    events.push(GameEvent::ItemEquipped {
      entity_id: player_id,
      item_id,
      slot,
    });
    Ok(())
  }

  /// Executes unequipping an item from a slot into inventory.
  fn execute_player_unequip(
    &mut self,
    player_id: drl_protocol::EntityId,
    slot: drl_protocol::EquipmentSlot,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let player = self
      .state
      .world
      .get_actor_mut(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?;

    if player.inventory().is_full() {
      return Err(CommandError::InventoryFull);
    }

    let item = player.equipment_mut().unequip(slot)?;
    let item_id = item.id();
    player.inventory_mut().add_item(item)?;

    events.push(GameEvent::ItemUnequipped {
      entity_id: player_id,
      item_id,
      slot,
    });
    Ok(())
  }

  /// Executes using a consumable item from inventory.
  fn execute_player_use(
    &mut self,
    player_id: drl_protocol::EntityId,
    item_id: drl_protocol::ItemId,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let player = self
      .state
      .world
      .get_actor(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?;

    let item = player
      .inventory()
      .get_item(item_id)
      .ok_or(CommandError::ItemNotFound(item_id))?;

    let item_name = item.name().to_string();

    if item.is_phase_device() {
      let old_pos = player.position();
      let new_pos = self
        .state
        .world
        .find_random_walkable_unoccupied_cell(&mut self.state.rng)
        .ok_or_else(|| {
          CommandError::InvalidCommand("no valid teleport destination available".to_string())
        })?;

      let player_mut = self.state.world.get_actor_mut(player_id).unwrap();
      player_mut.inventory_mut().remove_item(item_id)?;
      player_mut.set_position(new_pos);
      self.state.world.update_visibility();

      events.push(GameEvent::PlayerTeleported {
        from: old_pos,
        to: new_pos,
      });
      events.push(GameEvent::ItemUsed {
        entity_id: player_id,
        item_id,
        item_name,
      });
      Ok(())
    } else if let Some(props) = item.consumable_properties() {
      let heal_amount = props.heal_amount;
      let player_mut = self.state.world.get_actor_mut(player_id).unwrap();
      player_mut.heal(heal_amount);
      player_mut.inventory_mut().remove_item(item_id)?;

      events.push(GameEvent::ItemUsed {
        entity_id: player_id,
        item_id,
        item_name,
      });
      Ok(())
    } else {
      Err(CommandError::CannotUse(item_id))
    }
  }

  /// Executes reloading the player's equipped weapon.
  fn execute_player_reload(
    &mut self,
    player_id: drl_protocol::EntityId,
    events: &mut Vec<GameEvent>,
  ) -> Result<ActionCost, CommandError> {
    let acid_spitter_item_id = self
      .state
      .world
      .get_actor(player_id)
      .and_then(|player| player.equipment().weapon())
      .filter(|weapon| weapon.archetype() == drl_protocol::ItemArchetype::AcidSpitter)
      .map(Item::id);
    if let Some(item_id) = acid_spitter_item_id {
      return self.execute_acid_spitter_reload(player_id, item_id, events);
    }

    let player = self
      .state
      .world
      .get_actor_mut(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?;

    let weapon = player
      .equipment_mut()
      .weapon_mut()
      .ok_or(CommandError::NoEquippedWeapon)?;

    let (ammo_type, needed, reload_cost) = {
      let Some(props) = weapon.weapon_properties() else {
        return Err(CommandError::NoEquippedWeapon);
      };
      if !props.is_ranged {
        return Err(CommandError::NoEquippedWeapon);
      }
      let needed = props.clip_capacity.saturating_sub(props.current_clip);
      if needed == 0 {
        return Err(CommandError::ClipAlreadyFull);
      }
      let ammo_type = props.ammo_type.ok_or(CommandError::NoMatchingAmmo)?;
      (ammo_type, needed, props.reload_cost)
    };

    let taken = player.inventory_mut().take_ammo(ammo_type, needed);
    if taken == 0 {
      return Err(CommandError::NoMatchingAmmo);
    }

    let weapon = player.equipment_mut().weapon_mut().unwrap();
    weapon.load_ammo_into_clip(taken);

    let (current_clip, max_clip) = {
      let props = weapon.weapon_properties().unwrap();
      (props.current_clip, props.clip_capacity)
    };

    events.push(GameEvent::WeaponReloaded {
      entity_id: player_id,
      ammo_loaded: taken,
      current_clip,
      max_clip,
    });

    Ok(reload_cost)
  }

  /// Executes Acid Spitter's terrain-fed reload callback as a typed transition.
  fn execute_acid_spitter_reload(
    &mut self,
    player_id: drl_protocol::EntityId,
    item_id: drl_protocol::ItemId,
    events: &mut Vec<GameEvent>,
  ) -> Result<ActionCost, CommandError> {
    let (position, current_clip, clip_capacity, reload_cost, tile_kind) = {
      let player = self
        .state
        .world
        .get_actor(player_id)
        .ok_or(CommandError::EntityNotFound(player_id))?;
      let weapon = player
        .equipment()
        .weapon()
        .filter(|item| item.id() == item_id)
        .ok_or(CommandError::NoEquippedWeapon)?;
      let properties = weapon
        .weapon_properties()
        .ok_or(CommandError::NoEquippedWeapon)?;
      let tile_kind = self
        .state
        .world
        .map()
        .get_tile(player.position())
        .ok_or(CommandError::OutOfBounds(player.position()))
        .map(Tile::to_kind)?;
      (
        player.position(),
        properties.current_clip,
        properties.clip_capacity,
        properties.reload_cost,
        tile_kind,
      )
    };

    let outcome = crate::acid_spitter::apply(
      current_clip,
      clip_capacity,
      self
        .state
        .world
        .get_actor(player_id)
        .map_or(0, |player| player.score_count()),
      tile_kind,
    )
    .map_err(|error| match error {
      AcidSpitterReloadError::ClipFull => CommandError::ClipAlreadyFull,
      AcidSpitterReloadError::NotOnAcid => CommandError::NoMatchingAmmo,
    })?;

    let resulting_tile = match outcome.resulting_tile {
      TileKind::Water => Tile::Water,
      _ => return Err(CommandError::NoMatchingAmmo),
    };
    if !self
      .state
      .world
      .map_mut()
      .set_tile(position, resulting_tile)
    {
      return Err(CommandError::OutOfBounds(position));
    }

    let (score_count_remaining, current_clip) = {
      let player = self
        .state
        .world
        .get_actor_mut(player_id)
        .ok_or(CommandError::EntityNotFound(player_id))?;
      player.set_score_count(outcome.score_count_remaining);
      let weapon = player
        .equipment_mut()
        .weapon_mut()
        .ok_or(CommandError::NoEquippedWeapon)?;
      weapon.load_ammo_into_clip(ACID_SPITTER_RELOAD_AMOUNT);
      let current_clip = weapon
        .weapon_properties()
        .map_or(outcome.current_clip, |properties| properties.current_clip);
      (player.score_count(), current_clip)
    };

    events.push(GameEvent::AcidSpitterReloaded {
      entity_id: player_id,
      item_id,
      position,
      ammo_loaded: ACID_SPITTER_RELOAD_AMOUNT,
      current_clip,
      max_clip: clip_capacity,
      score_count_remaining,
    });
    Ok(reload_cost)
  }

  /// Executes player descending stairs to transition to the next level.
  fn execute_player_descend(
    &mut self,
    player_id: drl_protocol::EntityId,
    events: &mut Vec<GameEvent>,
  ) -> Result<ActionCost, CommandError> {
    let p_pos = self
      .state
      .world
      .get_actor(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?
      .position();

    let is_on_stairs = self
      .state
      .world
      .map()
      .get_tile(p_pos)
      .is_some_and(|tile| tile == crate::grid::Tile::StairsDown);

    if !is_on_stairs {
      return Err(CommandError::NotOnStairs(p_pos));
    }

    let current_level = self.state.world.level_id();
    let next_level = current_level.next();

    // Extract player actor from current world to preserve state
    let player_actor = self
      .state
      .world
      .take_player()
      .ok_or(CommandError::EntityNotFound(player_id))?;

    // Generate new level layout
    let config = standard_procedural().config_for_dimensions(
      self.state.world.map().width(),
      self.state.world.map().height(),
    );
    let mut next_item_counter = 1000 * next_level.as_u32() as u64;
    let generated = LevelGenerator::generate(&config, &mut self.state.rng, &mut next_item_counter);

    // Replace world with next floor
    self.state.world = World::from_generated_level(next_level, generated, Some(player_actor));

    events.push(GameEvent::LevelTransitioned {
      from_level: current_level,
      to_level: next_level,
    });

    Ok(ActionCost::STANDARD)
  }

  /// Executes a melee attack between attacker and defender.
  fn execute_melee_attack(
    &mut self,
    attacker_id: drl_protocol::EntityId,
    target_id: drl_protocol::EntityId,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    if let Some(target) = self.state.world.get_actor(target_id)
      && target.death_drop().is_some()
    {
      self.validate_death_drop_position(target.position())?;
    }

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
        damage_type: None,
      });

      if is_lethal {
        let death_drop = self
          .state
          .world
          .get_actor(target_id)
          .and_then(|a| a.death_drop());
        let drop_pos = self.state.world.get_actor(target_id).map(|a| a.position());
        let cause = death_cause.unwrap_or(DeathCause::MeleeAttack { attacker_id });
        events.push(GameEvent::ActorDied {
          entity_id: target_id,
          cause,
        });

        if let (Some(drop_kind), Some(pos)) = (death_drop, drop_pos) {
          self.spawn_death_drop(target_id, pos, drop_kind, events)?;
        }

        if self.state.world.player_id() == Some(target_id) {
          self.state.is_game_over = true;
        }
      } else {
        let attacker_kb = self
          .state
          .world
          .get_actor(attacker_id)
          .map_or(0, |a| a.knockback());
        if attacker_kb > 0 {
          self.apply_knockback(attacker_id, target_id, attacker_kb, events)?;
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
  ) -> Result<ActionCost, CommandError> {
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

    if !crate::fov::has_line_of_sight(self.state.world.map(), p_pos, target_pos) {
      return Err(CommandError::LineOfSightBlocked(target_pos));
    }

    let distance = p_pos.distance_chebyshev(target_pos);

    // Prepare the full validation boundary before consuming ammo or RNG.
    let (fire_cost, shot_count, null_pointer_item_id) = {
      let player = self
        .state
        .world
        .get_actor(player_id)
        .ok_or(CommandError::EntityNotFound(player_id))?;
      let weapon = player
        .equipment()
        .weapon()
        .ok_or(CommandError::NoEquippedWeapon)?;
      let props = weapon
        .weapon_properties()
        .ok_or(CommandError::NoEquippedWeapon)?;

      if !props.is_ranged {
        return Err(CommandError::NoEquippedWeapon);
      }
      if props.current_clip == 0 {
        return Err(CommandError::NoAmmoInClip);
      }
      if distance > props.range {
        return Err(CommandError::TargetOutOfRange(target_pos));
      }
      let shot_count = props.shot_count();
      if props.current_clip < shot_count {
        return Err(CommandError::NoAmmoInClip);
      }

      let null_pointer_item_id =
        (weapon.archetype() == drl_protocol::ItemArchetype::NullPointer).then_some(weapon.id());
      (props.fire_cost, shot_count, null_pointer_item_id)
    };

    // Keep all ordinary command validation ahead of the death-drop preflight,
    // then reject an impossible drop before consuming clip state or combat RNG.
    if self
      .state
      .world
      .get_actor(target_monster_id)
      .is_some_and(|target| target.death_drop().is_some())
    {
      self.validate_death_drop_position(target_pos)?;
    }

    // Commit the prepared shot only after every fallible validation succeeds.
    {
      let player = self
        .state
        .world
        .get_actor_mut(player_id)
        .ok_or(CommandError::EntityNotFound(player_id))?;
      let weapon = player
        .equipment_mut()
        .weapon_mut()
        .ok_or(CommandError::NoEquippedWeapon)?;
      let props = weapon
        .weapon_properties_mut()
        .ok_or(CommandError::NoEquippedWeapon)?;
      props.current_clip -= shot_count;
    }

    for _ in 0..shot_count {
      let (outcome, damage) = {
        let player = self
          .state
          .world
          .get_actor(player_id)
          .ok_or(CommandError::EntityNotFound(player_id))?;
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
        let damage = match outcome {
          AttackOutcome::Hit { damage, .. } => damage,
          _ => 0,
        };
        (outcome, damage)
      };

      events.push(GameEvent::AttackResolved {
        attacker_id: player_id,
        target_id: target_monster_id,
        outcome,
        is_ranged: true,
      });

      if let AttackOutcome::Hit { .. } = outcome
        && let Some(item_id) = null_pointer_item_id
      {
        self.execute_null_pointer_hit(player_id, item_id, target_monster_id, events)?;
      }

      if damage == 0 {
        continue;
      }

      let (taken, actual_lethal, death_cause) =
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
        damage_type: None,
      });

      if actual_lethal {
        let death_drop = self
          .state
          .world
          .get_actor(target_monster_id)
          .and_then(|a| a.death_drop());
        let death_position = self
          .state
          .world
          .get_actor(target_monster_id)
          .map(|a| a.position())
          .unwrap_or(target_pos);
        events.push(GameEvent::ActorDied {
          entity_id: target_monster_id,
          cause: death_cause.unwrap_or(DeathCause::RangedAttack {
            attacker_id: player_id,
          }),
        });
        if let Some(drop_kind) = death_drop {
          self.spawn_death_drop(target_monster_id, death_position, drop_kind, events)?;
        }
        break;
      }

      let player_kb = self
        .state
        .world
        .get_actor(player_id)
        .map_or(0, |a| a.knockback());
      if player_kb > 0 {
        self.apply_knockback(player_id, target_monster_id, player_kb, events)?;
      }
    }

    Ok(fire_cost)
  }

  /// Applies Null Pointer's typed target branch and records its deferred blast.
  fn execute_null_pointer_hit(
    &mut self,
    entity_id: drl_protocol::EntityId,
    item_id: drl_protocol::ItemId,
    target_id: drl_protocol::EntityId,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let target_is_boss = self
      .state
      .world
      .get_actor(target_id)
      .ok_or(CommandError::EntityNotFound(target_id))?
      .is_boss();
    let target = self
      .state
      .world
      .get_actor_mut(target_id)
      .ok_or(CommandError::EntityNotFound(target_id))?;
    let score_count_remaining =
      NullPointerHitTransition::apply(target.score_count_mut(), target_is_boss);
    events.push(GameEvent::NullPointerHit {
      entity_id,
      item_id,
      target_id,
      target_is_boss,
      score_count_remaining,
    });
    events.push(GameEvent::NullPointerExplosionScheduled {
      entity_id,
      target_id,
      delay: NULL_POINTER_EXPLOSION_DELAY,
      radius: NULL_POINTER_EXPLOSION_RADIUS,
      damage: NULL_POINTER_EXPLOSION_DAMAGE,
    });
    Ok(())
  }

  /// Spawns a loot drop on the ground when a monster dies.
  fn spawn_death_drop(
    &mut self,
    entity_id: drl_protocol::EntityId,
    pos: Position,
    kind: drl_protocol::ItemSpawnKind,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    self.validate_death_drop_position(pos)?;
    let item_id = self.state.world.allocate_item_id();
    let item = Item::from_spawn_kind(item_id, kind);
    let item_name = item.name().to_string();
    self.state.world.spawn_ground_item(pos, item)?;
    events.push(GameEvent::ItemDropped {
      entity_id,
      item_id,
      item_name,
      position: pos,
    });
    Ok(())
  }

  /// Validates the map destination used by a typed death drop.
  fn validate_death_drop_position(&self, pos: Position) -> Result<(), CommandError> {
    if !self.state.world.map().is_in_bounds(pos) {
      return Err(CommandError::OutOfBounds(pos));
    }
    if !self.state.world.map().is_walkable(pos) {
      return Err(CommandError::BlockedByTerrain(pos));
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

    let action = crate::ai::MonsterAi::decide_action(monster, &self.state.world, player_id);

    match action {
      crate::ai::MonsterAction::Melee(target_id) => {
        self.execute_melee_attack(monster_id, target_id, events)?;
      }
      crate::ai::MonsterAction::Ranged(target_pos) => {
        self.execute_monster_ranged_attack(monster_id, player_id, target_pos, events)?;
      }
      crate::ai::MonsterAction::Move(dir) => {
        let m_pos = monster.position();
        let new_pos = m_pos + dir;
        if let Some(m) = self.state.world.get_actor_mut(monster_id) {
          m.set_position(new_pos);
        }
        events.push(GameEvent::EntityMoved {
          entity_id: monster_id,
          from: m_pos,
          to: new_pos,
        });
      }
      crate::ai::MonsterAction::Wait => {
        let m_pos = monster.position();
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

  /// Executes a monster ranged attack targeting the player.
  fn execute_monster_ranged_attack(
    &mut self,
    monster_id: drl_protocol::EntityId,
    player_id: drl_protocol::EntityId,
    target_pos: Position,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    let m_pos = self
      .state
      .world
      .get_actor(monster_id)
      .ok_or(CommandError::EntityNotFound(monster_id))?
      .position();

    let distance = m_pos.distance_chebyshev(target_pos);

    let monster = self
      .state
      .world
      .get_actor(monster_id)
      .ok_or(CommandError::EntityNotFound(monster_id))?;
    let player = self
      .state
      .world
      .get_actor(player_id)
      .ok_or(CommandError::EntityNotFound(player_id))?;

    let outcome =
      CombatResolver::resolve_ranged_attack(monster, player, distance, &mut self.state.rng);

    events.push(GameEvent::AttackResolved {
      attacker_id: monster_id,
      target_id: player_id,
      outcome,
      is_ranged: true,
    });

    if let AttackOutcome::Hit { damage, is_lethal } = outcome
      && damage > 0
    {
      let (taken, _, _) =
        self
          .state
          .world
          .apply_damage(player_id, damage, DamageSource::Actor(monster_id))?;

      let remaining = self
        .state
        .world
        .get_actor(player_id)
        .map_or(0, |a| a.hp().current);

      events.push(GameEvent::DamageApplied {
        target_id: player_id,
        amount: taken,
        remaining_hp: remaining,
        source: DamageSource::Actor(monster_id),
        damage_type: None,
      });

      if is_lethal {
        events.push(GameEvent::ActorDied {
          entity_id: player_id,
          cause: DeathCause::RangedAttack {
            attacker_id: monster_id,
          },
        });
        self.state.is_game_over = true;
      } else {
        let monster_kb = self
          .state
          .world
          .get_actor(monster_id)
          .map_or(0, |a| a.knockback());
        if monster_kb > 0 {
          self.apply_knockback(monster_id, player_id, monster_kb, events)?;
        }
      }
    }

    Ok(())
  }

  /// Applies kinetic knockback to a defender, pushing them away from the attacker if the path is clear.
  fn apply_knockback(
    &mut self,
    attacker_id: drl_protocol::EntityId,
    defender_id: drl_protocol::EntityId,
    power: u32,
    events: &mut Vec<GameEvent>,
  ) -> Result<(), CommandError> {
    if power == 0 {
      return Ok(());
    }

    let attacker_pos = self
      .state
      .world
      .get_actor(attacker_id)
      .ok_or(CommandError::EntityNotFound(attacker_id))?
      .position();

    let defender_pos = self
      .state
      .world
      .get_actor(defender_id)
      .ok_or(CommandError::EntityNotFound(defender_id))?
      .position();

    if attacker_pos == defender_pos {
      return Ok(());
    }

    let dx = (defender_pos.x - attacker_pos.x).clamp(-1, 1);
    let dy = (defender_pos.y - attacker_pos.y).clamp(-1, 1);

    let mut current_pos = defender_pos;
    for _ in 0..power {
      let next_pos = Position::new(current_pos.x + dx, current_pos.y + dy);
      if self.state.world.map().is_in_bounds(next_pos)
        && self.state.world.map().is_walkable(next_pos)
        && self.state.world.living_actor_at(next_pos).is_none()
      {
        current_pos = next_pos;
      } else {
        break;
      }
    }

    if current_pos != defender_pos {
      let defender = self
        .state
        .world
        .get_actor_mut(defender_id)
        .ok_or(CommandError::EntityNotFound(defender_id))?;
      defender.set_position(current_pos);

      if defender.is_player() {
        self.state.world.update_visibility();
      }

      events.push(GameEvent::ActorKnockedBack {
        entity_id: defender_id,
        from: defender_pos,
        to: current_pos,
      });
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use drl_protocol::{EntityId, ItemId, ItemSpawnKind};

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

    // Move(None) is the direction-level wait form and has no reachable
    // rejection branch; it still consumes one accepted turn.
    let events_none = game.step(Command::Move(Direction::None)).unwrap();
    assert_eq!(game.turn().count, 2);
    assert!(
      events_none
        .iter()
        .any(|e| matches!(e, GameEvent::EntityWaited { .. }))
    );

    // Step Wait
    let events2 = game.step(Command::Wait).unwrap();
    assert_eq!(game.turn().count, 3);
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

  #[test]
  fn test_game_step_ranged_attack_blocked_by_wall() {
    let mut game = Game::new(300, 10, 10, Position::new(2, 2)).unwrap();
    // Spawn monster at (5, 2)
    game
      .world_mut()
      .spawn_monster(Position::new(5, 2), "Imp", 20, 100, (2, 4))
      .unwrap();

    // Build wall between player (2, 2) and monster (5, 2) at (3, 2)
    game
      .world_mut()
      .map_mut()
      .set_tile(Position::new(3, 2), crate::grid::Tile::Wall);

    // Attack should be rejected because line of sight is blocked
    let err = game
      .step(Command::AttackRanged(Position::new(5, 2)))
      .unwrap_err();
    assert_eq!(err, CommandError::LineOfSightBlocked(Position::new(5, 2)));
  }

  #[test]
  fn test_game_step_pickup_and_drop() {
    let mut game = Game::new(400, 10, 10, Position::new(2, 2)).unwrap();
    let shotgun_id = game.world_mut().allocate_item_id();
    let shotgun = crate::item::Item::shotgun(shotgun_id);
    game
      .world_mut()
      .spawn_ground_item(Position::new(2, 2), shotgun)
      .unwrap();

    // Player picks up item from (2, 2)
    let events = game.step(Command::Pickup).unwrap();
    assert!(events.iter().any(|e| matches!(
      e,
      GameEvent::ItemPickedUp {
        item_id,
        ..
      } if *item_id == shotgun_id
    )));

    // Player drops the shotgun
    let events2 = game.step(Command::Drop(shotgun_id)).unwrap();
    assert!(events2.iter().any(|e| matches!(
      e,
      GameEvent::ItemDropped {
        item_id,
        ..
      } if *item_id == shotgun_id
    )));
  }

  #[test]
  fn test_game_step_pickup_rejection_preserves_partial_ammo_merge() {
    let mut game = Game::new(425, 10, 10, Position::new(2, 2)).unwrap();
    let player_id = game.world().player_id().unwrap();

    // The default 9mm stack has 30 rounds. Add 65 so a later pickup can
    // partially merge five rounds before the full backpack check is reached.
    let extra_ammo_id = game.world_mut().allocate_item_id();
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .inventory_mut()
      .add_item(crate::item::Item::ammo_9mm(extra_ammo_id, 65))
      .unwrap();

    // Fill the remaining inventory slots with non-ammunition items.
    for _ in 0..8 {
      let item_id = game.world_mut().allocate_item_id();
      game
        .world_mut()
        .get_actor_mut(player_id)
        .unwrap()
        .inventory_mut()
        .add_item(crate::item::Item::small_medpack(item_id))
        .unwrap();
    }

    let ground_ammo_id = game.world_mut().allocate_item_id();
    game
      .world_mut()
      .spawn_ground_item(
        Position::new(2, 2),
        crate::item::Item::ammo_9mm(ground_ammo_id, 10),
      )
      .unwrap();

    let before = game.clone();
    let err = game.step(Command::Pickup).unwrap_err();

    assert_eq!(err, CommandError::InventoryFull);
    assert_eq!(game, before);
  }

  #[test]
  fn test_death_drop_uses_canonical_item_factory() {
    let mut game = Game::new(450, 12, 12, Position::new(2, 2)).unwrap();
    let kinds = [
      ItemSpawnKind::Pistol,
      ItemSpawnKind::Shotgun,
      ItemSpawnKind::CombatKnife,
      ItemSpawnKind::Ammo9mm(7),
      ItemSpawnKind::AmmoShells(3),
      ItemSpawnKind::SmallMedPack,
      ItemSpawnKind::LargeMedPack,
      ItemSpawnKind::GreenArmor,
      ItemSpawnKind::PhaseDevice,
    ];

    for (index, kind) in kinds.into_iter().enumerate() {
      let item_id = ItemId::new(index as u64 + 4);
      let position = Position::new(index as i32 + 2, 3);
      let expected = Item::from_spawn_kind(item_id, kind);
      let mut events = Vec::new();

      game
        .spawn_death_drop(EntityId::new(99), position, kind, &mut events)
        .unwrap();

      assert_eq!(
        events,
        vec![GameEvent::ItemDropped {
          entity_id: EntityId::new(99),
          item_id,
          item_name: expected.name().to_string(),
          position,
        }]
      );
      let (actual_position, actual_item) = game.world().ground_items().get(&item_id).unwrap();
      assert_eq!(*actual_position, position);
      assert_eq!(actual_item.to_view(), expected.to_view());
    }
  }

  #[test]
  fn test_game_step_use_medpack() {
    let mut game = Game::new(500, 10, 10, Position::new(2, 2)).unwrap();
    // Damage player by 20
    let p_id = game.world().player_id().unwrap();
    game
      .world_mut()
      .get_actor_mut(p_id)
      .unwrap()
      .hp_mut()
      .take_damage(20);
    assert_eq!(game.world().player().unwrap().hp().current, 30);

    // Find small medpack in inventory
    let med_id = game
      .world()
      .player()
      .unwrap()
      .inventory()
      .find_first_by_category(drl_protocol::ItemCategory::MedPack)
      .unwrap();

    let events = game.step(Command::Use(med_id)).unwrap();
    assert!(events.iter().any(|e| matches!(
      e,
      GameEvent::ItemUsed {
        item_id,
        ..
      } if *item_id == med_id
    )));
    // HP restored by 10 (30 -> 40)
    assert_eq!(game.world().player().unwrap().hp().current, 40);
  }

  #[test]
  fn test_game_step_ranged_attack_ammo_consumption_and_reload() {
    let mut game = Game::new(600, 15, 15, Position::new(2, 2)).unwrap();
    let m_id = game
      .world_mut()
      .spawn_monster(Position::new(5, 2), "Zombie", 500, 0, (1, 2))
      .unwrap();

    // Weapon starts at 10/10 ammo. Fire 10 times.
    for _ in 0..10 {
      let target_pos = game.world().get_actor(m_id).unwrap().position();
      game.step(Command::AttackRanged(target_pos)).unwrap();
    }

    // 11th shot should fail with NoAmmoInClip
    let target_pos = game.world().get_actor(m_id).unwrap().position();
    let err = game.step(Command::AttackRanged(target_pos)).unwrap_err();
    assert_eq!(err, CommandError::NoAmmoInClip);

    // Reload weapon from 30 reserve 9mm ammo in inventory
    let reload_events = game.step(Command::Reload).unwrap();
    assert!(reload_events.iter().any(|e| matches!(
      e,
      GameEvent::WeaponReloaded {
        ammo_loaded: 10,
        current_clip: 10,
        ..
      }
    )));

    // Player can now fire again
    let fire_events = game.step(Command::AttackRanged(target_pos)).unwrap();
    assert!(fire_events.iter().any(|e| matches!(
      e,
      GameEvent::AttackResolved {
        target_id,
        is_ranged: true,
        ..
      } if *target_id == m_id
    )));
  }

  #[test]
  fn test_game_new_procedural() {
    let config = LevelGeneratorConfig::default();
    let game = Game::new_procedural(777, config).unwrap();
    assert_eq!(game.world().level_id(), LevelId::new(1));
    assert!(game.world().player().is_some());
    assert!(game.world().player().unwrap().is_alive());
  }

  #[test]
  fn test_game_step_descend_when_not_on_stairs_fails() {
    let mut game = Game::new(100, 10, 10, Position::new(2, 2)).unwrap();
    // Tile at (2, 2) is Floor, not StairsDown
    let err = game.step(Command::Descend).unwrap_err();
    assert_eq!(err, CommandError::NotOnStairs(Position::new(2, 2)));
    assert_eq!(game.world().level_id(), LevelId::new(1));
  }

  #[test]
  fn test_game_step_descend_stairs_transitions_level() {
    let mut game = Game::new(200, 10, 10, Position::new(2, 2)).unwrap();
    // Place stairs down at player position (2, 2)
    game
      .world_mut()
      .map_mut()
      .set_tile(Position::new(2, 2), crate::grid::Tile::StairsDown);

    // Give player custom equipment/stats to verify preservation across transition
    let p_id = game.world().player_id().unwrap();
    game
      .world_mut()
      .get_actor_mut(p_id)
      .unwrap()
      .hp_mut()
      .take_damage(10);
    assert_eq!(game.world().player().unwrap().hp().current, 40);

    let events = game.step(Command::Descend).unwrap();
    assert!(events.iter().any(|e| matches!(
      e,
      GameEvent::LevelTransitioned {
        from_level,
        to_level,
      } if *from_level == LevelId::new(1) && *to_level == LevelId::new(2)
    )));

    // World is now Level 2
    assert_eq!(game.world().level_id(), LevelId::new(2));
    // Player HP and inventory are preserved
    let p2 = game.world().player().unwrap();
    assert_eq!(p2.hp().current, 40);
    assert!(p2.equipment().weapon().is_some());
    assert_eq!(p2.equipment().weapon().unwrap().name(), "Pistol");
  }
}
