//! WebGPU texture resources for the imported legacy layer sources.
//!
//! This module owns browser-only GPU objects. It deliberately accepts the
//! renderer-neutral `AtlasTextureSource` manifest and does not know about
//! simulation state or sprite blend equations.

use drl_assets::AtlasTextureSource;
use drl_render::{PixelRect, PixelViewport, RenderScene, layer_draw_plan, sprite_composite_plan};
use wasm_bindgen::JsValue;
use wgpu::util::DeviceExt;

/// One uploaded source and its view. The texture is retained so the view stays
/// valid for the renderer's lifetime.
struct UploadedTexture {
  source: AtlasTextureSource,
  _texture: wgpu::Texture,
  view: wgpu::TextureView,
}

/// Deterministic cache of decoded legacy sources uploaded to WebGPU.
pub(crate) struct GpuTextureCache {
  entries: Vec<UploadedTexture>,
}

/// One source-specific bind group used by the base-color pass.
pub(crate) struct TextureBinding {
  pub(crate) source: AtlasTextureSource,
  pub(crate) bind_group: wgpu::BindGroup,
}

impl GpuTextureCache {
  /// Decodes and uploads each unique source in manifest order.
  pub(crate) async fn load<I>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    sources: I,
  ) -> Result<Self, JsValue>
  where
    I: IntoIterator<Item = AtlasTextureSource>,
  {
    let mut entries: Vec<UploadedTexture> = Vec::new();
    for source in sources {
      if entries.iter().any(|entry| entry.source == source) {
        continue;
      }
      let image = crate::wasm::load_texture_source(source).await?;
      let size = wgpu::Extent3d {
        width: source.width,
        height: source.height,
        depth_or_array_layers: 1,
      };
      let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(source.path),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      });
      queue.copy_external_image_to_texture(
        &wgpu::CopyExternalImageSourceInfo {
          source: wgpu::ExternalImageSource::HTMLImageElement(image),
          origin: wgpu::Origin2d::ZERO,
          flip_y: false,
        },
        wgpu::CopyExternalImageDestInfo {
          texture: &texture,
          mip_level: 0,
          origin: wgpu::Origin3d::ZERO,
          aspect: wgpu::TextureAspect::All,
          color_space: wgpu::PredefinedColorSpace::Srgb,
          premultiplied_alpha: false,
        },
        size,
      );
      let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
      entries.push(UploadedTexture {
        source,
        _texture: texture,
        view,
      });
    }
    Ok(Self { entries })
  }

  /// Returns a view for one imported source, if it was loaded.
  pub(crate) fn view(&self, source: AtlasTextureSource) -> Option<&wgpu::TextureView> {
    self
      .entries
      .iter()
      .find(|entry| entry.source == source)
      .map(|entry| &entry.view)
  }

  /// Number of unique sources retained by the cache.
  pub(crate) const fn len(&self) -> usize {
    self.entries.len()
  }

  /// Creates stable source-to-view bind groups for a sampler and layout.
  pub(crate) fn bind_groups(
    &self,
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
  ) -> Vec<TextureBinding> {
    self
      .entries
      .iter()
      .map(|entry| TextureBinding {
        source: entry.source,
        bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
          label: Some(entry.source.path),
          layout,
          entries: &[
            wgpu::BindGroupEntry {
              binding: 0,
              resource: wgpu::BindingResource::TextureView(&entry.view),
            },
            wgpu::BindGroupEntry {
              binding: 1,
              resource: wgpu::BindingResource::Sampler(sampler),
            },
          ],
        }),
      })
      .collect()
  }
}

const BASE_TEXTURE_SHADER: &str = r#"
struct VertexInput {
  @location(0) position: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) lighting: vec4<f32>,
};

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) lighting: vec4<f32>,
};

@group(0) @binding(0) var base_texture: texture_2d<f32>;
@group(0) @binding(1) var base_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
  var output: VertexOutput;
  output.position = vec4<f32>(input.position, 0.0, 1.0);
  output.uv = input.uv;
  output.lighting = input.lighting;
  return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
  let sampled = textureSample(base_texture, base_sampler, input.uv);
  return vec4<f32>(sampled.rgb * input.lighting.rgb, sampled.a);
}
"#;

/// Pipeline and source-specific bind groups for the partial base-color pass.
pub(crate) struct BaseTexturePipeline {
  pipeline: wgpu::RenderPipeline,
  bindings: Vec<TextureBinding>,
}

