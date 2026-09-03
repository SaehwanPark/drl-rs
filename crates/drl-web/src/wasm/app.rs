//! winit application shell: window creation, keyboard capture, and resize.
//! Keys become semantic commands through `BrowserSession` only.

use super::*;

pub(crate) struct WinitInputApp {
  canvas: Option<HtmlCanvasElement>,
  window: Option<WinitWindow>,
}

impl WinitInputApp {
  pub(crate) fn new(canvas: HtmlCanvasElement) -> Self {
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
    KeyCode::KeyC => "c",
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
