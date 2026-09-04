//! Native `winit` application shell and event translation.

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

use crate::input::command_for_key;
use crate::renderer::{DesktopRenderer, RenderError, SurfaceStatus};
use crate::session::{DesktopSession, demo_scenario};

/// Errors that can prevent the native shell from starting.
#[derive(Debug)]
pub enum DesktopError {
  Scenario(String),
  Initialization(String),
  EventLoop(String),
}

impl Display for DesktopError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Scenario(error) => write!(formatter, "scenario: {error}"),
      Self::Initialization(error) => write!(formatter, "initialization: {error}"),
      Self::EventLoop(error) => write!(formatter, "event loop: {error}"),
    }
  }
}

impl Error for DesktopError {}

/// Native application state connecting one session to one presentation shell.
pub struct DesktopApp {
  session: DesktopSession,
  window: Option<Arc<Window>>,
  renderer: Option<DesktopRenderer>,
}

impl DesktopApp {
  /// Creates an app around an already-instantiated deterministic session.
  #[must_use]
  pub fn new(session: DesktopSession) -> Self {
    Self {
      session,
      window: None,
      renderer: None,
    }
  }

  fn redraw(&mut self, event_loop: &ActiveEventLoop) {
    let Some(window) = self.window.as_ref().cloned() else {
      return;
    };
    let Some(renderer) = self.renderer.as_mut() else {
      return;
    };
    let scene = self.session.scene();
    match renderer.render(&scene) {
      Ok(()) => {}
      Err(RenderError::Surface(SurfaceStatus::Lost | SurfaceStatus::Outdated)) => {
        renderer.resize(window.inner_size());
      }
      Err(RenderError::Surface(SurfaceStatus::Timeout | SurfaceStatus::Occluded)) => {}
      Err(error) => {
        eprintln!("drl-desktop: rendering stopped: {error:?}");
        event_loop.exit();
      }
    }
  }

  fn request_redraw(&self) {
    if let Some(window) = &self.window {
      window.request_redraw();
    }
  }
}

impl ApplicationHandler for DesktopApp {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_some() {
      return;
    }

    let attributes = Window::default_attributes()
      .with_title("DRL-Rust native preview")
      .with_inner_size(LogicalSize::new(960.0, 640.0))
      .with_min_inner_size(LogicalSize::new(320.0, 240.0));
    let window = match event_loop.create_window(attributes) {
      Ok(window) => Arc::new(window),
      Err(error) => {
        eprintln!("drl-desktop: window creation failed: {error}");
        event_loop.exit();
        return;
      }
    };
    let renderer = match DesktopRenderer::new(window.clone()) {
      Ok(renderer) => renderer,
      Err(error) => {
        eprintln!("drl-desktop: GPU initialization failed: {error}");
        event_loop.exit();
        return;
      }
    };
    self.window = Some(window);
    self.renderer = Some(renderer);
    self.request_redraw();
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    _window_id: WindowId,
    event: WindowEvent,
  ) {
    match event {
      WindowEvent::CloseRequested => event_loop.exit(),
      WindowEvent::Resized(size) => {
        if let Some(renderer) = self.renderer.as_mut() {
          renderer.resize(size);
        }
        self.request_redraw();
      }
      WindowEvent::ScaleFactorChanged { .. } => {
        if let (Some(renderer), Some(window)) = (self.renderer.as_mut(), self.window.as_ref()) {
          // `inner_size` is physical pixels. Do not multiply it by the new
          // factor, or fractional Wayland scaling would be applied twice.
          renderer.resize(window.inner_size());
        }
        self.request_redraw();
      }
      WindowEvent::KeyboardInput { event, .. }
        if event.state == ElementState::Pressed && !event.repeat =>
      {
        if let PhysicalKey::Code(key) = event.physical_key {
          if key == winit::keyboard::KeyCode::Escape {
            event_loop.exit();
          } else if let Some(command) = command_for_key(key, &self.session.observation()) {
            if let Err(error) = self.session.submit(command) {
              eprintln!("drl-desktop: command rejected: {error}");
            }
            self.request_redraw();
          }
        }
      }
      WindowEvent::RedrawRequested => self.redraw(event_loop),
      _ => {}
    }
  }
}

/// Runs the native preview shell with the deterministic demo scenario.
pub fn run() -> Result<(), DesktopError> {
  let scenario = demo_scenario().map_err(DesktopError::Scenario)?;
  let session = DesktopSession::new(&scenario)
    .map_err(|error| DesktopError::Initialization(error.to_string()))?;
  let event_loop = EventLoop::new().map_err(|error| DesktopError::EventLoop(error.to_string()))?;
  let mut app = DesktopApp::new(session);
  event_loop
    .run_app(&mut app)
    .map_err(|error| DesktopError::EventLoop(error.to_string()))
}
