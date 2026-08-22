//! Pure presentation planning for DRL-Rust.
//!
//! Scene construction consumes only protocol observations and events. A
//! browser or native renderer may turn the resulting scene into pixels, but
//! presentation timing can never advance the simulation.

mod animation;
mod outline;
mod particle_decal;

pub use animation::{AnimationPlayback, animation_frame_index_at_elapsed};
pub use outline::outline_mask_composite;
pub use particle_decal::{ParticleDecalStorageError, ParticleDecalStore};

use drl_assets::{
  AtlasId, AtlasTextureSource, LayerRole, SpriteAnimation, SpriteDescriptor, SpriteLayer, SpriteUv,
  actor_sprite, item_sprite, tile_sprite,
};
use drl_protocol::{
  Command, EntityId, GameEvent, HitPoints, ItemArchetype, ItemView, PlayerObservation, Position,
  TileKind,
};
use std::collections::BTreeSet;

const NEUTRAL_COLORIZATION_TINT: [u8; 4] = [0, 0, 0, 0];

/// Returns the evidence-backed tint for the currently implemented armor item.
/// Other archetypes remain neutral until their content mappings are migrated.
#[must_use]
pub const fn item_colorization_tint(archetype: ItemArchetype) -> [u8; 4] {
  match archetype {
    ItemArchetype::GreenArmor => [0, 255, 0, 255],
    ItemArchetype::PhaseDevice => [0, 0, 179, 255],
    _ => NEUTRAL_COLORIZATION_TINT,
  }
}

/// Returns the pinned legacy tint for the currently implemented stairs tile.
#[must_use]
pub const fn tile_colorization_tint(tile: TileKind) -> [u8; 4] {
  match tile {
    TileKind::StairsDown => [255, 255, 0, 255],
    _ => NEUTRAL_COLORIZATION_TINT,
  }
}

/// Returns the observed armor tint for the player sprite.
#[must_use]
pub fn equipped_colorization_tint(armor: Option<&ItemView>) -> [u8; 4] {
  armor
    .map(|item| item_colorization_tint(item.archetype))
    .unwrap_or(NEUTRAL_COLORIZATION_TINT)
}

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

/// Three color/frame phases used by the legacy explosion-mark effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExplosionMarkPhase {
  First,
  Second,
  Third,
}

/// Selects the source-derived explosion-mark phase.
///
/// The legacy effect normalizes zero duration to one millisecond and divides
/// elapsed time into three integer phases. Values at or beyond the duration
/// take the source's fallback second phase. This helper owns no lifecycle,
/// scheduling, palette selection, or sprite rendering.
#[must_use]
pub fn explosion_mark_phase(elapsed_ms: u64, duration_ms: u64) -> ExplosionMarkPhase {
  let duration_ms = if duration_ms == 0 { 1 } else { duration_ms };
  match (elapsed_ms as u128) * 3 / duration_ms as u128 {
    0 => ExplosionMarkPhase::First,
    1 => ExplosionMarkPhase::Second,
    2 => ExplosionMarkPhase::Third,
    _ => ExplosionMarkPhase::Second,
  }
}

/// Returns the tracked cell/item effect segment for caller-owned timing.
///
/// The legacy selector computes `(elapsed * target) div duration` and nudges
/// any non-terminal quotient one step toward the signed target. Zero duration
/// and results outside the Rust segment range are rejected; no sprite, level,
/// item, delay, or lifecycle state is consulted.
#[must_use]
pub fn effect_segment_index_at_elapsed(
  elapsed_units: u64,
  duration_units: u64,
  target_segment: i32,
) -> Option<i32> {
  if duration_units == 0 {
    return None;
  }

  let elapsed = elapsed_units as i128;
  let duration = duration_units as i128;
  let target = target_segment as i128;
  let quotient = elapsed * target / duration;
  let corrected = if quotient != target {
    quotient
      + if target > 0 {
        1
      } else if target < 0 {
        -1
      } else {
        0
      }
  } else {
    quotient
  };
  i32::try_from(corrected).ok()
}

/// Returns the caller-owned death-animation segment at elapsed time.
///
/// The legacy selector optionally holds the first segment through a lead
/// delay, then computes `(elapsed * count) div duration` and clamps to the
/// final segment. Reverse playback ignores the lead delay. Zero durations or
/// empty segment sets are rejected; no actor, sprite, light, or lifecycle
/// state is consulted.
#[must_use]
pub fn kill_animation_segment_index_at_elapsed(
  elapsed_ms: u64,
  total_duration_ms: u64,
  segment_count: u32,
  lead_delay_ms: u64,
  reverse: bool,
) -> Option<u32> {
  if total_duration_ms == 0 || segment_count == 0 {
    return None;
  }
  if !reverse && lead_delay_ms > total_duration_ms {
    return None;
  }

  let count = u128::from(segment_count);
  let terminal = count - 1;
  let segment = if !reverse && lead_delay_ms > 0 {
    if elapsed_ms <= lead_delay_ms {
      0
    } else {
      let elapsed = u128::from(elapsed_ms - lead_delay_ms);
      let duration = u128::from((total_duration_ms - lead_delay_ms).max(1));
      (elapsed * count / duration).min(terminal)
    }
  } else {
    (u128::from(elapsed_ms) * count / u128::from(total_duration_ms)).min(terminal)
  };

  u32::try_from(segment).ok()
}

/// Returns the caller-owned FX sprite frame at elapsed time.
///
/// The legacy selector computes `(elapsed * frame_count) div duration` and
/// clamps the result to the final frame before applying a sprite-column
/// offset. Zero durations and empty frame sets are rejected; sprite IDs,
/// atlas columns, and effect lifecycle state remain caller/backend concerns.
#[must_use]
pub fn fx_animation_frame_index_at_elapsed(
  elapsed_units: u64,
  duration_units: u64,
  frame_count: u16,
) -> Option<u16> {
  if duration_units == 0 || frame_count == 0 {
    return None;
  }

  let frame_count = u128::from(frame_count);
  let frame_index =
    (u128::from(elapsed_units) * frame_count / u128::from(duration_units)).min(frame_count - 1);
  u16::try_from(frame_index).ok()
}

/// Returns the caller-owned movement progress at elapsed time.
///
/// The legacy movement draw computes `Clampf(elapsed / duration, 0, 1)` before
/// interpolating position and light. Zero duration is rejected; this helper
/// owns no coordinates, entity state, lighting, interpolation, or lifecycle.
#[must_use]
pub fn move_animation_progress_at_elapsed(elapsed_units: u64, duration_units: u64) -> Option<f32> {
  if duration_units == 0 {
    return None;
  }

  let progress = elapsed_units as f64 / duration_units as f64;
  Some(progress.min(1.0) as f32)
}

/// Returns the caller-owned missile path step at elapsed time.
///
/// The legacy constructor derives `step_delay` as
/// `max(duration / max(path_length, 1), 1)`, then updates the path step with
/// `elapsed div step_delay`. Zero duration and path length therefore normalize
/// to a one-unit delay. An index outside the Rust `u16` step range is rejected;
/// path traversal, visibility, particles, and lifecycle remain outside this
/// arithmetic helper.
#[must_use]
pub fn missile_step_index_at_elapsed(
  elapsed_units: u64,
  duration_units: u64,
  path_length_units: u64,
) -> Option<u16> {
  let step_delay = (duration_units / path_length_units.max(1)).max(1);
  u16::try_from(elapsed_units / step_delay).ok()
}

/// Returns the caller-owned ray sample distance at a zero-based sample index.
///
/// The legacy ray draw starts at half the integer-divided grid size, requires
/// that pre-increment distance to be strictly below the endpoint length, then
/// adds a fixed 20-unit spacing before sampling. The final sample can therefore
/// overshoot the endpoint. Checked arithmetic rejects invalid or unrepresentable
/// values; endpoint metrics, interpolation, visibility, and rendering remain
/// outside this numeric helper.
#[must_use]
pub fn missile_ray_sample_distance_at_index(
  sample_index: u64,
  endpoint_length_units: u64,
  grid_size_units: u64,
) -> Option<u64> {
  let start_distance = grid_size_units / 2;
  let pre_increment_distance = start_distance.checked_add(sample_index.checked_mul(20)?)?;
  if pre_increment_distance >= endpoint_length_units {
    return None;
  }
  pre_increment_distance.checked_add(20)
}

/// Returns the caller-owned screen-shake fade envelope at elapsed time.
///
/// The legacy update uses `1 - (elapsed / duration)^2` while the animation is
/// active and leaves the offset at zero once elapsed time reaches the duration.
/// Zero duration therefore returns zero. Random frequencies, offsets, strength,
/// direction, scheduling, and sprite-map state remain outside this helper.
#[must_use]
pub fn screen_shake_fade_at_elapsed(elapsed_units: u64, duration_units: u64) -> f32 {
  if duration_units == 0 || elapsed_units >= duration_units {
    return 0.0;
  }

  let progress = elapsed_units as f64 / duration_units as f64;
  (1.0 - progress * progress) as f32
}

