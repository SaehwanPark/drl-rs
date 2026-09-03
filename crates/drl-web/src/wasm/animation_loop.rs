//! requestAnimationFrame scheduling and the presentation-only animation loop.
//! Timing never advances gameplay state.

use super::*;

pub(crate) fn render_scene(
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

pub(crate) fn start_animation_loop() -> Result<(), JsValue> {
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
