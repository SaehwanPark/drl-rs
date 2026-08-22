//! Browser-first DRL-Rust session boundary.
//!
//! `BrowserSession` is intentionally usable on native hosts for deterministic
//! tests. The WASM exports are a thin boot/input shell; gameplay state stays in
//! Rust and is never mirrored into a parallel JavaScript model.

use drl_assets::{AtlasId, AtlasTextureSource};
use drl_core::item::Item;
use drl_core::{Game, Tile};
use drl_protocol::{
  Command, Direction, ItemId, ItemSpawnKind, ItemSpawnSpec, MonsterKind, MonsterSpawnSpec,
  PlayerObservation, Position, ReplayLog,
};
use drl_render::{PresentationStep, RenderScene, effect_timeline_for_observations};

/// Fixed deterministic content slice used by the first browser playthrough.
pub const M4_SEED: u64 = 0x4452_4c5f_4d34;
pub const M4_WIDTH: u32 = 24;
pub const M4_HEIGHT: u32 = 16;
pub const M4_START: Position = Position::new(4, 8);

/// Static bundle root used by the browser texture loader.
pub const GRAPHICS_ASSET_ROOT: &str = "assets/legacy/drl/graphics/";

const REGISTERED_ATLASES: [AtlasId; 7] = [
  AtlasId::Dguy,
  AtlasId::Enemies,
  AtlasId::EnemiesBig,
  AtlasId::GunsAndPickups,
  AtlasId::Levels,
  AtlasId::DoorsAndDecorations,
  AtlasId::Fx,
];

/// Returns every unique imported layer source in stable atlas registration
/// order. A browser uploader can use this manifest without inspecting scenes.
#[must_use]
pub fn texture_source_manifest() -> Vec<AtlasTextureSource> {
  let mut sources = Vec::new();
  for atlas in REGISTERED_ATLASES {
    for &layer in atlas.layers() {
      let source = atlas.texture_source(layer);
      if !sources.contains(&source) {
        sources.push(source);
      }
    }
  }
  sources
}

/// A rejected browser asset path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureSourcePathError {
  pub path: String,
}

impl std::fmt::Display for TextureSourcePathError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(formatter, "invalid texture source path: {}", self.path)
  }
}

/// Validates a relative imported-asset basename for subpath-safe loading.
pub fn browser_asset_url(path: &str) -> Result<String, TextureSourcePathError> {
  let valid = !path.is_empty()
    && !path.starts_with('/')
    && !path.contains("..")
    && !path.contains(['\\', '/', '?', '#'])
    && !path.contains("://")
    && path
      .bytes()
      .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
  if !valid {
    return Err(TextureSourcePathError {
      path: path.to_string(),
    });
  }
  Ok(format!("{GRAPHICS_ASSET_ROOT}{path}"))
}

/// Returns the same-origin URL for an imported atlas layer.
pub fn texture_source_url(source: AtlasTextureSource) -> Result<String, TextureSourcePathError> {
  browser_asset_url(source.path)
}

/// A decoded-source dimension mismatch at the browser asset boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureSourceDimensionsError {
  pub path: &'static str,
  pub expected: (u32, u32),
  pub actual: (u32, u32),
}

impl std::fmt::Display for TextureSourceDimensionsError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      formatter,
      "texture {} has dimensions {}x{}, expected {}x{}",
      self.path, self.actual.0, self.actual.1, self.expected.0, self.expected.1
    )
  }
}

/// Validates decoded image dimensions against the pinned asset metadata.
pub fn validate_texture_source_dimensions(
  source: AtlasTextureSource,
  actual_width: u32,
  actual_height: u32,
) -> Result<(), TextureSourceDimensionsError> {
  let expected = (source.width, source.height);
  if expected == (actual_width, actual_height) {
    Ok(())
  } else {
    Err(TextureSourceDimensionsError {
      path: source.path,
      expected,
      actual: (actual_width, actual_height),
    })
  }
}

/// A browser-facing simulation session with transactional command handling.
#[derive(Debug, Clone)]
pub struct BrowserSession {
  game: Game,
  last_error: Option<String>,
  commands: Vec<Command>,
}