/// Converts a one-based legacy cell coordinate into a centered pixel origin.
///
/// The legacy particle-burst path uses `((cell - 1) * 32 + 16)` for each axis
/// and a zero Z coordinate. Inputs are explicitly one-based legacy cells;
/// checked signed arithmetic rejects values that cannot be represented. The
/// helper does not convert current Rust positions or spawn/configure particles.
#[must_use]
pub fn particle_burst_origin_at_legacy_cell(
  legacy_cell_x: i32,
  legacy_cell_y: i32,
) -> Option<[i32; 3]> {
  let x = legacy_cell_x
    .checked_sub(1)?
    .checked_mul(32)?
    .checked_add(16)?;
  let y = legacy_cell_y
    .checked_sub(1)?
    .checked_mul(32)?
    .checked_add(16)?;
  Some([x, y, 0])
}

/// Resolves the caller-owned direction for one legacy particle burst sample.
///
/// The legacy burst normalizes the requested XY direction when its length is
/// positive, otherwise it clears only XY and retains the emitter Z direction.
/// A positive distance scale then replaces Z with `emitter_z * arc / scale`.
/// Random range selection, spread, decals, and particle-engine ownership stay
/// outside this deterministic helper.
#[must_use]
pub fn particle_burst_direction(
  emitter_direction: [f32; 3],
  requested_direction: [f32; 2],
  arc: f32,
  distance_scale: f32,
) -> [f32; 3] {
  let length = (requested_direction[0] * requested_direction[0]
    + requested_direction[1] * requested_direction[1])
    .sqrt();
  let mut direction = emitter_direction;
  if length > 0.0 {
    direction[0] = requested_direction[0] / length;
    direction[1] = requested_direction[1] / length;
  } else {
    direction[0] = 0.0;
    direction[1] = 0.0;
  }
  if distance_scale > 0.0 {
    direction[2] = emitter_direction[2] * arc / distance_scale;
  }
  direction
}

/// Samples one caller-owned legacy particle range from a unit interval value.
///
/// The source range helper evaluates `min + unit_sample * (max - min)`.
/// Callers provide the deterministic unit sample (normally `[0, 1)` from their
/// own RNG); this helper does not clamp the sample or own random state.
#[must_use]
pub fn particle_burst_range_sample(range: [f32; 2], unit_sample: f32) -> f32 {
  range[0] + unit_sample * (range[1] - range[0])
}

/// The cell and pixel coordinates used by a legacy particle decal callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParticleDecalPlacement {
  pub cell: [i32; 2],
  pub pixel: [i32; 2],
}

/// Maps a caller-rounded particle world position to legacy decal placement.
///
/// The source callback adds 16 pixels to each rounded position and uses
/// truncating integer division by 32, producing one-based cell coordinates.
/// Checked addition rejects positions whose offset cannot be represented;
/// map bounds, liquid/block flags, and decal storage remain caller-owned.
#[must_use]
pub fn particle_decal_placement_at_rounded_world(
  rounded_world_position: [i32; 2],
) -> Option<ParticleDecalPlacement> {
  let pixel_x = rounded_world_position[0].checked_add(16)?;
  let pixel_y = rounded_world_position[1].checked_add(16)?;
  Some(ParticleDecalPlacement {
    cell: [pixel_x / 32, pixel_y / 32],
    pixel: [pixel_x, pixel_y],
  })
}

/// Returns only the one-based cell from the legacy decal placement.
#[must_use]
pub fn particle_decal_cell_at_rounded_world(rounded_world_position: [i32; 2]) -> Option<[i32; 2]> {
  particle_decal_placement_at_rounded_world(rounded_world_position).map(|placement| placement.cell)
}

/// Reports whether a caller-resolved cell can receive a particle decal.
///
/// The legacy callback accepts only an in-bounds cell that is neither liquid
/// nor movement-blocking. Cell lookup, flag resolution, decal selection, and
/// storage remain caller-owned so this gate stays independent of simulation
/// state and renderer storage.
#[must_use]
pub const fn particle_decal_cell_is_eligible(
  cell_is_in_bounds: bool,
  cell_is_liquid: bool,
  cell_blocks_movement: bool,
) -> bool {
  cell_is_in_bounds && !cell_is_liquid && !cell_blocks_movement
}

/// The renderer-neutral request produced for one accepted particle decal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParticleDecalInsertion {
  pub placement: ParticleDecalPlacement,
  pub sprite_id: u32,
}

/// Builds a caller-owned insertion request for one eligible particle decal.
///
/// The legacy callback stores the caller-provided sprite at the placement
/// derived from the rounded world position after its map/flag guards pass.
/// This helper preserves that request without selecting sprites, storing
/// decals, spawning particles, or rendering them.
#[must_use]
pub fn particle_decal_insertion_at_rounded_world(
  rounded_world_position: [i32; 2],
  cell_is_in_bounds: bool,
  cell_is_liquid: bool,
  cell_blocks_movement: bool,
  sprite_id: u32,
) -> Option<ParticleDecalInsertion> {
  let placement = particle_decal_placement_at_rounded_world(rounded_world_position)?;
  if !particle_decal_cell_is_eligible(cell_is_in_bounds, cell_is_liquid, cell_blocks_movement) {
    return None;
  }
  Some(ParticleDecalInsertion {
    placement,
    sprite_id,
  })
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

/// Returns the source-derived instantaneous low-health pulse target.
///
/// The legacy presentation updates this target only below one third health,
/// using an integer-divided half-health denominator and a five-radian-per-
/// second sine phase. This helper intentionally exposes only that pure target:
/// callers provide elapsed time, while smoothing, texture compositing, and
/// post-processing remain outside the renderer-neutral boundary.
#[must_use]
pub fn low_health_pulse_target_alpha(player_hp: Option<HitPoints>, elapsed_ms: u64) -> f32 {
  let Some(hp) = player_hp else {
    return 0.0;
  };

  if hp.current >= hp.max / 3 {
    return 0.0;
  }

  let half_max = hp.max / 2;
  if half_max == 0 {
    return 0.0;
  }

  let health_ratio = hp.current as f32 / half_max as f32;
  let phase = (elapsed_ms as f64 / 1_000.0) * 5.0;
  let target = 0.8 - health_ratio + phase.sin() as f32 * 0.2;
  target.clamp(0.0, 1.0)
}

/// Caller-owned low-health pulse values after one presentation-time step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LowHealthPulseState {
  /// Smoothed pulse alpha before any draw-time clamping.
  pub alpha: f32,
  /// Pending event target after its independent decay.
  pub pending_target_alpha: f32,
}

/// Applies the pinned low-health pulse smoothing and pending-target decay.
///
/// The legacy presentation moves its current alpha toward its caller-selected
/// target by at most `elapsed_ms / 500`, then decays the pending target by that
/// same step. The values are intentionally not clamped here: the legacy
/// compositor clamps only when drawing, and pulse events may carry values above
/// one. Callers own target selection, state, and the clock.
#[must_use]
pub fn low_health_pulse_state_step(
  current_alpha: f32,
  selected_target_alpha: f32,
  pending_target_alpha: f32,
  elapsed_ms: u64,
) -> LowHealthPulseState {
  let step = elapsed_ms as f32 / 500.0;
  let alpha = if selected_target_alpha > current_alpha {
    current_alpha + (selected_target_alpha - current_alpha).min(step)
  } else if selected_target_alpha < current_alpha {
    current_alpha - (current_alpha - selected_target_alpha).min(step)
  } else {
    current_alpha
  };

  let pending_target_alpha = if pending_target_alpha > 0.0 {
    pending_target_alpha - pending_target_alpha.min(step)
  } else {
    pending_target_alpha
  };

  LowHealthPulseState {
    alpha,
    pending_target_alpha,
  }
}

/// The pinned shader's declared blur-weight array, in source order.
///
/// Its `weights[abs(i)]` loop samples only entries 0–2; entries 3–4 are
/// retained as observed implementation artifacts.
pub const POST_PROCESS_BLUR_DECLARED_WEIGHTS: [f32; 5] =
  [0.227_027, 0.316_216, 0.070_270, 0.050_987, 0.016_216];

/// Effective symmetric weights for center, one-pixel, and two-pixel offsets.
pub const POST_PROCESS_BLUR_WEIGHTS: [f32; 3] = [0.227_027, 0.316_216, 0.070_270];

/// Axis used by the two tracked post-process blur passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostProcessBlurAxis {
  Horizontal,
  Vertical,
}

/// One normalized tap in a post-process blur plan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PostProcessBlurTap {
  pub offset: [f32; 2],
  pub weight: f32,
}

/// Index of the tap whose source alpha is preserved by the legacy blur pass.
pub const POST_PROCESS_BLUR_CENTER_INDEX: usize = 2;

