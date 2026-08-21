//! Pure presentation planning for DRL-Rust.
//!
//! Scene construction consumes only protocol observations and events. A
//! browser or native renderer may turn the resulting scene into pixels, but
//! presentation timing can never advance the simulation.

use drl_assets::{SpriteDescriptor, actor_sprite, item_sprite, tile_sprite};
use drl_protocol::{Command, GameEvent, ItemView, PlayerObservation, Position, TileKind};

/// A complete before/command/events/after boundary for one presentation step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationStep {
  pub before: PlayerObservation,
  pub command: Command,
  pub events: Vec<GameEvent>,
  pub after: PlayerObservation,
}

/// Bounded, deterministic presentation effects derived from simulation events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PresentationEffect {
  Move,
  MeleeAttack,
  RangedAttack,
  Hit,
  Death,
  Pickup,
  Drop,
  Equip,
  Use,
  Reload,
  Teleport,
  LevelTransition,
  Knockback,
}

/// A tile ready for a renderer to draw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneTile {
  pub position: Position,
  pub kind: TileKind,
  pub visible: bool,
  pub explored: bool,
  pub sprite: SpriteDescriptor,
}

/// An actor ready for a renderer to draw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneActor {
  pub id: drl_protocol::EntityId,
  pub position: Position,
  pub is_player: bool,
  pub hp: Option<drl_protocol::HitPoints>,
  pub sprite: SpriteDescriptor,
}

/// An item ready for a renderer to draw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneItem {
  pub position: Position,
  pub item: ItemView,
  pub sprite: SpriteDescriptor,
}

/// HUD values that can be represented in semantic DOM or pixel UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudState {
  pub turn: u64,
  pub player_hp: Option<drl_protocol::HitPoints>,
  pub weapon: Option<ItemView>,
  pub armor: Option<ItemView>,
  pub inventory_size: usize,
}

/// Deterministic render input built from a player observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderScene {
  pub map_width: u32,
  pub map_height: u32,
  pub player_position: Position,
  /// Visible positions eligible for the bounded target overlay.
  pub target_positions: Vec<Position>,
  pub tiles: Vec<SceneTile>,
  pub actors: Vec<SceneActor>,
  pub items: Vec<SceneItem>,
  pub hud: HudState,
}

impl RenderScene {
  /// Builds a scene without consulting hidden simulation state.
  #[must_use]
  pub fn from_observation(observation: &PlayerObservation) -> Self {
    let tiles = observation
      .visible_tiles
      .iter()
      .map(|tile| SceneTile {
        position: tile.position,
        kind: tile.kind,
        visible: tile.is_visible,
        explored: true,
        sprite: tile_sprite(tile.kind),
      })
      .collect();
    let actors = observation
      .visible_actors
      .iter()
      .map(|actor| SceneActor {
        id: actor.id,
        position: actor.position,
        is_player: actor.is_player,
        hp: actor.hp,
        sprite: actor_sprite(actor.monster_kind),
      })
      .collect();
    let target_positions = observation
      .visible_actors
      .iter()
      .filter(|actor| !actor.is_player)
      .map(|actor| actor.position)
      .collect();
    let items = observation
      .ground_items
      .iter()
      .map(|ground| SceneItem {
        position: ground.position,
        item: ground.item.clone(),
        sprite: item_sprite(ground.item.archetype),
      })
      .collect();

    Self {
      map_width: observation.map_width,
      map_height: observation.map_height,
      player_position: observation.player_position,
      target_positions,
      tiles,
      actors,
      items,
      hud: HudState {
        turn: observation.turn.count,
        player_hp: observation.player_hp,
        weapon: observation.equipped_weapon.clone(),
        armor: observation.equipped_armor.clone(),
        inventory_size: observation.inventory.len(),
      },
    }
  }
}

/// Returns a stable event list for audio/effect mapping.
#[must_use]
pub fn event_sequence(step: &PresentationStep) -> &[GameEvent] {
  &step.events
}

/// Maps events to bounded visual effects without consulting simulation state.
#[must_use]
pub fn effects_for_events(events: &[GameEvent]) -> Vec<PresentationEffect> {
  events
    .iter()
    .filter_map(|event| match event {
      GameEvent::EntityMoved { .. } => Some(PresentationEffect::Move),
      GameEvent::AttackResolved { is_ranged, .. } => Some(if *is_ranged {
        PresentationEffect::RangedAttack
      } else {
        PresentationEffect::MeleeAttack
      }),
      GameEvent::DamageApplied { .. } => Some(PresentationEffect::Hit),
      GameEvent::ActorDied { .. } => Some(PresentationEffect::Death),
      GameEvent::ItemPickedUp { .. } => Some(PresentationEffect::Pickup),
      GameEvent::ItemDropped { .. } => Some(PresentationEffect::Drop),
      GameEvent::ItemEquipped { .. } | GameEvent::ItemUnequipped { .. } => {
        Some(PresentationEffect::Equip)
      }
      GameEvent::ItemUsed { .. } => Some(PresentationEffect::Use),
      GameEvent::WeaponReloaded { .. } => Some(PresentationEffect::Reload),
      GameEvent::LevelTransitioned { .. } => Some(PresentationEffect::LevelTransition),
      GameEvent::PlayerTeleported { .. } => Some(PresentationEffect::Teleport),
      GameEvent::ActorKnockedBack { .. } => Some(PresentationEffect::Knockback),
      GameEvent::TurnStarted { .. }
      | GameEvent::EntityWaited { .. }
      | GameEvent::ActionCostPaid { .. }
      | GameEvent::TurnEnded { .. } => None,
    })
    .collect()
}

/// Returns the renderer component name.
#[must_use]
pub fn renderer_name() -> &'static str {
  "drl-render"
}

#[cfg(test)]
mod tests {
  use super::*;
  use drl_core::Game;

  #[test]
  fn scene_contains_only_observed_content() {
    let game = Game::new_arena(42, 12, 10).expect("arena");
    let observation = game.observe_player();
    let scene = RenderScene::from_observation(&observation);
    assert_eq!(scene.map_width, 12);
    assert_eq!(scene.map_height, 10);
    assert!(scene.actors.iter().all(|actor| {
      observation
        .visible_actors
        .iter()
        .any(|view| view.id == actor.id)
    }));
  }

  #[test]
  fn effects_are_event_ordered_and_deterministic() {
    let events = [
      GameEvent::EntityMoved {
        entity_id: drl_protocol::EntityId::new(1),
        from: Position::new(1, 1),
        to: Position::new(2, 1),
      },
      GameEvent::ActorKnockedBack {
        entity_id: drl_protocol::EntityId::new(2),
        from: Position::new(3, 1),
        to: Position::new(4, 1),
      },
    ];
    assert_eq!(
      effects_for_events(&events),
      vec![PresentationEffect::Move, PresentationEffect::Knockback]
    );
  }
}
