//! Platform-neutral GPU helpers shared by the WebGPU shell: the bounded
//! base/mask/emissive/outline shader contract, its UV/clip-space mappings, and
//! the backend status reported to the DOM error screen.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

use drl_assets::SpriteUv;
use drl_render::{LightingBand, PixelRect};

/// Returns the six UV coordinates for a top-left-origin textured quad.
#[must_use]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) const fn base_texture_uvs(uv: SpriteUv) -> [[f32; 2]; 6] {
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
pub(crate) fn base_texture_lighting_factor(band: LightingBand) -> f32 {
  band.factor() as f32 / 100.0
}

/// Applies the legacy emissive floor to a fair RGB lighting scalar.
#[allow(dead_code)]
pub(crate) fn emissive_lighting_floor(lighting: f32, emissive: f32) -> f32 {
  lighting.max(emissive)
}

/// Matches the legacy shader's minimum surviving fragment alpha.
#[allow(dead_code)]
pub(crate) fn retains_textured_fragment(alpha: f32) -> bool {
  alpha >= 0.1
}

/// Shared WGSL source for the bounded base/mask/emissive/outline textured pass.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) const BASE_TEXTURE_SHADER: &str = r#"
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
pub(crate) const fn base_texture_ndc_rect(
  rect: PixelRect,
  canvas_width: u32,
  canvas_height: u32,
) -> [f32; 4] {
  let width = if canvas_width == 0 { 1 } else { canvas_width } as f32;
  let height = if canvas_height == 0 { 1 } else { canvas_height } as f32;
  [
    -1.0 + 2.0 * rect.x as f32 / width,
    1.0 - 2.0 * rect.y.saturating_add(rect.height) as f32 / height,
    -1.0 + 2.0 * rect.x.saturating_add(rect.width) as f32 / width,
    1.0 - 2.0 * rect.y as f32 / height,
  ]
}

/// Browser GPU backend state exposed to the DOM error screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuStatus {
  Ready,
  Unsupported,
  Lost,
}
