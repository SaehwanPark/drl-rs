//! Browser/WASM shell module map.
//!
//! The shell owns browser I/O only. Gameplay state, presentation plans, and
//! semantic commands stay in the platform-neutral modules so the same contracts
//! also drive a future native desktop shell.

mod animation_loop;
mod app;
mod exports;
mod renderer;
mod scene;
mod shell_dom;
pub(crate) mod storage;
mod textures;

pub use exports::{
  boot, clear_save, dispatch_inventory, dispatch_key, key_command, load, resize, restart, save,
  set_muted, set_volume, unlock_audio,
};
pub use renderer::WebGpuRenderer;
pub use textures::load_texture_source;

use drl_assets::AtlasTextureSource;
use drl_protocol::{Command, ItemId, PlayerObservation, Position};
use drl_render::{
  AnimationPlayback, MinimapState, ParticleDecalSprite, ParticleDecalStore, PixelViewport,
  RenderScene, scene_clear_color,
};
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

use super::texture::{BaseTexturePipeline, GpuTextureCache};
use super::*;
use animation_loop::{render_scene, start_animation_loop};
use app::WinitInputApp;
use scene::{scene_vertices, target_vertices};
use shell_dom::{
  clear_persistence_diagnostic, set_diagnostic, set_status, update_dom, update_target_status,
};
use storage::{
  append_persistence_warning, persist_session, read_persisted_session, rejected_save_message,
  remove_persisted_session, remove_rejected_session, save_after_command,
};

thread_local! {
  static SESSION: RefCell<Option<BrowserSession>> = const { RefCell::new(None) };
  static RENDERER: RefCell<Option<WebGpuRenderer>> = const { RefCell::new(None) };
  static AUDIO: RefCell<Option<drl_audio::WebAudioMixer>> = const { RefCell::new(None) };
  static TARGET: RefCell<Option<Position>> = const { RefCell::new(None) };
  static ANIMATION_CLOCK: RefCell<AnimationClock> = const { RefCell::new(AnimationClock { start_ms: None }) };
  static ANIMATION_LOOP: RefCell<Option<Closure<dyn FnMut(f64)>>> = const { RefCell::new(None) };
  static VISIBILITY_LISTENER: RefCell<Option<Closure<dyn FnMut()>>> = const { RefCell::new(None) };
}

pub(crate) const SAVE_STORAGE_KEY: &str = "drl-rust:m4-session:v1";
pub(crate) const REJECTED_SAVE_STORAGE_KEY: &str = "drl-rust:m4-session:v1:rejected";