/// Builds the tracked five-tap blur plan for a caller-supplied screen size.
#[must_use]
pub fn post_process_blur_taps(
  axis: PostProcessBlurAxis,
  screen_width: u32,
  screen_height: u32,
) -> Option<[PostProcessBlurTap; 5]> {
  if screen_width == 0 || screen_height == 0 {
    return None;
  }

  let scale = match axis {
    PostProcessBlurAxis::Horizontal => 1.0 / screen_width as f32,
    PostProcessBlurAxis::Vertical => 1.0 / screen_height as f32,
  };
  let offsets = [-2.0_f32, -1.0, 0.0, 1.0, 2.0];
  let weights = [
    POST_PROCESS_BLUR_WEIGHTS[2],
    POST_PROCESS_BLUR_WEIGHTS[1],
    POST_PROCESS_BLUR_WEIGHTS[0],
    POST_PROCESS_BLUR_WEIGHTS[1],
    POST_PROCESS_BLUR_WEIGHTS[2],
  ];
  Some(std::array::from_fn(|index| {
    let distance = offsets[index] * scale;
    let offset = match axis {
      PostProcessBlurAxis::Horizontal => [distance, 0.0],
      PostProcessBlurAxis::Vertical => [0.0, distance],
    };
    PostProcessBlurTap {
      offset,
      weight: weights[index],
    }
  }))
}

/// Applies the tracked blur reduction to five caller-supplied RGBA samples.
///
/// RGB uses the effective weights without renormalization; alpha is copied
/// from the center sample, matching the legacy shader's `texel.w` rule.
#[must_use]
pub fn post_process_blur_rgba(samples: [[f32; 4]; 5]) -> [f32; 4] {
  let weights = [
    POST_PROCESS_BLUR_WEIGHTS[2],
    POST_PROCESS_BLUR_WEIGHTS[1],
    POST_PROCESS_BLUR_WEIGHTS[0],
    POST_PROCESS_BLUR_WEIGHTS[1],
    POST_PROCESS_BLUR_WEIGHTS[2],
  ];
  let mut rgb = [0.0; 3];
  for (sample, weight) in samples.into_iter().zip(weights) {
    for (channel, value) in rgb.iter_mut().zip(sample[..3].iter()) {
      *channel += value * weight;
    }
  }
  [
    rgb[0],
    rgb[1],
    rgb[2],
    samples[POST_PROCESS_BLUR_CENTER_INDEX][3],
  ]
}

/// Applies the observed post-process glow add to an RGB color.
///
/// Callers provide finite presentation values; non-finite input policy is not
/// part of this source-derived contract.
#[must_use]
pub fn post_process_glow_color(
  base_rgb: [f32; 3],
  blur_rgba: [f32; 4],
  glow_enabled: bool,
) -> [f32; 3] {
  if !glow_enabled {
    return base_rgb;
  }

  let glow = 1.6 * blur_rgba[3];
  [
    base_rgb[0] + blur_rgba[0] * glow,
    base_rgb[1] + blur_rgba[1] * glow,
    base_rgb[2] + blur_rgba[2] * glow,
  ]
}

/// Returns the clamped, channel-swizzled coordinate used for the legacy LUT.
///
/// Callers provide finite presentation values; non-finite input policy is not
/// part of this source-derived contract.
#[must_use]
pub fn post_process_lut_coordinate(color_rgb: [f32; 3]) -> [f32; 3] {
  let scale = 30.0 / 32.0;
  let offset = 1.0 / 32.0;
  [
    (color_rgb[0] * scale + offset).clamp(0.0, 1.0),
    (color_rgb[2] * scale + offset).clamp(0.0, 1.0),
    (color_rgb[1] * scale + offset).clamp(0.0, 1.0),
  ]
}

/// One stage in the pinned post-process draw sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostProcessPass {
  DirectScene,
  CaptureScene,
  HorizontalBlur,
  VerticalBlur,
  Composite,
}

/// Renderer-neutral order for the optional post-process stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PostProcessPassPlan {
  passes: [PostProcessPass; 4],
  pass_count: u8,
  /// Whether the blur stages and glow contribution are enabled.
  pub glow_enabled: bool,
  /// Whether the composite stage should sample a caller-provided LUT.
  pub lut_enabled: bool,
}

impl PostProcessPassPlan {
  /// Returns the active stages in their observed execution order.
  #[must_use]
  pub fn ordered_passes(&self) -> &[PostProcessPass] {
    &self.passes[..usize::from(self.pass_count.min(self.passes.len() as u8))]
  }
}

/// Plans the pinned scene, optional blur, and composite sequence.
///
/// With neither feature enabled, the legacy path draws the scene directly.
/// When glow or LUT processing is enabled, the scene is captured first,
/// optional horizontal/vertical blur stages run in that order, and the final
/// composite stage consumes the captured inputs. This function owns no GPU
/// resources, sampling, scheduling, or capture-parity claim.
#[must_use]
pub fn post_process_pass_plan(glow_enabled: bool, lut_enabled: bool) -> PostProcessPassPlan {
  let mut passes = [PostProcessPass::DirectScene; 4];
  let mut pass_count = 1_u8;

  if glow_enabled || lut_enabled {
    passes[0] = PostProcessPass::CaptureScene;
  }

  if glow_enabled {
    passes[usize::from(pass_count)] = PostProcessPass::HorizontalBlur;
    pass_count += 1;
    passes[usize::from(pass_count)] = PostProcessPass::VerticalBlur;
    pass_count += 1;
  }

  if glow_enabled || lut_enabled {
    passes[usize::from(pass_count)] = PostProcessPass::Composite;
    pass_count += 1;
  }

  PostProcessPassPlan {
    passes,
    pass_count,
    glow_enabled,
    lut_enabled,
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
  /// Stable per-sprite group assigned by `layer_draw_plan`.
  pub sprite_index: u32,
  pub atlas: AtlasId,
  pub layer: SpriteLayer,
  pub role: LayerRole,
  pub source: AtlasTextureSource,
  pub lighting: LightingBand,
  pub colorization_tint: [u8; 4],
  pub animation: Option<SpriteAnimation>,
  pub destination: PixelRect,
  pub uv: SpriteUv,
}

/// One sprite's grouped texture inputs for a future compositor.
///
/// A backend must sample the optional masks as their named roles rather than
/// alpha-overlaying each source as an independent color quad. This record is
/// still metadata only: it does not load, decode, or blend image data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteComposite {
  pub sprite_index: u32,
  pub atlas: AtlasId,
  pub destination: PixelRect,
  pub uv: SpriteUv,
  pub lighting: LightingBand,
  pub colorization_tint: [u8; 4],
  pub animation: Option<SpriteAnimation>,
  pub base: AtlasTextureSource,
  pub mask: Option<AtlasTextureSource>,
  pub shadow: Option<AtlasTextureSource>,
  pub emissive: Option<AtlasTextureSource>,
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
  pub colorization_tint: [u8; 4],
}

/// An item ready for a renderer to draw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneItem {
  pub position: Position,
  pub item: ItemView,
  pub sprite: SpriteDescriptor,
  pub colorization_tint: [u8; 4],
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
        colorization_tint: if actor.is_player {
          equipped_colorization_tint(observation.equipped_armor.as_ref())
        } else {
          NEUTRAL_COLORIZATION_TINT
        },
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
        colorization_tint: item_colorization_tint(ground.item.archetype),
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

#[derive(Debug, Clone, Copy)]
enum FrameSelection {
  Static,
  Progress(f32),
  Elapsed {
    elapsed_ms: u64,
    playback: AnimationPlayback,
  },
}

impl FrameSelection {
  const fn is_dynamic(self) -> bool {
    !matches!(self, Self::Static)
  }
}

fn append_layer_draws(
  plan: &mut Vec<LayerDraw>,
  sprite_index: u32,
  descriptor: SpriteDescriptor,
  lighting: LightingBand,
  colorization_tint: [u8; 4],
  destination: Option<PixelRect>,
  selection: FrameSelection,
) -> Option<()> {
  let Some(destination) = destination else {
    return Some(());
  };
  let Some(uv) = descriptor_frame_uv(descriptor, selection) else {
    return if selection.is_dynamic() {
      None
    } else {
      Some(())
    };
  };
  plan.extend(descriptor.layers.iter().copied().map(|layer| LayerDraw {
    sprite_index,
    atlas: descriptor.atlas,
    layer,
    role: layer.role(),
    source: descriptor.atlas.texture_source(layer),
    lighting,
    colorization_tint,
    animation: descriptor.animation,
    destination,
    uv,
  }));
  Some(())
}

fn descriptor_frame_uv(
  descriptor: SpriteDescriptor,
  selection: FrameSelection,
) -> Option<SpriteUv> {
  let frame_index = match (descriptor.animation, selection) {
    (Some(animation), FrameSelection::Progress(progress)) => {
      animation_frame_index(progress, animation.frame_count)?
    }
    (
      Some(animation),
      FrameSelection::Elapsed {
        elapsed_ms,
        playback,
      },
    ) => animation_frame_index_at_elapsed(animation, elapsed_ms, playback)?,
    _ => 0,
  };
  let frame = descriptor.frame_rect(frame_index)?;
  let (atlas_width, atlas_height) = descriptor.atlas.dimensions();
  frame.uv_rect(atlas_width, atlas_height)
}

