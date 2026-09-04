//! Shared fallback geometry for platform frontends.
//!
//! This plan is intentionally simpler than the atlas compositor: it provides a
//! deterministic colored scene for a frontend boundary proof without moving
//! GPU, window, or asset-loading policy into `drl-render`.

use crate::{PixelRect, PixelViewport, RenderScene, SceneTile, shade_color};
use drl_protocol::TileKind;

/// One colored square in the renderer-neutral fallback scene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneQuad {
  /// Physical pixel destination selected by the shared integer viewport.
  pub rect: PixelRect,
  /// Fraction of the normalized tile bounds to inset on each axis.
  pub inset_fraction: f32,
  /// RGBA display color used by the fallback pipeline.
  pub color: [f32; 4],
}

fn tile_color(tile: &SceneTile) -> [f32; 4] {
  let color = match tile.kind {
    TileKind::Wall => [0.08, 0.09, 0.12, 1.0],
    TileKind::DoorClosed => [0.24, 0.16, 0.09, 1.0],
    TileKind::DoorOpen => [0.18, 0.20, 0.18, 1.0],
    TileKind::StairsDown => [0.28, 0.24, 0.08, 1.0],
    TileKind::Lava => [0.45, 0.12, 0.04, 1.0],
    TileKind::Acid => [0.12, 0.45, 0.12, 1.0],
    TileKind::Water => [0.12, 0.28, 0.55, 1.0],
    TileKind::Mud => [0.38, 0.26, 0.16, 1.0],
    TileKind::Floor => [0.16, 0.18, 0.22, 1.0],
  };
  shade_color(color, tile.lighting_band())
}

fn append_quad(
  plan: &mut Vec<SceneQuad>,
  viewport: PixelViewport,
  position: drl_protocol::Position,
  inset_fraction: f32,
  color: [f32; 4],
) {
  if let Some(rect) = viewport.tile_rect(position) {
    plan.push(SceneQuad {
      rect,
      inset_fraction,
      color,
    });
  }
}

fn append_target_quads(plan: &mut Vec<SceneQuad>, viewport: PixelViewport, scene: &RenderScene) {
  for &target in &scene.target_positions {
    append_quad(plan, viewport, target, 0.08, [1.0, 0.82, 0.18, 0.35]);
  }
}

/// Builds target-overlay geometry from the same renderer-owned policy used by
/// the colored fallback. Textured frontends use this bounded subset after
/// their atlas compositor has drawn the observed scene.
#[must_use]
pub fn target_quad_plan(
  scene: &RenderScene,
  canvas_width: u32,
  canvas_height: u32,
) -> Vec<SceneQuad> {
  let viewport = PixelViewport::fit(
    scene.map_width,
    scene.map_height,
    canvas_width,
    canvas_height,
  );
  let mut plan = Vec::new();
  append_target_quads(&mut plan, viewport, scene);
  plan
}