impl BrowserSession {
  /// Creates the fixed M4 arena and its representative loot/combat content.
  pub fn new() -> Result<Self, drl_protocol::CommandError> {
    Ok(Self {
      game: Self::fixed_game()?,
      last_error: None,
      commands: Vec::new(),
    })
  }

  /// Builds the same fixed content for direct-core parity tests and tools.
  pub fn fixed_game() -> Result<Game, drl_protocol::CommandError> {
    let mut game = Game::new(M4_SEED, M4_WIDTH, M4_HEIGHT, M4_START)?;
    let stairs = Position::new(19, 8);
    game
      .world_mut()
      .map_mut()
      .set_tile(stairs, Tile::StairsDown);

    let loot_position = Position::new(7, 8);
    for kind in [
      drl_protocol::ItemSpawnKind::Shotgun,
      drl_protocol::ItemSpawnKind::GreenArmor,
      drl_protocol::ItemSpawnKind::SmallMedPack,
    ] {
      let id = game.world_mut().allocate_item_id();
      game
        .world_mut()
        .spawn_ground_item(loot_position, Item::from_spawn_kind(id, kind))?;
    }

    let monster_position = Position::new(13, 8);
    let id = game.world_mut().allocate_entity_id();
    let monster = drl_core::Actor::from_monster_kind(id, monster_position, MonsterKind::Imp);
    game.world_mut().actors_mut().insert(id, monster);
    Ok(game)
  }

  /// Returns the current fair player observation.
  #[must_use]
  pub fn observation(&self) -> PlayerObservation {
    self.game.observe_player()
  }

  /// Returns the current render scene derived from the fair observation.
  #[must_use]
  pub fn scene(&self) -> RenderScene {
    RenderScene::from_observation(&self.observation())
  }

  /// Returns the most recent rejected-command message, if any.
  #[must_use]
  pub fn last_error(&self) -> Option<&str> {
    self.last_error.as_deref()
  }

  /// Returns true after the deterministic session reaches player death.
  #[must_use]
  pub fn is_game_over(&self) -> bool {
    self.game.is_game_over()
  }

  /// Submits one semantic command. Failed commands roll back the session.
  pub fn submit(&mut self, command: Command) -> Result<PresentationStep, String> {
    let before = self.observation();
    let checkpoint = self.game.clone();
    match self.game.step(command) {
      Ok(events) => {
        self.last_error = None;
        self.commands.push(command);
        let after = self.observation();
        let effects = effect_timeline_for_observations(&before, &after, &events);
        Ok(PresentationStep {
          before,
          command,
          events,
          effects,
          after,
        })
      }
      Err(error) => {
        self.game = checkpoint;
        let message = error.to_string();
        self.last_error = Some(message.clone());
        Err(message)
      }
    }
  }

  /// Restarts the deterministic M4 session.
  pub fn restart(&mut self) -> Result<(), drl_protocol::CommandError> {
    *self = Self::new()?;
    Ok(())
  }

  /// Returns a replay-schema representation of the fixed browser session.
  ///
  /// The log uses the existing V1 schema; it does not create a browser-specific
  /// wire format or expose authoritative state to JavaScript.
  #[must_use]
  pub fn replay_log(&self) -> ReplayLog {
    let mut replay = ReplayLog::new(M4_SEED, M4_WIDTH, M4_HEIGHT, M4_START);
    replay.record_stairs(Position::new(19, 8));
    replay.record_monster(
      MonsterSpawnSpec::new(
        Position::new(13, 8),
        "Imp",
        MonsterKind::Imp.default_hp(),
        MonsterKind::Imp.default_speed(),
        MonsterKind::Imp.default_melee_damage(),
      )
      .with_ranged_combat((5, 10), 8, 70)
      .with_death_drop(Some(ItemSpawnKind::SmallMedPack)),
    );
    let loot_position = Position::new(7, 8);
    replay.record_item(ItemSpawnSpec::new(loot_position, ItemSpawnKind::Shotgun));
    replay.record_item(ItemSpawnSpec::new(loot_position, ItemSpawnKind::GreenArmor));
    replay.record_item(ItemSpawnSpec::new(
      loot_position,
      ItemSpawnKind::SmallMedPack,
    ));
    for command in &self.commands {
      replay.record_command(*command);
    }
    replay
  }