fn build_layer_draw_plan(
  scene: &RenderScene,
  viewport: PixelViewport,
  selection: FrameSelection,
) -> Option<Vec<LayerDraw>> {
  let mut plan = Vec::new();
  let mut sprite_index = 0_u32;
  for tile in &scene.tiles {
    if !tile.visible && !tile.explored {
      continue;
    }
    append_layer_draws(
      &mut plan,
      sprite_index,
      tile.sprite,
      tile.lighting_band(),
      tile_colorization_tint(tile.kind),
      viewport.tile_rect(tile.position),
      selection,
    )?;
    sprite_index = sprite_index.saturating_add(1);
  }
  for item in &scene.items {
    append_layer_draws(
      &mut plan,
      sprite_index,
      item.sprite,
      LightingBand::Visible,
      item.colorization_tint,
      viewport.tile_rect(item.position),
      selection,
    )?;
    sprite_index = sprite_index.saturating_add(1);
  }
  for actor in &scene.actors {
    append_layer_draws(
      &mut plan,
      sprite_index,
      actor.sprite,
      LightingBand::Visible,
      actor.colorization_tint,
      viewport.tile_rect(actor.position),
      selection,
    )?;
    sprite_index = sprite_index.saturating_add(1);
  }
  Some(plan)
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
  build_layer_draw_plan(scene, viewport, FrameSelection::Static).unwrap_or_default()
}

/// Builds a deterministic layer plan for caller-supplied normalized animation
/// progress. This does not own a clock or connect effects to sprites.
#[must_use]
pub fn layer_draw_plan_at_progress(
  scene: &RenderScene,
  viewport: PixelViewport,
  progress: f32,
) -> Option<Vec<LayerDraw>> {
  if !progress.is_finite() || progress < 0.0 || progress >= 1.0 {
    return None;
  }
  build_layer_draw_plan(scene, viewport, FrameSelection::Progress(progress))
}

/// Builds a deterministic layer plan for caller-supplied elapsed animation
/// time and explicit playback policy. This owns no clock or effect mapping.
#[must_use]
pub fn layer_draw_plan_at_elapsed(
  scene: &RenderScene,
  viewport: PixelViewport,
  elapsed_ms: u64,
  playback: AnimationPlayback,
) -> Option<Vec<LayerDraw>> {
  build_layer_draw_plan(
    scene,
    viewport,
    FrameSelection::Elapsed {
      elapsed_ms,
      playback,
    },
  )
}

fn composite_group(draws: &[LayerDraw]) -> Option<SpriteComposite> {
  let first = draws.first()?;
  let expected_layers = first.atlas.layers();
  if draws.len() != expected_layers.len()
    || draws.iter().any(|draw| {
      draw.sprite_index != first.sprite_index
        || draw.atlas != first.atlas
        || draw.destination != first.destination
        || draw.uv != first.uv
        || draw.lighting != first.lighting
        || draw.colorization_tint != first.colorization_tint
        || draw.animation != first.animation
    })
  {
    return None;
  }
  for (draw, expected_layer) in draws.iter().zip(expected_layers.iter().copied()) {
    if draw.layer != expected_layer || draw.role != expected_layer.role() {
      return None;
    }
  }
  let mut base = None;
  let mut mask = None;
  let mut shadow = None;
  let mut emissive = None;
  for draw in draws {
    match draw.role {
      LayerRole::BaseColor if base.is_none() => base = Some(draw.source),
      LayerRole::ColorizationMask if mask.is_none() => mask = Some(draw.source),
      LayerRole::OutlineMask if shadow.is_none() => shadow = Some(draw.source),
      LayerRole::EmissiveMask if emissive.is_none() => emissive = Some(draw.source),
      _ => return None,
    }
  }
  Some(SpriteComposite {
    sprite_index: first.sprite_index,
    atlas: first.atlas,
    destination: first.destination,
    uv: first.uv,
    lighting: first.lighting,
    colorization_tint: first.colorization_tint,
    animation: first.animation,
    base: base?,
    mask,
    shadow,
    emissive,
  })
}

