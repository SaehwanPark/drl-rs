//! Minimal native `wgpu` renderer for the shared fallback scene.

use std::sync::Arc;

use drl_render::{RenderScene, SceneQuad, scene_clear_color, scene_quad_plan};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;

const VERTEX_STRIDE: u64 = 24;

/// Surface outcomes the event-loop shell can handle without touching gameplay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceStatus {
  Timeout,
  Occluded,
  Outdated,
  Lost,
  Validation,
}

/// Presentation failure returned by the native renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
  Surface(SurfaceStatus),
}

/// Clamps a physical window extent for safe surface configuration.
///
/// The input is already in physical framebuffer pixels. In particular, a
/// compositor scale factor must not be multiplied into this value again.
#[must_use]
pub fn framebuffer_extent(size: PhysicalSize<u32>) -> PhysicalSize<u32> {
  PhysicalSize::new(size.width.max(1), size.height.max(1))
}

/// Native surface/device/pipeline owner. It owns no simulation state.
pub struct DesktopRenderer {
  _instance: wgpu::Instance,
  surface: wgpu::Surface<'static>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  pipeline: wgpu::RenderPipeline,
}

impl DesktopRenderer {
  /// Creates a Vulkan-or-Metal renderer for a native window.
  pub fn new(window: Arc<Window>) -> Result<Self, String> {
    pollster::block_on(Self::new_async(window))
  }

  async fn new_async(window: Arc<Window>) -> Result<Self, String> {
    let size = framebuffer_extent(window.inner_size());
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::VULKAN | wgpu::Backends::METAL,
      flags: wgpu::InstanceFlags::default(),
      memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
      backend_options: wgpu::BackendOptions::default(),
      display: None,
    });
    let surface = instance
      .create_surface(window)
      .map_err(|error| format!("surface creation failed: {error}"))?;
    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
        apply_limit_buckets: true,
      })
      .await
      .map_err(|error| format!("native GPU unavailable: {error}"))?;
    let (device, queue) = adapter
      .request_device(&wgpu::DeviceDescriptor {
        label: Some("drl-desktop-device"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
      })
      .await
      .map_err(|error| format!("native GPU device failed: {error}"))?;
    let mut config = surface
      .get_default_config(&adapter, size.width, size.height)
      .ok_or_else(|| "native surface has no compatible format".to_string())?;
    config.width = size.width;
    config.height = size.height;
    surface.configure(&device, &config);
    let pipeline = create_pipeline(&device, config.format);
    Ok(Self {
      _instance: instance,
      surface,
      device,
      queue,
      config,
      pipeline,
    })
  }

  /// Resizes the presentation surface from physical window pixels only.
  pub fn resize(&mut self, size: PhysicalSize<u32>) {
    let size = framebuffer_extent(size);
    self.config.width = size.width;
    self.config.height = size.height;
    self.surface.configure(&self.device, &self.config);
  }

  /// Renders one fair `RenderScene` and presents it to the native surface.
  pub fn render(&self, scene: &RenderScene) -> Result<(), RenderError> {
    let frame = match self.surface.get_current_texture() {
      wgpu::CurrentSurfaceTexture::Success(frame)
      | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
      wgpu::CurrentSurfaceTexture::Timeout => {
        return Err(RenderError::Surface(SurfaceStatus::Timeout));
      }
      wgpu::CurrentSurfaceTexture::Occluded => {
        return Err(RenderError::Surface(SurfaceStatus::Occluded));
      }
      wgpu::CurrentSurfaceTexture::Outdated => {
        return Err(RenderError::Surface(SurfaceStatus::Outdated));
      }
      wgpu::CurrentSurfaceTexture::Lost => {
        return Err(RenderError::Surface(SurfaceStatus::Lost));
      }
      wgpu::CurrentSurfaceTexture::Validation => {
        return Err(RenderError::Surface(SurfaceStatus::Validation));
      }
    };
    let view = frame
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());
    let [r, g, b, a] = scene_clear_color(scene.hud.player_hp);
    let clear = wgpu::Color {
      r: f64::from(r),
      g: f64::from(g),
      b: f64::from(b),
      a: f64::from(a),
    };
    let mut encoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("drl-desktop-frame"),
      });
    let clear_attachments = [Some(wgpu::RenderPassColorAttachment {
      view: &view,
      depth_slice: None,
      resolve_target: None,
      ops: wgpu::Operations {
        load: wgpu::LoadOp::Clear(clear),
        store: wgpu::StoreOp::Store,
      },
    })];
    {
      let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("drl-desktop-clear"),
        color_attachments: &clear_attachments,
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
      });
    }

    let vertices = scene_vertex_bytes(scene, self.config.width, self.config.height);
    if !vertices.is_empty() {
      let vertex_buffer = self
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
          label: Some("drl-desktop-scene-vertices"),
          contents: &vertices,
          usage: wgpu::BufferUsages::VERTEX,
        });
      let vertex_count = (vertices.len() / VERTEX_STRIDE as usize) as u32;
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("drl-desktop-scene"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
      });
      pass.set_pipeline(&self.pipeline);
      pass.set_vertex_buffer(0, vertex_buffer.slice(..));
      pass.draw(0..vertex_count, 0..1);
    }
    self.queue.submit([encoder.finish()]);
    self.queue.present(frame);
    Ok(())
  }
}

