//! Pure presentation planning for DRL-Rust.
//!
//! Scene construction consumes only protocol observations and events. A
//! browser or native renderer may turn the resulting scene into pixels, but
//! presentation timing can never advance the simulation.

use drl_assets::{
  AtlasId, AtlasTextureSource, LayerRole, SpriteDescriptor, SpriteLayer, SpriteUv, actor_sprite,
  item_sprite, tile_sprite,
};
use drl_protocol::{
  Command, EntityId, GameEvent, HitPoints, ItemView, PlayerObservation, Position, TileKind,
};

/// A complete before/command/events/after boundary for one presentation step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationStep {
  pub before: PlayerObservation,
  pub command: Command,
  pub events: Vec<GameEvent>,
  pub effects: Vec<EffectSpan>,
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

impl PresentationEffect {
  /// Returns the frontend-neutral logical duration for this effect.
  #[must_use]
  pub const fn duration_ticks(self) -> u16 {
    match self {
      Self::Move => 1,
      Self::MeleeAttack | Self::RangedAttack => 2,
      Self::Hit => 1,
      Self::Death => 4,
      Self::Pickup | Self::Drop | Self::Equip | Self::Use => 2,
      Self::Reload => 3,
      Self::Teleport | Self::LevelTransition => 4,
      Self::Knockback => 2,
    }
  }
}

/// One sequential presentation effect span in logical frontend ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectSpan {
  pub effect: PresentationEffect,
  pub start_tick: u32,
  pub duration_ticks: u16,
}

/// Visibility-derived presentation bands for deterministic scene shading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LightingBand {
  /// The player currently sees this tile.
  Visible,
  /// The player remembers this tile, but it is outside current sight.
  Explored,
}

impl LightingBand {
  /// Returns the percentage shade factor used by the presentation layer.
  #[must_use]
  pub const fn factor(self) -> u8 {
    match self {
      Self::Visible => 100,
      Self::Explored => 45,
    }
  }
}

/// Applies the shared visibility shade to an RGBA color.
#[must_use]
pub fn shade_color(color: [f32; 4], band: LightingBand) -> [f32; 4] {
  let factor = band.factor() as f32 / 100.0;
  [
    color[0] * factor,
    color[1] * factor,
    color[2] * factor,
    color[3],
  ]
}

/// Coarse scene tone selected from the fair player health observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneTone {
  Normal,
  LowHealth,
}

/// Selects the current scene tone without consulting hidden simulation state.
#[must_use]
pub const fn scene_tone(player_hp: Option<HitPoints>) -> SceneTone {
  match player_hp {
    Some(hp) if hp.current < hp.max / 4 => SceneTone::LowHealth,
    _ => SceneTone::Normal,
  }
}

/// Returns the deterministic clear color for a player health tone.
#[must_use]
pub const fn scene_clear_color(player_hp: Option<HitPoints>) -> [f32; 4] {
  match scene_tone(player_hp) {
    SceneTone::Normal => [0.025, 0.035, 0.055, 1.0],
    SceneTone::LowHealth => [0.12, 0.015, 0.015, 1.0],
  }
}

/// Integer pixel bounds for one logical map cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelRect {
  pub x: u32,
  pub y: u32,
  pub width: u32,
  pub height: u32,
}

/// One renderer-neutral atlas layer draw operation.
///
/// The destination is already resolved to the physical pixel grid while the
/// UVs remain normalized image-space coordinates. A backend may batch or
/// upload these operations, but this plan itself never loads or samples an
/// image and never advances simulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerDraw {
  pub atlas: AtlasId,
  pub layer: SpriteLayer,
  pub role: LayerRole,
  pub source: AtlasTextureSource,
  pub lighting: LightingBand,
  pub destination: PixelRect,
  pub uv: SpriteUv,
}

/// Deterministic pixel-grid layout for a scene and its physical canvas.
///
/// The viewport uses one square integer-sized cell for every map tile. Any
/// unused pixels are centered as letterbox margins instead of stretching the
/// map differently on each axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelViewport {
  pub map_width: u32,
  pub map_height: u32,
  pub canvas_width: u32,
  pub canvas_height: u32,
  pub tile_size: u32,
  pub offset_x: u32,
  pub offset_y: u32,
}