impl BaseTexturePipeline {
  /// Builds a nearest-filtered base-color pipeline and its source bindings.
  pub(crate) fn new(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    cache: Option<&GpuTextureCache>,
  ) -> Self {
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("drl-web-base-texture-layout"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("drl-web-nearest-sampler"),
      mag_filter: wgpu::FilterMode::Nearest,
      min_filter: wgpu::FilterMode::Nearest,
      mipmap_filter: wgpu::MipmapFilterMode::Nearest,
      ..Default::default()
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("drl-web-base-texture-shader"),
      source: wgpu::ShaderSource::Wgsl(BASE_TEXTURE_SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("drl-web-base-texture-pipeline-layout"),
      bind_group_layouts: &[Some(&layout)],
      immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("drl-web-base-texture-pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        buffers: &[Some(wgpu::VertexBufferLayout {
          array_stride: 32,
          step_mode: wgpu::VertexStepMode::Vertex,
          attributes: &[
            wgpu::VertexAttribute {
              format: wgpu::VertexFormat::Float32x2,
              offset: 0,
              shader_location: 0,
            },
            wgpu::VertexAttribute {
              format: wgpu::VertexFormat::Float32x2,
              offset: 8,
              shader_location: 1,
            },
            wgpu::VertexAttribute {
              format: wgpu::VertexFormat::Float32x4,
              offset: 16,
              shader_location: 2,
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
    });
    let bindings = cache.map_or_else(Vec::new, |cache| {
      cache.bind_groups(device, &layout, &sampler)
    });
    Self { pipeline, bindings }
  }

  /// Returns true only when every fair composite can be drawn from the cache.
  pub(crate) fn covers_scene(
    &self,
    scene: &RenderScene,
    canvas_width: u32,
    canvas_height: u32,
  ) -> bool {
    let (vertices, batches) = base_texture_vertices(scene, canvas_width, canvas_height);
    !vertices.is_empty()
      && !self.bindings.is_empty()
      && batches.iter().all(|batch| {
        self
          .bindings
          .iter()
          .any(|binding| binding.source == batch.source)
      })
  }

  /// Draws fair base-color composites over the geometry fallback.
  pub(crate) fn draw(
    &self,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    scene: &RenderScene,
    canvas_width: u32,
    canvas_height: u32,
  ) {
    let (vertices, batches) = base_texture_vertices(scene, canvas_width, canvas_height);
    if vertices.is_empty() || self.bindings.is_empty() {
      return;
    }
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("drl-web-base-texture-vertices"),
      contents: &vertices,
      usage: wgpu::BufferUsages::VERTEX,
    });
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("drl-web-base-texture"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view,
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
    for batch in batches {
      if let Some(binding) = self
        .bindings
        .iter()
        .find(|binding| binding.source == batch.source)
      {
        pass.set_bind_group(0, &binding.bind_group, &[]);
        pass.draw(batch.start..batch.start + batch.count, 0..1);
      }
    }
  }
}

struct TextureBatch {
  source: AtlasTextureSource,
  start: u32,
  count: u32,
}

fn push_texture_vertex(vertices: &mut Vec<u8>, x: f32, y: f32, u: f32, v: f32, lighting: f32) {
  vertices.extend_from_slice(&x.to_ne_bytes());
  vertices.extend_from_slice(&y.to_ne_bytes());
  vertices.extend_from_slice(&u.to_ne_bytes());
  vertices.extend_from_slice(&v.to_ne_bytes());
  for component in [lighting, lighting, lighting, 1.0] {
    vertices.extend_from_slice(&component.to_ne_bytes());
  }
}

fn push_texture_quad(
  vertices: &mut Vec<u8>,
  rect: PixelRect,
  viewport: &PixelViewport,
  uv: drl_assets::SpriteUv,
  lighting: f32,
) {
  let [left, bottom, right, top] =
    crate::base_texture_ndc_rect(rect, viewport.canvas_width, viewport.canvas_height);
  let [[u0, v0], [u1, v1], [u2, v2], [u3, v3], [u4, v4], [u5, v5]] = crate::base_texture_uvs(uv);
  push_texture_vertex(vertices, left, bottom, u0, v0, lighting);
  push_texture_vertex(vertices, right, bottom, u1, v1, lighting);
  push_texture_vertex(vertices, right, top, u2, v2, lighting);
  push_texture_vertex(vertices, left, bottom, u3, v3, lighting);
  push_texture_vertex(vertices, right, top, u4, v4, lighting);
  push_texture_vertex(vertices, left, top, u5, v5, lighting);
}

fn base_texture_vertices(
  scene: &RenderScene,
  canvas_width: u32,
  canvas_height: u32,
) -> (Vec<u8>, Vec<TextureBatch>) {
  let viewport = PixelViewport::fit(
    scene.map_width,
    scene.map_height,
    canvas_width,
    canvas_height,
  );
  let plan = layer_draw_plan(scene, viewport);
  let composites = sprite_composite_plan(&plan);
  let mut vertices = Vec::new();
  let mut batches = Vec::new();
  for composite in composites {
    let start = (vertices.len() / 32) as u32;
    push_texture_quad(
      &mut vertices,
      composite.destination,
      &viewport,
      composite.uv,
      crate::base_texture_lighting_factor(composite.lighting),
    );
    batches.push(TextureBatch {
      source: composite.base,
      start,
      count: 6,
    });
  }
  (vertices, batches)
}