fn create_pipeline(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
  let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("drl-desktop-scene-shader"),
    source: wgpu::ShaderSource::Wgsl(
      r#"
struct VertexInput {
  @location(0) position: vec2<f32>,
  @location(1) color: vec4<f32>,
};

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
  var output: VertexOutput;
  output.position = vec4<f32>(input.position, 0.0, 1.0);
  output.color = input.color;
  return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
  return input.color;
}
"#
      .into(),
    ),
  });
  device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("drl-desktop-scene-pipeline"),
    layout: None,
    vertex: wgpu::VertexState {
      module: &shader,
      entry_point: Some("vs_main"),
      compilation_options: wgpu::PipelineCompilationOptions::default(),
      buffers: &[Some(wgpu::VertexBufferLayout {
        array_stride: VERTEX_STRIDE,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
          wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: 0,
            shader_location: 0,
          },
          wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 8,
            shader_location: 1,
          },
        ],
      })],
    },
    primitive: wgpu::PrimitiveState::default(),
    depth_stencil: None,
    multisample: wgpu::MultisampleState::default(),
    fragment: Some(wgpu::FragmentState {
      module: &shader,
      entry_point: Some("fs_main"),
      compilation_options: wgpu::PipelineCompilationOptions::default(),
      targets: &[Some(wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
      })],
    }),
    multiview_mask: None,
    cache: None,
  })
}

fn push_vertex(vertices: &mut Vec<u8>, x: f32, y: f32, color: [f32; 4]) {
  vertices.extend_from_slice(&x.to_ne_bytes());
  vertices.extend_from_slice(&y.to_ne_bytes());
  for component in color {
    vertices.extend_from_slice(&component.to_ne_bytes());
  }
}

fn push_quad(vertices: &mut Vec<u8>, quad: SceneQuad, width: u32, height: u32) {
  let width = width.max(1) as f32;
  let height = height.max(1) as f32;
  let left = -1.0 + 2.0 * quad.rect.x as f32 / width;
  let right = -1.0 + 2.0 * quad.rect.x.saturating_add(quad.rect.width) as f32 / width;
  let top = 1.0 - 2.0 * quad.rect.y as f32 / height;
  let bottom = 1.0 - 2.0 * quad.rect.y.saturating_add(quad.rect.height) as f32 / height;
  let inset_x = (right - left) * quad.inset_fraction;
  let inset_y = (top - bottom) * quad.inset_fraction;
  let left = left + inset_x;
  let right = right - inset_x;
  let bottom = bottom + inset_y;
  let top = top - inset_y;
  push_vertex(vertices, left, bottom, quad.color);
  push_vertex(vertices, right, bottom, quad.color);
  push_vertex(vertices, right, top, quad.color);
  push_vertex(vertices, left, bottom, quad.color);
  push_vertex(vertices, right, top, quad.color);
  push_vertex(vertices, left, top, quad.color);
}

fn scene_vertex_bytes(scene: &RenderScene, width: u32, height: u32) -> Vec<u8> {
  let mut vertices = Vec::new();
  for quad in scene_quad_plan(scene, width, height) {
    push_quad(&mut vertices, quad, width, height);
  }
  vertices
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::session::{DesktopSession, demo_scenario};
  use drl_render::scene_quad_plan;

  #[test]
  fn physical_extent_is_clamped_without_scale_factor_reapplication() {
    assert_eq!(
      framebuffer_extent(PhysicalSize::new(0, 0)),
      PhysicalSize::new(1, 1)
    );
    assert_eq!(
      framebuffer_extent(PhysicalSize::new(1440, 810)),
      PhysicalSize::new(1440, 810)
    );
  }

  #[test]
  fn native_vertices_are_derived_from_shared_scene_quads() {
    let scenario = demo_scenario().expect("demo scenario");
    let session = DesktopSession::new(&scenario).expect("desktop session");
    let scene = session.scene();
    let quads = scene_quad_plan(&scene, 960, 640);
    assert_eq!(
      scene_vertex_bytes(&scene, 960, 640).len(),
      quads.len() * 6 * VERTEX_STRIDE as usize
    );
  }
}