impl PixelViewport {
  /// Fits a map into a physical canvas using the largest square tile size.
  #[must_use]
  pub fn fit(map_width: u32, map_height: u32, canvas_width: u32, canvas_height: u32) -> Self {
    let map_width = map_width.max(1);
    let map_height = map_height.max(1);
    let canvas_width = canvas_width.max(1);
    let canvas_height = canvas_height.max(1);
    let tile_size = (canvas_width / map_width).min(canvas_height / map_height);
    let board_width = tile_size.saturating_mul(map_width);
    let board_height = tile_size.saturating_mul(map_height);
    Self {
      map_width,
      map_height,
      canvas_width,
      canvas_height,
      tile_size,
      offset_x: canvas_width.saturating_sub(board_width) / 2,
      offset_y: canvas_height.saturating_sub(board_height) / 2,
    }
  }

  /// Returns the integer pixel rectangle for a map position.
  #[must_use]
  pub fn tile_rect(self, position: Position) -> Option<PixelRect> {
    if self.tile_size == 0 {
      return None;
    }
    let x = u32::try_from(position.x).ok()?;
    let y = u32::try_from(position.y).ok()?;
    if x >= self.map_width || y >= self.map_height {
      return None;
    }
    Some(PixelRect {
      x: self
        .offset_x
        .saturating_add(x.saturating_mul(self.tile_size)),
      y: self
        .offset_y
        .saturating_add(y.saturating_mul(self.tile_size)),
      width: self.tile_size,
      height: self.tile_size,
    })
  }
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

impl SceneTile {
  /// Maps fair tile visibility to the shared presentation lighting band.
  #[must_use]
  pub const fn lighting_band(&self) -> LightingBand {
    if self.visible {
      LightingBand::Visible
    } else {
      LightingBand::Explored
    }
  }
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
    // Ground-item memory may include an explored tile that is currently out
    // of sight. Do not draw or otherwise surface those items in the browser
    // scene; the frontend must never turn remembered state into hidden-world
    // disclosure.
    let items = observation
      .ground_items
      .iter()
      .filter(|ground| {
        observation
          .visible_tiles
          .iter()
          .any(|tile| tile.position == ground.position && tile.is_visible)
      })
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

fn append_layer_draws(
  plan: &mut Vec<LayerDraw>,
  descriptor: SpriteDescriptor,
  lighting: LightingBand,
  destination: Option<PixelRect>,
) {
  let Some(destination) = destination else {
    return;
  };
  let (atlas_width, atlas_height) = descriptor.atlas.dimensions();
  let Some(uv) = descriptor.rect.uv_rect(atlas_width, atlas_height) else {
    return;
  };
  plan.extend(descriptor.layers.iter().copied().map(|layer| LayerDraw {
    atlas: descriptor.atlas,
    layer,
    role: layer.role(),
    source: descriptor.atlas.texture_source(layer),
    lighting,
    destination,
    uv,
  }));
}

/// Builds a stable atlas/layer draw plan from fair scene data.
///
/// Entries are emitted back-to-front in scene order: tiles, visible items,
/// then visible actors. Explored-but-hidden tiles remain eligible for the
/// fogged scene because they are part of the fair observation memory; a tile
/// that is neither visible nor explored is omitted. Invalid atlas geometry is
/// omitted defensively rather than producing a malformed backend command.
#[must_use]
pub fn layer_draw_plan(scene: &RenderScene, viewport: PixelViewport) -> Vec<LayerDraw> {
  let mut plan = Vec::new();
  for tile in &scene.tiles {
    if !tile.visible && !tile.explored {
      continue;
    }
    append_layer_draws(
      &mut plan,
      tile.sprite,
      tile.lighting_band(),
      viewport.tile_rect(tile.position),
    );
  }
  for item in &scene.items {
    append_layer_draws(
      &mut plan,
      item.sprite,
      LightingBand::Visible,
      viewport.tile_rect(item.position),
    );
  }
  for actor in &scene.actors {
    append_layer_draws(
      &mut plan,
      actor.sprite,
      LightingBand::Visible,
      viewport.tile_rect(actor.position),
    );
  }
  plan
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

/// Builds deterministic sequential timing spans from simulation events.
#[must_use]
pub fn effect_timeline(events: &[GameEvent]) -> Vec<EffectSpan> {
  let mut start_tick = 0_u32;
  effects_for_events(events)
    .into_iter()
    .map(|effect| {
      let duration_ticks = effect.duration_ticks();
      let span = EffectSpan {
        effect,
        start_tick,
        duration_ticks,
      };
      start_tick = start_tick.saturating_add(u32::from(duration_ticks));
      span
    })
    .collect()
}

/// One active effect at a frontend presentation tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectFrame {
  pub effect: PresentationEffect,
  /// Normalized progress inside the span, bounded to `[0, 1)`.
  pub progress: f32,
}

/// Returns active effect frames in the stable order of the supplied timeline.
///
/// Presentation ticks are frontend timing units only. A span whose end would
/// overflow `u32`, or whose duration is zero, is omitted defensively.
#[must_use]
pub fn active_effect_frames(spans: &[EffectSpan], presentation_tick: u32) -> Vec<EffectFrame> {
  spans
    .iter()
    .filter_map(|span| {
      let duration = u32::from(span.duration_ticks);
      if duration == 0 {
        return None;
      }
      let end_tick = span.start_tick.checked_add(duration)?;
      if presentation_tick < span.start_tick || presentation_tick >= end_tick {
        return None;
      }
      let elapsed = presentation_tick - span.start_tick;
      Some(EffectFrame {
        effect: span.effect,
        progress: elapsed as f32 / duration as f32,
      })
    })
    .collect()
}

fn event_entity_ids(event: &GameEvent) -> [Option<EntityId>; 2] {
  match event {
    GameEvent::EntityMoved { entity_id, .. }
    | GameEvent::EntityWaited { entity_id, .. }
    | GameEvent::ActorDied { entity_id, .. }
    | GameEvent::ActionCostPaid { entity_id, .. }
    | GameEvent::ItemPickedUp { entity_id, .. }
    | GameEvent::ItemDropped { entity_id, .. }
    | GameEvent::ItemEquipped { entity_id, .. }
    | GameEvent::ItemUnequipped { entity_id, .. }
    | GameEvent::ItemUsed { entity_id, .. }
    | GameEvent::WeaponReloaded { entity_id, .. }
    | GameEvent::ActorKnockedBack { entity_id, .. } => [Some(*entity_id), None],
    GameEvent::AttackResolved {
      attacker_id,
      target_id,
      ..
    } => [Some(*attacker_id), Some(*target_id)],
    GameEvent::DamageApplied { target_id, .. } => [Some(*target_id), None],
    GameEvent::TurnStarted { .. }
    | GameEvent::LevelTransitioned { .. }
    | GameEvent::PlayerTeleported { .. }
    | GameEvent::TurnEnded { .. } => [None, None],
  }
}

fn event_is_observable(
  before: &PlayerObservation,
  after: &PlayerObservation,
  event: &GameEvent,
) -> bool {
  if matches!(
    event,
    GameEvent::LevelTransitioned { .. } | GameEvent::PlayerTeleported { .. }
  ) {
    return true;
  }
  let ids = event_entity_ids(event);
  if matches!(
    event,
    GameEvent::DamageApplied { .. } | GameEvent::ActorDied { .. }
  ) {
    return ids.into_iter().flatten().any(|entity_id| {
      before
        .visible_actors
        .iter()
        .any(|actor| actor.id == entity_id)
    });
  }
  ids.into_iter().flatten().any(|entity_id| {
    let visible_before = before
      .visible_actors
      .iter()
      .any(|actor| actor.id == entity_id);
    let visible_after = after
      .visible_actors
      .iter()
      .any(|actor| actor.id == entity_id);
    visible_before && visible_after
  })
}

/// Builds effect spans using endpoint-visible actors and pre-step visible
/// targets for terminal hit/death events.
///
/// Direct player transitions remain observable even when no actor identity is
/// present in the event. Hidden actor events are excluded before timing spans
/// are assigned, so future frame mapping cannot disclose hidden activity.
#[must_use]
pub fn effect_timeline_for_observations(
  before: &PlayerObservation,
  after: &PlayerObservation,
  events: &[GameEvent],
) -> Vec<EffectSpan> {
  let observable_events = events
    .iter()
    .filter(|event| event_is_observable(before, after, event))
    .cloned()
    .collect::<Vec<_>>();
  effect_timeline(&observable_events)
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
  fn pixel_viewport_centers_square_integer_cells() {
    let viewport = PixelViewport::fit(24, 16, 960, 504);
    assert_eq!(viewport.tile_size, 31);
    assert_eq!(viewport.offset_x, 108);
    assert_eq!(viewport.offset_y, 4);
    assert_eq!(
      viewport.tile_rect(Position::new(0, 0)),
      Some(PixelRect {
        x: 108,
        y: 4,
        width: 31,
        height: 31,
      })
    );
    assert_eq!(
      viewport.tile_rect(Position::new(23, 15)),
      Some(PixelRect {
        x: 821,
        y: 469,
        width: 31,
        height: 31,
      })
    );
  }

  #[test]
  fn pixel_viewport_clamps_empty_dimensions_and_rejects_out_of_bounds_tiles() {
    let viewport = PixelViewport::fit(0, 0, 0, 0);
    assert_eq!(viewport.tile_size, 1);
    assert!(viewport.tile_rect(Position::new(0, 0)).is_some());
    assert_eq!(viewport.tile_rect(Position::new(-1, 0)), None);
    assert_eq!(viewport.tile_rect(Position::new(1, 0)), None);

    let undersized = PixelViewport::fit(24, 16, 1, 1);
    assert_eq!(undersized.tile_size, 0);
    assert_eq!(undersized.tile_rect(Position::new(0, 0)), None);
  }

  #[test]
  fn lighting_band_shading_is_visibility_derived_and_stable() {
    let visible = SceneTile {
      position: Position::new(0, 0),
      kind: TileKind::Floor,
      visible: true,
      explored: true,
      sprite: tile_sprite(TileKind::Floor),
    };
    let explored = SceneTile {
      visible: false,
      ..visible.clone()
    };
    assert_eq!(visible.lighting_band(), LightingBand::Visible);
    assert_eq!(explored.lighting_band(), LightingBand::Explored);
    assert_eq!(
      shade_color([0.2, 0.4, 0.8, 1.0], LightingBand::Visible),
      [0.2, 0.4, 0.8, 1.0]
    );
    let shaded = shade_color([0.2, 0.4, 0.8, 1.0], LightingBand::Explored);
    assert!((shaded[0] - 0.09).abs() < 1e-6);
    assert!((shaded[1] - 0.18).abs() < 1e-6);
    assert!((shaded[2] - 0.36).abs() < 1e-6);
    assert_eq!(shaded[3], 1.0);
  }

  #[test]
  fn scene_clear_tone_preserves_quarter_health_threshold() {
    assert_eq!(scene_tone(None), SceneTone::Normal);
    assert_eq!(scene_tone(Some(HitPoints::new(12, 50))), SceneTone::Normal);
    assert_eq!(
      scene_tone(Some(HitPoints::new(11, 50))),
      SceneTone::LowHealth
    );
    assert_eq!(
      scene_clear_color(Some(HitPoints::new(11, 50))),
      [0.12, 0.015, 0.015, 1.0]
    );
    assert_eq!(
      scene_clear_color(Some(HitPoints::new(50, 50))),
      [0.025, 0.035, 0.055, 1.0]
    );
  }

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
  fn layer_draw_plan_is_ordered_and_uses_descriptor_geometry() {
    let game = Game::new_arena(42, 12, 10).expect("arena");
    let scene = RenderScene::from_observation(&game.observe_player());
    let viewport = PixelViewport::fit(scene.map_width, scene.map_height, 960, 640);
    let plan = layer_draw_plan(&scene, viewport);
    let first_tile = scene.tiles.first().expect("visible tile");
    let first_destination = viewport
      .tile_rect(first_tile.position)
      .expect("tile in viewport");
    let first_uv = first_tile
      .sprite
      .rect
      .uv_rect(
        first_tile.sprite.atlas.dimensions().0,
        first_tile.sprite.atlas.dimensions().1,
      )
      .expect("measured tile UVs");

    assert!(plan.len() >= first_tile.sprite.layers.len());
    for (draw, layer) in plan
      .iter()
      .zip(first_tile.sprite.layers.iter().copied())
      .take(first_tile.sprite.layers.len())
    {
      assert_eq!(draw.atlas, first_tile.sprite.atlas);
      assert_eq!(draw.layer, layer);
      assert_eq!(draw.role, layer.role());
      assert_eq!(draw.source, draw.atlas.texture_source(draw.layer));
      assert_eq!(draw.lighting, first_tile.lighting_band());
      assert_eq!(draw.destination, first_destination);
      assert_eq!(draw.uv, first_uv);
    }
    let tile_draws = scene
      .tiles
      .iter()
      .filter(|tile| tile.visible || tile.explored)
      .map(|tile| tile.sprite.layers.len())
      .sum::<usize>();
    let item_draws = scene
      .items
      .iter()
      .map(|item| item.sprite.layers.len())
      .sum::<usize>();
    let actor_draws = scene
      .actors
      .iter()
      .map(|actor| actor.sprite.layers.len())
      .sum::<usize>();
    assert_eq!(plan.len(), tile_draws + item_draws + actor_draws);
    let first_actor = scene.actors.first().expect("player actor");
    assert_eq!(
      plan[tile_draws + item_draws].atlas,
      first_actor.sprite.atlas
    );
    assert_eq!(
      plan[tile_draws + item_draws].lighting,
      LightingBand::Visible
    );
    assert_eq!(plan, layer_draw_plan(&scene, viewport));

    let mut explored_scene = scene.clone();
    explored_scene.tiles[0].visible = false;
    explored_scene.tiles[0].explored = true;
    let explored_plan = layer_draw_plan(&explored_scene, viewport);
    assert!(
      explored_plan
        .iter()
        .take(first_tile.sprite.layers.len())
        .all(|draw| draw.lighting == LightingBand::Explored)
    );
  }

  #[test]
  fn layer_draw_plan_omits_unknown_tiles_and_zero_sized_viewports() {
    let game = Game::new_arena(42, 12, 10).expect("arena");
    let observation = game.observe_player();
    let mut scene = RenderScene::from_observation(&observation);
    scene.tiles.push(SceneTile {
      position: Position::new(0, 0),
      kind: TileKind::Floor,
      visible: false,
      explored: false,
      sprite: tile_sprite(TileKind::Floor),
    });
    let viewport = PixelViewport::fit(scene.map_width, scene.map_height, 960, 640);
    let plan = layer_draw_plan(&scene, viewport);
    let expected = RenderScene::from_observation(&observation);
    assert_eq!(plan, layer_draw_plan(&expected, viewport));

    let undersized = PixelViewport::fit(scene.map_width, scene.map_height, 1, 1);
    assert!(layer_draw_plan(&scene, undersized).is_empty());
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
    assert_eq!(
      effect_timeline(&events),
      vec![
        EffectSpan {
          effect: PresentationEffect::Move,
          start_tick: 0,
          duration_ticks: 1,
        },
        EffectSpan {
          effect: PresentationEffect::Knockback,
          start_tick: 1,
          duration_ticks: 2,
        },
      ]
    );
  }

  #[test]
  fn active_effect_frames_are_normalized_and_overflow_safe() {
    let spans = [
      EffectSpan {
        effect: PresentationEffect::Move,
        start_tick: 2,
        duration_ticks: 2,
      },
      EffectSpan {
        effect: PresentationEffect::Hit,
        start_tick: 4,
        duration_ticks: 1,
      },
      EffectSpan {
        effect: PresentationEffect::Death,
        start_tick: 5,
        duration_ticks: 0,
      },
      EffectSpan {
        effect: PresentationEffect::Teleport,
        start_tick: u32::MAX,
        duration_ticks: 2,
      },
    ];

    assert_eq!(active_effect_frames(&spans, 1), Vec::new());
    assert_eq!(
      active_effect_frames(&spans, 2),
      vec![EffectFrame {
        effect: PresentationEffect::Move,
        progress: 0.0,
      }]
    );
    let halfway = active_effect_frames(&spans, 3);
    assert_eq!(halfway[0].effect, PresentationEffect::Move);
    assert!((halfway[0].progress - 0.5).abs() < f32::EPSILON);
    assert_eq!(
      active_effect_frames(&spans, 4),
      vec![EffectFrame {
        effect: PresentationEffect::Hit,
        progress: 0.0,
      }]
    );
    assert!(active_effect_frames(&spans, 5).is_empty());
    assert_eq!(
      active_effect_frames(&spans, 3),
      active_effect_frames(&spans, 3)
    );
  }

  #[test]
  fn observed_effect_timeline_excludes_visibility_boundary_events() {
    let game = Game::new_arena(42, 12, 10).expect("arena");
    let observation = game.observe_player();
    let player_id = observation
      .visible_actors
      .iter()
      .find(|actor| actor.is_player)
      .expect("player actor")
      .id;
    let hidden_event = GameEvent::EntityMoved {
      entity_id: EntityId::new(999),
      from: Position::new(8, 8),
      to: Position::new(9, 8),
    };
    let visible_event = GameEvent::EntityMoved {
      entity_id: player_id,
      from: observation.player_position,
      to: observation.player_position,
    };
    let events = [hidden_event.clone(), visible_event];
    assert_eq!(
      effect_timeline_for_observations(&observation, &observation, &events),
      vec![EffectSpan {
        effect: PresentationEffect::Move,
        start_tick: 0,
        duration_ticks: 1,
      }]
    );

    let mut visible_before = observation.clone();
    let mut transient_actor = visible_before
      .visible_actors
      .iter()
      .find(|actor| actor.is_player)
      .expect("player actor")
      .clone();
    transient_actor.id = EntityId::new(999);
    transient_actor.is_player = false;
    transient_actor.name = "transient actor".to_string();
    transient_actor.monster_kind = Some(drl_protocol::MonsterKind::Imp);
    visible_before.visible_actors.push(transient_actor.clone());
    let hidden_after = observation.clone();
    assert!(
      effect_timeline_for_observations(
        &visible_before,
        &hidden_after,
        std::slice::from_ref(&hidden_event),
      )
      .is_empty()
    );

    let hidden_before = observation;
    let mut visible_after = hidden_before.clone();
    visible_after.visible_actors.push(transient_actor);
    assert!(
      effect_timeline_for_observations(&hidden_before, &visible_after, &[hidden_event]).is_empty()
    );
  }

  #[test]
  fn observed_effect_timeline_preserves_visible_terminal_events() {
    let game = Game::new_arena(42, 12, 10).expect("arena");
    let observation = game.observe_player();
    let player_id = observation
      .visible_actors
      .iter()
      .find(|actor| actor.is_player)
      .expect("player actor")
      .id;
    let mut visible_before = observation.clone();
    let mut defeated_actor = visible_before
      .visible_actors
      .iter()
      .find(|actor| actor.is_player)
      .expect("player actor")
      .clone();
    defeated_actor.id = EntityId::new(999);
    defeated_actor.is_player = false;
    defeated_actor.name = "defeated actor".to_string();
    defeated_actor.monster_kind = Some(drl_protocol::MonsterKind::Imp);
    visible_before.visible_actors.push(defeated_actor);
    let hidden_after = observation.clone();
    let defeated_events = [
      GameEvent::DamageApplied {
        target_id: EntityId::new(999),
        amount: 4,
        remaining_hp: 0,
        source: drl_protocol::DamageSource::Actor(EntityId::new(1)),
      },
      GameEvent::ActorDied {
        entity_id: EntityId::new(999),
        cause: drl_protocol::DeathCause::MeleeAttack {
          attacker_id: EntityId::new(1),
        },
      },
    ];
    assert_eq!(
      effect_timeline_for_observations(&visible_before, &hidden_after, &defeated_events),
      vec![
        EffectSpan {
          effect: PresentationEffect::Hit,
          start_tick: 0,
          duration_ticks: 1,
        },
        EffectSpan {
          effect: PresentationEffect::Death,
          start_tick: 1,
          duration_ticks: 4,
        },
      ]
    );

    let player_death_events = [
      GameEvent::DamageApplied {
        target_id: player_id,
        amount: 50,
        remaining_hp: 0,
        source: drl_protocol::DamageSource::Actor(EntityId::new(1)),
      },
      GameEvent::ActorDied {
        entity_id: player_id,
        cause: drl_protocol::DeathCause::MeleeAttack {
          attacker_id: EntityId::new(1),
        },
      },
    ];
    let player_hidden_after = drl_protocol::PlayerObservation {
      visible_actors: Vec::new(),
      ..observation
    };
    assert_eq!(
      effect_timeline_for_observations(
        &player_hidden_after,
        &player_hidden_after,
        &player_death_events
      ),
      Vec::<EffectSpan>::new()
    );
    assert_eq!(
      effect_timeline_for_observations(&hidden_after, &player_hidden_after, &player_death_events),
      vec![
        EffectSpan {
          effect: PresentationEffect::Hit,
          start_tick: 0,
          duration_ticks: 1,
        },
        EffectSpan {
          effect: PresentationEffect::Death,
          start_tick: 1,
          duration_ticks: 4,
        },
      ]
    );
  }
}
