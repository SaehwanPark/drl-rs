//! Browser-first DRL-Rust session boundary.
//!
//! `BrowserSession` is intentionally usable on native hosts for deterministic
//! tests. The WASM exports are a thin boot/input shell; gameplay state stays in
//! Rust and is never mirrored into a parallel JavaScript model.
//!
//! Module map (the browser shell keeps platform I/O only):
//!
//! - `session`: transactional session boundary over the deterministic `Game`;
//! - `input`: keyboard/DOM controls to semantic `Command` values;
//! - `dom`: accessible DOM projections of fair observations;
//! - `assets`: stable atlas identities and same-origin asset URLs;
//! - `animation`: presentation clock helpers;
//! - `gpu`: platform-neutral GPU/shader contract helpers and `GpuStatus`;
//! - `persistence`: snapshot codec and semantic-identity validation;
//! - `texture`: WebGPU texture upload/pipeline helpers (WASM only);
//! - `wasm`: the `winit`/WebGPU/DOM browser shell (WASM only).

mod animation;
mod assets;
mod dom;
mod gpu;
mod input;
mod persistence;
mod session;
#[cfg(target_arch = "wasm32")]
mod texture;
#[cfg(target_arch = "wasm32")]
pub(crate) mod wasm;

pub use assets::{
  GRAPHICS_ASSET_ROOT, TextureSourceDimensionsError, TextureSourcePathError, browser_asset_url,
  texture_source_manifest, texture_source_url, validate_texture_source_dimensions,
};
pub use gpu::GpuStatus;
pub use input::InventoryAction;
pub use persistence::SnapshotError;
pub use session::{BrowserSession, M4_HEIGHT, M4_SEED, M4_START, M4_WIDTH};

// Crate-internal surface shared by the browser shell and the boundary tests.
// Names stay here only while another module actually resolves them at the crate
// root, so a target-specific helper never looks like public API.
#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) use animation::AnimationClock;
#[cfg(test)]
pub(crate) use animation::animation_elapsed_ms;
#[cfg(target_arch = "wasm32")]
pub(crate) use assets::REGISTERED_ATLASES;
#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) use dom::{inventory_markup, minimap_markup};
#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) use gpu::{
  BASE_TEXTURE_SHADER, base_texture_lighting_factor, base_texture_ndc_rect, base_texture_uvs,
};
#[cfg(test)]
pub(crate) use gpu::{emissive_lighting_floor, retains_textured_fragment};
#[cfg(test)]
pub(crate) use session::chainfire_ammo_cost;

#[cfg(test)]
mod tests;
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests;

#[cfg(target_arch = "wasm32")]
pub use wasm::{
  WebGpuRenderer, boot, clear_save, dispatch_inventory, dispatch_key, key_command, load,
  load_texture_source, resize, restart, save, set_muted, set_volume, unlock_audio,
};
