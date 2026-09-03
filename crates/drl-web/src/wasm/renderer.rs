//! WebGPU renderer for the platform-neutral `RenderScene`. It owns no
//! simulation state and derives every vertex from fair presentation plans.

use super::*;

/// Minimal WebGPU renderer that owns no simulation state.
pub struct WebGpuRenderer {
  _instance: wgpu::Instance,
  surface: wgpu::Surface<'static>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  pipeline: wgpu::RenderPipeline,
  base_texture: BaseTexturePipeline,
  canvas: HtmlCanvasElement,
  textures: Option<GpuTextureCache>,
  texture_upload_error: Option<String>,
}

const SCENE_SHADER: &str = r#"
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
"#;

impl WebGpuRenderer {
  /// Requests the browser WebGPU adapter for a canvas.
  pub async fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::BROWSER_WEBGPU,
      flags: wgpu::InstanceFlags::default(),
      memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
      backend_options: wgpu::BackendOptions::default(),
      display: None,
    });
    let surface = instance
      .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
      .map_err(|error| JsValue::from_str(&format!("surface creation failed: {error}")))?;
    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
        apply_limit_buckets: true,
      })
      .await
      .map_err(|error| JsValue::from_str(&format!("WebGPU unavailable: {error}")))?;
    let (device, queue) = adapter
      .request_device(&wgpu::DeviceDescriptor {
        label: Some("drl-web-device"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
      })
      .await
      .map_err(|error| JsValue::from_str(&format!("WebGPU device failed: {error}")))?;
    let width = canvas.width().max(1);
    let height = canvas.height().max(1);
    let config = surface
      .get_default_config(&adapter, width, height)
      .ok_or_else(|| JsValue::from_str("WebGPU canvas format unavailable"))?;
    surface.configure(&device, &config);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("drl-web-scene-shader"),
      source: wgpu::ShaderSource::Wgsl(SCENE_SHADER.into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("drl-web-scene-pipeline"),
      layout: None,
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        buffers: &[Some(wgpu::VertexBufferLayout {
          array_stride: 24,
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
          format: config.format,
          blend: Some(wgpu::BlendState::ALPHA_BLENDING),
          write_mask: wgpu::ColorWrites::ALL,
        })],
      }),
      multiview_mask: None,
      cache: None,
    });
    let (textures, texture_upload_error) =
      match GpuTextureCache::load(&device, &queue, texture_source_manifest()).await {
        Ok(cache) => (Some(cache), None),
        Err(error) => (
          None,
          Some(
            error
              .as_string()
              .unwrap_or_else(|| "texture upload failed".to_string()),
          ),
        ),
      };
    let base_texture = BaseTexturePipeline::new(&device, &queue, config.format, textures.as_ref());
    Ok(Self {
      _instance: instance,
      surface,
      device,
      queue,
      config,
      pipeline,
      base_texture,
      canvas,
      textures,
      texture_upload_error,
    })
  }

  /// Returns the number of unique imported sources uploaded at startup.
  pub fn texture_source_count(&self) -> usize {
    self.textures.as_ref().map_or(0, GpuTextureCache::len)
  }

  /// Reports whether a decoded source has a retained GPU view.
  pub fn has_texture_source(&self, source: AtlasTextureSource) -> bool {
    self
      .textures
      .as_ref()
      .is_some_and(|textures| textures.view(source).is_some())
  }

  /// Returns the non-fatal upload error, if geometry fallback is active.
  pub fn texture_upload_error(&self) -> Option<&str> {
    self.texture_upload_error.as_deref()
  }

  /// Resizes only the presentation surface; it never touches simulation.
  pub fn resize(&mut self, width: u32, height: u32, dpr: f64) {
    let scale = dpr.max(1.0);
    self.config.width = ((width as f64) * scale).round().max(1.0) as u32;
    self.config.height = ((height as f64) * scale).round().max(1.0) as u32;
    self.canvas.set_width(self.config.width);
    self.canvas.set_height(self.config.height);
    self.surface.configure(&self.device, &self.config);
  }

  /// Clears the canvas and presents one deterministic frame.
  pub fn render(&self, scene: &RenderScene) -> Result<(), JsValue> {
    self.render_with_elapsed(scene, None, None)
  }

  /// Presents a frame with caller-owned retained particle decals.
  pub fn render_with_particle_decals(
    &self,
    scene: &RenderScene,
    store: &ParticleDecalStore,
    sprites: &[ParticleDecalSprite],
  ) -> Result<(), JsValue> {
    self.render_with_elapsed(scene, None, Some((store, sprites)))
  }

  /// Presents one frame using caller-supplied elapsed animation time.
  ///
  /// The renderer reads no clock and does not schedule redraws; callers own
  /// elapsed-time and playback policy decisions.
  pub fn render_at_elapsed(
    &self,
    scene: &RenderScene,
    elapsed_ms: u64,
    playback: AnimationPlayback,
  ) -> Result<(), JsValue> {
    self.render_with_elapsed(scene, Some((elapsed_ms, playback)), None)
  }

  /// Presents an elapsed-time frame with caller-owned retained decals.
  pub fn render_at_elapsed_with_particle_decals(
    &self,
    scene: &RenderScene,
    elapsed_ms: u64,
    playback: AnimationPlayback,
    store: &ParticleDecalStore,
    sprites: &[ParticleDecalSprite],
  ) -> Result<(), JsValue> {
    self.render_with_elapsed(scene, Some((elapsed_ms, playback)), Some((store, sprites)))
  }

  fn render_with_elapsed(
    &self,
    scene: &RenderScene,
    elapsed: Option<(u64, AnimationPlayback)>,
    particle_decals: Option<(&ParticleDecalStore, &[ParticleDecalSprite])>,
  ) -> Result<(), JsValue> {
    let frame = match self.surface.get_current_texture() {
      wgpu::CurrentSurfaceTexture::Success(frame)
      | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
      status => {
        return Err(JsValue::from_str(&format!(
          "GPU frame unavailable: {status:?}"
        )));
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
        label: Some("drl-web-frame"),
      });
    let attachments = [Some(wgpu::RenderPassColorAttachment {
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
        label: Some("drl-web-clear"),
        color_attachments: &attachments,
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
      });
    }
    let textured_scene = match (elapsed, particle_decals) {
      (None, None) => self
        .base_texture
        .covers_scene(scene, self.config.width, self.config.height),
      (Some((elapsed_ms, playback)), None) => self.base_texture.covers_scene_at_elapsed(
        scene,
        self.config.width,
        self.config.height,
        elapsed_ms,
        playback,
      ),
      (None, Some((store, sprites))) => self.base_texture.covers_scene_with_particle_decals(
        scene,
        self.config.width,
        self.config.height,
        store,
        sprites,
      ),
      (Some((elapsed_ms, playback)), Some((store, sprites))) => {
        self.base_texture.covers_scene_with_selection(
          scene,
          self.config.width,
          self.config.height,
          Some((elapsed_ms, playback)),
          Some((store, sprites)),
        )
      }
    };
    if textured_scene {
      match (elapsed, particle_decals) {
        (None, None) => self.base_texture.draw(
          &self.device,
          &mut encoder,
          &view,
          scene,
          self.config.width,
          self.config.height,
        ),
        (Some((elapsed_ms, playback)), None) => self.base_texture.draw_at_elapsed(
          &self.device,
          &mut encoder,
          &view,
          scene,
          self.config.width,
          self.config.height,
          elapsed_ms,
          playback,
        ),
        (None, Some((store, sprites))) => self.base_texture.draw_with_particle_decals(
          &self.device,
          &mut encoder,
          &view,
          scene,
          self.config.width,
          self.config.height,
          store,
          sprites,
        ),
        (Some((elapsed_ms, playback)), Some((store, sprites))) => {
          self.base_texture.draw_at_elapsed_with_particle_decals(
            &self.device,
            &mut encoder,
            &view,
            scene,
            self.config.width,
            self.config.height,
            elapsed_ms,
            playback,
            store,
            sprites,
          )
        }
      }
    }
    let vertices = if textured_scene {
      target_vertices(scene, self.config.width, self.config.height)
    } else {
      scene_vertices(scene, self.config.width, self.config.height)
    };
    if !vertices.is_empty() {
      let vertex_buffer = self
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
          label: Some("drl-web-scene-vertices"),
          contents: &vertices,
          usage: wgpu::BufferUsages::VERTEX,
        });
      let vertex_count = (vertices.len() / 24) as u32;
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("drl-web-scene"),
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
