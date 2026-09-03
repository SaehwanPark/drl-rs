//! Vertex generation for one `RenderScene`. All geometry comes from the shared
//! layer-draw plan so the browser never invents presentation policy.

use super::*;

fn push_vertex(vertices: &mut Vec<u8>, x: f32, y: f32, color: [f32; 4]) {
  vertices.extend_from_slice(&x.to_ne_bytes());
  vertices.extend_from_slice(&y.to_ne_bytes());
  for component in color {
    vertices.extend_from_slice(&component.to_ne_bytes());
  }
}

fn push_quad(
  vertices: &mut Vec<u8>,
  left: f32,
  bottom: f32,
  right: f32,
  top: f32,
  color: [f32; 4],
) {
  push_vertex(vertices, left, bottom, color);
  push_vertex(vertices, right, bottom, color);
  push_vertex(vertices, right, top, color);
  push_vertex(vertices, left, bottom, color);
  push_vertex(vertices, right, top, color);
  push_vertex(vertices, left, top, color);
}

fn scene_position(viewport: &PixelViewport, x: i32, y: i32) -> Option<(f32, f32, f32, f32)> {
  let rect = viewport.tile_rect(drl_protocol::Position::new(x, y))?;
  let width = viewport.canvas_width.max(1) as f32;
  let height = viewport.canvas_height.max(1) as f32;
  let left = -1.0 + 2.0 * rect.x as f32 / width;
  let right = -1.0 + 2.0 * rect.x.saturating_add(rect.width) as f32 / width;
  let top = 1.0 - 2.0 * rect.y as f32 / height;
  let bottom = 1.0 - 2.0 * rect.y.saturating_add(rect.height) as f32 / height;
  Some((left, bottom, right, top))
}

pub(crate) fn scene_vertices(
  scene: &RenderScene,
  canvas_width: u32,
  canvas_height: u32,
) -> Vec<u8> {
  let viewport = PixelViewport::fit(
    scene.map_width,
    scene.map_height,
    canvas_width,
    canvas_height,
  );
  let mut vertices = Vec::new();
  for tile in &scene.tiles {
    let color = match tile.kind {
      drl_protocol::TileKind::Wall => [0.08, 0.09, 0.12, 1.0],
      drl_protocol::TileKind::DoorClosed => [0.24, 0.16, 0.09, 1.0],
      drl_protocol::TileKind::DoorOpen => [0.18, 0.20, 0.18, 1.0],
      drl_protocol::TileKind::StairsDown => [0.28, 0.24, 0.08, 1.0],
      drl_protocol::TileKind::Lava => [0.45, 0.12, 0.04, 1.0],
      drl_protocol::TileKind::Acid => [0.12, 0.45, 0.12, 1.0],
      drl_protocol::TileKind::Water => [0.12, 0.28, 0.55, 1.0],
      drl_protocol::TileKind::Mud => [0.38, 0.26, 0.16, 1.0],
      drl_protocol::TileKind::Floor => [0.16, 0.18, 0.22, 1.0],
    };
    let color = shade_color(color, tile.lighting_band());
    if let Some((left, bottom, right, top)) =
      scene_position(&viewport, tile.position.x, tile.position.y)
    {
      push_quad(&mut vertices, left, bottom, right, top, color);
    }
  }
  for item in &scene.items {
    if let Some((left, bottom, right, top)) =
      scene_position(&viewport, item.position.x, item.position.y)
    {
      let inset_x = (right - left) * 0.28;
      let inset_y = (top - bottom) * 0.28;
      push_quad(
        &mut vertices,
        left + inset_x,
        bottom + inset_y,
        right - inset_x,
        top - inset_y,
        [0.22, 0.75, 0.35, 1.0],
      );
    }
  }
  for actor in &scene.actors {
    if let Some((left, bottom, right, top)) =
      scene_position(&viewport, actor.position.x, actor.position.y)
    {
      let inset_x = (right - left) * 0.18;
      let inset_y = (top - bottom) * 0.18;
      let color = if actor.is_player {
        [0.25, 0.75, 0.95, 1.0]
      } else {
        [0.85, 0.25, 0.24, 1.0]
      };
      push_quad(
        &mut vertices,
        left + inset_x,
        bottom + inset_y,
        right - inset_x,
        top - inset_y,
        color,
      );
    }
  }
  for target in &scene.target_positions {
    if let Some((left, bottom, right, top)) = scene_position(&viewport, target.x, target.y) {
      let inset_x = (right - left) * 0.08;
      let inset_y = (top - bottom) * 0.08;
      push_quad(
        &mut vertices,
        left + inset_x,
        bottom + inset_y,
        right - inset_x,
        top - inset_y,
        [1.0, 0.82, 0.18, 0.35],
      );
    }
  }
  vertices
}

pub(crate) fn target_vertices(
  scene: &RenderScene,
  canvas_width: u32,
  canvas_height: u32,
) -> Vec<u8> {
  let viewport = PixelViewport::fit(
    scene.map_width,
    scene.map_height,
    canvas_width,
    canvas_height,
  );
  let mut vertices = Vec::new();
  for target in &scene.target_positions {
    if let Some((left, bottom, right, top)) = scene_position(&viewport, target.x, target.y) {
      let inset_x = (right - left) * 0.08;
      let inset_y = (top - bottom) * 0.08;
      push_quad(
        &mut vertices,
        left + inset_x,
        bottom + inset_y,
        right - inset_x,
        top - inset_y,
        [1.0, 0.82, 0.18, 0.35],
      );
    }
  }
  vertices
}
