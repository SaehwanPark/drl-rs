//! Vertex generation for one `RenderScene`. All geometry comes from the shared
//! layer-draw plan so the browser never invents presentation policy.

use super::*;
use drl_render::{PixelRect, scene_quad_plan, target_quad_plan};

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

fn normalized_rect(canvas_width: u32, canvas_height: u32, rect: PixelRect) -> (f32, f32, f32, f32) {
  let width = canvas_width.max(1) as f32;
  let height = canvas_height.max(1) as f32;
  let left = -1.0 + 2.0 * rect.x as f32 / width;
  let right = -1.0 + 2.0 * rect.x.saturating_add(rect.width) as f32 / width;
  let top = 1.0 - 2.0 * rect.y as f32 / height;
  let bottom = 1.0 - 2.0 * rect.y.saturating_add(rect.height) as f32 / height;
  (left, bottom, right, top)
}

fn push_scene_quad(
  vertices: &mut Vec<u8>,
  quad: drl_render::SceneQuad,
  canvas_width: u32,
  canvas_height: u32,
) {
  let (left, bottom, right, top) = normalized_rect(canvas_width, canvas_height, quad.rect);
  let inset_x = (right - left) * quad.inset_fraction;
  let inset_y = (top - bottom) * quad.inset_fraction;
  push_quad(
    vertices,
    left + inset_x,
    bottom + inset_y,
    right - inset_x,
    top - inset_y,
    quad.color,
  );
}

pub(crate) fn scene_vertices(
  scene: &RenderScene,
  canvas_width: u32,
  canvas_height: u32,
) -> Vec<u8> {
  let mut vertices = Vec::new();
  for quad in scene_quad_plan(scene, canvas_width, canvas_height) {
    push_scene_quad(&mut vertices, quad, canvas_width, canvas_height);
  }
  vertices
}

pub(crate) fn target_vertices(
  scene: &RenderScene,
  canvas_width: u32,
  canvas_height: u32,
) -> Vec<u8> {
  let mut vertices = Vec::new();
  for quad in target_quad_plan(scene, canvas_width, canvas_height) {
    push_scene_quad(&mut vertices, quad, canvas_width, canvas_height);
  }
  vertices
}