  /// Maps keyboard names to semantic commands without advancing the game.
  #[must_use]
  pub fn command_for_key(key: &str, observation: &PlayerObservation) -> Option<Command> {
    let direction = match key {
      "ArrowUp" | "w" | "W" | "8" => Some(Direction::North),
      "ArrowRight" | "d" | "D" | "6" => Some(Direction::East),
      "ArrowDown" | "s" | "S" | "2" => Some(Direction::South),
      "ArrowLeft" | "a" | "A" | "4" => Some(Direction::West),
      "7" => Some(Direction::NorthWest),
      "9" => Some(Direction::NorthEast),
      "1" => Some(Direction::SouthWest),
      "3" => Some(Direction::SouthEast),
      _ => None,
    };
    if let Some(direction) = direction {
      return Some(Command::Move(direction));
    }
    match key {
      "." | "5" | "Space" => Some(Command::Wait),
      "g" | "G" => Some(Command::Pickup),
      "r" | "R" => Some(Command::Reload),
      ">" => Some(Command::Descend),
      "f" | "F" => observation
        .visible_actors
        .iter()
        .find(|actor| !actor.is_player)
        .map(|actor| Command::AttackRanged(actor.position)),
      _ => None,
    }
  }

  /// Creates an explicit ranged target command for a DOM/canvas click.
  #[must_use]
  pub const fn target_command(position: Position, confirmed: bool) -> Option<Command> {
    if confirmed {
      Some(Command::AttackRanged(position))
    } else {
      None
    }
  }

  /// Maps an inventory action from a semantic item id.
  #[must_use]
  pub const fn inventory_command(action: InventoryAction, item_id: ItemId) -> Command {
    match action {
      InventoryAction::Equip => Command::Equip(item_id),
      InventoryAction::Use => Command::Use(item_id),
      InventoryAction::Drop => Command::Drop(item_id),
    }
  }
}

/// DOM inventory action supported by the first slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryAction {
  Equip,
  Use,
  Drop,
}

/// Browser GPU backend state exposed to the DOM error screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuStatus {
  Ready,
  Unsupported,
  Lost,
}

#[cfg(target_arch = "wasm32")]
mod texture;

#[cfg(target_arch = "wasm32")]
pub(crate) mod wasm {
  use super::texture::GpuTextureCache;
  use super::*;
  use drl_render::{PixelViewport, scene_clear_color, shade_color};
  use std::cell::RefCell;
  use wasm_bindgen::prelude::*;
  use wasm_bindgen_futures::JsFuture;
  use web_sys::{HtmlCanvasElement, HtmlImageElement, Window};
  use wgpu::util::DeviceExt;
  use winit::application::ApplicationHandler;
  use winit::event::{ElementState, WindowEvent};
  use winit::event_loop::{ActiveEventLoop, EventLoop};
  use winit::keyboard::{KeyCode, PhysicalKey};
  use winit::platform::web::{EventLoopExtWebSys, WindowAttributesExtWebSys};
  use winit::window::{Window as WinitWindow, WindowId};

  thread_local! {
    static SESSION: RefCell<Option<BrowserSession>> = const { RefCell::new(None) };
    static RENDERER: RefCell<Option<WebGpuRenderer>> = const { RefCell::new(None) };
    static AUDIO: RefCell<Option<drl_audio::WebAudioMixer>> = const { RefCell::new(None) };
    static TARGET: RefCell<Option<Position>> = const { RefCell::new(None) };
  }

  /// Loads and decodes one same-origin imported atlas layer.
  ///
  /// The returned DOM image is ready for a future WebGPU upload. Dimensions
  /// are checked against the pinned manifest before the image crosses the
  /// renderer boundary.
  pub async fn load_texture_source(
    source: AtlasTextureSource,
  ) -> Result<HtmlImageElement, JsValue> {
    let image = HtmlImageElement::new()?;
    let url = texture_source_url(source).map_err(|error| JsValue::from_str(&error.to_string()))?;
    image.set_src(&url);
    JsFuture::from(image.decode()).await?;
    validate_texture_source_dimensions(source, image.natural_width(), image.natural_height())
      .map_err(|error| JsValue::from_str(&error.to_string()))?;
    // WebGPU's external-image source reports the element's pixel dimensions;
    // pin them to the validated manifest before issuing the copy.
    image.set_width(source.width);
    image.set_height(source.height);
    Ok(image)
  }