/// Builds the deterministic colored geometry fallback consumed by browser and
/// native frontend shells.
///
/// Draw order is tiles, items, actors, then target overlays. The source scene
/// is already a fair `RenderScene`; this function does not inspect simulation
/// state or infer hidden entities. Tiles outside the observed/explored set are
/// omitted to preserve the fair presentation boundary for manually built
/// scenes as well as observation-derived scenes.
#[must_use]
pub fn scene_quad_plan(
  scene: &RenderScene,
  canvas_width: u32,
  canvas_height: u32,
) -> Vec<SceneQuad> {
  let viewport = PixelViewport::fit(
    scene.map_width,
    scene.map_height,
    canvas_width,
    canvas_height,
  );
  let mut plan = Vec::new();
  for tile in &scene.tiles {
    if tile.visible || tile.explored {
      append_quad(&mut plan, viewport, tile.position, 0.0, tile_color(tile));
    }
  }
  for item in &scene.items {
    append_quad(
      &mut plan,
      viewport,
      item.position,
      0.28,
      [0.22, 0.75, 0.35, 1.0],
    );
  }
  for actor in &scene.actors {
    let color = if actor.is_player {
      [0.25, 0.75, 0.95, 1.0]
    } else {
      [0.85, 0.25, 0.24, 1.0]
    };
    append_quad(&mut plan, viewport, actor.position, 0.18, color);
  }
  append_target_quads(&mut plan, viewport, scene);
  plan
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{HudState, SceneActor, SceneTile};
  use drl_assets::{actor_sprite, tile_sprite};
  use drl_protocol::{EntityId, HitPoints, MonsterKind, Position};

  fn sample_scene() -> RenderScene {
    RenderScene {
      map_width: 3,
      map_height: 2,
      player_position: Position::new(0, 0),
      target_positions: vec![Position::new(2, 1)],
      tiles: vec![
        SceneTile {
          position: Position::new(0, 0),
          kind: TileKind::Floor,
          visible: true,
          explored: true,
          sprite: tile_sprite(TileKind::Floor),
        },
        SceneTile {
          position: Position::new(1, 0),
          kind: TileKind::Wall,
          visible: false,
          explored: true,
          sprite: tile_sprite(TileKind::Wall),
        },
      ],
      actors: vec![SceneActor {
        id: EntityId(1),
        position: Position::new(0, 0),
        is_player: true,
        hp: Some(HitPoints::full(10)),
        sprite: actor_sprite(Some(MonsterKind::FormerHuman)),
        colorization_tint: [0, 0, 0, 0],
      }],
      items: Vec::new(),
      hud: HudState {
        turn: 0,
        player_hp: Some(HitPoints::full(10)),
        weapon: None,
        armor: None,
        inventory_size: 0,
      },
    }
  }

  #[test]
  fn plan_uses_integer_viewport_and_stable_layer_order() {
    let plan = scene_quad_plan(&sample_scene(), 30, 20);
    assert_eq!(plan.len(), 4);
    assert_eq!(
      plan[0].rect,
      PixelRect {
        x: 0,
        y: 0,
        width: 10,
        height: 10
      }
    );
    assert_eq!(
      plan[1].rect,
      PixelRect {
        x: 10,
        y: 0,
        width: 10,
        height: 10
      }
    );
    assert_eq!(
      plan[2].rect,
      PixelRect {
        x: 0,
        y: 0,
        width: 10,
        height: 10
      }
    );
    assert_eq!(
      plan[3].rect,
      PixelRect {
        x: 20,
        y: 10,
        width: 10,
        height: 10
      }
    );
    assert_eq!(plan[0].inset_fraction, 0.0);
    assert_eq!(plan[2].inset_fraction, 0.18);
    assert_eq!(plan[3].inset_fraction, 0.08);
    assert_eq!(plan[0].color, [0.16, 0.18, 0.22, 1.0]);
    assert_eq!(plan[1].color[0], 0.036);
    assert_eq!(plan[1].color[1], 0.0405);
    assert!((plan[1].color[2] - 0.054).abs() < 1e-6);
    assert_eq!(plan[1].color[3], 1.0);
    assert_eq!(plan[2].color, [0.25, 0.75, 0.95, 1.0]);
    assert_eq!(plan[3].color, [1.0, 0.82, 0.18, 0.35]);
    assert_eq!(target_quad_plan(&sample_scene(), 30, 20), vec![plan[3]]);
  }

  #[test]
  fn plan_omits_unobserved_scene_tiles() {
    let mut scene = sample_scene();
    scene.tiles[1].visible = false;
    scene.tiles[1].explored = false;
    let plan = scene_quad_plan(&scene, 30, 20);
    assert_eq!(plan.len(), 3);
    assert_eq!(plan[0].rect.x, 0);
    assert_eq!(plan[1].inset_fraction, 0.18);
    assert_eq!(plan[2].inset_fraction, 0.08);
  }
}