/// Groups a layer draw plan into one record per complete sprite.
///
/// Groups must be contiguous, use the atlas-registered layer order, and carry
/// one stable sprite index. Malformed or repeated groups are omitted rather
/// than allowing a backend to sample the wrong source role.
#[must_use]
pub fn sprite_composite_plan(plan: &[LayerDraw]) -> Vec<SpriteComposite> {
  let mut composites = Vec::new();
  let mut seen_groups = BTreeSet::new();
  let mut start = 0;
  while start < plan.len() {
    let sprite_index = plan[start].sprite_index;
    let mut end = start + 1;
    while end < plan.len() && plan[end].sprite_index == sprite_index {
      end += 1;
    }
    if seen_groups.insert(sprite_index)
      && let Some(composite) = composite_group(&plan[start..end])
    {
      composites.push(composite);
    }
    start = end;
  }
  composites
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

/// Maps normalized frontend progress to a caller-supplied zero-based frame.
///
/// Frame counts are intentionally supplied by the caller: this helper does
/// not infer asset animation metadata or legacy timing.
#[must_use]
pub fn animation_frame_index(progress: f32, frame_count: u16) -> Option<u16> {
  if frame_count == 0 || !progress.is_finite() || !(0.0..1.0).contains(&progress) {
    return None;
  }
  let index = (progress * f32::from(frame_count)).floor() as u16;
  Some(index.min(frame_count.saturating_sub(1)))
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
  use drl_core::{Game, scenario::Scenario};
  use drl_protocol::{ItemSpawnKind, PlayerSpawnConfig};

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
  fn low_health_pulse_target_preserves_observed_threshold_and_phase() {
    assert_eq!(low_health_pulse_target_alpha(None, 0), 0.0);
    assert_eq!(
      low_health_pulse_target_alpha(Some(HitPoints::new(16, 50)), 0),
      0.0
    );

    let at_zero = low_health_pulse_target_alpha(Some(HitPoints::new(15, 50)), 0);
    assert!((at_zero - 0.2).abs() < 0.001);

    let odd_half_denominator = low_health_pulse_target_alpha(Some(HitPoints::new(1, 51)), 0);
    assert!((odd_half_denominator - 0.76).abs() < 0.001);

    let near_sine_peak = low_health_pulse_target_alpha(Some(HitPoints::new(15, 50)), 314);
    assert!((near_sine_peak - 0.4).abs() < 0.002);
  }

  #[test]
  fn low_health_pulse_target_is_bounded_for_edge_inputs() {
    assert_eq!(
      low_health_pulse_target_alpha(Some(HitPoints::new(0, 0)), u64::MAX),
      0.0
    );
    assert_eq!(
      low_health_pulse_target_alpha(Some(HitPoints::new(0, 2)), 0),
      0.0
    );

    let trough = low_health_pulse_target_alpha(Some(HitPoints::new(15, 50)), 942);
    assert!(trough < 0.001);

    for elapsed_ms in [0, 314, 628, 942, 1_256, 60_000, u64::MAX] {
      let target = low_health_pulse_target_alpha(Some(HitPoints::new(0, 50)), elapsed_ms);
      assert!((0.0..=1.0).contains(&target));
    }
  }

  #[test]
  fn low_health_pulse_state_step_moves_and_decays_independently() {
    assert_eq!(
      low_health_pulse_state_step(0.25, 0.5, 0.75, 0),
      LowHealthPulseState {
        alpha: 0.25,
        pending_target_alpha: 0.75,
      }
    );

    let stepped = low_health_pulse_state_step(0.1, 0.8, 0.8, 100);
    assert!((stepped.alpha - 0.3).abs() < 0.000_001);
    assert!((stepped.pending_target_alpha - 0.6).abs() < 0.000_001);

    let downward = low_health_pulse_state_step(0.9, 0.0, 0.0, 100);
    assert!((downward.alpha - 0.7).abs() < 0.000_001);
    assert_eq!(downward.pending_target_alpha, 0.0);
  }

  #[test]
  fn low_health_pulse_state_step_handles_large_and_negative_values() {
    let snapped = low_health_pulse_state_step(1.5, 2.5, 1.5, 1_000);
    assert_eq!(snapped.alpha, 2.5);
    assert_eq!(snapped.pending_target_alpha, 0.0);

    let negative = low_health_pulse_state_step(0.5, 0.0, -0.5, 100);
    assert!((negative.alpha - 0.3).abs() < 0.000_001);
    assert_eq!(negative.pending_target_alpha, -0.5);
  }

  #[test]
  fn post_process_glow_preserves_observed_weights_and_toggle() {
    assert_eq!(
      POST_PROCESS_BLUR_DECLARED_WEIGHTS,
      [0.227_027, 0.316_216, 0.070_270, 0.050_987, 0.016_216]
    );
    assert_eq!(POST_PROCESS_BLUR_WEIGHTS, [0.227_027, 0.316_216, 0.070_270]);

    let base = [0.1, 0.2, 0.3];
    let blur = [0.5, 0.25, 0.75, 0.5];
    assert_eq!(post_process_glow_color(base, blur, false), base);

    let blended = post_process_glow_color(base, blur, true);
    for (actual, expected) in blended.into_iter().zip([0.5, 0.4, 0.9]) {
      assert!((actual - expected).abs() < 0.000_001);
    }
    assert_eq!(
      post_process_glow_color(base, [1.0, 1.0, 1.0, 0.0], true),
      base
    );
  }

  #[test]
  fn post_process_lut_coordinate_preserves_swizzle_and_clamp() {
    assert_eq!(
      post_process_lut_coordinate([0.0, 0.5, 1.0]),
      [1.0 / 32.0, 31.0 / 32.0, 0.5]
    );
    assert_eq!(
      post_process_lut_coordinate([-1.0, 2.0, 0.0]),
      [0.0, 1.0 / 32.0, 1.0]
    );
  }

  #[test]
  fn post_process_pass_plan_preserves_all_gate_combinations() {
    let direct = post_process_pass_plan(false, false);
    assert_eq!(direct.ordered_passes(), &[PostProcessPass::DirectScene]);
    assert!(!direct.glow_enabled);
    assert!(!direct.lut_enabled);

    let lut_only = post_process_pass_plan(false, true);
    assert_eq!(
      lut_only.ordered_passes(),
      &[PostProcessPass::CaptureScene, PostProcessPass::Composite]
    );
    assert!(!lut_only.glow_enabled);
    assert!(lut_only.lut_enabled);

    let glow_only = post_process_pass_plan(true, false);
    assert_eq!(
      glow_only.ordered_passes(),
      &[
        PostProcessPass::CaptureScene,
        PostProcessPass::HorizontalBlur,
        PostProcessPass::VerticalBlur,
        PostProcessPass::Composite,
      ]
    );
    assert!(glow_only.glow_enabled);
    assert!(!glow_only.lut_enabled);

    let glow_and_lut = post_process_pass_plan(true, true);
    assert_eq!(glow_and_lut.ordered_passes(), glow_only.ordered_passes());
    assert_eq!(glow_and_lut.ordered_passes().len(), 4);
    assert_eq!(glow_and_lut, post_process_pass_plan(true, true));
    assert!(glow_and_lut.glow_enabled);
    assert!(glow_and_lut.lut_enabled);
  }

  #[test]
  fn post_process_blur_taps_plan_both_axes_and_center_alpha() {
    assert_eq!(POST_PROCESS_BLUR_CENTER_INDEX, 2);

    let horizontal = post_process_blur_taps(PostProcessBlurAxis::Horizontal, 320, 200)
      .expect("nonzero dimensions produce taps");
    let vertical = post_process_blur_taps(PostProcessBlurAxis::Vertical, 320, 200)
      .expect("nonzero dimensions produce taps");

    assert!((horizontal[0].offset[0] + 2.0 / 320.0).abs() < 0.000_001);
    assert_eq!(horizontal[0].offset[1], 0.0);
    assert_eq!(horizontal[2].offset, [0.0, 0.0]);
    assert!((vertical[4].offset[1] - 2.0 / 200.0).abs() < 0.000_001);
    assert_eq!(vertical[4].offset[0], 0.0);

    let weights = horizontal.map(|tap| tap.weight);
    assert_eq!(
      weights,
      [0.070_270, 0.316_216, 0.227_027, 0.316_216, 0.070_270]
    );
    assert_eq!(
      horizontal,
      post_process_blur_taps(PostProcessBlurAxis::Horizontal, 320, 200).unwrap()
    );
  }

  #[test]
  fn post_process_blur_taps_reject_zero_dimensions() {
    assert_eq!(
      post_process_blur_taps(PostProcessBlurAxis::Horizontal, 0, 200),
      None
    );
    assert_eq!(
      post_process_blur_taps(PostProcessBlurAxis::Vertical, 320, 0),
      None
    );
  }

  #[test]
  fn post_process_blur_rgba_weights_rgb_and_preserves_center_alpha() {
    let samples = [
      [1.0, 2.0, 3.0, 0.1],
      [4.0, 5.0, 6.0, 0.2],
      [7.0, 8.0, 9.0, 0.3],
      [10.0, 11.0, 12.0, 0.4],
      [13.0, 14.0, 15.0, 0.5],
    ];
    let blurred = post_process_blur_rgba(samples);
    assert!((blurred[0] - 7.0).abs() < 0.000_01);
    assert!((blurred[1] - 8.0).abs() < 0.000_01);
    assert!((blurred[2] - 9.0).abs() < 0.000_01);
    assert_eq!(blurred[3], 0.3);

    let constant = post_process_blur_rgba([[1.0, 1.0, 1.0, 0.0]; 5]);
    assert!(constant[..3].iter().all(|channel| *channel < 1.0));
    assert!(constant[..3].iter().all(|channel| *channel > 0.999));
  }

  #[test]
  fn item_colorization_tint_maps_only_verified_green_armor() {
    assert_eq!(
      item_colorization_tint(ItemArchetype::GreenArmor),
      [0, 255, 0, 255]
    );
    assert_eq!(
      item_colorization_tint(ItemArchetype::PhaseDevice),
      [0, 0, 179, 255]
    );
    assert_eq!(item_colorization_tint(ItemArchetype::Pistol), [0, 0, 0, 0]);
  }

  #[test]
  fn tile_colorization_tint_maps_only_pinned_stairs() {
    assert_eq!(
      tile_colorization_tint(TileKind::StairsDown),
      [255, 255, 0, 255]
    );
    for tile in [
      TileKind::Floor,
      TileKind::Wall,
      TileKind::DoorClosed,
      TileKind::DoorOpen,
    ] {
      assert_eq!(tile_colorization_tint(tile), [0, 0, 0, 0]);
    }
  }

  #[test]
  fn equipped_colorization_tint_uses_observed_armor_only() {
    let armor = ItemView {
      id: drl_protocol::ItemId(7),
      archetype: ItemArchetype::GreenArmor,
      name: "Green Armor".to_owned(),
      category: drl_protocol::ItemCategory::Armor,
      count: 1,
      description: "Armor".to_owned(),
      clip: None,
      damage: None,
      armor_value: Some(100),
      heal_amount: None,
      knockback: None,
    };
    assert_eq!(equipped_colorization_tint(None), [0, 0, 0, 0]);
    assert_eq!(equipped_colorization_tint(Some(&armor)), [0, 255, 0, 255]);
  }

  #[test]
  fn scene_and_composites_forward_observed_green_armor_tint() {
    let mut scenario =
      Scenario::from_ascii("green armor", "", "#####\n#@..#\n#####").expect("scenario");
    scenario.player_config = Some(PlayerSpawnConfig {
      equipped_armor: Some(ItemSpawnKind::GreenArmor),
      ..PlayerSpawnConfig::default()
    });
    let observation = scenario.instantiate().expect("game").observe_player();
    let scene = RenderScene::from_observation(&observation);
    assert_eq!(
      scene
        .actors
        .iter()
        .find(|actor| actor.is_player)
        .map(|actor| actor.colorization_tint),
      Some([0, 255, 0, 255])
    );
    let viewport = PixelViewport::fit(scene.map_width, scene.map_height, 96, 32);
    let composites = sprite_composite_plan(&layer_draw_plan(&scene, viewport));
    assert!(
      composites
        .iter()
        .any(|composite| composite.colorization_tint == [0, 255, 0, 255])
    );
  }

  #[test]
  fn visible_phase_device_forwards_quantized_ground_tint_only() {
    let scenario =
      Scenario::from_ascii("phase device", "", "#####\n#@P.#\n#####").expect("scenario");
    let observation = scenario.instantiate().expect("game").observe_player();
    let scene = RenderScene::from_observation(&observation);
    assert_eq!(
      scene
        .items
        .iter()
        .find(|item| item.item.archetype == ItemArchetype::PhaseDevice)
        .map(|item| item.colorization_tint),
      Some([0, 0, 179, 255])
    );
    assert!(
      scene
        .actors
        .iter()
        .all(|actor| { !actor.is_player || actor.colorization_tint == [0, 0, 0, 0] })
    );
    let viewport = PixelViewport::fit(scene.map_width, scene.map_height, 160, 96);
    let composites = sprite_composite_plan(&layer_draw_plan(&scene, viewport));
    assert!(
      composites
        .iter()
        .any(|composite| composite.colorization_tint == [0, 0, 179, 255])
    );
  }

  #[test]
  fn stairs_tile_tint_reaches_mask_layer_and_composite() {
    let stairs = TileKind::StairsDown;
    let scene = RenderScene {
      map_width: 1,
      map_height: 1,
      player_position: Position::new(0, 0),
      target_positions: Vec::new(),
      tiles: vec![SceneTile {
        position: Position::new(0, 0),
        kind: stairs,
        visible: true,
        explored: true,
        sprite: tile_sprite(stairs),
      }],
      actors: Vec::new(),
      items: Vec::new(),
      hud: HudState {
        turn: 0,
        player_hp: None,
        weapon: None,
        armor: None,
        inventory_size: 0,
      },
    };
    let viewport = PixelViewport::fit(1, 1, 32, 32);
    let plan = layer_draw_plan(&scene, viewport);
    let expected_tint = [255, 255, 0, 255];
    assert!(
      plan
        .iter()
        .all(|draw| draw.colorization_tint == expected_tint)
    );
    let mask = plan
      .iter()
      .find(|draw| draw.role == LayerRole::ColorizationMask)
      .expect("stairs mask layer");
    assert_eq!(mask.source, mask.atlas.texture_source(mask.layer));
    assert_eq!(
      mask.destination,
      viewport
        .tile_rect(Position::new(0, 0))
        .expect("stairs destination")
    );
    let composites = sprite_composite_plan(&plan);
    assert_eq!(composites.len(), 1);
    assert_eq!(composites[0].colorization_tint, expected_tint);
    assert_eq!(composites[0].mask, Some(mask.source));
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
    assert_eq!(
      plan[tile_draws + item_draws].animation,
      first_actor.sprite.animation
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

    let composites = sprite_composite_plan(&plan);
    assert_eq!(
      composites.len(),
      scene
        .tiles
        .iter()
        .filter(|tile| tile.visible || tile.explored)
        .count()
        + scene.items.len()
        + scene.actors.len()
    );
    assert_eq!(composites[0].sprite_index, plan[0].sprite_index);
    assert_eq!(composites[0].atlas, first_tile.sprite.atlas);
    assert_eq!(
      composites[0].base,
      first_tile.sprite.atlas.texture_source(SpriteLayer::Base)
    );
    assert_eq!(
      composites[0].mask,
      Some(first_tile.sprite.atlas.texture_source(SpriteLayer::Mask))
    );
    let first_actor_composite = composites
      .iter()
      .find(|composite| composite.animation == first_actor.sprite.animation)
      .expect("player composite");
    assert_eq!(
      first_actor_composite.animation,
      first_actor.sprite.animation
    );
  }

  #[test]
  fn progress_layer_plan_selects_only_evidenced_animation_rows() {
    let game = Game::new_arena(42, 12, 10).expect("arena");
    let scene = RenderScene::from_observation(&game.observe_player());
    let viewport = PixelViewport::fit(scene.map_width, scene.map_height, 960, 640);
    let frame_zero = layer_draw_plan(&scene, viewport);
    assert_eq!(
      layer_draw_plan_at_progress(&scene, viewport, 0.0),
      Some(frame_zero.clone())
    );
    assert_eq!(
      layer_draw_plan_at_progress(&scene, viewport, 0.4999),
      Some(frame_zero.clone())
    );
    for invalid in [-0.01, 1.0, f32::NAN, f32::INFINITY] {
      assert!(layer_draw_plan_at_progress(&scene, viewport, invalid).is_none());
    }

    let player = scene.actors.first().expect("player");
    let player_draw = frame_zero
      .iter()
      .find(|draw| draw.atlas == player.sprite.atlas && draw.animation == player.sprite.animation)
      .expect("player draw");
    let frame_one = layer_draw_plan_at_progress(&scene, viewport, 0.5).expect("frame one plan");
    let expected_uv = player
      .sprite
      .frame_rect(1)
      .expect("player frame one")
      .uv_rect(
        player.sprite.atlas.dimensions().0,
        player.sprite.atlas.dimensions().1,
      )
      .expect("player frame one UV");
    let selected_player: Vec<_> = frame_one
      .iter()
      .filter(|draw| draw.sprite_index == player_draw.sprite_index)
      .collect();
    assert!(!selected_player.is_empty());
    assert!(selected_player.iter().all(|draw| draw.uv == expected_uv));

    let static_draw = frame_zero
      .iter()
      .find(|draw| draw.animation.is_none())
      .expect("static tile draw");
    let static_selected = frame_one
      .iter()
      .find(|draw| draw.sprite_index == static_draw.sprite_index)
      .expect("static tile at frame one progress");
    assert_eq!(static_selected.uv, static_draw.uv);
    assert_eq!(
      layer_draw_plan_at_progress(&scene, viewport, 0.5),
      layer_draw_plan_at_progress(&scene, viewport, 0.5)
    );

    let mut malformed_scene = scene;
    let mut malformed_player = malformed_scene.actors[0].sprite;
    malformed_player.rect = drl_assets::SpriteRect::new(0, 32, 32, 32);
    malformed_scene.actors[0].sprite = malformed_player;
    assert!(layer_draw_plan_at_progress(&malformed_scene, viewport, 0.5).is_none());
  }

  #[test]
  fn elapsed_layer_plan_selects_rows_without_changing_static_draws() {
    let game = Game::new_arena(42, 12, 10).expect("arena");
    let scene = RenderScene::from_observation(&game.observe_player());
    let viewport = PixelViewport::fit(scene.map_width, scene.map_height, 960, 640);
    let frame_zero = layer_draw_plan(&scene, viewport);
    let frame_at_499 = layer_draw_plan_at_elapsed(&scene, viewport, 499, AnimationPlayback::Loop)
      .expect("frame zero plan");
    assert_eq!(frame_at_499, frame_zero);

    let frame_at_500 = layer_draw_plan_at_elapsed(&scene, viewport, 500, AnimationPlayback::Loop)
      .expect("frame one plan");
    let player = scene.actors.first().expect("player actor");
    let player_draw = frame_zero
      .iter()
      .find(|draw| draw.atlas == player.sprite.atlas && draw.animation == player.sprite.animation)
      .expect("player draw");
    let expected_uv = player
      .sprite
      .frame_rect(1)
      .expect("player frame one")
      .uv_rect(
        player.sprite.atlas.dimensions().0,
        player.sprite.atlas.dimensions().1,
      )
      .expect("player frame one UV");
    let selected_player: Vec<_> = frame_at_500
      .iter()
      .filter(|draw| draw.sprite_index == player_draw.sprite_index)
      .collect();
    assert!(!selected_player.is_empty());
    assert!(selected_player.iter().all(|draw| draw.uv == expected_uv));

    let frame_at_cycle =
      layer_draw_plan_at_elapsed(&scene, viewport, 1_000, AnimationPlayback::Loop)
        .expect("wrapped frame zero plan");
    assert_eq!(frame_at_cycle, frame_zero);
    let clamped = layer_draw_plan_at_elapsed(&scene, viewport, u64::MAX, AnimationPlayback::Clamp)
      .expect("clamped final-frame plan");
    let max_loop = layer_draw_plan_at_elapsed(&scene, viewport, u64::MAX, AnimationPlayback::Loop);
    assert_eq!(max_loop, Some(frame_at_500.clone()));
    assert_eq!(
      max_loop,
      layer_draw_plan_at_elapsed(&scene, viewport, u64::MAX, AnimationPlayback::Loop)
    );

    let static_draw = frame_zero
      .iter()
      .find(|draw| draw.animation.is_none())
      .expect("static tile draw");
    let static_selected = clamped
      .iter()
      .find(|draw| draw.sprite_index == static_draw.sprite_index)
      .expect("static draw at elapsed time");
    assert_eq!(static_selected.uv, static_draw.uv);

    let player_composite = sprite_composite_plan(&clamped)
      .into_iter()
      .find(|composite| composite.sprite_index == player_draw.sprite_index)
      .expect("player composite");
    assert_eq!(player_composite.uv, selected_player[0].uv);

    let mut malformed_scene = scene;
    let mut malformed_player = malformed_scene.actors[0].sprite;
    malformed_player.rect = drl_assets::SpriteRect::new(0, 32, 32, 32);
    malformed_scene.actors[0].sprite = malformed_player;
    assert!(
      layer_draw_plan_at_elapsed(&malformed_scene, viewport, 500, AnimationPlayback::Loop,)
        .is_none()
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
  fn sprite_composite_plan_rejects_incomplete_or_malformed_groups() {
    let game = Game::new_arena(42, 12, 10).expect("arena");
    let scene = RenderScene::from_observation(&game.observe_player());
    let viewport = PixelViewport::fit(scene.map_width, scene.map_height, 960, 640);
    let plan = layer_draw_plan(&scene, viewport);
    let group_len = scene.tiles[0].sprite.layers.len();
    let complete: Vec<_> = plan.iter().take(group_len).copied().collect();
    assert_eq!(sprite_composite_plan(&complete).len(), 1);

    let partial = &complete[..group_len - 1];
    assert!(sprite_composite_plan(partial).is_empty());

    let mut reordered = complete.clone();
    reordered.swap(0, 1);
    assert!(sprite_composite_plan(&reordered).is_empty());

    let mut duplicate = complete.clone();
    duplicate[1] = duplicate[0];
    assert!(sprite_composite_plan(&duplicate).is_empty());

    let mut mismatched = complete.clone();
    mismatched[0].sprite_index = mismatched[0].sprite_index.saturating_add(1);
    assert!(sprite_composite_plan(&mismatched).is_empty());

    let mut repeated = complete.clone();
    repeated.extend(complete);
    assert!(sprite_composite_plan(&repeated).is_empty());
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
  fn explosion_mark_phase_matches_three_integer_buckets_and_fallback() {
    assert_eq!(explosion_mark_phase(0, 10), ExplosionMarkPhase::First);
    assert_eq!(explosion_mark_phase(3, 10), ExplosionMarkPhase::First);
    assert_eq!(explosion_mark_phase(4, 10), ExplosionMarkPhase::Second);
    assert_eq!(explosion_mark_phase(6, 10), ExplosionMarkPhase::Second);
    assert_eq!(explosion_mark_phase(7, 10), ExplosionMarkPhase::Third);
    assert_eq!(explosion_mark_phase(9, 10), ExplosionMarkPhase::Third);
    assert_eq!(explosion_mark_phase(10, 10), ExplosionMarkPhase::Second);
  }

  #[test]
  fn explosion_mark_phase_normalizes_zero_and_avoids_elapsed_overflow() {
    assert_eq!(explosion_mark_phase(0, 0), ExplosionMarkPhase::First);
    assert_eq!(explosion_mark_phase(1, 0), ExplosionMarkPhase::Second);
    assert_eq!(
      explosion_mark_phase(u64::MAX, 1),
      ExplosionMarkPhase::Second
    );
    assert_eq!(
      explosion_mark_phase(u64::MAX - 1, u64::MAX),
      ExplosionMarkPhase::Third
    );
    assert_eq!(
      explosion_mark_phase(u64::MAX, u64::MAX),
      ExplosionMarkPhase::Second
    );
  }

  #[test]
  fn effect_segment_index_at_elapsed_preserves_signed_quotient_and_correction() {
    assert_eq!(effect_segment_index_at_elapsed(0, 10, 4), Some(1));
    assert_eq!(effect_segment_index_at_elapsed(3, 10, 4), Some(2));
    assert_eq!(effect_segment_index_at_elapsed(7, 10, 4), Some(3));
    assert_eq!(effect_segment_index_at_elapsed(10, 10, 4), Some(4));
    assert_eq!(effect_segment_index_at_elapsed(11, 10, 4), Some(4));
    assert_eq!(effect_segment_index_at_elapsed(20, 10, 4), Some(9));

    assert_eq!(effect_segment_index_at_elapsed(0, 10, -4), Some(-1));
    assert_eq!(effect_segment_index_at_elapsed(3, 10, -4), Some(-2));
    assert_eq!(effect_segment_index_at_elapsed(7, 10, -4), Some(-3));
    assert_eq!(effect_segment_index_at_elapsed(10, 10, -4), Some(-4));
    assert_eq!(effect_segment_index_at_elapsed(20, 10, -4), Some(-9));
  }

  #[test]
  fn effect_segment_index_at_elapsed_rejects_zero_duration_and_stays_overflow_safe() {
    assert_eq!(effect_segment_index_at_elapsed(0, 0, 4), None);
    assert_eq!(effect_segment_index_at_elapsed(u64::MAX, 1, 0), Some(0));
    assert_eq!(
      effect_segment_index_at_elapsed(u64::MAX, u64::MAX, i32::MAX),
      Some(i32::MAX)
    );
    assert_eq!(effect_segment_index_at_elapsed(u64::MAX, 1, i32::MAX), None);
    assert_eq!(effect_segment_index_at_elapsed(2, 1, i32::MIN), None);
  }

  #[test]
  fn kill_animation_segment_index_respects_lead_delay_and_clamp() {
    assert_eq!(
      kill_animation_segment_index_at_elapsed(0, 100, 3, 20, false),
      Some(0)
    );
    assert_eq!(
      kill_animation_segment_index_at_elapsed(20, 100, 3, 20, false),
      Some(0)
    );
    assert_eq!(
      kill_animation_segment_index_at_elapsed(47, 100, 3, 20, false),
      Some(1)
    );
    assert_eq!(
      kill_animation_segment_index_at_elapsed(80, 100, 3, 20, false),
      Some(2)
    );
    assert_eq!(
      kill_animation_segment_index_at_elapsed(100, 100, 3, 20, false),
      Some(2)
    );
  }

  #[test]
  fn kill_animation_segment_index_ignores_lead_for_reverse_and_rejects_invalid_inputs() {
    assert_eq!(
      kill_animation_segment_index_at_elapsed(33, 100, 3, 20, true),
      Some(0)
    );
    assert_eq!(
      kill_animation_segment_index_at_elapsed(34, 100, 3, 20, true),
      Some(1)
    );
    assert_eq!(
      kill_animation_segment_index_at_elapsed(u64::MAX, u64::MAX, u32::MAX, 0, true),
      Some(u32::MAX - 1)
    );
    assert_eq!(
      kill_animation_segment_index_at_elapsed(0, 0, 3, 0, false),
      None
    );
    assert_eq!(
      kill_animation_segment_index_at_elapsed(0, 10, 0, 0, false),
      None
    );
    assert_eq!(
      kill_animation_segment_index_at_elapsed(0, 10, 3, 11, false),
      None
    );
  }

  #[test]
  fn fx_animation_frame_index_selects_and_clamps_integer_frames() {
    assert_eq!(fx_animation_frame_index_at_elapsed(0, 100, 4), Some(0));
    assert_eq!(fx_animation_frame_index_at_elapsed(24, 100, 4), Some(0));
    assert_eq!(fx_animation_frame_index_at_elapsed(25, 100, 4), Some(1));
    assert_eq!(fx_animation_frame_index_at_elapsed(75, 100, 4), Some(3));
    assert_eq!(fx_animation_frame_index_at_elapsed(100, 100, 4), Some(3));
    assert_eq!(fx_animation_frame_index_at_elapsed(101, 100, 4), Some(3));
    assert_eq!(fx_animation_frame_index_at_elapsed(0, 10, 3), Some(0));
    assert_eq!(fx_animation_frame_index_at_elapsed(3, 10, 3), Some(0));
    assert_eq!(fx_animation_frame_index_at_elapsed(4, 10, 3), Some(1));
    assert_eq!(fx_animation_frame_index_at_elapsed(6, 10, 3), Some(1));
    assert_eq!(fx_animation_frame_index_at_elapsed(7, 10, 3), Some(2));
    assert_eq!(
      fx_animation_frame_index_at_elapsed(u64::MAX, 10, 1),
      Some(0)
    );
  }

  #[test]
  fn fx_animation_frame_index_rejects_empty_metadata_and_stays_overflow_safe() {
    assert_eq!(fx_animation_frame_index_at_elapsed(0, 0, 4), None);
    assert_eq!(fx_animation_frame_index_at_elapsed(0, 100, 0), None);
    assert_eq!(
      fx_animation_frame_index_at_elapsed(u64::MAX, 1, u16::MAX),
      Some(u16::MAX - 1)
    );
    assert_eq!(
      fx_animation_frame_index_at_elapsed(u64::MAX, u64::MAX, u16::MAX),
      Some(u16::MAX - 1)
    );
  }

  #[test]
  fn move_animation_progress_clamps_normalized_elapsed_time() {
    assert_eq!(move_animation_progress_at_elapsed(0, 100), Some(0.0));
    assert_eq!(move_animation_progress_at_elapsed(25, 100), Some(0.25));
    assert_eq!(move_animation_progress_at_elapsed(1, 3), Some(1.0 / 3.0));
    assert_eq!(move_animation_progress_at_elapsed(100, 100), Some(1.0));
    assert_eq!(move_animation_progress_at_elapsed(101, 100), Some(1.0));
    assert_eq!(move_animation_progress_at_elapsed(u64::MAX, 1), Some(1.0));
  }

  #[test]
  fn move_animation_progress_rejects_zero_duration() {
    assert_eq!(move_animation_progress_at_elapsed(0, 0), None);
    assert_eq!(move_animation_progress_at_elapsed(u64::MAX, 0), None);
  }

  #[test]
  fn missile_step_index_uses_normalized_step_delay() {
    assert_eq!(missile_step_index_at_elapsed(0, 100, 10), Some(0));
    assert_eq!(missile_step_index_at_elapsed(9, 100, 10), Some(0));
    assert_eq!(missile_step_index_at_elapsed(10, 100, 10), Some(1));
    assert_eq!(missile_step_index_at_elapsed(99, 100, 10), Some(9));
    assert_eq!(missile_step_index_at_elapsed(100, 100, 10), Some(10));
    assert_eq!(missile_step_index_at_elapsed(5, 5, 100), Some(5));
  }

  #[test]
  fn missile_step_index_normalizes_zero_inputs_and_stays_overflow_safe() {
    assert_eq!(missile_step_index_at_elapsed(0, 0, 0), Some(0));
    assert_eq!(missile_step_index_at_elapsed(u64::MAX, 0, 0), None);
    assert_eq!(
      missile_step_index_at_elapsed(u64::MAX, u64::MAX, 1),
      Some(1)
    );
    assert_eq!(
      missile_step_index_at_elapsed(u64::MAX, u64::MAX, u64::MAX),
      None
    );
  }

  #[test]
  fn missile_ray_sample_preserves_spacing_boundaries_and_overshoot() {
    assert_eq!(missile_ray_sample_distance_at_index(0, 10, 20), None);
    assert_eq!(missile_ray_sample_distance_at_index(0, 31, 20), Some(30));
    assert_eq!(missile_ray_sample_distance_at_index(1, 40, 20), Some(50));
    assert_eq!(missile_ray_sample_distance_at_index(1, 30, 20), None);
  }

  #[test]
  fn missile_ray_sample_floors_grid_start_and_rejects_overflow() {
    assert_eq!(missile_ray_sample_distance_at_index(0, 31, 21), Some(30));
    assert_eq!(missile_ray_sample_distance_at_index(0, 0, 0), None);
    assert_eq!(missile_ray_sample_distance_at_index(0, 20, 0), Some(20));
    assert_eq!(
      missile_ray_sample_distance_at_index(u64::MAX, u64::MAX, u64::MAX),
      None
    );
    assert_eq!(
      missile_ray_sample_distance_at_index((u64::MAX - 15) / 20, u64::MAX, 0,),
      None
    );
  }

  #[test]
  fn screen_shake_fade_follows_quadratic_active_envelope() {
    assert_eq!(screen_shake_fade_at_elapsed(0, 100), 1.0);
    assert_eq!(screen_shake_fade_at_elapsed(50, 100), 0.75);
    assert!((screen_shake_fade_at_elapsed(99, 100) - 0.0199).abs() < 0.0001);
  }

  #[test]
  fn screen_shake_fade_zeroes_at_expiry_and_for_zero_duration() {
    assert_eq!(screen_shake_fade_at_elapsed(100, 100), 0.0);
    assert_eq!(screen_shake_fade_at_elapsed(101, 100), 0.0);
    assert_eq!(screen_shake_fade_at_elapsed(u64::MAX, 0), 0.0);
  }

  #[test]
  fn particle_burst_origin_centers_one_based_legacy_cells() {
    assert_eq!(
      particle_burst_origin_at_legacy_cell(1, 1),
      Some([16, 16, 0])
    );
    assert_eq!(
      particle_burst_origin_at_legacy_cell(3, 4),
      Some([80, 112, 0])
    );
    assert_eq!(
      particle_burst_origin_at_legacy_cell(0, 0),
      Some([-16, -16, 0])
    );
  }

  #[test]
  fn particle_burst_origin_rejects_signed_overflow() {
    assert_eq!(particle_burst_origin_at_legacy_cell(i32::MIN, 1), None);
    assert_eq!(particle_burst_origin_at_legacy_cell(i32::MAX, 1), None);
  }

  #[test]
  fn particle_burst_direction_normalizes_xy_and_scales_z() {
    assert_eq!(
      particle_burst_direction([1.0, 2.0, 3.0], [3.0, 4.0], 2.0, 4.0),
      [0.6, 0.8, 1.5]
    );
  }

  #[test]
  fn particle_burst_direction_clears_zero_xy_and_keeps_z_without_scale() {
    assert_eq!(
      particle_burst_direction([1.0, 2.0, 3.0], [0.0, 0.0], 2.0, 0.0),
      [0.0, 0.0, 3.0]
    );
    assert_eq!(
      particle_burst_direction([1.0, 2.0, 3.0], [0.0, 0.0], 2.0, -1.0),
      [0.0, 0.0, 3.0]
    );
  }

  #[test]
  fn particle_burst_direction_preserves_unit_axis_and_zero_emitter_z() {
    assert_eq!(
      particle_burst_direction([0.0, 0.0, 0.0], [-2.0, 0.0], 5.0, 2.0),
      [-1.0, 0.0, 0.0]
    );
  }

  #[test]
  fn particle_burst_range_sample_interpolates_source_bounds() {
    assert_eq!(particle_burst_range_sample([2.0, 6.0], 0.0), 2.0);
    assert_eq!(particle_burst_range_sample([2.0, 6.0], 0.25), 3.0);
    assert_eq!(particle_burst_range_sample([2.0, 6.0], 0.75), 5.0);
    assert_eq!(particle_burst_range_sample([3.0, 3.0], 0.5), 3.0);
  }

  #[test]
  fn particle_burst_range_sample_preserves_reversed_and_unclamped_inputs() {
    assert_eq!(particle_burst_range_sample([6.0, 2.0], 0.25), 5.0);
    assert_eq!(particle_burst_range_sample([2.0, 6.0], 1.25), 7.0);
  }

  #[test]
  fn particle_decal_cell_maps_rounded_world_to_one_based_cells() {
    assert_eq!(particle_decal_cell_at_rounded_world([16, 16]), Some([1, 1]));
    assert_eq!(particle_decal_cell_at_rounded_world([48, 80]), Some([2, 3]));
    assert_eq!(
      particle_decal_cell_at_rounded_world([-16, -16]),
      Some([0, 0])
    );
  }

  #[test]
  fn particle_decal_cell_rejects_offset_overflow() {
    assert_eq!(particle_decal_cell_at_rounded_world([i32::MAX, 0]), None);
  }

  #[test]
  fn particle_decal_placement_preserves_cell_and_pixel_targets() {
    assert_eq!(
      particle_decal_placement_at_rounded_world([48, 80]),
      Some(ParticleDecalPlacement {
        cell: [2, 3],
        pixel: [64, 96],
      })
    );
    assert_eq!(
      particle_decal_placement_at_rounded_world([-16, -16]),
      Some(ParticleDecalPlacement {
        cell: [0, 0],
        pixel: [0, 0],
      })
    );
  }

  #[test]
  fn particle_decal_cell_eligibility_matches_legacy_guards() {
    assert!(particle_decal_cell_is_eligible(true, false, false));
    assert!(!particle_decal_cell_is_eligible(false, false, false));
    assert!(!particle_decal_cell_is_eligible(true, true, false));
    assert!(!particle_decal_cell_is_eligible(true, false, true));
    assert!(!particle_decal_cell_is_eligible(true, true, true));
  }

  #[test]
  fn particle_decal_insertion_preserves_placement_and_sprite() {
    assert_eq!(
      particle_decal_insertion_at_rounded_world([48, 80], true, false, false, 0),
      Some(ParticleDecalInsertion {
        placement: ParticleDecalPlacement {
          cell: [2, 3],
          pixel: [64, 96],
        },
        sprite_id: 0,
      })
    );
    assert_eq!(
      particle_decal_insertion_at_rounded_world([48, 80], true, false, false, 42)
        .map(|insertion| insertion.sprite_id),
      Some(42)
    );
  }

  #[test]
  fn particle_decal_insertion_rejects_ineligible_cells() {
    for flags in [
      (false, false, false),
      (true, true, false),
      (true, false, true),
      (true, true, true),
    ] {
      assert_eq!(
        particle_decal_insertion_at_rounded_world([48, 80], flags.0, flags.1, flags.2, 42),
        None
      );
    }
  }

  #[test]
  fn particle_decal_insertion_rejects_offset_overflow() {
    assert_eq!(
      particle_decal_insertion_at_rounded_world([i32::MAX, 0], true, false, false, 42),
      None
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
  fn animation_frame_index_is_bounded_and_deterministic() {
    assert_eq!(animation_frame_index(0.0, 4), Some(0));
    assert_eq!(animation_frame_index(0.249_999, 4), Some(0));
    assert_eq!(animation_frame_index(0.25, 4), Some(1));
    assert_eq!(animation_frame_index(0.5, 4), Some(2));
    assert_eq!(animation_frame_index(0.999_999, 4), Some(3));
    assert_eq!(
      animation_frame_index(0.999_999, 4),
      animation_frame_index(0.999_999, 4)
    );
  }

  #[test]
  fn animation_frame_index_rejects_invalid_progress_and_counts() {
    assert_eq!(animation_frame_index(0.0, 0), None);
    assert_eq!(animation_frame_index(-0.01, 4), None);
    assert_eq!(animation_frame_index(1.0, 4), None);
    assert_eq!(animation_frame_index(f32::NAN, 4), None);
    assert_eq!(animation_frame_index(f32::INFINITY, 4), None);
    assert_eq!(animation_frame_index(f32::NEG_INFINITY, 4), None);
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