  /// Minimal WebGPU renderer that owns no simulation state.
  pub struct WebGpuRenderer {
    _instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
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
      Ok(Self {
        _instance: instance,
        surface,
        device,
        queue,
        config,
        pipeline,
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
      let vertices = scene_vertices(scene, self.config.width, self.config.height);
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

  fn scene_vertices(scene: &RenderScene, canvas_width: u32, canvas_height: u32) -> Vec<u8> {
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

  struct WinitInputApp {
    canvas: Option<HtmlCanvasElement>,
    window: Option<WinitWindow>,
  }

  impl WinitInputApp {
    fn new(canvas: HtmlCanvasElement) -> Self {
      Self {
        canvas: Some(canvas),
        window: None,
      }
    }
  }

  fn key_name(code: KeyCode) -> Option<&'static str> {
    Some(match code {
      KeyCode::ArrowUp => "ArrowUp",
      KeyCode::ArrowRight => "ArrowRight",
      KeyCode::ArrowDown => "ArrowDown",
      KeyCode::ArrowLeft => "ArrowLeft",
      KeyCode::KeyW => "w",
      KeyCode::KeyA => "a",
      KeyCode::KeyS => "s",
      KeyCode::KeyD => "d",
      KeyCode::Numpad8 => "8",
      KeyCode::Numpad6 => "6",
      KeyCode::Numpad2 => "2",
      KeyCode::Numpad4 => "4",
      KeyCode::Numpad7 => "7",
      KeyCode::Numpad9 => "9",
      KeyCode::Numpad1 => "1",
      KeyCode::Numpad3 => "3",
      KeyCode::Numpad5 => "5",
      KeyCode::NumpadDecimal => ".",
      KeyCode::Period => ".",
      KeyCode::Space => "Space",
      KeyCode::Enter | KeyCode::NumpadEnter => "Enter",
      KeyCode::Escape => "Escape",
      KeyCode::KeyG => "g",
      KeyCode::KeyR => "r",
      KeyCode::KeyF => "f",
      KeyCode::BracketRight => ">",
      _ => return None,
    })
  }

  impl ApplicationHandler for WinitInputApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
      if self.window.is_some() {
        return;
      }
      let Some(canvas) = self.canvas.take() else {
        return;
      };
      let attributes = WinitWindow::default_attributes()
        .with_canvas(Some(canvas))
        .with_focusable(true)
        .with_prevent_default(true);
      match event_loop.create_window(attributes) {
        Ok(window) => self.window = Some(window),
        Err(error) => {
          if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            set_status(&document, &format!("Browser input unavailable: {error}"));
          }
        }
      }
    }

