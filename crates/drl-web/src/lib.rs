//! Browser-first DRL-Rust session boundary.
//!
//! `BrowserSession` is intentionally usable on native hosts for deterministic
//! tests. The WASM exports are a thin boot/input shell; gameplay state stays in
//! Rust and is never mirrored into a parallel JavaScript model.

use drl_assets::{AtlasId, AtlasTextureSource, SpriteUv};
use drl_core::item::Item;
use drl_core::{Game, Tile};
use drl_protocol::{
  Command, Direction, ItemId, ItemSpawnKind, ItemSpawnSpec, ItemView, MonsterKind,
  MonsterSpawnSpec, PlayerObservation, Position, ReplayLog,
};
use drl_render::{
  LightingBand, MinimapMarker, MinimapState, ParticleDecalSprite, ParticleDecalStorageError,
  ParticleDecalStore, PixelRect, PresentationStep, RenderScene, effect_timeline_for_observations,
};

mod persistence;
pub use persistence::SnapshotError;

/// Escapes user-visible item names before they cross the HTML string boundary.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn escape_html(value: &str) -> String {
  let mut escaped = String::with_capacity(value.len());
  for character in value.chars() {
    match character {
      '&' => escaped.push_str("&amp;"),
      '<' => escaped.push_str("&lt;"),
      '>' => escaped.push_str("&gt;"),
      '"' => escaped.push_str("&quot;"),
      '\'' => escaped.push_str("&#39;"),
      _ => escaped.push(character),
    }
  }
  escaped
}

/// Builds item-qualified inventory controls for the browser's semantic DOM.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn inventory_markup(items: &[ItemView]) -> String {
  use std::fmt::Write;

  let mut controls = String::new();
  for item in items {
    let name = escape_html(&item.name);
    let item_id = item.id.as_u64();
    write!(
      controls,
      "<div id=\"inventory-item-{item_id}\" role=\"group\" aria-label=\"Inventory item: {name}\"><span>{name}</span><button type=\"button\" data-action=\"equip\" data-item-id=\"{item_id}\" aria-label=\"Equip {name}\">Equip</button><button type=\"button\" data-action=\"use\" data-item-id=\"{item_id}\" aria-label=\"Use {name}\">Use</button><button type=\"button\" data-action=\"drop\" data-item-id=\"{item_id}\" aria-label=\"Drop {name}\">Drop</button></div>"
    )
    .expect("writing inventory markup to a String cannot fail");
  }
  controls
}

const MAX_MINIMAP_CELLS: u64 = 4096;

/// Renders the fair minimap projection as a bounded, accessible text grid.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn minimap_markup(state: &MinimapState) -> String {
  let cell_count = u64::from(state.map_width) * u64::from(state.map_height);
  if state.map_width == 0 || state.map_height == 0 || cell_count > MAX_MINIMAP_CELLS {
    return "Minimap unavailable.".to_string();
  }

  let width = state.map_width as usize;
  let height = state.map_height as usize;
  let mut glyphs = vec![' '; cell_count as usize];
  for cell in &state.cells {
    let Some(x) = usize::try_from(cell.position.x).ok() else {
      continue;
    };
    let Some(y) = usize::try_from(cell.position.y).ok() else {
      continue;
    };
    if x >= width || y >= height {
      continue;
    }
    let glyph = match cell.marker {
      Some(MinimapMarker::Player) => '@',
      Some(MinimapMarker::VisibleActor) => 'a',
      None => match cell.tile_kind {
        drl_protocol::TileKind::Floor => '.',
        drl_protocol::TileKind::Wall => '#',
        drl_protocol::TileKind::DoorClosed => '+',
        drl_protocol::TileKind::DoorOpen => '/',
        drl_protocol::TileKind::StairsDown => '>',
        drl_protocol::TileKind::Lava => '=',
        drl_protocol::TileKind::Acid => 'a',
        drl_protocol::TileKind::Water => '~',
        drl_protocol::TileKind::Mud => 'u',
      },
    };
    glyphs[y * width + x] = glyph;
  }

  let mut markup = String::with_capacity((width + 1) * height);
  for row in glyphs.chunks(width) {
    if !markup.is_empty() {
      markup.push('\n');
    }
    for glyph in row {
      markup.push(*glyph);
    }
  }
  markup
}

/// Returns the six UV coordinates for a top-left-origin textured quad.
#[must_use]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const fn base_texture_uvs(uv: SpriteUv) -> [[f32; 2]; 6] {
  [
    [uv.u_min, uv.v_max],
    [uv.u_max, uv.v_max],
    [uv.u_max, uv.v_min],
    [uv.u_min, uv.v_max],
    [uv.u_max, uv.v_min],
    [uv.u_min, uv.v_min],
  ]
}

/// Returns the shared fair lighting factor used by the textured pass.
#[must_use]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn base_texture_lighting_factor(band: LightingBand) -> f32 {
  band.factor() as f32 / 100.0
}

/// Applies the legacy emissive floor to a fair RGB lighting scalar.
#[allow(dead_code)]
fn emissive_lighting_floor(lighting: f32, emissive: f32) -> f32 {
  lighting.max(emissive)
}

/// Matches the legacy shader's minimum surviving fragment alpha.
#[allow(dead_code)]
fn retains_textured_fragment(alpha: f32) -> bool {
  alpha >= 0.1
}

/// Shared WGSL source for the bounded base/mask/emissive/outline textured pass.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const BASE_TEXTURE_SHADER: &str = r#"
struct VertexInput {
  @location(0) position: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) lighting: vec4<f32>,
  @location(3) colorization: vec4<f32>,
};

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) lighting: vec4<f32>,
  @location(2) colorization: vec4<f32>,
};

@group(0) @binding(0) var base_texture: texture_2d<f32>;
@group(0) @binding(1) var emissive_texture: texture_2d<f32>;
@group(0) @binding(2) var mask_texture: texture_2d<f32>;
@group(0) @binding(3) var outline_texture: texture_2d<f32>;
@group(0) @binding(4) var base_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
  var output: VertexOutput;
  output.position = vec4<f32>(input.position, 0.0, 1.0);
  output.uv = input.uv;
  output.lighting = input.lighting;
  output.colorization = input.colorization;
  return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
  let sampled = textureSample(base_texture, base_sampler, input.uv);
  let emissive = textureSample(emissive_texture, base_sampler, input.uv).r;
  let mask = textureSample(mask_texture, base_sampler, input.uv);
  let outline = textureSample(outline_texture, base_sampler, input.uv);
  let colorized = sampled.rgb + mask.rgb * input.colorization.rgb;
  let lighting = max(input.lighting.rgb, vec3<f32>(emissive));
  let outline_alpha = outline.a * (1.0 - sampled.a);
  let output_alpha = sampled.a + outline_alpha;
  let output_rgb = (colorized * sampled.a + outline.rgb * outline_alpha)
    / max(output_alpha, 0.0001);
  let output = vec4<f32>(output_rgb * lighting, output_alpha);
  if (output.a < 0.1) {
    discard;
  }
  return output;
}
"#;

/// Converts a physical destination rectangle into clip-space bounds.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const fn base_texture_ndc_rect(rect: PixelRect, canvas_width: u32, canvas_height: u32) -> [f32; 4] {
  let width = if canvas_width == 0 { 1 } else { canvas_width } as f32;
  let height = if canvas_height == 0 { 1 } else { canvas_height } as f32;
  [
    -1.0 + 2.0 * rect.x as f32 / width,
    1.0 - 2.0 * rect.y.saturating_add(rect.height) as f32 / height,
    -1.0 + 2.0 * rect.x.saturating_add(rect.width) as f32 / width,
    1.0 - 2.0 * rect.y as f32 / height,
  ]
}

/// Converts a browser animation timestamp into bounded elapsed milliseconds.
///
/// The timestamp source and scheduling policy remain outside this pure helper.
#[must_use]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn animation_elapsed_ms(start_ms: f64, timestamp_ms: f64) -> Option<u64> {
  if !start_ms.is_finite() || !timestamp_ms.is_finite() || timestamp_ms < start_ms {
    return None;
  }
  let elapsed_ms = (timestamp_ms - start_ms).floor();
  if elapsed_ms >= u64::MAX as f64 {
    Some(u64::MAX)
  } else {
    Some(elapsed_ms.max(0.0) as u64)
  }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
struct AnimationClock {
  start_ms: Option<f64>,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
impl AnimationClock {
  fn reset(&mut self) {
    self.start_ms = None;
  }

  fn visibility_changed(&mut self) {
    self.reset();
  }

  fn elapsed_ms(&mut self, hidden: bool, timestamp_ms: f64) -> Option<u64> {
    if hidden {
      self.reset();
      return None;
    }
    let start_ms = *self.start_ms.get_or_insert(timestamp_ms);
    animation_elapsed_ms(start_ms, timestamp_ms)
  }
}

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
  particle_decals: ParticleDecalStore,
  particle_decal_sprites: Vec<ParticleDecalSprite>,
}

impl BrowserSession {
  /// Creates the fixed M4 arena and its representative loot/combat content.
  pub fn new() -> Result<Self, drl_protocol::CommandError> {
    Ok(Self::from_game(Self::fixed_game()?))
  }

