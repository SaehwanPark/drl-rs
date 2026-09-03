//! `wasm_bindgen` exports: the thin boot/control surface used by
//! `web/bootstrap.js`.

use super::*;

/// Starts the browser shell after the HTML start button has granted audio.
#[wasm_bindgen]
pub async fn boot() -> Result<JsValue, JsValue> {
  let window: Window = web_sys::window().ok_or_else(|| JsValue::from_str("window unavailable"))?;
  let document = window
    .document()
    .ok_or_else(|| JsValue::from_str("document unavailable"))?;
  let canvas = document
    .get_element_by_id("game-canvas")
    .ok_or_else(|| JsValue::from_str("#game-canvas is missing"))?
    .dyn_into::<HtmlCanvasElement>()?;
  canvas.set_width(768);
  canvas.set_height(512);
  let mut session = BrowserSession::new().map_err(|error| JsValue::from_str(&error.to_string()))?;
  let restore_message = match read_persisted_session() {
    Ok(Some(token)) => match session.restore_snapshot_with_format(&token) {
      Ok(_) => " Restored the saved session.".to_string(),
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
  if restore_message.starts_with(" Saved session ignored") {
    set_diagnostic(
      &document,
      "Saved session incompatible",
      &restore_message,
      "Use Clear save to remove it, then save a new session from this build.",
    );
  }
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
    if matches!(
      command,
      Command::AttackRanged(_) | Command::AttackRangedChainfire(_)
    ) {
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
        let clear_warning = clear_warning.or(quarantine_warning);
        if clear_warning.is_none() {
          if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            clear_persistence_diagnostic(&document);
          }
        }
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
    Ok(_) => {
      if let Some(document) = web_sys::window().and_then(|window| window.document()) {
        clear_persistence_diagnostic(&document);
      }
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
      "Session loaded from this device.".to_string()
    }
    Err(error) => {
      let status = rejected_save_message(&token, &error);
      if let Some(document) = web_sys::window().and_then(|window| window.document()) {
        set_diagnostic(
          &document,
          "Saved session incompatible",
          &status,
          "Use Clear save to remove it, then save a new session from this build.",
        );
      }
      status
    }
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