    fn window_event(
      &mut self,
      _event_loop: &ActiveEventLoop,
      _window_id: WindowId,
      event: WindowEvent,
    ) {
      match event {
        WindowEvent::KeyboardInput { event, .. }
          if event.state == ElementState::Pressed && !event.repeat =>
        {
          let PhysicalKey::Code(code) = event.physical_key else {
            return;
          };
          if let Some(key) = key_name(code) {
            let message = dispatch_key(key);
            if let Some(document) = web_sys::window().and_then(|window| window.document()) {
              set_status(&document, &message);
            }
          }
        }
        WindowEvent::Resized(size) => resize(size.width, size.height, 1.0),
        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
          if let Some(window) = self.window.as_ref() {
            let size = window.inner_size();
            // `inner_size` is already physical pixels here. Applying the
            // scale factor again would double-count Retina/zoom changes.
            let _ = scale_factor;
            resize(size.width, size.height, 1.0);
          }
        }
        _ => {}
      }
    }
  }

  fn update_dom(document: &web_sys::Document, observation: &PlayerObservation) {
    if let Some(hp) = document.get_element_by_id("game-hp") {
      let value = observation.player_hp.map_or_else(
        || "HP: —".to_string(),
        |hp| format!("HP: {}/{}", hp.current, hp.max),
      );
      hp.set_text_content(Some(&value));
    }
    if let Some(turn) = document.get_element_by_id("game-turn") {
      turn.set_text_content(Some(&format!("Turn: {}", observation.turn.count)));
    }
    if let Some(weapon) = document.get_element_by_id("game-weapon") {
      let value = observation.equipped_weapon.as_ref().map_or_else(
        || "Weapon: —".to_string(),
        |item| format!("Weapon: {}", item.name),
      );
      weapon.set_text_content(Some(&value));
    }
    if let Some(targets) = document.get_element_by_id("target-indicator") {
      let count = observation
        .visible_actors
        .iter()
        .filter(|actor| !actor.is_player)
        .count();
      let value = if count == 0 {
        "Targets: none visible".to_string()
      } else {
        format!("Targets: {count} visible (F selects nearest)")
      };
      targets.set_text_content(Some(&value));
    }
    if let Some(inventory) = document.get_element_by_id("inventory") {
      let controls = observation
        .inventory
        .iter()
        .map(|item| {
          format!(
            "<p>{}</p><button type=\"button\" data-action=\"equip\" data-item-id=\"{}\">Equip</button><button type=\"button\" data-action=\"use\" data-item-id=\"{}\">Use</button><button type=\"button\" data-action=\"drop\" data-item-id=\"{}\">Drop</button>",
            item.name,
            item.id.as_u64(),
            item.id.as_u64(),
            item.id.as_u64()
          )
        })
        .collect::<Vec<_>>()
        .join("");
      inventory.set_inner_html(&controls);
    }
  }

  fn update_target_status(document: &web_sys::Document, message: &str) {
    if let Some(targets) = document.get_element_by_id("target-indicator") {
      targets.set_text_content(Some(message));
    }
  }

  fn set_status(document: &web_sys::Document, message: &str) {
    if let Some(status) = document.get_element_by_id("game-status") {
      status.set_text_content(Some(message));
    }
    if let Some(log) = document.get_element_by_id("game-log") {
      log.set_text_content(Some(message));
    }
  }

  fn render_scene(scene: &RenderScene) {
    let result = RENDERER.with(|renderer_slot| {
      renderer_slot
        .borrow()
        .as_ref()
        .map_or(Ok(()), |renderer| renderer.render(scene))
    });
    if let Err(error) = result
      && let Some(document) = web_sys::window().and_then(|window| window.document())
    {
      set_status(
        &document,
        &format!("WebGPU presentation unavailable; gameplay is unchanged: {error:?}"),
      );
    }
  }

  /// Starts the browser shell after the HTML start button has granted audio.
  #[wasm_bindgen]
  pub async fn boot() -> Result<JsValue, JsValue> {
    let window: Window =
      web_sys::window().ok_or_else(|| JsValue::from_str("window unavailable"))?;
    let document = window
      .document()
      .ok_or_else(|| JsValue::from_str("document unavailable"))?;
    let canvas = document
      .get_element_by_id("game-canvas")
      .ok_or_else(|| JsValue::from_str("#game-canvas is missing"))?
      .dyn_into::<HtmlCanvasElement>()?;
    canvas.set_width(768);
    canvas.set_height(512);
    let session = BrowserSession::new().map_err(|error| JsValue::from_str(&error.to_string()))?;
    let turn = session.observation().turn.count;
    let renderer = WebGpuRenderer::new(canvas.clone()).await?;
    renderer.render(&session.scene())?;
    let texture_count = renderer.texture_source_count();
    let texture_upload_error = renderer.texture_upload_error().map(str::to_owned);
    // Audio is an optional presentation effect. Browser policy, an unavailable
    // AudioContext, or a suspended context must never prevent the simulation
    // session from starting or accepting commands.
    let mut mixer = drl_audio::WebAudioMixer::new().ok();
    let audio_unlocked = if let Some(mixer) = mixer.as_mut() {
      mixer.unlock().await.is_ok()
    } else {
      false
    };
    let audio_available = mixer.is_some();
    SESSION.with(|slot| *slot.borrow_mut() = Some(session));
    RENDERER.with(|slot| *slot.borrow_mut() = Some(renderer));
    AUDIO.with(|slot| *slot.borrow_mut() = mixer);
    TARGET.with(|slot| *slot.borrow_mut() = None);
    let event_loop = EventLoop::new()
      .map_err(|error| JsValue::from_str(&format!("input loop unavailable: {error}")))?;
    event_loop.spawn_app(WinitInputApp::new(canvas));
    let status = document
      .get_element_by_id("game-status")
      .ok_or_else(|| JsValue::from_str("#game-status is missing"))?;
    let audio_message = match (audio_available, audio_unlocked) {
      (true, true) => "Ready — use arrows/WASD or numpad. Audio is gesture-gated.",
      (true, false) => "Ready — use arrows/WASD or numpad. Audio is suspended; gameplay continues.",
      (false, _) => "Ready — use arrows/WASD or numpad. Audio is unavailable; gameplay continues.",
    };
    let message = match texture_upload_error {
      Some(error) => {
        format!("{audio_message} Texture upload unavailable; geometry fallback active ({error}).")
      }
      None => format!("{audio_message} Textures uploaded: {texture_count}."),
    };
    status.set_text_content(Some(&message));
    SESSION.with(|slot| {
      if let Some(session) = slot.borrow().as_ref() {
        update_dom(&document, &session.observation());
      }
    });
    Ok(JsValue::from_str(&format!("turn={turn}")))
  }

  /// A small exported key contract used by the HTML shell and WASM tests.
  #[wasm_bindgen]
  pub fn key_command(key: &str) -> String {
    let observation = BrowserSession::new().expect("fixed session").observation();
    BrowserSession::command_for_key(key, &observation)
      .map_or_else(|| "none".to_string(), |command| format!("{command:?}"))
  }

  /// Submits one focused keyboard command and redraws without exposing game
  /// state to JavaScript.
  #[wasm_bindgen]
  pub fn dispatch_key(key: &str) -> String {
    SESSION.with(|session_slot| {
      let mut session_ref = session_slot.borrow_mut();
      let Some(session) = session_ref.as_mut() else {
        return "Press Start first.".to_string();
      };
      let observation = session.observation();
      if key == "Escape" {
        TARGET.with(|target_slot| *target_slot.borrow_mut() = None);
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
          update_target_status(&document, "Targets: selection cancelled");
        }
        return "Targeting cancelled.".to_string();
      }
      if key == "f" || key == "F" {
        let target = observation
          .visible_actors
          .iter()
          .find(|actor| !actor.is_player)
          .map(|actor| actor.position);
        TARGET.with(|target_slot| *target_slot.borrow_mut() = target);
        let Some(target) = target else {
          return "No visible target.".to_string();
        };
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
          update_target_status(
            &document,
            &format!(
              "Target selected: ({}, {}). Press Enter to fire or Escape to cancel",
              target.x, target.y
            ),
          );
        }
        return format!("Target selected at ({}, {}).", target.x, target.y);
      }
      let command = if key == "Enter" {
        let Some(target) = TARGET.with(|target_slot| *target_slot.borrow()) else {
          return "No target selected.".to_string();
        };
        Command::AttackRanged(target)
      } else {
        let Some(command) = BrowserSession::command_for_key(key, &observation) else {
          return format!("Unbound key: {key}");
        };
        command
      };
      if matches!(command, Command::AttackRanged(_)) {
        TARGET.with(|target_slot| *target_slot.borrow_mut() = None);
      }
      match session.submit(command) {
        Ok(step) => {
          if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            update_dom(&document, &step.after);
            if key == "Enter" {
              update_target_status(&document, "Targets: fired");
            }
          }
          AUDIO.with(|audio_slot| {
            if let Some(mixer) = audio_slot.borrow().as_ref() {
              for cue in drl_audio::cues_for_events(&step.events) {
                let _ = mixer.play(cue);
              }
            }
          });
          render_scene(&RenderScene::from_observation(&step.after));
          if session.is_game_over() {
            "Game over — press Restart to try again.".to_string()
          } else {
            format!("Turn {}: {:?}", step.after.turn.count, command)
          }
        }
        Err(error) => format!("Command rejected: {error}"),
      }
    })
  }

  /// Executes an inventory action from a semantic DOM control.
  #[wasm_bindgen]
  pub fn dispatch_inventory(action: &str, item_id: u64) -> String {
    SESSION.with(|session_slot| {
      let mut session_ref = session_slot.borrow_mut();
      let Some(session) = session_ref.as_mut() else {
        return "Press Start first.".to_string();
      };
      let Some(action) = (match action {
        "equip" => Some(InventoryAction::Equip),
        "use" => Some(InventoryAction::Use),
        "drop" => Some(InventoryAction::Drop),
        _ => None,
      }) else {
        return format!("Unbound inventory action: {action}");
      };
      let command = BrowserSession::inventory_command(action, ItemId::new(item_id));
      match session.submit(command) {
        Ok(step) => {
          if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            update_dom(&document, &step.after);
          }
          AUDIO.with(|audio_slot| {
            if let Some(mixer) = audio_slot.borrow().as_ref() {
              for cue in drl_audio::cues_for_events(&step.events) {
                let _ = mixer.play(cue);
              }
            }
          });
          render_scene(&RenderScene::from_observation(&step.after));
          if session.is_game_over() {
            "Game over — press Restart to try again.".to_string()
          } else {
            format!("Turn {}: {:?}", step.after.turn.count, command)
          }
        }
        Err(error) => format!("Inventory action rejected: {error}"),
      }
    })
  }

  /// Resizes only the canvas surface. Visibility and DPR are presentation
  /// concerns and never submit a simulation command.
  #[wasm_bindgen]
  pub fn resize(width: u32, height: u32, dpr: f64) {
    RENDERER.with(|renderer_slot| {
      if let Some(renderer) = renderer_slot.borrow_mut().as_mut() {
        renderer.resize(width, height, dpr);
      }
    });
  }

  /// Restarts the fixed session and redraws the initial observation.
  #[wasm_bindgen]
  pub fn restart() -> String {
    SESSION.with(|session_slot| {
      let mut session_ref = session_slot.borrow_mut();
      let Some(session) = session_ref.as_mut() else {
        return "Press Start first.".to_string();
      };
      match session.restart() {
        Ok(()) => {
          let observation = session.observation();
          if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            update_dom(&document, &observation);
          }
          render_scene(&RenderScene::from_observation(&observation));
          "Restarted deterministic M4 session.".to_string()
        }
        Err(error) => format!("Restart failed: {error}"),
      }
    })
  }

  /// Changes the user-visible mute state without affecting gameplay.
  #[wasm_bindgen]
  pub fn set_muted(muted: bool) -> String {
    AUDIO.with(|audio_slot| {
      let mut audio_ref = audio_slot.borrow_mut();
      let Some(mixer) = audio_ref.as_mut() else {
        return "Audio unavailable; gameplay continues.".to_string();
      };
      let settings = mixer.settings();
      mixer.set_settings(muted, settings.volume);
      if muted {
        "Audio muted."
      } else {
        "Audio enabled."
      }
      .to_string()
    })
  }

  /// Changes the user-visible volume without affecting gameplay.
  #[wasm_bindgen]
  pub fn set_volume(volume: f32) -> String {
    AUDIO.with(|audio_slot| {
      let mut audio_ref = audio_slot.borrow_mut();
      let Some(mixer) = audio_ref.as_mut() else {
        return "Audio unavailable; gameplay continues.".to_string();
      };
      let settings = mixer.settings();
      mixer.set_settings(settings.muted, volume);
      format!("Audio volume: {:.0}%.", mixer.settings().volume * 100.0)
    })
  }

  /// Retries a suspended Web Audio context from a later trusted gesture.
  #[wasm_bindgen]
  pub async fn unlock_audio() -> String {
    let mixer = AUDIO.with(|audio_slot| audio_slot.borrow_mut().take());
    let Some(mut mixer) = mixer else {
      return "Audio unavailable; gameplay continues.".to_string();
    };
    let result = mixer.unlock().await;
    let unlocked = result.is_ok();
    AUDIO.with(|audio_slot| *audio_slot.borrow_mut() = Some(mixer));
    if unlocked {
      "Audio unlocked.".to_string()
    } else {
      "Audio remains suspended; gameplay continues.".to_string()
    }
  }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::{
  WebGpuRenderer, boot, dispatch_inventory, dispatch_key, key_command, load_texture_source, resize,
  restart, set_muted, set_volume, unlock_audio,
};

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn texture_source_urls_are_same_origin_and_dimensions_are_checked() {
    let source = drl_assets::AtlasId::Enemies.texture_source(drl_assets::SpriteLayer::Base);
    assert_eq!(
      texture_source_url(source).expect("manifest path"),
      "assets/legacy/drl/graphics/enemies.png"
    );
    assert_eq!(
      browser_asset_url("dguy.png").expect("safe path"),
      "assets/legacy/drl/graphics/dguy.png"
    );
    for path in [
      "/dguy.png",
      "../dguy.png",
      "foo/../bar.png",
      "dguy.png?x=1",
      "dguy.png#x",
      r"..\dguy.png",
    ] {
      assert!(browser_asset_url(path).is_err(), "{path}");
    }
    assert!(validate_texture_source_dimensions(source, 512, 192).is_ok());
    let error = validate_texture_source_dimensions(source, 256, 192).unwrap_err();
    assert_eq!(error.path, "enemies.png");
    assert_eq!(error.expected, (512, 192));
    assert_eq!(error.actual, (256, 192));
    assert!(error.to_string().contains("expected 512x192"));
  }

  #[test]
  fn texture_source_manifest_is_stable_and_deduplicated() {
    let sources = texture_source_manifest();
    assert_eq!(sources.len(), 24);
    assert_eq!(sources.first().expect("base source").path, "dguy.png");
    assert_eq!(sources.last().expect("last source").path, "fx_emissive.png");
    assert!(sources.windows(2).all(|window| window[0] != window[1]));
    assert_eq!(
      sources
        .iter()
        .filter(|source| source.path == "levels.png")
        .count(),
      1
    );
  }

  #[test]
  fn rejected_commands_do_not_advance_the_session() {
    let mut session = BrowserSession::new().expect("fixed session");
    let before = session.observation();
    let error = session.submit(Command::Descend).unwrap_err();
    assert!(!error.is_empty());
    assert_eq!(session.observation(), before);
  }

  #[test]
  fn keyboard_mapping_covers_diagonal_numpad_and_actions() {
    let observation = BrowserSession::new().expect("fixed session").observation();
    assert_eq!(
      BrowserSession::command_for_key("7", &observation),
      Some(Command::Move(Direction::NorthWest))
    );
    assert_eq!(
      BrowserSession::command_for_key("g", &observation),
      Some(Command::Pickup)
    );
    assert_eq!(
      BrowserSession::command_for_key("r", &observation),
      Some(Command::Reload)
    );
  }

  #[test]
  fn browser_session_matches_direct_core_for_identical_commands() {
    let mut browser = BrowserSession::new().expect("fixed session");
    let mut direct = BrowserSession::fixed_game().expect("fixed core game");
    let commands = [
      Command::Wait,
      Command::Move(Direction::East),
      Command::Move(Direction::East),
      Command::Move(Direction::East),
      Command::Pickup,
      Command::Pickup,
      Command::Pickup,
    ];
    for command in commands {
      let expected_events = direct.step(command).expect("direct command");
      let step = browser.submit(command).expect("browser command");
      assert_eq!(step.events, expected_events);
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events)
      );
      assert_eq!(step.after, direct.observe_player());
    }
    let replay = browser.replay_log();
    let (replayed, _) = drl_core::ReplayEngine::run(&replay).expect("replay browser run");
    let browser_observation = browser.observation();
    let replay_observation = replayed.observe_player();
    assert_eq!(browser_observation, replay_observation);
    assert!(drl_core::ReplayEngine::verify_determinism(&replay).expect("replay determinism"));
  }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
  use wasm_bindgen_test::*;

  wasm_bindgen_test_configure!(run_in_browser);

  #[wasm_bindgen_test]
  fn key_contract_is_stable() {
    assert!(crate::key_command("ArrowUp").contains("Move"));
  }
}