  /// Wraps an already-instantiated deterministic game at the browser boundary.
  ///
  /// The helper keeps browser presentation tests on the same authoritative
  /// `Game` state without adding a second scenario or replay representation.
  fn from_game(game: Game) -> Self {
    Self {
      game,
      last_error: None,
      commands: Vec::new(),
      particle_decals: ParticleDecalStore::new(256),
      particle_decal_sprites: Vec::new(),
    }
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

  /// Returns retained presentation-only decal requests for the browser pass.
  #[must_use]
  pub fn particle_decal_store(&self) -> &ParticleDecalStore {
    &self.particle_decals
  }

  /// Returns the caller-owned opaque sprite-handle descriptor table.
  #[must_use]
  pub fn particle_decal_sprites(&self) -> &[ParticleDecalSprite] {
    &self.particle_decal_sprites
  }

  /// Retains one presentation-only decal request without touching gameplay.
  pub fn try_insert_particle_decal(
    &mut self,
    insertion: drl_render::ParticleDecalInsertion,
  ) -> Result<(), ParticleDecalStorageError> {
    self.particle_decals.try_insert(insertion)
  }

  /// Replaces the caller-owned descriptor table used by decal rendering.
  pub fn set_particle_decal_sprites(&mut self, sprites: Vec<ParticleDecalSprite>) {
    self.particle_decal_sprites = sprites;
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

  /// Encodes successful fixed-session commands into a versioned save token.
  pub fn snapshot_token(&self) -> Result<String, SnapshotError> {
    persistence::encode_snapshot(&self.commands)
  }

  /// Rebuilds this session from a versioned token without exposing game state.
  pub fn restore_snapshot(&mut self, token: &str) -> Result<(), SnapshotError> {
    self.restore_snapshot_with_format(token).map(|_| ())
  }

  fn restore_snapshot_with_format(
    &mut self,
    token: &str,
  ) -> Result<persistence::SnapshotFormat, SnapshotError> {
    let decoded = persistence::decode_snapshot_with_format(token)?;
    let mut restored =
      Self::new().map_err(|error| SnapshotError::Initialization(error.to_string()))?;
    for command in decoded.commands {
      restored
        .submit(command)
        .map_err(SnapshotError::CommandRejected)?;
    }
    let format = decoded.format;
    *self = restored;
    Ok(format)
  }

  /// Returns a replay-schema representation of the fixed browser session.
  ///
  /// The log uses the existing versioned replay schema; it does not create a
  /// browser-specific wire format or expose authoritative state to JavaScript.
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
  use super::texture::{BaseTexturePipeline, GpuTextureCache};
  use super::*;
  use drl_render::{AnimationPlayback, PixelViewport, scene_clear_color, shade_color};
  use std::cell::RefCell;
  use wasm_bindgen::prelude::*;
  use wasm_bindgen_futures::JsFuture;
  use web_sys::{HtmlCanvasElement, HtmlElement, HtmlImageElement, Storage, Window};
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
    static ANIMATION_CLOCK: RefCell<AnimationClock> = const { RefCell::new(AnimationClock { start_ms: None }) };
    static ANIMATION_LOOP: RefCell<Option<Closure<dyn FnMut(f64)>>> = const { RefCell::new(None) };
    static VISIBILITY_LISTENER: RefCell<Option<Closure<dyn FnMut()>>> = const { RefCell::new(None) };
  }

  const SAVE_STORAGE_KEY: &str = "drl-rust:m4-session:v1";
  const REJECTED_SAVE_STORAGE_KEY: &str = "drl-rust:m4-session:v1:rejected";

  fn browser_storage() -> Result<Storage, SnapshotError> {
    let window = web_sys::window()
      .ok_or_else(|| SnapshotError::Initialization("window unavailable".to_string()))?;
    window
      .local_storage()
      .map_err(|error| {
        SnapshotError::Initialization(format!("localStorage unavailable: {error:?}"))
      })?
      .ok_or_else(|| SnapshotError::Initialization("localStorage unavailable".to_string()))
  }

  fn persist_session(session: &BrowserSession) -> Result<(), SnapshotError> {
    let token = session.snapshot_token()?;
    browser_storage()?
      .set_item(SAVE_STORAGE_KEY, &token)
      .map_err(|error| SnapshotError::Initialization(format!("save failed: {error:?}")))
  }

  fn migrate_legacy_snapshot(
    session: &BrowserSession,
    format: persistence::SnapshotFormat,
  ) -> Option<String> {
    (format == persistence::SnapshotFormat::V1)
      .then(|| persist_session(session).err())
      .flatten()
      .map(|error| {
        format!(
          " Legacy save restored, but migration to the current format failed ({error}); use Save to retry."
        )
      })
  }

  fn remove_persisted_session() -> Result<(), SnapshotError> {
    browser_storage()?
      .remove_item(SAVE_STORAGE_KEY)
      .map_err(|error| SnapshotError::Initialization(format!("clear failed: {error:?}")))
  }

  fn remove_rejected_session() -> Result<(), SnapshotError> {
    browser_storage()?
      .remove_item(REJECTED_SAVE_STORAGE_KEY)
      .map_err(|error| SnapshotError::Initialization(format!("quarantine clear failed: {error:?}")))
  }

  fn quarantine_persisted_session(token: &str, error: &SnapshotError) -> Result<(), SnapshotError> {
    let storage = browser_storage()?;
    let record = persistence::encode_quarantine_record(token, error);
    storage
      .set_item(REJECTED_SAVE_STORAGE_KEY, &record)
      .map_err(|storage_error| {
        SnapshotError::Initialization(format!("quarantine write failed: {storage_error:?}"))
      })?;
    storage
      .remove_item(SAVE_STORAGE_KEY)
      .map_err(|storage_error| {
        SnapshotError::Initialization(format!("active save clear failed: {storage_error:?}"))
      })
  }

  fn rejected_save_message(token: &str, error: &SnapshotError) -> String {
    match quarantine_persisted_session(token, error) {
      Ok(()) => format!(" Saved session ignored ({error}); rejected save quarantined."),
      Err(recovery_error) => {
        format!(" Saved session ignored ({error}); rejected save may remain ({recovery_error}).")
      }
    }
  }

  fn append_persistence_warning(status: String, warning: Option<String>) -> String {
    match warning {
      Some(warning) => format!("{status}{warning}"),
      None => status,
    }
  }

  fn save_after_command(session: &BrowserSession) -> Option<String> {
    persist_session(session).err().map(|error| {
      format!(" Save warning: current session was not persisted ({error}); use Save to retry.")
    })
  }

  fn read_persisted_session() -> Result<Option<String>, SnapshotError> {
    browser_storage()?
      .get_item(SAVE_STORAGE_KEY)
      .map_err(|error| SnapshotError::Initialization(format!("load failed: {error:?}")))
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
      let base_texture =
        BaseTexturePipeline::new(&device, &queue, config.format, textures.as_ref());
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
        (None, None) => {
          self
            .base_texture
            .covers_scene(scene, self.config.width, self.config.height)
        }
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

  fn target_vertices(scene: &RenderScene, canvas_width: u32, canvas_height: u32) -> Vec<u8> {
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
    if let Some(minimap) = document.get_element_by_id("minimap") {
      let state = MinimapState::from_observation(observation);
      minimap.set_text_content(Some(&minimap_markup(&state)));
    }
    if let Some(inventory) = document.get_element_by_id("inventory") {
      inventory.set_inner_html(&inventory_markup(&observation.inventory));
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
  }

  fn set_diagnostic(document: &web_sys::Document, title: &str, detail: &str, action: &str) {
    if let Some(panel) = document.get_element_by_id("game-diagnostics") {
      let _ = panel.remove_attribute("hidden");
    }
    if let Some(title_node) = document.get_element_by_id("diagnostics-title") {
      title_node.set_text_content(Some(title));
    }
    if let Some(detail_node) = document.get_element_by_id("diagnostics-detail") {
      detail_node.set_text_content(Some(detail));
    }
    if let Some(action_node) = document.get_element_by_id("diagnostics-action") {
      action_node.set_text_content(Some(action));
    }
    if let Some(panel) = document
      .get_element_by_id("game-diagnostics")
      .and_then(|panel| panel.dyn_into::<HtmlElement>().ok())
    {
      let _ = panel.focus();
    }
  }

  fn render_scene(
    scene: &RenderScene,
    store: &ParticleDecalStore,
    sprites: &[ParticleDecalSprite],
  ) {
    let result = RENDERER.with(|renderer_slot| {
      renderer_slot.borrow().as_ref().map_or(Ok(()), |renderer| {
        renderer.render_with_particle_decals(scene, store, sprites)
      })
    });
    if let Err(error) = result
      && let Some(document) = web_sys::window().and_then(|window| window.document())
    {
      set_status(
        &document,
        &format!("WebGPU presentation unavailable; gameplay is unchanged: {error:?}"),
      );
      set_diagnostic(
        &document,
        "WebGPU presentation unavailable",
        &format!("The renderer reported a local presentation error ({error:?})."),
        "Gameplay is unchanged; retry after checking the desktop Chromium WebGPU environment.",
      );
    }
  }

  fn render_animation_frame(timestamp_ms: f64) {
    let Some(window) = web_sys::window() else {
      return;
    };
    let Some(document) = window.document() else {
      return;
    };
    let Some(elapsed_ms) = ANIMATION_CLOCK.with(|clock| {
      clock
        .borrow_mut()
        .elapsed_ms(document.hidden(), timestamp_ms)
    }) else {
      return;
    };
    let result = SESSION.with(|session_slot| {
      let session_ref = session_slot.borrow();
      let Some(session) = session_ref.as_ref() else {
        return Ok(());
      };
      let scene = session.scene();
      RENDERER.with(|renderer_slot| {
        renderer_slot.borrow().as_ref().map_or(Ok(()), |renderer| {
          renderer.render_at_elapsed_with_particle_decals(
            &scene,
            elapsed_ms,
            AnimationPlayback::Loop,
            session.particle_decal_store(),
            session.particle_decal_sprites(),
          )
        })
      })
    });
    if let Err(error) = result {
      set_status(
        &document,
        &format!("WebGPU animation frame unavailable; gameplay is unchanged: {error:?}"),
      );
      set_diagnostic(
        &document,
        "WebGPU animation unavailable",
        &format!("A local animation frame could not be presented ({error:?})."),
        "Gameplay is unchanged; continue without animation or reload the page.",
      );
    }
  }

  fn request_next_animation_frame() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window unavailable"))?;
    let callback = Closure::wrap(Box::new(|timestamp_ms: f64| {
      render_animation_frame(timestamp_ms);
      if let Err(error) = request_next_animation_frame()
        && let Some(document) = web_sys::window().and_then(|window| window.document())
      {
        set_status(
          &document,
          &format!("Browser animation scheduling unavailable: {error:?}"),
        );
        set_diagnostic(
          &document,
          "Browser animation scheduling unavailable",
          &format!("The browser rejected a local animation-frame request ({error:?})."),
          "Gameplay state is not advanced by the failed request; reload to retry presentation.",
        );
        ANIMATION_LOOP.with(|slot| *slot.borrow_mut() = None);
      }
    }) as Box<dyn FnMut(f64)>);
    window.request_animation_frame(callback.as_ref().unchecked_ref())?;
    ANIMATION_LOOP.with(|slot| *slot.borrow_mut() = Some(callback));
    Ok(())
  }

  fn install_visibility_listener() -> Result<(), JsValue> {
    if VISIBILITY_LISTENER.with(|slot| slot.borrow().is_some()) {
      return Ok(());
    }
    let document = web_sys::window()
      .ok_or_else(|| JsValue::from_str("window unavailable"))?
      .document()
      .ok_or_else(|| JsValue::from_str("document unavailable"))?;
    let callback = Closure::wrap(Box::new(|| {
      ANIMATION_CLOCK.with(|clock| clock.borrow_mut().visibility_changed());
    }) as Box<dyn FnMut()>);
    document
      .add_event_listener_with_callback("visibilitychange", callback.as_ref().unchecked_ref())?;
    VISIBILITY_LISTENER.with(|slot| *slot.borrow_mut() = Some(callback));
    Ok(())
  }

  fn start_animation_loop() -> Result<(), JsValue> {
    if ANIMATION_LOOP.with(|slot| slot.borrow().is_some()) {
      return Ok(());
    }
    if let Err(error) = install_visibility_listener()
      && let Some(document) = web_sys::window().and_then(|window| window.document())
    {
      set_status(
        &document,
        &format!("Browser visibility lifecycle unavailable; animation continues: {error:?}"),
      );
      set_diagnostic(
        &document,
        "Browser visibility lifecycle unavailable",
        &format!("The page could not install its local visibility listener ({error:?})."),
        "Gameplay can continue; reload to retry presentation lifecycle handling.",
      );
    }
    ANIMATION_CLOCK.with(|clock| clock.borrow_mut().reset());
    request_next_animation_frame()
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
    let mut session =
      BrowserSession::new().map_err(|error| JsValue::from_str(&error.to_string()))?;
    let restore_message = match read_persisted_session() {
      Ok(Some(token)) => match session.restore_snapshot_with_format(&token) {
        Ok(format) => {
          let status = if format == persistence::SnapshotFormat::V1 {
            " Restored and migrated the legacy saved session.".to_string()
          } else {
            " Restored the saved session.".to_string()
          };
          append_persistence_warning(status, migrate_legacy_snapshot(&session, format))
        }
        Err(error) => rejected_save_message(&token, &error),
      },
      Ok(None) => String::new(),
      Err(error) => format!(" Saved session unavailable ({error})."),
    };
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
        format!(
          "{audio_message}{restore_message} Texture upload unavailable; geometry fallback active ({error})."
        )
      }
      None => format!("{audio_message}{restore_message} Textures uploaded: {texture_count}."),
    };
    status.set_text_content(Some(&message));
    if let Err(error) = start_animation_loop() {
      set_status(
        &document,
        &format!("Browser animation scheduling unavailable; gameplay continues: {error:?}"),
      );
      set_diagnostic(
        &document,
        "Browser animation scheduling unavailable",
        &format!("The browser rejected the initial animation-frame request ({error:?})."),
        "Gameplay continues without animation; reload to retry presentation scheduling.",
      );
    }
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
          let persistence_warning = save_after_command(session);
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
          render_scene(
            &RenderScene::from_observation(&step.after),
            session.particle_decal_store(),
            session.particle_decal_sprites(),
          );
          let status = if session.is_game_over() {
            "Game over — press Restart to try again.".to_string()
          } else {
            format!("Turn {}: {:?}", step.after.turn.count, command)
          };
          if let Some(warning) = persistence_warning.as_deref()
            && let Some(document) = web_sys::window().and_then(|window| window.document())
          {
            set_status(&document, warning);
          }
          append_persistence_warning(status, persistence_warning)
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
          let persistence_warning = save_after_command(session);
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
          render_scene(
            &RenderScene::from_observation(&step.after),
            session.particle_decal_store(),
            session.particle_decal_sprites(),
          );
          let status = if session.is_game_over() {
            "Game over — press Restart to try again.".to_string()
          } else {
            format!("Turn {}: {:?}", step.after.turn.count, command)
          };
          if let Some(warning) = persistence_warning.as_deref()
            && let Some(document) = web_sys::window().and_then(|window| window.document())
          {
            set_status(&document, warning);
          }
          append_persistence_warning(status, persistence_warning)
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
          let clear_warning = remove_persisted_session().err().map(|error| {
            format!(
              " Save clear warning: the previous save may remain ({error}); use Clear Save to retry."
            )
          });
          let quarantine_warning = remove_rejected_session().err().map(|error| {
            format!(" Rejected-save quarantine clear warning: {error}; use Clear Save to retry.")
          });
          ANIMATION_CLOCK.with(|clock| clock.borrow_mut().reset());
          let observation = session.observation();
          if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            update_dom(&document, &observation);
          }
          render_scene(
            &RenderScene::from_observation(&observation),
            session.particle_decal_store(),
            session.particle_decal_sprites(),
          );
          let status = "Restarted deterministic M4 session.".to_string();
          let clear_warning = clear_warning.or(quarantine_warning);
          if let Some(warning) = clear_warning.as_deref()
            && let Some(document) = web_sys::window().and_then(|window| window.document())
          {
            set_status(&document, warning);
          }
          append_persistence_warning(status, clear_warning)
        }
        Err(error) => format!("Restart failed: {error}"),
      }
    })
  }

  /// Saves the successful fixed-session command history to versioned localStorage.
  #[wasm_bindgen]
  pub fn save() -> String {
    let result = SESSION.with(|session_slot| {
      let session_ref = session_slot.borrow();
      let session = session_ref
        .as_ref()
        .ok_or_else(|| SnapshotError::Initialization("Press Start first.".to_string()))?;
      persist_session(session)
    });
    match result {
      Ok(()) => "Session saved on this device.".to_string(),
      Err(error) => error.to_string(),
    }
  }

  /// Loads and transactionally restores the versioned localStorage snapshot.
  #[wasm_bindgen]
  pub fn load() -> String {
    let token = match read_persisted_session() {
      Ok(Some(token)) => token,
      Ok(None) => return "No saved session found.".to_string(),
      Err(error) => return error.to_string(),
    };
    let result = SESSION.with(|session_slot| {
      let mut session_ref = session_slot.borrow_mut();
      let session = session_ref
        .as_mut()
        .ok_or_else(|| SnapshotError::Initialization("Press Start first.".to_string()))?;
      session.restore_snapshot_with_format(&token)
    });
    match result {
      Ok(format) => {
        let migration_warning = SESSION.with(|session_slot| {
          session_slot
            .borrow()
            .as_ref()
            .and_then(|session| migrate_legacy_snapshot(session, format))
        });
        ANIMATION_CLOCK.with(|clock| clock.borrow_mut().reset());
        TARGET.with(|target_slot| *target_slot.borrow_mut() = None);
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
          SESSION.with(|session_slot| {
            if let Some(session) = session_slot.borrow().as_ref() {
              update_dom(&document, &session.observation());
              render_scene(
                &RenderScene::from_observation(&session.observation()),
                session.particle_decal_store(),
                session.particle_decal_sprites(),
              );
            }
          });
        }
        let status = if format == persistence::SnapshotFormat::V1 {
          "Session loaded and migrated from the legacy format.".to_string()
        } else {
          "Session loaded from this device.".to_string()
        };
        append_persistence_warning(status, migration_warning)
      }
      Err(error) => rejected_save_message(&token, &error),
    }
  }

  /// Removes the local save without changing the active simulation.
  #[wasm_bindgen]
  pub fn clear_save() -> String {
    let active_error = remove_persisted_session().err();
    let quarantine_error = remove_rejected_session().err();
    match (active_error, quarantine_error) {
      (None, None) => "Saved session cleared.".to_string(),
      (Some(error), None) | (None, Some(error)) => error.to_string(),
      (Some(active), Some(quarantine)) => {
        format!("Save clear failed: {active}; {quarantine}")
      }
    }
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
  WebGpuRenderer, boot, clear_save, dispatch_inventory, dispatch_key, key_command, load,
  load_texture_source, resize, restart, save, set_muted, set_volume, unlock_audio,
};

#[cfg(test)]
mod tests {
  use super::*;
  use drl_protocol::{ItemArchetype, ItemCategory, PlayerSpawnConfig, Position, TileKind};

  fn test_item(name: &str) -> ItemView {
    ItemView {
      id: ItemId::new(7),
      archetype: ItemArchetype::Pistol,
      name: name.to_string(),
      category: ItemCategory::Weapon,
      count: 1,
      description: String::new(),
      clip: None,
      damage: None,
      armor_value: None,
      heal_amount: None,
      knockback: None,
    }
  }

  #[test]
  fn inventory_markup_qualifies_actions_and_escapes_names() {
    let markup = inventory_markup(&[test_item("Pistol <&\"'")]);
    assert!(markup.contains("id=\"inventory-item-7\""));
    assert!(markup.contains("role=\"group\""));
    assert!(markup.contains("aria-label=\"Inventory item: Pistol &lt;&amp;&quot;&#39;\""));
    assert!(markup.contains("aria-label=\"Equip Pistol &lt;&amp;&quot;&#39;\""));
    assert!(markup.contains("aria-label=\"Use Pistol &lt;&amp;&quot;&#39;\""));
    assert!(markup.contains("aria-label=\"Drop Pistol &lt;&amp;&quot;&#39;\""));
    assert!(!markup.contains("Pistol <&\"'"));
  }

  #[test]
  fn minimap_markup_renders_only_projected_cells_and_markers() {
    let markup = minimap_markup(&MinimapState {
      map_width: 4,
      map_height: 2,
      cells: vec![
        drl_render::MinimapCell {
          position: Position::new(0, 0),
          tile_kind: TileKind::Wall,
          is_visible: true,
          marker: None,
        },
        drl_render::MinimapCell {
          position: Position::new(1, 0),
          tile_kind: TileKind::Floor,
          is_visible: true,
          marker: Some(MinimapMarker::Player),
        },
        drl_render::MinimapCell {
          position: Position::new(2, 0),
          tile_kind: TileKind::Floor,
          is_visible: true,
          marker: Some(MinimapMarker::VisibleActor),
        },
        drl_render::MinimapCell {
          position: Position::new(3, 1),
          tile_kind: TileKind::StairsDown,
          is_visible: false,
          marker: None,
        },
      ],
    });

    assert_eq!(markup, "#@a \n   >");
    assert!(!markup.contains("?"));
  }

  #[test]
  fn minimap_markup_bounds_dom_work_for_malformed_dimensions() {
    assert_eq!(
      minimap_markup(&MinimapState {
        map_width: 65,
        map_height: 65,
        cells: Vec::new(),
      }),
      "Minimap unavailable."
    );
  }

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
  fn base_texture_uvs_preserve_top_left_orientation() {
    let uv = SpriteUv {
      u_min: 0.1,
      v_min: 0.2,
      u_max: 0.3,
      v_max: 0.4,
    };
    assert_eq!(
      base_texture_uvs(uv),
      [
        [0.1, 0.4],
        [0.3, 0.4],
        [0.3, 0.2],
        [0.1, 0.4],
        [0.3, 0.2],
        [0.1, 0.2],
      ]
    );
  }

  #[test]
  fn base_texture_lighting_factor_matches_fair_bands() {
    assert_eq!(base_texture_lighting_factor(LightingBand::Visible), 1.0);
    assert_eq!(base_texture_lighting_factor(LightingBand::Explored), 0.45);
  }

  #[test]
  fn emissive_role_raises_but_never_reduces_fair_light() {
    assert_eq!(emissive_lighting_floor(0.45, 0.8), 0.8);
    assert_eq!(emissive_lighting_floor(1.0, 0.8), 1.0);
  }

  #[test]
  fn emissive_role_pairing_uses_registered_atlas_source() {
    let base = AtlasId::Enemies.texture_source(drl_assets::SpriteLayer::Base);
    let emissive = AtlasId::Enemies.texture_source(drl_assets::SpriteLayer::Emissive);
    assert_eq!(base.path, "enemies.png");
    assert_eq!(emissive.path, "enemies_emissive.png");
    assert_eq!((base.width, base.height), (emissive.width, emissive.height));
  }

  #[test]
  fn outline_role_registration_preserves_optional_atlas_boundary() {
    assert!(
      AtlasId::Enemies
        .layers()
        .contains(&drl_assets::SpriteLayer::Shadow)
    );
    assert!(
      !AtlasId::Levels
        .layers()
        .contains(&drl_assets::SpriteLayer::Shadow)
    );
    let source = AtlasId::Enemies.texture_source(drl_assets::SpriteLayer::Shadow);
    assert_eq!(source.path, "enemies_shadow.png");
    assert_eq!((source.width, source.height), (512, 192));
  }

  #[test]
  fn textured_alpha_cutoff_matches_legacy_boundary() {
    assert!(!retains_textured_fragment(0.0));
    assert!(!retains_textured_fragment(0.099));
    assert!(retains_textured_fragment(0.1));
    assert!(retains_textured_fragment(1.0));
  }

  #[test]
  fn textured_shader_contract_keeps_verified_compositing_terms() {
    assert!(BASE_TEXTURE_SHADER.contains("textureSample(base_texture"));
    assert!(BASE_TEXTURE_SHADER.contains("textureSample(emissive_texture"));
    assert!(BASE_TEXTURE_SHADER.contains("textureSample(mask_texture"));
    assert!(BASE_TEXTURE_SHADER.contains("outline_texture: texture_2d<f32>"));
    assert!(BASE_TEXTURE_SHADER.contains("textureSample(outline_texture"));
    assert!(BASE_TEXTURE_SHADER.contains("mask.rgb * input.colorization.rgb"));
    assert!(BASE_TEXTURE_SHADER.contains("output.colorization = input.colorization"));
    assert!(BASE_TEXTURE_SHADER.contains("max(input.lighting.rgb"));
    assert!(BASE_TEXTURE_SHADER.contains("outline.a * (1.0 - sampled.a)"));
    assert!(BASE_TEXTURE_SHADER.contains("colorized * sampled.a + outline.rgb * outline_alpha"));
    assert!(BASE_TEXTURE_SHADER.contains("output_rgb * lighting, output_alpha"));
    assert!(BASE_TEXTURE_SHADER.contains("if (output.a < 0.1)"));
    assert!(BASE_TEXTURE_SHADER.contains("return output;"));
  }

  #[test]
  fn base_texture_ndc_rect_preserves_destination_orientation() {
    let rect = PixelRect {
      x: 10,
      y: 20,
      width: 30,
      height: 40,
    };
    let [left, bottom, right, top] = base_texture_ndc_rect(rect, 100, 100);
    assert!((left + 0.8).abs() < f32::EPSILON);
    assert!((bottom + 0.2).abs() < f32::EPSILON);
    assert!((right + 0.2).abs() < f32::EPSILON);
    assert!((top - 0.6).abs() < f32::EPSILON);
  }

  #[test]
  fn animation_elapsed_ms_is_monotonic_bounded_and_clock_free() {
    assert_eq!(animation_elapsed_ms(100.0, 100.0), Some(0));
    assert_eq!(animation_elapsed_ms(100.0, 100.9), Some(0));
    assert_eq!(animation_elapsed_ms(100.0, 101.1), Some(1));
    assert_eq!(animation_elapsed_ms(100.0, 99.0), None);
    assert_eq!(animation_elapsed_ms(f64::NAN, 100.0), None);
    assert_eq!(animation_elapsed_ms(100.0, f64::INFINITY), None);
    assert_eq!(animation_elapsed_ms(0.0, u64::MAX as f64), Some(u64::MAX));
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
  fn snapshot_round_trip_replays_fixed_session_deterministically() {
    let mut session = BrowserSession::new().expect("fixed session");
    for command in [
      Command::Move(Direction::East),
      Command::Move(Direction::East),
      Command::Move(Direction::East),
      Command::Pickup,
    ] {
      session.submit(command).expect("legal command");
    }
    let expected_observation = session.observation();
    let expected_replay = session.replay_log();
    let token = session.snapshot_token().expect("snapshot encoding");
    assert_eq!(token, "DRL-RUST-BROWSER-SAVE/2:fixed-m4-v1:4:mr;mr;mr;p");

    let mut restored = BrowserSession::new().expect("fixed session");
    restored.restore_snapshot(&token).expect("snapshot restore");
    assert_eq!(restored.observation(), expected_observation);
    assert_eq!(restored.replay_log(), expected_replay);
    assert_eq!(restored.snapshot_token().expect("re-encode"), token);
  }

  #[test]
  fn v1_snapshot_restores_and_reencodes_as_v2() {
    let legacy = "DRL-RUST-BROWSER-SAVE/1:fixed-m4-v1:mr;mr;mr;p";
    let mut expected = BrowserSession::new().expect("fixed session");
    for command in [
      Command::Move(Direction::East),
      Command::Move(Direction::East),
      Command::Move(Direction::East),
      Command::Pickup,
    ] {
      expected.submit(command).expect("legal command");
    }

    let mut restored = BrowserSession::new().expect("fixed session");
    restored
      .restore_snapshot(legacy)
      .expect("legacy snapshot restore");
    assert_eq!(restored.observation(), expected.observation());
    assert_eq!(restored.replay_log(), expected.replay_log());
    assert_eq!(
      restored.snapshot_token().expect("migrated encoding"),
      "DRL-RUST-BROWSER-SAVE/2:fixed-m4-v1:4:mr;mr;mr;p"
    );
    assert_eq!(
      persistence::decode_snapshot_with_format(legacy)
        .expect("legacy decode")
        .format,
      persistence::SnapshotFormat::V1
    );
  }

  #[test]
  fn snapshot_rejects_corruption_and_unknown_versions() {
    let mut session = BrowserSession::new().expect("fixed session");
    assert_eq!(
      session.restore_snapshot("DRL-RUST-BROWSER-SAVE/3:fixed-m4-v1:w"),
      Err(SnapshotError::UnsupportedVersion("3".to_string()))
    );
    assert_eq!(
      session.restore_snapshot("DRL-RUST-BROWSER-SAVE/1:other:w"),
      Err(SnapshotError::UnsupportedContent("other".to_string()))
    );
    assert_eq!(
      session.restore_snapshot("not-a-snapshot"),
      Err(SnapshotError::Malformed)
    );
    assert_eq!(
      session.restore_snapshot("DRL-RUST-BROWSER-SAVE/1:fixed-m4-v1:w;;p"),
      Err(SnapshotError::Malformed)
    );
    assert_eq!(
      session.restore_snapshot("DRL-RUST-BROWSER-SAVE/2:fixed-m4-v1:2:w"),
      Err(SnapshotError::Malformed)
    );
    assert_eq!(
      session.restore_snapshot("DRL-RUST-BROWSER-SAVE/2:fixed-m4-v1:nope:w"),
      Err(SnapshotError::Malformed)
    );
    assert_eq!(
      session.restore_snapshot("DRL-RUST-BROWSER-SAVE/2:fixed-m4-v1:1:"),
      Err(SnapshotError::Malformed)
    );
    let oversized = format!("DRL-RUST-BROWSER-SAVE/1:fixed-m4-v1:{}", "w;".repeat(8193));
    assert_eq!(
      session.restore_snapshot(&oversized),
      Err(SnapshotError::TooLarge)
    );
  }

  #[test]
  fn rejected_snapshot_keeps_the_active_session_unchanged() {
    let mut session = BrowserSession::new().expect("fixed session");
    session
      .submit(Command::Move(Direction::East))
      .expect("legal command");
    let before_observation = session.observation();
    let before_replay = session.replay_log();
    let before_token = session.snapshot_token().expect("snapshot encoding");

    assert_eq!(
      session.restore_snapshot("DRL-RUST-BROWSER-SAVE/1:fixed-m4-v1:w;;p"),
      Err(SnapshotError::Malformed)
    );
    assert_eq!(session.observation(), before_observation);
    assert_eq!(session.replay_log(), before_replay);
    assert_eq!(
      session.snapshot_token().expect("snapshot encoding"),
      before_token
    );
  }

  #[test]
  fn snapshot_codec_covers_every_command_variant() {
    let commands = [
      Command::Move(Direction::None),
      Command::Move(Direction::NorthWest),
      Command::AttackMelee(Direction::SouthEast),
      Command::AttackRanged(Position::new(-3, 8)),
      Command::Wait,
      Command::Pickup,
      Command::Drop(ItemId::new(4)),
      Command::Equip(ItemId::new(5)),
      Command::Unequip(drl_protocol::EquipmentSlot::Weapon),
      Command::Unequip(drl_protocol::EquipmentSlot::Armor),
      Command::Use(ItemId::new(6)),
      Command::Invoke(ItemId::new(7)),
      Command::AltReload {
        item_id: ItemId::new(8),
        confirmed: true,
      },
      Command::Reload,
      Command::Descend,
    ];
    let token = persistence::encode_snapshot(&commands).expect("codec encoding");
    assert_eq!(
      persistence::decode_snapshot_with_format(&token)
        .expect("codec decoding")
        .commands,
      commands
    );
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
  fn browser_decal_requests_are_presentation_only() {
    let mut session = BrowserSession::new().expect("fixed session");
    let before = session.observation();
    session
      .try_insert_particle_decal(drl_render::ParticleDecalInsertion {
        placement: drl_render::ParticleDecalPlacement {
          cell: [1, 1],
          pixel: [32, 32],
        },
        sprite_id: 100_001,
      })
      .expect("retain presentation request");

    assert_eq!(session.observation(), before);
    assert_eq!(session.particle_decal_store().len(), 1);
    assert!(session.particle_decal_sprites().is_empty());
  }

  #[test]
  fn animation_clock_rebases_after_hidden_frames() {
    let mut clock = AnimationClock::default();
    assert_eq!(clock.elapsed_ms(false, 100.0), Some(0));
    assert_eq!(clock.elapsed_ms(false, 101.0), Some(1));
    assert_eq!(clock.elapsed_ms(true, 500.0), None);
    assert_eq!(clock.elapsed_ms(false, 501.0), Some(0));
    assert_eq!(clock.elapsed_ms(false, 502.0), Some(1));
    clock.reset();
    assert_eq!(clock.elapsed_ms(false, 900.0), Some(0));
  }

  #[test]
  fn animation_clock_rebases_on_visibility_lifecycle_change() {
    let mut clock = AnimationClock::default();
    assert_eq!(clock.elapsed_ms(false, 100.0), Some(0));
    assert_eq!(clock.elapsed_ms(false, 101.0), Some(1));
    clock.visibility_changed();
    assert_eq!(clock.elapsed_ms(false, 500.0), Some(0));
    clock.visibility_changed();
    assert_eq!(clock.elapsed_ms(false, 900.0), Some(0));
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

  #[test]
  fn subtle_knife_browser_boundary_matches_direct_core_presentation() {
    let mut setup_replay =
      ReplayLog::new(784, 30, 30, Position::new(15, 15)).with_player_config(PlayerSpawnConfig {
        hp: 50,
        max_hp: 50,
        speed: 100,
        initial_items: Vec::new(),
        equipped_weapon: Some(ItemSpawnKind::SubtleKnife),
        equipped_armor: None,
        equipped_armor_durability: None,
      });
    setup_replay.record_tile(Position::new(17, 15), TileKind::Wall);
    setup_replay.record_monster(MonsterSpawnSpec::new(
      Position::new(16, 15),
      "Visible Imp",
      30,
      1,
      (1, 1),
    ));
    setup_replay.record_monster(MonsterSpawnSpec::new(
      Position::new(18, 15),
      "Occluded Imp",
      30,
      1,
      (1, 1),
    ));

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let command = Command::Invoke(ItemId::new(4));

    let expected_events = direct.step(command).expect("direct invoke");
    let step = browser.submit(command).expect("browser invoke");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    let mut command_replay = setup_replay;
    command_replay.record_command(command);
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed.observe_player(), direct.observe_player());
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn trigun_vertical_browser_boundary_matches_direct_core_presentation() {
    let mut setup_replay =
      ReplayLog::new(42, 12, 4, Position::new(1, 1)).with_player_config(PlayerSpawnConfig {
        hp: 20,
        max_hp: 50,
        speed: 100,
        initial_items: Vec::new(),
        equipped_weapon: Some(ItemSpawnKind::Trigun),
        equipped_armor: None,
        equipped_armor_durability: None,
      });
    setup_replay.record_tile(Position::new(8, 1), TileKind::Wall);
    setup_replay.record_monster(
      MonsterSpawnSpec::new(Position::new(4, 1), "Imp", 20, 100, (3, 8))
        .with_ranged_combat((2, 5), 7, 70)
        .with_death_drop(Some(ItemSpawnKind::SmallMedPack)),
    );
    setup_replay.record_monster(
      MonsterSpawnSpec::new(Position::new(9, 1), "Imp", 20, 100, (3, 8))
        .with_ranged_combat((2, 5), 7, 70)
        .with_death_drop(Some(ItemSpawnKind::SmallMedPack)),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let initial_observation = initial.observe_player();
    let visible_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| actor.position() == Position::new(4, 1))
      .expect("visible actor")
      .id();
    let hidden_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| actor.position() == Position::new(9, 1))
      .expect("occluded actor")
      .id();
    let trigun_id = initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("equipped Trigun")
      .id();
    assert!(
      initial_observation
        .visible_actors
        .iter()
        .any(|actor| actor.id == visible_id)
    );
    assert!(
      !initial_observation
        .visible_actors
        .iter()
        .any(|actor| actor.id == hidden_id)
    );

    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let command = Command::AltReload {
      item_id: trigun_id,
      confirmed: true,
    };

    let expected_events = direct.step(command).expect("direct alternate reload");
    let step = browser.submit(command).expect("browser alternate reload");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    assert!(direct.is_game_over());
    assert!(browser.is_game_over());
    assert_eq!(
      direct.world().get_actor(visible_id).unwrap().hp().current,
      20
    );
    assert_eq!(
      direct.world().get_actor(hidden_id).unwrap().hp().current,
      20
    );

    let mut command_replay = setup_replay;
    command_replay.record_command(command);
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed.observe_player(), direct.observe_player());
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn nuclear_plasma_overload_browser_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let mut setup_replay =
      ReplayLog::new(794, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
        hp: 50,
        max_hp: 50,
        speed: 100,
        initial_items: Vec::new(),
        equipped_weapon: Some(ItemSpawnKind::NuclearPlasmaRifle),
        equipped_armor: None,
        equipped_armor_durability: None,
      });
    setup_replay.record_tile(player_position, TileKind::Acid);

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let plasma_id = initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("equipped Nuclear Plasma Rifle")
      .id();
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let command = Command::AltReload {
      item_id: plasma_id,
      confirmed: true,
    };

    let expected_events = direct.step(command).expect("direct overload");
    let step = browser.submit(command).expect("browser overload");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    assert!(direct.is_game_over());
    assert!(browser.is_game_over());
    assert_eq!(direct.world().player().unwrap().equipment().weapon(), None);

    let mut command_replay = setup_replay;
    command_replay.record_command(command);
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed, direct);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn nuclear_bfg_overload_browser_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let mut setup_replay =
      ReplayLog::new(795, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
        hp: 50,
        max_hp: 50,
        speed: 100,
        initial_items: Vec::new(),
        equipped_weapon: Some(ItemSpawnKind::NuclearBfg9000),
        equipped_armor: None,
        equipped_armor_durability: None,
      });
    setup_replay.record_tile(player_position, TileKind::Acid);

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let bfg_id = initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("equipped Nuclear BFG 9000")
      .id();
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let command = Command::AltReload {
      item_id: bfg_id,
      confirmed: true,
    };

    let expected_events = direct.step(command).expect("direct overload");
    let step = browser.submit(command).expect("browser overload");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    assert!(direct.is_game_over());
    assert!(browser.is_game_over());
    assert_eq!(direct.world().player().unwrap().equipment().weapon(), None);

    let mut command_replay = setup_replay;
    command_replay.record_command(command);
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed, direct);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn acid_spitter_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let mut setup_replay =
      ReplayLog::new(42, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
        hp: 50,
        max_hp: 50,
        speed: 100,
        initial_items: Vec::new(),
        equipped_weapon: Some(ItemSpawnKind::AcidSpitter),
        equipped_armor: None,
        equipped_armor_durability: None,
      });
    setup_replay.record_tile(player_position, TileKind::Acid);
    setup_replay.record_tile(player_position + Direction::East, TileKind::Water);

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "AcidSpitterVertical",
      "Acid Spitter reload converts the current cell to Water",
      "########\n#@w....#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.tiles.insert(player_position, drl_core::Tile::Acid);
    scenario.player_config = Some(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::AcidSpitter),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );
    let acid_spitter_id = initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("equipped Acid Spitter")
      .id();
    assert_eq!(
      initial.world().map().get_tile(player_position),
      Some(Tile::Acid)
    );
    assert_eq!(
      initial
        .world()
        .map()
        .get_tile(player_position + Direction::East),
      Some(Tile::Water)
    );

    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let command = Command::Reload;
    let expected_events = direct.step(command).expect("direct terrain reload");
    let step = browser.submit(command).expect("browser terrain reload");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(
      step.effects,
      vec![drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Reload,
        start_tick: 0,
        duration_ticks: 3,
      }]
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    assert_eq!(
      direct.world().map().get_tile(player_position),
      Some(Tile::Water)
    );
    assert!(
      step
        .after
        .visible_tiles
        .iter()
        .any(|tile| { tile.position == player_position && tile.kind == TileKind::Water })
    );
    assert_eq!(browser.observation().player_position, player_position);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .id(),
      acid_spitter_id
    );

    let reload_index = expected_events
      .iter()
      .position(|event| matches!(event, drl_protocol::GameEvent::AcidSpitterReloaded { .. }))
      .expect("terrain reload event");
    let cost_index = expected_events
      .iter()
      .position(|event| matches!(event, drl_protocol::GameEvent::ActionCostPaid { .. }))
      .expect("reload action cost");
    let turn_end_index = expected_events
      .iter()
      .position(|event| matches!(event, drl_protocol::GameEvent::TurnEnded { .. }))
      .expect("reload turn end");
    assert!(reload_index < cost_index);
    assert!(cost_index < turn_end_index);

    let mut command_replay = setup_replay;
    command_replay.record_command(command);
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed.observe_player(), direct.observe_player());
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn null_pointer_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(2, 1);
    let mut setup_replay =
      ReplayLog::new(25, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
        hp: 50,
        max_hp: 50,
        speed: 100,
        initial_items: Vec::new(),
        equipped_weapon: Some(ItemSpawnKind::NullPointer),
        equipped_armor: None,
        equipped_armor_durability: None,
      });
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Boss Target", 20, 100, (3, 8))
        .with_ranged_combat((2, 5), 7, 70)
        .with_death_drop(Some(ItemSpawnKind::SmallMedPack))
        .with_boss(true),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "NullPointerVertical",
      "Boss target for the typed Null Pointer encounter",
      "########\n#@i....#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 25;
    scenario.monsters[0].name = "Boss Target".to_string();
    scenario.monsters[0].is_boss = true;
    scenario.player_config = Some(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::NullPointer),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let player_id = initial.world().player_id().expect("player");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| actor.position() == target_position)
      .expect("boss target")
      .id();
    let item_id = initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("equipped Null Pointer")
      .id();
    assert!(
      initial
        .observe_player()
        .visible_actors
        .iter()
        .any(|actor| actor.id == target_id)
    );

    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let command = Command::AttackRanged(target_position);
    let expected_events = direct.step(command).expect("direct ranged hit");
    let step = browser.submit(command).expect("browser ranged hit");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(
      step.effects,
      vec![
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::RangedAttack,
          start_tick: 0,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::MeleeAttack,
          start_tick: 2,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Hit,
          start_tick: 4,
          duration_ticks: 1,
        },
      ]
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    assert_eq!(
      direct.world().get_actor(target_id).unwrap().score_count(),
      1000
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .id(),
      item_id
    );
    assert_eq!(direct.world().player().unwrap().id(), player_id);

    let attack_index = expected_events
      .iter()
      .position(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::AttackResolved {
            is_ranged: true,
            ..
          }
        )
      })
      .expect("ranged attack event");
    let hit_index = expected_events
      .iter()
      .position(|event| matches!(event, drl_protocol::GameEvent::NullPointerHit { .. }))
      .expect("Null Pointer hit event");
    let explosion_index = expected_events
      .iter()
      .position(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::NullPointerExplosionScheduled { .. }
        )
      })
      .expect("deferred explosion event");
    assert!(attack_index < hit_index);
    assert!(hit_index < explosion_index);

    let mut command_replay = setup_replay;
    command_replay.record_command(command);
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed, direct);
    assert_eq!(replayed.observe_player(), direct.observe_player());
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn grammaton_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(3, 1);
    let mut setup_replay =
      ReplayLog::new(4, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
        hp: 50,
        max_hp: 50,
        speed: 100,
        initial_items: Vec::new(),
        equipped_weapon: Some(ItemSpawnKind::GrammatonBeretta),
        equipped_armor: None,
        equipped_armor_durability: None,
      });
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Burst Target", 200, 1, (3, 8))
        .with_ranged_combat((2, 5), 7, 70)
        .with_death_drop(Some(ItemSpawnKind::SmallMedPack)),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "GrammatonVertical",
      "Burst-mode Grammaton encounter against a visible target",
      "########\n#@.i...#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 4;
    scenario.monsters[0].name = "Burst Target".to_string();
    scenario.monsters[0].hp = 200;
    scenario.monsters[0].speed = 1;
    scenario.player_config = Some(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::GrammatonBeretta),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let grammaton_id = initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("equipped Grammaton")
      .id();
    let mode_command = Command::AltReload {
      item_id: grammaton_id,
      confirmed: true,
    };
    let attack_command = Command::AttackRanged(target_position);
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);

    let mode_events = direct.step(mode_command).expect("direct mode cycle");
    let mode_step = browser.submit(mode_command).expect("browser mode cycle");
    assert_eq!(mode_step.events, mode_events);
    assert_eq!(mode_step.after, direct.observe_player());
    assert!(mode_step.effects.is_empty());
    assert!(mode_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::GrammatonFireModeChanged {
          item_id,
          mode: drl_protocol::WeaponFireMode::Burst,
          score_count_remaining: -200,
          ..
        } if *item_id == grammaton_id
      )
    }));

    let expected_events = direct.step(attack_command).expect("direct burst attack");
    let step = browser
      .submit(attack_command)
      .expect("browser burst attack");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(
      step.effects,
      vec![
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::RangedAttack,
          start_tick: 0,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Hit,
          start_tick: 2,
          duration_ticks: 1,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::RangedAttack,
          start_tick: 3,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Hit,
          start_tick: 5,
          duration_ticks: 1,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::RangedAttack,
          start_tick: 6,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Hit,
          start_tick: 8,
          duration_ticks: 1,
        },
      ]
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(
      direct
        .world()
        .player()
        .expect("player")
        .equipment()
        .weapon()
        .expect("weapon")
        .weapon_properties()
        .expect("weapon properties")
        .current_clip,
      15
    );

    let mut command_replay = setup_replay;
    command_replay.record_command(mode_command);
    command_replay.record_command(attack_command);
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    let mut expected_full_events = mode_events;
    expected_full_events.extend(expected_events);
    assert_eq!(replay_events, expected_full_events);
    assert_eq!(replayed, direct);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn jackhammer_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(3, 1);
    let mut setup_replay =
      ReplayLog::new(3, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
        hp: 50,
        max_hp: 50,
        speed: 100,
        initial_items: Vec::new(),
        equipped_weapon: Some(ItemSpawnKind::Jackhammer),
        equipped_armor: None,
        equipped_armor_durability: None,
      });
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Single Target", 100, 1, (3, 8))
        .with_ranged_combat((2, 5), 7, 70)
        .with_death_drop(Some(ItemSpawnKind::SmallMedPack)),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "JackhammerVertical",
      "Single-mode Jackhammer encounter against a visible target",
      "########\n#@.i...#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 3;
    scenario.monsters[0].name = "Single Target".to_string();
    scenario.monsters[0].hp = 100;
    scenario.monsters[0].speed = 1;
    scenario.player_config = Some(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Jackhammer),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let jackhammer_id = initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("equipped Jackhammer")
      .id();
    let mode_command = Command::AltReload {
      item_id: jackhammer_id,
      confirmed: true,
    };
    let attack_command = Command::AttackRanged(target_position);
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);

    let mode_events = direct.step(mode_command).expect("direct mode toggle");
    let mode_step = browser.submit(mode_command).expect("browser mode toggle");
    assert_eq!(mode_step.events, mode_events);
    assert_eq!(mode_step.after, direct.observe_player());
    assert!(mode_step.effects.is_empty());
    assert!(mode_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::JackhammerFireModeChanged {
          item_id,
          mode: drl_protocol::WeaponFireMode::Single,
          score_count_remaining: -1,
          ..
        } if *item_id == jackhammer_id
      )
    }));

    let expected_events = direct.step(attack_command).expect("direct single attack");
    let step = browser
      .submit(attack_command)
      .expect("browser single attack");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(
      step.effects,
      vec![
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::RangedAttack,
          start_tick: 0,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Hit,
          start_tick: 2,
          duration_ticks: 1,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Knockback,
          start_tick: 3,
          duration_ticks: 2,
        },
      ]
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(
      direct
        .world()
        .player()
        .expect("player")
        .equipment()
        .weapon()
        .expect("weapon")
        .weapon_properties()
        .expect("weapon properties")
        .current_clip,
      9
    );
    assert_eq!(
      direct
        .world()
        .actors()
        .values()
        .find(|actor| actor.name() == "Single Target")
        .expect("target")
        .position(),
      Position::new(4, 1)
    );

    let mut command_replay = setup_replay;
    command_replay.record_command(mode_command);
    command_replay.record_command(attack_command);
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    let mut expected_full_events = mode_events;
    expected_full_events.extend(expected_events);
    assert_eq!(replay_events, expected_full_events);
    assert_eq!(replayed, direct);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn lava_armor_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let mut setup_replay =
      ReplayLog::new(17, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
        hp: 50,
        max_hp: 50,
        speed: 100,
        initial_items: Vec::new(),
        equipped_weapon: Some(ItemSpawnKind::Pistol),
        equipped_armor: Some(ItemSpawnKind::LavaArmor),
        equipped_armor_durability: Some(97),
      });
    setup_replay.record_tile(player_position, drl_protocol::TileKind::Lava);
    setup_replay.record_tile(
      player_position + Direction::East,
      drl_protocol::TileKind::Lava,
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "LavaArmorVertical",
      "Lava Armor recharge encounter on a canonical Lava tile",
      "########\n#@=....#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 17;
    scenario.tiles.insert(player_position, drl_core::Tile::Lava);
    scenario.player_config = Some(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Pistol),
      equipped_armor: Some(ItemSpawnKind::LavaArmor),
      equipped_armor_durability: Some(97),
    });
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let commands = [Command::Wait; 5];
    let mut expected_events = Vec::new();
    for (index, command) in commands.iter().copied().enumerate() {
      let direct_events = direct.step(command).expect("direct wait");
      let step = browser.submit(command).expect("browser wait");
      assert_eq!(step.events, direct_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(step.effects, Vec::new());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &direct_events,)
      );
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      if index < 4 {
        assert_eq!(
          direct.world().player().unwrap().lava_recharge_timer(),
          (index + 1) as u32
        );
      } else {
        assert!(direct_events.iter().any(|event| {
          matches!(
            event,
            drl_protocol::GameEvent::LavaArmorRecharged {
              durability_restored: 3,
              durability_remaining: 100,
              timer: 0,
              ..
            }
          )
        }));
        assert_eq!(direct.world().player().unwrap().lava_recharge_timer(), 0);
      }
      expected_events.extend(direct_events);
    }

    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);
    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed, direct);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn blaster_recharge_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(2, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Blaster),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let target =
      drl_protocol::MonsterSpawnSpec::new(target_position, "Recharge Target", 1_000, 1, (0, 0));
    let mut setup_replay =
      ReplayLog::new(31, 8, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(target.clone());

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "BlasterRechargeVertical",
      "Blaster recharge after an accepted-command interval",
      "########\n#@i....#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 31;
    scenario.monsters[0] = target;
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut commands = Vec::with_capacity(40);
    commands.push(Command::AttackRanged(target_position));
    commands.extend(std::iter::repeat_n(Command::Wait, 39));
    let mut expected_events = Vec::new();
    for (index, command) in commands.iter().copied().enumerate() {
      let direct_events = direct.step(command).expect("direct command");
      let step = browser.submit(command).expect("browser command");
      assert_eq!(step.events, direct_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &direct_events,)
      );
      if index > 0 {
        assert_eq!(step.effects, Vec::new());
      }
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      if index < 39 {
        assert!(
          !direct_events
            .iter()
            .any(|event| matches!(event, drl_protocol::GameEvent::WeaponRecharged { .. }))
        );
      } else {
        assert!(direct_events.iter().any(|event| {
          matches!(
            event,
            drl_protocol::GameEvent::WeaponRecharged {
              ammo_recharged: 1,
              current_clip: 10,
              max_clip: 10,
              timer: 30,
              ..
            }
          )
        }));
      }
      expected_events.extend(direct_events);
    }

    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);
    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed, direct);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn nuclear_plasma_recharge_browser_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(2, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::NuclearPlasmaRifle),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let target =
      drl_protocol::MonsterSpawnSpec::new(target_position, "Recharge Target", 1_000, 1, (0, 0));
    let mut setup_replay =
      ReplayLog::new(32, 8, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(target);
    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());

    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut commands = Vec::with_capacity(42);
    commands.push(Command::AttackRanged(target_position));
    commands.extend(std::iter::repeat_n(Command::Wait, 41));
    let mut expected_events = Vec::new();
    for (index, command) in commands.iter().copied().enumerate() {
      let direct_events = direct.step(command).expect("direct command");
      let step = browser.submit(command).expect("browser command");
      assert_eq!(step.events, direct_events);
      assert_eq!(step.after, direct.observe_player());
      if index < 41 {
        assert!(
          !direct_events
            .iter()
            .any(|event| matches!(event, drl_protocol::GameEvent::WeaponRecharged { .. }))
        );
      } else {
        assert!(direct_events.iter().any(|event| {
          matches!(
            event,
            drl_protocol::GameEvent::WeaponRecharged {
              ammo_recharged: 1,
              current_clip: 24,
              max_clip: 24,
              timer: 40,
              ..
            }
          )
        }));
      }
      expected_events.extend(direct_events);
    }

    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);
    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed, direct);
    assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).unwrap());
  }

  #[test]
  fn nuclear_bfg_recharge_browser_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(2, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::NuclearBfg9000),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let target =
      drl_protocol::MonsterSpawnSpec::new(target_position, "Recharge Target", 1_000, 1, (0, 0));
    let mut setup_replay =
      ReplayLog::new(33, 8, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(target);
    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());

    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut commands = Vec::with_capacity(5);
    commands.push(Command::AttackRanged(target_position));
    commands.extend(std::iter::repeat_n(Command::Wait, 4));
    let mut expected_events = Vec::new();
    for (index, command) in commands.iter().copied().enumerate() {
      let direct_events = direct.step(command).expect("direct command");
      let step = browser.submit(command).expect("browser command");
      assert_eq!(step.events, direct_events);
      assert_eq!(step.after, direct.observe_player());
      if index < 4 {
        assert!(
          !direct_events
            .iter()
            .any(|event| matches!(event, drl_protocol::GameEvent::WeaponRecharged { .. }))
        );
      } else {
        assert!(direct_events.iter().any(|event| {
          matches!(
            event,
            drl_protocol::GameEvent::WeaponRecharged {
              ammo_recharged: 1,
              current_clip: 40,
              max_clip: 40,
              timer: 0,
              ..
            }
          )
        }));
      }
      expected_events.extend(direct_events);
    }

    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);
    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed, direct);
    assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).unwrap());
  }

  #[test]
  fn if_noreload_denial_browser_boundary_matches_direct_core() {
    for kind in [
      ItemSpawnKind::Blaster,
      ItemSpawnKind::NuclearPlasmaRifle,
      ItemSpawnKind::NuclearBfg9000,
    ] {
      let replay =
        ReplayLog::new(1_764, 8, 4, Position::new(1, 1)).with_player_config(PlayerSpawnConfig {
          hp: 50,
          max_hp: 50,
          speed: 100,
          initial_items: Vec::new(),
          equipped_weapon: Some(kind),
          equipped_armor: None,
          equipped_armor_durability: None,
        });
      let (initial, setup_events) = drl_core::ReplayEngine::run(&replay).expect("replay setup");
      assert!(setup_events.is_empty());

      let mut direct = initial.clone();
      let before = direct.clone();
      assert!(matches!(
        direct.step(Command::Reload),
        Err(drl_protocol::CommandError::CannotReload(_))
      ));
      assert_eq!(direct, before);

      let mut browser = BrowserSession::from_game(initial);
      let observation_before = browser.observation();
      let replay_before = browser.replay_log().clone();
      let weapon_id = observation_before
        .equipped_weapon
        .as_ref()
        .expect("configured weapon")
        .id;
      let error = browser.submit(Command::Reload).unwrap_err();
      assert_eq!(
        error,
        drl_protocol::CommandError::CannotReload(weapon_id).to_string()
      );
      assert_eq!(browser.observation(), observation_before);
      assert_eq!(browser.replay_log(), replay_before);
    }
  }

  #[test]
  fn medical_powerarmor_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let setup_replay =
      ReplayLog::new(23, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
        hp: 20,
        max_hp: 50,
        speed: 100,
        initial_items: Vec::new(),
        equipped_weapon: Some(ItemSpawnKind::Pistol),
        equipped_armor: Some(ItemSpawnKind::MedicalPowerarmor),
        equipped_armor_durability: Some(100),
      });

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let scenario = drl_core::scenario::Scenario::from_ascii(
      "MedicalPowerarmorVertical",
      "Medical Powerarmor periodic repair encounter",
      "########\n#@.....#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    let mut scenario = scenario;
    scenario.seed = 23;
    scenario.player_config = Some(PlayerSpawnConfig {
      hp: 20,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Pistol),
      equipped_armor: Some(ItemSpawnKind::MedicalPowerarmor),
      equipped_armor_durability: Some(100),
    });
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let commands = [Command::Wait; 30];
    let mut expected_events = Vec::new();
    for (index, command) in commands.iter().copied().enumerate() {
      let direct_events = direct.step(command).expect("direct wait");
      let step = browser.submit(command).expect("browser wait");
      assert_eq!(step.events, direct_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(step.effects, Vec::new());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &direct_events,)
      );
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      if index < 29 {
        assert_eq!(
          direct.world().player().unwrap().medical_repair_timer(),
          (index + 1) as u32
        );
      } else {
        assert!(direct_events.iter().any(|event| {
          matches!(
            event,
            drl_protocol::GameEvent::MedicalPowerarmorRepaired {
              healed: 1,
              remaining_hp: 21,
              durability_remaining: 99,
              timer: 20,
              ..
            }
          )
        }));
        assert_eq!(direct.world().player().unwrap().medical_repair_timer(), 20);
      }
      expected_events.extend(direct_events);
    }

    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);
    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed, direct);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn maleks_armor_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let setup_replay =
      ReplayLog::new(24, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
        hp: 50,
        max_hp: 50,
        speed: 100,
        initial_items: Vec::new(),
        equipped_weapon: Some(ItemSpawnKind::Pistol),
        equipped_armor: Some(ItemSpawnKind::MaleksArmor),
        equipped_armor_durability: Some(99),
      });

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "MalekArmorVertical",
      "Malek's Armor periodic durability recharge encounter",
      "########\n#@.....#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 24;
    scenario.player_config = Some(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Pistol),
      equipped_armor: Some(ItemSpawnKind::MaleksArmor),
      equipped_armor_durability: Some(99),
    });
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let commands = [Command::Wait; 56];
    let mut expected_events = Vec::new();
    for (index, command) in commands.iter().copied().enumerate() {
      let direct_events = direct.step(command).expect("direct wait");
      let step = browser.submit(command).expect("browser wait");
      assert_eq!(step.events, direct_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(step.effects, Vec::new());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &direct_events,)
      );
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      if index < 54 {
        assert_eq!(
          direct.world().player().unwrap().malek_recharge_timer(),
          (index + 1) as u32
        );
      } else if index == 54 {
        assert!(direct_events.iter().any(|event| {
          matches!(
            event,
            drl_protocol::GameEvent::MalekArmorRecharged {
              durability_restored: 1,
              durability_remaining: 100,
              timer: 50,
              ..
            }
          )
        }));
        assert_eq!(direct.world().player().unwrap().malek_recharge_timer(), 50);
      } else {
        assert!(
          !direct_events
            .iter()
            .any(|event| matches!(event, drl_protocol::GameEvent::MalekArmorRecharged { .. }))
        );
        assert_eq!(direct.world().player().unwrap().malek_recharge_timer(), 50);
      }
      expected_events.extend(direct_events);
    }

    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);
    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed, direct);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn former_human_profile_progression_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let setup_replay =
      ReplayLog::new(0, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
        hp: 50,
        max_hp: 50,
        speed: 100,
        initial_items: Vec::new(),
        equipped_weapon: Some(ItemSpawnKind::Pistol),
        equipped_armor: None,
        equipped_armor_durability: None,
      });
    let mut setup_replay = setup_replay;
    setup_replay.record_stairs(Position::new(5, 1));
    setup_replay.record_monster(
      MonsterSpawnSpec::new(Position::new(4, 1), "Progression Target", 10, 100, (2, 5))
        .with_ranged_combat((1, 4), 6, 65)
        .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let scenario = drl_core::scenario::Scenario::from_ascii(
      "FormerHumanProfileProgressionVertical",
      "Pistol progression through a Former Human profile, dropped ammunition, and stairs",
      "########\n#@..h>.#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    let mut scenario = scenario;
    scenario.seed = 0;
    scenario.monsters[0].name = "Progression Target".to_string();
    scenario.player_config = Some(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Pistol),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let monster_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("Former Human identity")
      .id();
    let target = Position::new(4, 1);
    let commands = vec![
      Command::Move(Direction::East),
      Command::AttackRanged(target),
      Command::AttackRanged(target),
      Command::Move(Direction::East),
      Command::Move(Direction::East),
      Command::Pickup,
      Command::Move(Direction::East),
      Command::Descend,
    ];

    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut expected_events = Vec::new();
    let expected_effects = [
      vec![
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Move,
          start_tick: 0,
          duration_ticks: 1,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::RangedAttack,
          start_tick: 1,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Hit,
          start_tick: 3,
          duration_ticks: 1,
        },
      ],
      vec![
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::RangedAttack,
          start_tick: 0,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Hit,
          start_tick: 2,
          duration_ticks: 1,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::RangedAttack,
          start_tick: 3,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Hit,
          start_tick: 5,
          duration_ticks: 1,
        },
      ],
      vec![
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::RangedAttack,
          start_tick: 0,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Hit,
          start_tick: 2,
          duration_ticks: 1,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Death,
          start_tick: 3,
          duration_ticks: 4,
        },
      ],
      vec![drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Move,
        start_tick: 0,
        duration_ticks: 1,
      }],
      vec![drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Move,
        start_tick: 0,
        duration_ticks: 1,
      }],
      vec![drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Pickup,
        start_tick: 0,
        duration_ticks: 2,
      }],
      vec![drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Move,
        start_tick: 0,
        duration_ticks: 1,
      }],
      vec![drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::LevelTransition,
        start_tick: 0,
        duration_ticks: 4,
      }],
    ];
    for (command, literal_effects) in commands.iter().copied().zip(expected_effects) {
      let direct_events = direct.step(command).expect("progression command");
      let step = browser
        .submit(command)
        .expect("browser progression command");
      assert_eq!(step.events, direct_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &direct_events,)
      );
      assert_eq!(step.effects, literal_effects);
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      expected_events.extend(direct_events);
    }

    assert_eq!(direct.world().level_id().0, 2);
    assert_eq!(direct.world().player().unwrap().hp().current, 44);
    assert!(
      direct
        .world()
        .player()
        .unwrap()
        .inventory()
        .has_ammo(drl_protocol::AmmoType::Ammo9mm, 10)
    );
    assert!(expected_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::ActorDied { entity_id, .. } if *entity_id == monster_id
      )
    }));
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);

    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed, direct);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn phase_device_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let mut setup_replay = ReplayLog::new(9999, 8, 4, player_position);
    setup_replay.record_item(ItemSpawnSpec::new(
      Position::new(2, 1),
      ItemSpawnKind::PhaseDevice,
    ));

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let scenario = drl_core::scenario::Scenario::from_ascii(
      "PhaseDeviceVertical",
      "Phase Device escape from a fixed arena",
      "########\n#@P....#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    let mut scenario = scenario;
    scenario.seed = 9999;
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let player_id = initial.world().player_id().expect("player identity");
    let device_id = initial
      .world()
      .ground_items()
      .keys()
      .next()
      .copied()
      .expect("phase device identity");
    let commands = vec![
      Command::Move(Direction::East),
      Command::Pickup,
      Command::Use(device_id),
    ];
    let expected_effects = [
      vec![drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Move,
        start_tick: 0,
        duration_ticks: 1,
      }],
      vec![drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Pickup,
        start_tick: 0,
        duration_ticks: 2,
      }],
      vec![
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Teleport,
          start_tick: 0,
          duration_ticks: 4,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Use,
          start_tick: 4,
          duration_ticks: 2,
        },
      ],
    ];

    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut expected_events = Vec::new();
    for (command, literal_effects) in commands.iter().copied().zip(expected_effects) {
      let direct_events = direct.step(command).expect("phase device command");
      let step = browser
        .submit(command)
        .expect("browser phase device command");
      assert_eq!(step.events, direct_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &direct_events,)
      );
      assert_eq!(step.effects, literal_effects);
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      expected_events.extend(direct_events);
    }

    assert_eq!(
      direct.world().player().unwrap().position(),
      Position::new(6, 2)
    );
    assert!(direct.world().is_explored(Position::new(6, 2)));
    assert!(
      direct
        .world()
        .player()
        .unwrap()
        .inventory()
        .get_item(device_id)
        .is_none()
    );
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);

    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("phase device command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed, direct);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
    assert!(expected_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::PlayerTeleported { from, to }
          if *from == Position::new(2, 1) && *to == Position::new(6, 2)
      )
    }));
    assert!(expected_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::ItemUsed { entity_id, item_id, .. }
          if *entity_id == player_id && *item_id == device_id
      )
    }));
  }

  #[test]
  fn shotgun_knockback_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Shotgun),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let target_position = Position::new(3, 1);
    let mut setup_replay =
      ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Knockback Target", 15, 100, (3, 6))
        .with_ranged_combat((2, 6), 5, 60)
        .with_death_drop(Some(ItemSpawnKind::AmmoShells(10))),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "ShotgunKnockbackVertical",
      "Shotgun knockback against a Former Sergeant profile",
      "########\n#@.s...#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 0;
    scenario.monsters[0].name = "Knockback Target".to_string();
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("knockback target")
      .id();
    let attack_command = Command::AttackRanged(target_position);
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);

    let expected_events = direct
      .step(attack_command)
      .expect("direct Shotgun knockback attack");
    let step = browser
      .submit(attack_command)
      .expect("browser Shotgun knockback attack");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(
      step.effects,
      vec![
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::RangedAttack,
          start_tick: 0,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Hit,
          start_tick: 2,
          duration_ticks: 1,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Knockback,
          start_tick: 3,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::RangedAttack,
          start_tick: 5,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Hit,
          start_tick: 7,
          duration_ticks: 1,
        },
      ]
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, vec![attack_command]);
    assert_eq!(
      direct.world().player().unwrap().hp().current,
      47,
      "Former Sergeant profile response should hit once"
    );
    assert_eq!(
      direct.world().get_actor(target_id).unwrap().position(),
      Position::new(4, 1)
    );
    assert_eq!(direct.world().get_actor(target_id).unwrap().hp().current, 3);
    assert!(expected_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::ActorKnockedBack { entity_id, from, to }
          if *entity_id == target_id
            && *from == target_position
            && *to == Position::new(4, 1)
      )
    }));
    assert!(expected_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == target_id && *event_target == player_id
      )
    }));

    let mut command_replay = setup_replay;
    command_replay.record_command(attack_command);
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed, direct);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn green_armor_protection_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Pistol),
      equipped_armor: Some(ItemSpawnKind::GreenArmor),
      equipped_armor_durability: None,
    };
    let target_position = Position::new(3, 1);
    let mut setup_replay =
      ReplayLog::new(4, 8, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Armor Target", 15, 100, (3, 6))
        .with_ranged_combat((2, 6), 5, 60)
        .with_death_drop(Some(ItemSpawnKind::AmmoShells(10))),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "GreenArmorProtectionVertical",
      "Green Armor mitigation against a Former Sergeant profile",
      "########\n#@.s...#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 4;
    scenario.monsters[0].name = "Armor Target".to_string();
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("armor target")
      .id();
    let command = Command::Wait;
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);

    let expected_events = direct.step(command).expect("direct armor response");
    let step = browser.submit(command).expect("browser armor response");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(
      step.effects,
      vec![
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::RangedAttack,
          start_tick: 0,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Hit,
          start_tick: 2,
          duration_ticks: 1,
        },
      ]
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, vec![command]);
    let armor = step.after.equipped_armor.as_ref().expect("Green Armor");
    assert_eq!(armor.name, "Green Armor");
    assert_eq!(armor.armor_value, Some(5));
    assert_eq!(direct.world().player().unwrap().hp().current, 49);
    assert!(expected_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { damage: 3, is_lethal: false },
          is_ranged: true,
        } if *attacker_id == target_id && *event_target == player_id
      )
    }));
    assert!(expected_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::DamageApplied {
          target_id: event_target,
          amount: 1,
          remaining_hp: 49,
          source: drl_protocol::DamageSource::Actor(source_id),
          ..
        } if *event_target == player_id && *source_id == target_id
      )
    }));

    let mut command_replay = setup_replay;
    command_replay.record_command(command);
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed, direct);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn small_medpack_recovery_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let player_config = PlayerSpawnConfig {
      hp: 45,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::SmallMedPack],
      equipped_weapon: None,
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let setup_replay =
      ReplayLog::new(2, 8, 4, player_position).with_player_config(player_config.clone());
    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let medpack_id = *initial
      .world()
      .player()
      .expect("player")
      .inventory()
      .items()
      .keys()
      .next()
      .expect("Small MedPack");
    assert_eq!(medpack_id, drl_protocol::ItemId::new(4));

    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "SmallMedPackRecoveryVertical",
      "Small MedPack recovery at the health cap",
      "########\n#@.....#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 2;
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let player_id = initial.world().player_id().expect("player identity");
    let command = Command::Use(medpack_id);
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);

    let expected_events = direct.step(command).expect("direct medpack use");
    let step = browser.submit(command).expect("browser medpack use");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(
      step.effects,
      vec![drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Use,
        start_tick: 0,
        duration_ticks: 2,
      }]
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, vec![command]);
    assert_eq!(step.after.player_hp.unwrap().current, 50);
    assert!(step.after.inventory.is_empty());
    assert!(expected_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::ItemUsed { entity_id, item_id, item_name }
          if *entity_id == player_id
            && *item_id == medpack_id
            && item_name == "Small MedPack"
      )
    }));

    let mut command_replay = setup_replay;
    command_replay.record_command(command);
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, expected_events);
    assert_eq!(replayed, direct);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn demon_medpack_recovery_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let player_config = PlayerSpawnConfig {
      hp: 46,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::SmallMedPack],
      equipped_weapon: None,
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let target_position = Position::new(2, 1);
    let mut setup_replay =
      ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Rush Demon", 30, 140, (5, 10))
        .with_death_drop(Some(ItemSpawnKind::LargeMedPack)),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let medpack_id = *initial
      .world()
      .player()
      .expect("player")
      .inventory()
      .items()
      .keys()
      .next()
      .expect("Small MedPack");
    assert_eq!(medpack_id, ItemId::new(4));

    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "DemonMedPackRecoveryVertical",
      "Demon melee pressure around Small MedPack recovery",
      "########\n#@d....#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 0;
    scenario.monsters[0].name = "Rush Demon".to_string();
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("rush demon")
      .id();
    let commands = [Command::Wait, Command::Use(medpack_id)];
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut all_events = Vec::new();
    let mut all_effects = Vec::new();
    let mut effect_offset = 0;

    for command in commands {
      let expected_events = direct.step(command).expect("direct demon encounter");
      let step = browser.submit(command).expect("browser demon encounter");
      assert_eq!(step.events, expected_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
      );
      let step_duration = step
        .effects
        .iter()
        .map(|span| u32::from(span.duration_ticks))
        .sum::<u32>();
      all_events.extend(expected_events);
      all_effects.extend(step.effects.into_iter().map(|span| drl_render::EffectSpan {
        start_tick: span.start_tick + effect_offset,
        ..span
      }));
      effect_offset += step_duration;
    }

    assert_eq!(direct.world().player().unwrap().hp().current, 41);
    assert!(direct.world().player().unwrap().inventory().is_empty());
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(
      browser.scene(),
      RenderScene::from_observation(&direct.observe_player())
    );
    assert_eq!(browser.replay_log().commands, commands);
    assert_eq!(all_events.len(), 14);
    assert!(all_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { damage: 6, is_lethal: false },
          is_ranged: false,
        } if *attacker_id == target_id && *event_target == player_id
      )
    }));
    assert!(all_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::ItemUsed { entity_id, item_id, item_name }
          if *entity_id == player_id
            && *item_id == medpack_id
            && item_name == "Small MedPack"
      )
    }));
    assert_eq!(
      all_effects,
      vec![
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::MeleeAttack,
          start_tick: 0,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Hit,
          start_tick: 2,
          duration_ticks: 1,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Use,
          start_tick: 3,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::MeleeAttack,
          start_tick: 5,
          duration_ticks: 2,
        },
        drl_render::EffectSpan {
          effect: drl_render::PresentationEffect::Hit,
          start_tick: 7,
          duration_ticks: 1,
        },
      ]
    );

    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, all_events);
    assert_eq!(replayed, direct);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn pistol_reload_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(20)],
      equipped_weapon: Some(ItemSpawnKind::Pistol),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let target_position = Position::new(3, 1);
    let mut setup_replay =
      ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
        .with_ranged_combat((1, 4), 6, 65)
        .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let ammo_id = ItemId::new(4);
    let pistol_id = ItemId::new(5);
    assert_eq!(
      initial
        .world()
        .player()
        .expect("player")
        .inventory()
        .get_item(ammo_id)
        .expect("9mm reserve")
        .count(),
      20
    );
    assert_eq!(
      initial
        .world()
        .player()
        .expect("player")
        .equipment()
        .weapon()
        .expect("Pistol")
        .id(),
      pistol_id
    );

    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "PistolReloadVertical",
      "Pistol clip depletion and deterministic reload",
      "########\n#@.h...#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 0;
    scenario.monsters[0].name = "Static Target".to_string();
    scenario.monsters[0].hp = 500;
    scenario.monsters[0].speed = 1;
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("static target")
      .id();
    let target = Position::new(3, 1);
    let mut commands = vec![Command::AttackRanged(target); 10];
    commands.push(Command::Reload);
    let ranged_attack = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::RangedAttack,
      start_tick: 0,
      duration_ticks: 2,
    };
    let hit = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Hit,
      start_tick: 2,
      duration_ticks: 1,
    };
    let reload = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Reload,
      start_tick: 0,
      duration_ticks: 3,
    };
    let expected_effects = [
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack],
      vec![ranged_attack],
      vec![reload],
    ];
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut all_events = Vec::new();
    let mut reload_effects = Vec::new();

    for (index, command) in commands.iter().copied().enumerate() {
      let expected_events = direct.step(command).expect("direct pistol command");
      let step = browser.submit(command).expect("browser pistol command");
      assert_eq!(step.events, expected_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
      );
      assert_eq!(step.effects, expected_effects[index]);
      if command == Command::Reload {
        reload_effects = step.effects;
      }
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      all_events.extend(expected_events);
    }

    assert_eq!(direct.world().player().unwrap().hp().current, 50);
    assert_eq!(
      direct.world().get_actor(target_id).unwrap().hp().current,
      458
    );
    let weapon = direct
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap();
    assert_eq!(weapon.current_clip, 10);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .inventory()
        .get_item(ammo_id)
        .unwrap()
        .count(),
      10
    );
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);
    assert_eq!(
      browser.observation().equipped_weapon.unwrap().clip,
      Some((10, 10))
    );
    assert_eq!(
      browser
        .observation()
        .inventory
        .iter()
        .find(|item| item.id == ammo_id)
        .unwrap()
        .count,
      10
    );
    assert_eq!(reload_effects, vec![reload]);
    assert_eq!(
      all_events
        .iter()
        .filter(|event| matches!(event, drl_protocol::GameEvent::AttackResolved { attacker_id, target_id: event_target, is_ranged: true, .. } if *attacker_id == player_id && *event_target == target_id))
        .count(),
      10
    );
    assert!(all_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 10,
          current_clip: 10,
          max_clip: 10,
        } if *entity_id == player_id
      )
    }));
    let reload_index = all_events
      .iter()
      .position(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::WeaponReloaded {
            entity_id,
            ammo_loaded: 10,
            current_clip: 10,
            max_clip: 10,
          } if *entity_id == player_id
        )
      })
      .expect("reload event");
    assert!(matches!(
      all_events.get(reload_index + 1),
      Some(drl_protocol::GameEvent::ActionCostPaid {
        entity_id,
        cost: drl_protocol::ActionCost(1000),
      }) if *entity_id == player_id
    ));
    assert!(matches!(
      all_events.get(reload_index + 2),
      Some(drl_protocol::GameEvent::TurnEnded { .. })
    ));

    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replay_events, all_events);
    assert_eq!(replayed, direct);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn plasma_rifle_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(12)],
      equipped_weapon: Some(ItemSpawnKind::PlasmaRifle),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let target_position = Position::new(3, 1);
    let mut setup_replay =
      ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
        .with_ranged_combat((1, 4), 6, 65)
        .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let cells_id = ItemId::new(4);
    let plasma_id = ItemId::new(5);
    assert_eq!(
      initial
        .world()
        .player()
        .expect("player")
        .inventory()
        .get_item(cells_id)
        .expect("cell reserve")
        .count(),
      12
    );
    assert_eq!(
      initial
        .world()
        .player()
        .expect("player")
        .equipment()
        .weapon()
        .expect("Plasma Rifle")
        .id(),
      plasma_id
    );

    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "PlasmaRifleCellVertical",
      "Plasma Rifle cell clip depletion and deterministic reload",
      "########\n#@.h...#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 0;
    scenario.monsters[0].name = "Static Target".to_string();
    scenario.monsters[0].hp = 500;
    scenario.monsters[0].speed = 1;
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("static target")
      .id();
    let target = Position::new(3, 1);
    let mut commands = vec![Command::AttackRanged(target); 6];
    commands.push(Command::Reload);
    let ranged_attack = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::RangedAttack,
      start_tick: 0,
      duration_ticks: 2,
    };
    let hit = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Hit,
      start_tick: 2,
      duration_ticks: 1,
    };
    let reload = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Reload,
      start_tick: 0,
      duration_ticks: 3,
    };
    let expected_effects = [
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack],
      vec![reload],
    ];
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut all_events = Vec::new();
    let mut reload_effects = Vec::new();

    for (index, command) in commands.iter().copied().enumerate() {
      let expected_events = direct.step(command).expect("direct Plasma Rifle command");
      let step = browser
        .submit(command)
        .expect("browser Plasma Rifle command");
      assert_eq!(step.events, expected_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
      );
      assert_eq!(step.effects, expected_effects[index]);
      if command == Command::Reload {
        reload_effects = step.effects;
      }
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      all_events.extend(expected_events);
    }

    assert_eq!(direct.world().player().unwrap().hp().current, 50);
    assert_eq!(
      direct.world().get_actor(target_id).unwrap().hp().current,
      480
    );
    let weapon = direct
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap();
    assert_eq!(weapon.current_clip, 6);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .inventory()
        .get_item(cells_id)
        .unwrap()
        .count(),
      6
    );
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);
    assert_eq!(
      browser.observation().equipped_weapon.unwrap().clip,
      Some((6, 6))
    );
    assert_eq!(
      browser
        .observation()
        .inventory
        .iter()
        .find(|item| item.id == cells_id)
        .unwrap()
        .count,
      6
    );
    assert_eq!(reload_effects, vec![reload]);
    assert_eq!(
      all_events
        .iter()
        .filter(|event| matches!(event, drl_protocol::GameEvent::AttackResolved { attacker_id, target_id: event_target, is_ranged: true, .. } if *attacker_id == player_id && *event_target == target_id))
        .count(),
      6
    );
    let reload_index = all_events
      .iter()
      .position(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::WeaponReloaded {
            entity_id,
            ammo_loaded: 6,
            current_clip: 6,
            max_clip: 6,
          } if *entity_id == player_id
        )
      })
      .expect("reload event");
    assert_eq!(
      all_events[..reload_index]
        .iter()
        .filter(|event| {
          matches!(
            event,
            drl_protocol::GameEvent::AttackResolved {
              attacker_id,
              target_id: event_target,
              is_ranged: true,
              ..
            } if *attacker_id == player_id && *event_target == target_id
          )
        })
        .count(),
      6
    );
    assert!(matches!(
      all_events.get(reload_index + 1),
      Some(drl_protocol::GameEvent::ActionCostPaid {
        entity_id,
        cost: drl_protocol::ActionCost(1000),
      }) if *entity_id == player_id
    ));
    assert!(matches!(
      all_events.get(reload_index + 2),
      Some(drl_protocol::GameEvent::TurnEnded { .. })
    ));

    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, all_events);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn rocket_launcher_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoRockets(2)],
      equipped_weapon: Some(ItemSpawnKind::RocketLauncher),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let target_position = Position::new(3, 1);
    let mut setup_replay =
      ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
        .with_ranged_combat((1, 4), 6, 65)
        .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let rockets_id = ItemId::new(4);
    let launcher_id = ItemId::new(5);
    assert_eq!(
      initial
        .world()
        .player()
        .expect("player")
        .inventory()
        .get_item(rockets_id)
        .expect("rocket reserve")
        .count(),
      2
    );
    assert_eq!(
      initial
        .world()
        .player()
        .expect("player")
        .equipment()
        .weapon()
        .expect("Rocket Launcher")
        .id(),
      launcher_id
    );

    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "RocketLauncherOneShotVertical",
      "Rocket Launcher one-shot clip depletion and deterministic reload",
      "########\n#@.h...#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 0;
    scenario.monsters[0].name = "Static Target".to_string();
    scenario.monsters[0].hp = 500;
    scenario.monsters[0].speed = 1;
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("static target")
      .id();
    let target = Position::new(3, 1);
    let commands = vec![Command::AttackRanged(target), Command::Reload];
    let ranged_attack = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::RangedAttack,
      start_tick: 0,
      duration_ticks: 2,
    };
    let hit = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Hit,
      start_tick: 2,
      duration_ticks: 1,
    };
    let reload = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Reload,
      start_tick: 0,
      duration_ticks: 3,
    };
    let expected_effects = [vec![ranged_attack, hit], vec![reload]];
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut all_events = Vec::new();
    let mut reload_effects = Vec::new();

    for (index, command) in commands.iter().copied().enumerate() {
      let expected_events = direct
        .step(command)
        .expect("direct Rocket Launcher command");
      let step = browser
        .submit(command)
        .expect("browser Rocket Launcher command");
      assert_eq!(step.events, expected_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
      );
      assert_eq!(step.effects, expected_effects[index]);
      if command == Command::Reload {
        reload_effects = step.effects;
      }
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      all_events.extend(expected_events);
    }

    assert_eq!(direct.world().player().unwrap().hp().current, 50);
    assert_eq!(
      direct.world().get_actor(target_id).unwrap().hp().current,
      471
    );
    let weapon = direct
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap();
    assert_eq!(weapon.current_clip, 1);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .inventory()
        .get_item(rockets_id)
        .unwrap()
        .count(),
      1
    );
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);
    assert_eq!(
      browser.observation().equipped_weapon.unwrap().clip,
      Some((1, 1))
    );
    assert_eq!(
      browser
        .observation()
        .inventory
        .iter()
        .find(|item| item.id == rockets_id)
        .unwrap()
        .count,
      1
    );
    assert_eq!(reload_effects, vec![reload]);
    assert_eq!(
      all_events
        .iter()
        .filter(|event| matches!(event, drl_protocol::GameEvent::AttackResolved { attacker_id, target_id: event_target, is_ranged: true, .. } if *attacker_id == player_id && *event_target == target_id))
        .count(),
      1
    );
    let reload_index = all_events
      .iter()
      .position(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::WeaponReloaded {
            entity_id,
            ammo_loaded: 1,
            current_clip: 1,
            max_clip: 1,
          } if *entity_id == player_id
        )
      })
      .expect("reload event");
    assert_eq!(
      all_events[..reload_index]
        .iter()
        .filter(|event| {
          matches!(
            event,
            drl_protocol::GameEvent::AttackResolved {
              attacker_id,
              target_id: event_target,
              is_ranged: true,
              ..
            } if *attacker_id == player_id && *event_target == target_id
          )
        })
        .count(),
      1
    );
    assert!(matches!(
      all_events.get(reload_index + 1),
      Some(drl_protocol::GameEvent::ActionCostPaid {
        entity_id,
        cost: drl_protocol::ActionCost(1000),
      }) if *entity_id == player_id
    ));
    assert!(matches!(
      all_events.get(reload_index + 2),
      Some(drl_protocol::GameEvent::TurnEnded { .. })
    ));

    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, all_events);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn missile_launcher_single_shell_reload_browser_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(3, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoRockets(2)],
      equipped_weapon: Some(ItemSpawnKind::MissileLauncher),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(1, 8, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Static Target", 1_000, 1, (2, 5))
        .with_ranged_combat((1, 4), 6, 65)
        .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
    );
    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());

    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "MissileLauncherSingleShellVertical",
      "Missile Launcher single-shell reload after clip depletion",
      "########\n#@.h...#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 1;
    scenario.monsters[0].name = "Static Target".to_string();
    scenario.monsters[0].hp = 1_000;
    scenario.monsters[0].speed = 1;
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let commands = vec![
      Command::AttackRanged(target_position),
      Command::AttackRanged(target_position),
      Command::AttackRanged(target_position),
      Command::AttackRanged(target_position),
      Command::Reload,
      Command::Reload,
    ];
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut all_events = Vec::new();
    for command in commands.iter().copied() {
      let expected_events = direct
        .step(command)
        .expect("direct Missile Launcher command");
      let step = browser
        .submit(command)
        .expect("browser Missile Launcher command");
      assert_eq!(step.events, expected_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
      );
      all_events.extend(expected_events);
    }

    let weapon = direct
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap();
    assert_eq!(weapon.current_clip, 2);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .inventory()
        .total_ammo(drl_protocol::AmmoType::Rocket),
      0
    );
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);

    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, all_events);
    assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).unwrap());
  }

  #[test]
  fn missile_launcher_alt_reload_browser_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(3, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoRockets(4)],
      equipped_weapon: Some(ItemSpawnKind::MissileLauncher),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(1, 8, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Static Target", 1_000, 1, (2, 5))
        .with_ranged_combat((1, 4), 6, 65)
        .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
    );
    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());

    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "MissileLauncherAltReloadVertical",
      "Missile Launcher alternate full reload after clip depletion",
      "########\n#@.h...#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 1;
    scenario.monsters[0].name = "Static Target".to_string();
    scenario.monsters[0].hp = 1_000;
    scenario.monsters[0].speed = 1;
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let player_id = initial.world().player_id().expect("player identity");
    let weapon_id = initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Missile Launcher")
      .id();
    let commands = [
      Command::AttackRanged(target_position),
      Command::AttackRanged(target_position),
      Command::AttackRanged(target_position),
      Command::AttackRanged(target_position),
      Command::AltReload {
        item_id: weapon_id,
        confirmed: false,
      },
    ];
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut all_events = Vec::new();
    for command in commands {
      let expected_events = direct
        .step(command)
        .expect("direct Missile Launcher command");
      let step = browser
        .submit(command)
        .expect("browser Missile Launcher command");
      assert_eq!(step.events, expected_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
      );
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      all_events.extend(expected_events);
    }

    let weapon = direct
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap();
    assert_eq!(weapon.current_clip, 4);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .inventory()
        .total_ammo(drl_protocol::AmmoType::Rocket),
      0
    );
    let reload_index = all_events
      .iter()
      .position(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::WeaponReloaded {
            entity_id,
            ammo_loaded: 4,
            current_clip: 4,
            max_clip: 4,
          } if *entity_id == player_id
        )
      })
      .expect("alternate reload event");
    assert!(matches!(
      all_events.get(reload_index + 1),
      Some(drl_protocol::GameEvent::ActionCostPaid {
        entity_id,
        cost: drl_protocol::ActionCost(2_500),
      }) if *entity_id == player_id
    ));
    assert!(matches!(
      all_events.get(reload_index + 2),
      Some(drl_protocol::GameEvent::TurnEnded { .. })
    ));
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);

    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, all_events);
    assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).unwrap());
  }

  #[test]
  fn chainsaw_melee_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(1, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Chainsaw),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let target_position = Position::new(2, 1);
    let mut setup_replay =
      ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (5, 10))
        .with_death_drop(Some(ItemSpawnKind::LargeMedPack)),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let chainsaw_id = ItemId::new(4);
    assert_eq!(
      initial
        .world()
        .player()
        .expect("player")
        .equipment()
        .weapon()
        .expect("Chainsaw")
        .id(),
      chainsaw_id
    );

    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "ChainsawMeleeVertical",
      "Chainsaw melee damage against a static Demon-profile target",
      "########\n#@d....#\n#......#\n########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 0;
    scenario.monsters[0].name = "Static Target".to_string();
    scenario.monsters[0].hp = 500;
    scenario.monsters[0].speed = 1;
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("static target")
      .id();
    let command = Command::AttackMelee(Direction::East);
    let expected_effects = vec![
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::MeleeAttack,
        start_tick: 0,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 2,
        duration_ticks: 1,
      },
    ];
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let expected_events = direct.step(command).expect("direct Chainsaw command");
    let step = browser.submit(command).expect("browser Chainsaw command");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(step.effects, expected_effects);
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, vec![command]);
    assert_eq!(
      direct.world().get_actor(target_id).unwrap().hp().current,
      480
    );
    assert!(matches!(
      expected_events.get(1),
      Some(drl_protocol::GameEvent::AttackResolved {
        attacker_id,
        target_id: event_target,
        outcome: drl_protocol::AttackOutcome::Hit { damage: 20, is_lethal: false },
        is_ranged: false,
      }) if *attacker_id == player_id && *event_target == target_id
    ));
    assert!(matches!(
      expected_events.get(3),
      Some(drl_protocol::GameEvent::ActionCostPaid {
        entity_id,
        cost: drl_protocol::ActionCost(1000),
      }) if *entity_id == player_id
    ));

    let mut command_replay = setup_replay;
    command_replay.record_command(command);
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, expected_events);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn shotgun_reload_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(2, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoShells(10)],
      equipped_weapon: Some(ItemSpawnKind::Shotgun),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let target_position = Position::new(7, 1);
    let mut setup_replay =
      ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
        .with_ranged_combat((1, 4), 6, 65)
        .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let shells_id = ItemId::new(4);
    let shotgun_id = ItemId::new(5);
    assert_eq!(
      initial
        .world()
        .player()
        .expect("player")
        .inventory()
        .get_item(shells_id)
        .expect("shell reserve")
        .count(),
      10
    );
    assert_eq!(
      initial
        .world()
        .player()
        .expect("player")
        .equipment()
        .weapon()
        .expect("Shotgun")
        .id(),
      shotgun_id
    );

    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "ShotgunReloadVertical",
      "Shotgun shell clip depletion and deterministic reload",
      "#########\n#.@....h#\n#.......#\n#########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 0;
    scenario.monsters[0].name = "Static Target".to_string();
    scenario.monsters[0].hp = 500;
    scenario.monsters[0].speed = 1;
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("static target")
      .id();
    let target = Position::new(7, 1);
    let mut commands = vec![Command::AttackRanged(target); 8];
    commands.push(Command::Reload);
    let ranged_attack = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::RangedAttack,
      start_tick: 0,
      duration_ticks: 2,
    };
    let hit = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Hit,
      start_tick: 2,
      duration_ticks: 1,
    };
    let reload = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Reload,
      start_tick: 0,
      duration_ticks: 3,
    };
    let expected_effects = [
      vec![ranged_attack],
      vec![ranged_attack],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack],
      vec![ranged_attack, hit],
      vec![reload],
    ];
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut all_events = Vec::new();
    let mut reload_effects = Vec::new();

    for (index, command) in commands.iter().copied().enumerate() {
      let expected_events = direct.step(command).expect("direct Shotgun command");
      let step = browser.submit(command).expect("browser Shotgun command");
      assert_eq!(step.events, expected_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
      );
      assert_eq!(step.effects, expected_effects[index]);
      if command == Command::Reload {
        reload_effects = step.effects;
      }
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      all_events.extend(expected_events);
    }

    assert_eq!(direct.world().player().unwrap().hp().current, 50);
    assert_eq!(
      direct.world().get_actor(target_id).unwrap().hp().current,
      429
    );
    assert_eq!(
      direct.world().get_actor(target_id).unwrap().position(),
      target
    );
    assert!(!all_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::ActorKnockedBack { entity_id, .. }
          if *entity_id == target_id
      )
    }));
    let weapon = direct
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap();
    assert_eq!(weapon.current_clip, 8);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .inventory()
        .get_item(shells_id)
        .unwrap()
        .count(),
      2
    );
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);
    assert_eq!(
      browser.observation().equipped_weapon.unwrap().clip,
      Some((8, 8))
    );
    assert_eq!(
      browser
        .observation()
        .inventory
        .iter()
        .find(|item| item.id == shells_id)
        .unwrap()
        .count,
      2
    );
    assert_eq!(reload_effects, vec![reload]);
    assert_eq!(
      all_events
        .iter()
        .filter(|event| matches!(event, drl_protocol::GameEvent::AttackResolved { attacker_id, target_id: event_target, is_ranged: true, .. } if *attacker_id == player_id && *event_target == target_id))
        .count(),
      8
    );
    let reload_index = all_events
      .iter()
      .position(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::WeaponReloaded {
            entity_id,
            ammo_loaded: 8,
            current_clip: 8,
            max_clip: 8,
          } if *entity_id == player_id
        )
      })
      .expect("reload event");
    assert_eq!(
      all_events[..reload_index]
        .iter()
        .filter(|event| {
          matches!(
            event,
            drl_protocol::GameEvent::AttackResolved {
              attacker_id,
              target_id: event_target,
              is_ranged: true,
              ..
            } if *attacker_id == player_id && *event_target == target_id
          )
        })
        .count(),
      8
    );
    assert!(matches!(
      all_events.get(reload_index + 1),
      Some(drl_protocol::GameEvent::ActionCostPaid {
        entity_id,
        cost: drl_protocol::ActionCost(1200),
      }) if *entity_id == player_id
    ));
    assert!(matches!(
      all_events.get(reload_index + 2),
      Some(drl_protocol::GameEvent::TurnEnded { .. })
    ));

    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, all_events);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn assault_shotgun_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(2, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoShells(8)],
      equipped_weapon: Some(ItemSpawnKind::AssaultShotgun),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let target_position = Position::new(7, 1);
    let mut setup_replay =
      ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
        .with_ranged_combat((1, 4), 6, 65)
        .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let shells_id = ItemId::new(4);
    let shotgun_id = ItemId::new(5);
    assert_eq!(
      initial
        .world()
        .player()
        .expect("player")
        .inventory()
        .get_item(shells_id)
        .expect("shell reserve")
        .count(),
      8
    );
    assert_eq!(
      initial
        .world()
        .player()
        .expect("player")
        .equipment()
        .weapon()
        .expect("Assault Shotgun")
        .id(),
      shotgun_id
    );

    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "AssaultShotgunVertical",
      "Assault Shotgun shell clip depletion and deterministic reload",
      "#########\n#.@....h#\n#.......#\n#########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 0;
    scenario.monsters[0].name = "Static Target".to_string();
    scenario.monsters[0].hp = 500;
    scenario.monsters[0].speed = 1;
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("static target")
      .id();
    let target = Position::new(7, 1);
    let mut commands = vec![Command::AttackRanged(target); 6];
    commands.push(Command::Reload);
    let ranged_attack = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::RangedAttack,
      start_tick: 0,
      duration_ticks: 2,
    };
    let hit = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Hit,
      start_tick: 2,
      duration_ticks: 1,
    };
    let reload = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Reload,
      start_tick: 0,
      duration_ticks: 3,
    };
    let expected_effects = [
      vec![ranged_attack],
      vec![ranged_attack],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![reload],
    ];
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut all_events = Vec::new();
    let mut reload_effects = Vec::new();

    for (index, command) in commands.iter().copied().enumerate() {
      let expected_events = direct
        .step(command)
        .expect("direct Assault Shotgun command");
      let step = browser
        .submit(command)
        .expect("browser Assault Shotgun command");
      assert_eq!(step.events, expected_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
      );
      assert_eq!(step.effects, expected_effects[index]);
      if command == Command::Reload {
        reload_effects = step.effects;
      }
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      all_events.extend(expected_events);
    }

    assert_eq!(direct.world().player().unwrap().hp().current, 50);
    assert_eq!(
      direct.world().get_actor(target_id).unwrap().hp().current,
      433
    );
    assert_eq!(
      direct.world().get_actor(target_id).unwrap().position(),
      target
    );
    assert!(!all_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::ActorKnockedBack { entity_id, .. }
          if *entity_id == target_id
      )
    }));
    let weapon = direct
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap();
    assert_eq!(weapon.current_clip, 1);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .inventory()
        .get_item(shells_id)
        .unwrap()
        .count(),
      7
    );
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);
    assert_eq!(
      browser.observation().equipped_weapon.unwrap().clip,
      Some((1, 6))
    );
    assert_eq!(
      browser
        .observation()
        .inventory
        .iter()
        .find(|item| item.id == shells_id)
        .unwrap()
        .count,
      7
    );
    assert_eq!(reload_effects, vec![reload]);
    assert_eq!(
      all_events
        .iter()
        .filter(|event| matches!(event, drl_protocol::GameEvent::AttackResolved { attacker_id, target_id: event_target, is_ranged: true, .. } if *attacker_id == player_id && *event_target == target_id))
        .count(),
      6
    );
    let reload_index = all_events
      .iter()
      .position(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::WeaponReloaded {
            entity_id,
            ammo_loaded: 1,
            current_clip: 1,
            max_clip: 6,
          } if *entity_id == player_id
        )
      })
      .expect("reload event");
    assert_eq!(
      all_events[..reload_index]
        .iter()
        .filter(|event| {
          matches!(
            event,
            drl_protocol::GameEvent::AttackResolved {
              attacker_id,
              target_id: event_target,
              is_ranged: true,
              ..
            } if *attacker_id == player_id && *event_target == target_id
          )
        })
        .count(),
      6
    );
    assert!(matches!(
      all_events.get(reload_index + 1),
      Some(drl_protocol::GameEvent::ActionCostPaid {
        entity_id,
        cost: drl_protocol::ActionCost(1000),
      }) if *entity_id == player_id
    ));
    assert!(matches!(
      all_events.get(reload_index + 2),
      Some(drl_protocol::GameEvent::TurnEnded { .. })
    ));

    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, all_events);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn assault_shotgun_alt_reload_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(2, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoShells(8)],
      equipped_weapon: Some(ItemSpawnKind::AssaultShotgun),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let target_position = Position::new(7, 1);
    let mut setup_replay =
      ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
        .with_ranged_combat((1, 4), 6, 65)
        .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let player_id = initial.world().player_id().expect("player identity");
    let weapon_id = initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Assault Shotgun")
      .id();
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("static target")
      .id();

    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "AssaultShotgunAltReloadVertical",
      "Assault Shotgun alternate full reload against a static target",
      "#########\n#.@....h#\n#.......#\n#########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 0;
    scenario.monsters[0].name = "Static Target".to_string();
    scenario.monsters[0].hp = 500;
    scenario.monsters[0].speed = 1;
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let target = Position::new(7, 1);
    let mut commands = vec![Command::AttackRanged(target); 6];
    commands.push(Command::AltReload {
      item_id: weapon_id,
      confirmed: false,
    });
    let ranged_attack = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::RangedAttack,
      start_tick: 0,
      duration_ticks: 2,
    };
    let hit = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Hit,
      start_tick: 2,
      duration_ticks: 1,
    };
    let reload = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Reload,
      start_tick: 0,
      duration_ticks: 3,
    };
    let expected_effects = [
      vec![ranged_attack],
      vec![ranged_attack],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![ranged_attack, hit],
      vec![reload],
    ];
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut all_events = Vec::new();

    for (index, command) in commands.iter().copied().enumerate() {
      let expected_events = direct
        .step(command)
        .expect("direct Assault Shotgun alternate reload command");
      let step = browser
        .submit(command)
        .expect("browser Assault Shotgun alternate reload command");
      assert_eq!(step.events, expected_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
      );
      assert_eq!(step.effects, expected_effects[index]);
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      all_events.extend(expected_events);
    }

    assert_eq!(
      direct.world().get_actor(target_id).unwrap().hp().current,
      433
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .inventory()
        .total_ammo(drl_protocol::AmmoType::Shells),
      2
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      6
    );
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);
    assert!(all_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::ActionCostPaid {
          entity_id,
          cost: drl_protocol::ActionCost(2_500),
        } if *entity_id == player_id
      )
    }));

    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, all_events);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn combat_shotgun_alt_reload_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(2, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoShells(10)],
      equipped_weapon: Some(ItemSpawnKind::CombatShotgun),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let target_position = Position::new(7, 1);
    let mut setup_replay =
      ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
        .with_ranged_combat((1, 4), 6, 65)
        .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
    );
    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let player_id = initial.world().player_id().expect("player identity");
    let weapon_id = initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Combat Shotgun")
      .id();

    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "CombatShotgunAltReloadVertical",
      "Combat Shotgun alternate full reload directly chambers an empty chamber",
      "#########\n#.@....h#\n#.......#\n#########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 0;
    scenario.monsters[0].name = "Static Target".to_string();
    scenario.monsters[0].hp = 500;
    scenario.monsters[0].speed = 1;
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let target = Position::new(7, 1);
    let mut commands = Vec::new();
    for index in 0..5 {
      commands.push(Command::AttackRanged(target));
      if index < 4 {
        commands.push(Command::Reload);
      }
    }
    commands.push(Command::AltReload {
      item_id: weapon_id,
      confirmed: false,
    });
    commands.push(Command::AttackRanged(target));

    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut all_events = Vec::new();
    for command in commands.iter().copied() {
      let expected_events = direct
        .step(command)
        .expect("direct Combat Shotgun alternate reload command");
      let step = browser
        .submit(command)
        .expect("browser Combat Shotgun alternate reload command");
      assert_eq!(step.events, expected_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
      );
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      all_events.extend(expected_events);
    }

    assert_eq!(
      direct
        .world()
        .player()
        .expect("player")
        .equipment()
        .weapon()
        .expect("Combat Shotgun")
        .weapon_properties()
        .expect("weapon properties")
        .current_clip,
      4
    );
    assert_eq!(
      direct
        .world()
        .player()
        .expect("player")
        .inventory()
        .total_ammo(drl_protocol::AmmoType::Shells),
      5
    );
    assert!(all_events.iter().any(|event| matches!(
      event,
      drl_protocol::GameEvent::WeaponReloaded {
        entity_id,
        ammo_loaded: 5,
        current_clip: 5,
        max_clip: 5,
      } if *entity_id == player_id
    )));
    assert!(all_events.iter().any(|event| matches!(
      event,
      drl_protocol::GameEvent::ActionCostPaid {
        entity_id,
        cost: drl_protocol::ActionCost(2_500),
      } if *entity_id == player_id
    )));
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);

    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, all_events);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn double_shotgun_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(2, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoShells(4)],
      equipped_weapon: Some(ItemSpawnKind::DoubleShotgun),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let target_position = Position::new(7, 1);
    let mut setup_replay =
      ReplayLog::new(1, 9, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
        .with_ranged_combat((1, 4), 6, 65)
        .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
    );

    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let shells_id = ItemId::new(4);
    let shotgun_id = ItemId::new(5);
    assert_eq!(
      initial
        .world()
        .player()
        .expect("player")
        .inventory()
        .get_item(shells_id)
        .expect("shell reserve")
        .count(),
      4
    );
    assert_eq!(
      initial
        .world()
        .player()
        .expect("player")
        .equipment()
        .weapon()
        .expect("Double Shotgun")
        .id(),
      shotgun_id
    );

    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "DoubleShotgunVertical",
      "Double Shotgun clip depletion and deterministic reload",
      "#########\n#.@....h#\n#.......#\n#########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 1;
    scenario.monsters[0].name = "Static Target".to_string();
    scenario.monsters[0].hp = 500;
    scenario.monsters[0].speed = 1;
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("static target")
      .id();
    let target = Position::new(7, 1);
    let mut commands = vec![Command::AttackRanged(target); 2];
    commands.push(Command::Reload);
    let ranged_attack = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::RangedAttack,
      start_tick: 0,
      duration_ticks: 2,
    };
    let hit = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Hit,
      start_tick: 2,
      duration_ticks: 1,
    };
    let reload = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Reload,
      start_tick: 0,
      duration_ticks: 3,
    };
    let expected_effects = [vec![ranged_attack], vec![ranged_attack, hit], vec![reload]];
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut all_events = Vec::new();
    let mut reload_effects = Vec::new();

    for (index, command) in commands.iter().copied().enumerate() {
      let expected_events = direct.step(command).expect("direct Double Shotgun command");
      let step = browser
        .submit(command)
        .expect("browser Double Shotgun command");
      assert_eq!(step.events, expected_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
      );
      assert_eq!(step.effects, expected_effects[index]);
      if command == Command::Reload {
        reload_effects = step.effects;
      }
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      all_events.extend(expected_events);
    }

    assert_eq!(direct.world().player().unwrap().hp().current, 50);
    assert_eq!(
      direct.world().get_actor(target_id).unwrap().hp().current,
      474
    );
    assert_eq!(
      direct.world().get_actor(target_id).unwrap().position(),
      target
    );
    assert!(!all_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::ActorKnockedBack { entity_id, .. }
          if *entity_id == target_id
      )
    }));
    let weapon = direct
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap();
    assert_eq!(weapon.current_clip, 2);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .inventory()
        .get_item(shells_id)
        .unwrap()
        .count(),
      2
    );
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);
    assert_eq!(
      browser.observation().equipped_weapon.unwrap().clip,
      Some((2, 2))
    );
    assert_eq!(
      browser
        .observation()
        .inventory
        .iter()
        .find(|item| item.id == shells_id)
        .unwrap()
        .count,
      2
    );
    assert_eq!(reload_effects, vec![reload]);
    assert_eq!(
      all_events
        .iter()
        .filter(|event| matches!(event, drl_protocol::GameEvent::AttackResolved { attacker_id, target_id: event_target, is_ranged: true, .. } if *attacker_id == player_id && *event_target == target_id))
        .count(),
      2
    );
    let reload_index = all_events
      .iter()
      .position(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::WeaponReloaded {
            entity_id,
            ammo_loaded: 2,
            current_clip: 2,
            max_clip: 2,
          } if *entity_id == player_id
        )
      })
      .expect("reload event");
    assert_eq!(
      all_events[..reload_index]
        .iter()
        .filter(|event| {
          matches!(
            event,
            drl_protocol::GameEvent::AttackResolved {
              attacker_id,
              target_id: event_target,
              is_ranged: true,
              ..
            } if *attacker_id == player_id && *event_target == target_id
          )
        })
        .count(),
      2
    );
    assert!(matches!(
      all_events.get(reload_index + 1),
      Some(drl_protocol::GameEvent::ActionCostPaid {
        entity_id,
        cost: drl_protocol::ActionCost(1000),
      }) if *entity_id == player_id
    ));
    assert!(matches!(
      all_events.get(reload_index + 2),
      Some(drl_protocol::GameEvent::TurnEnded { .. })
    ));

    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, all_events);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn combat_pump_vertical_browser_boundary_matches_direct_core_presentation() {
    let player_position = Position::new(2, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoShells(10)],
      equipped_weapon: Some(ItemSpawnKind::CombatShotgun),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let target_position = Position::new(7, 1);
    let mut setup_replay =
      ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config.clone());
    setup_replay.record_monster(
      MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
        .with_ranged_combat((1, 4), 6, 65)
        .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
    );
    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
    assert!(setup_events.is_empty());
    let shells_id = ItemId::new(4);
    let weapon_id = ItemId::new(5);
    let mut scenario = drl_core::scenario::Scenario::from_ascii(
      "CombatPumpVertical",
      "Combat Shotgun pump cycles, shell reload, and deterministic replay",
      "#########\n#.@....h#\n#.......#\n#########\n",
    )
    .expect("vertical scenario fixture");
    scenario.seed = 0;
    scenario.monsters[0].name = "Static Target".to_string();
    scenario.monsters[0].hp = 500;
    scenario.monsters[0].speed = 1;
    scenario.player_config = Some(player_config);
    assert_eq!(
      scenario.instantiate().expect("scenario initial state"),
      initial
    );

    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("static target")
      .id();
    let target = Position::new(7, 1);
    let mut commands = Vec::new();
    for index in 0..5 {
      commands.push(Command::AttackRanged(target));
      if index < 4 {
        commands.push(Command::Reload);
      }
    }
    commands.push(Command::Reload);
    let ranged_attack = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::RangedAttack,
      start_tick: 0,
      duration_ticks: 2,
    };
    let hit = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Hit,
      start_tick: 2,
      duration_ticks: 1,
    };
    let reload = drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Reload,
      start_tick: 0,
      duration_ticks: 3,
    };
    let expected_effects = [
      vec![ranged_attack],
      Vec::new(),
      vec![ranged_attack],
      Vec::new(),
      vec![ranged_attack, hit],
      Vec::new(),
      vec![ranged_attack, hit],
      Vec::new(),
      vec![ranged_attack, hit],
      vec![reload],
    ];
    let mut direct = initial.clone();
    let mut browser = BrowserSession::from_game(initial);
    let mut all_events = Vec::new();
    for (index, command) in commands.iter().copied().enumerate() {
      let expected_events = direct.step(command).expect("direct Combat Shotgun command");
      let step = browser
        .submit(command)
        .expect("browser Combat Shotgun command");
      assert_eq!(step.events, expected_events);
      assert_eq!(step.after, direct.observe_player());
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
      );
      assert_eq!(step.effects, expected_effects[index]);
      assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
      all_events.extend(expected_events);
    }

    assert_eq!(
      direct.world().get_actor(target_id).unwrap().hp().current,
      454
    );
    assert_eq!(
      direct.world().get_actor(target_id).unwrap().position(),
      target
    );
    assert!(!all_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::ActorKnockedBack { entity_id, .. }
          if *entity_id == target_id
      )
    }));
    let weapon_item = direct
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap();
    assert_eq!(weapon_item.id(), weapon_id);
    let weapon = weapon_item.weapon_properties().unwrap();
    assert_eq!(weapon.current_clip, 1);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .inventory()
        .get_item(shells_id)
        .unwrap()
        .count(),
      9
    );
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, commands);
    assert_eq!(
      browser.observation().equipped_weapon.unwrap().clip,
      Some((1, 5))
    );
    assert_eq!(
      browser
        .observation()
        .inventory
        .iter()
        .find(|item| item.id == shells_id)
        .unwrap()
        .count,
      9
    );
    assert_eq!(
      all_events
        .iter()
        .filter(|event| {
          matches!(
            event,
            drl_protocol::GameEvent::ActionCostPaid {
              entity_id,
              cost: drl_protocol::ActionCost(200),
            } if *entity_id == player_id
          )
        })
        .count(),
      4
    );
    let reload_index = all_events
      .iter()
      .position(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::WeaponReloaded {
            entity_id,
            ammo_loaded: 1,
            current_clip: 1,
            max_clip: 5,
          } if *entity_id == player_id
        )
      })
      .expect("reload event");
    assert!(matches!(
      all_events.get(reload_index + 1),
      Some(drl_protocol::GameEvent::ActionCostPaid {
        entity_id,
        cost: drl_protocol::ActionCost(1000),
      }) if *entity_id == player_id
    ));
    assert!(matches!(
      all_events.get(reload_index + 2),
      Some(drl_protocol::GameEvent::TurnEnded { .. })
    ));
    let mut command_replay = setup_replay;
    for command in commands {
      command_replay.record_command(command);
    }
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, all_events);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn standard_bfg_exact_hit_browser_boundary_matches_direct_core() {
    let player_position = Position::new(2, 1);
    let target_position = Position::new(5, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Bfg9000),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(MonsterSpawnSpec::new(
      target_position,
      "Static Target",
      500,
      1,
      (2, 4),
    ));
    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("exact-hit replay setup");
    assert!(setup_events.is_empty());

    let command = Command::AttackRanged(target_position);
    let mut direct = initial.clone();
    let expected_events = direct.step(command).expect("direct exact-hit command");
    assert!(expected_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
          ..
        }
      )
    }));

    let mut browser = BrowserSession::from_game(initial);
    let step = browser.submit(command).expect("browser exact-hit command");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, vec![command]);

    let mut command_replay = setup_replay;
    command_replay.record_command(command);
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("exact-hit command replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, expected_events);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
  }

  #[test]
  fn nuclear_bfg_exact_hit_browser_boundary_matches_direct_core() {
    let player_position = Position::new(2, 1);
    let target_position = Position::new(5, 1);
    let player_config = PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::NuclearBfg9000),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(MonsterSpawnSpec::new(
      target_position,
      "Static Target",
      500,
      1,
      (2, 4),
    ));
    let (initial, setup_events) =
      drl_core::ReplayEngine::run(&setup_replay).expect("nuclear exact-hit replay setup");
    assert!(setup_events.is_empty());

    let command = Command::AttackRanged(target_position);
    let mut direct = initial.clone();
    let expected_events = direct
      .step(command)
      .expect("direct nuclear exact-hit command");
    assert!(expected_events.iter().any(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
          ..
        }
      )
    }));

    let mut browser = BrowserSession::from_game(initial);
    let step = browser
      .submit(command)
      .expect("browser nuclear exact-hit command");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    assert_eq!(browser.observation(), direct.observe_player());
    assert_eq!(browser.replay_log().commands, vec![command]);

    let mut command_replay = setup_replay;
    command_replay.record_command(command);
    let (replayed, replay_events) =
      drl_core::ReplayEngine::run(&command_replay).expect("nuclear exact-hit command replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, expected_events);
    assert!(
      drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism")
    );
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
