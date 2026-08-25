//! Semantic audio cue mapping for DRL-Rust.
//!
//! The first browser slice uses generated tones. Legacy audio and music are
//! deliberately not bundled until their redistribution rights are recorded.

use drl_protocol::{AttackOutcome, GameEvent};

/// Semantic cue independent of a particular audio file or backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioCue {
  Move,
  Wait,
  AttackMelee,
  AttackRanged,
  Hit,
  Miss,
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

/// Maps deterministic simulation events to semantic cues.
#[must_use]
pub fn cues_for_events(events: &[GameEvent]) -> Vec<AudioCue> {
  let mut cues = Vec::new();
  for event in events {
    match event {
      GameEvent::EntityMoved { .. } => cues.push(AudioCue::Move),
      GameEvent::EntityWaited { .. } => cues.push(AudioCue::Wait),
      GameEvent::AttackResolved {
        outcome, is_ranged, ..
      } => {
        cues.push(if *is_ranged {
          AudioCue::AttackRanged
        } else {
          AudioCue::AttackMelee
        });
        match outcome {
          AttackOutcome::Hit { .. } | AttackOutcome::Blocked => cues.push(AudioCue::Hit),
          AttackOutcome::Miss => cues.push(AudioCue::Miss),
        }
      }
      GameEvent::ActorDied { .. } => cues.push(AudioCue::Death),
      GameEvent::ItemPickedUp { .. } => cues.push(AudioCue::Pickup),
      GameEvent::ItemDropped { .. } => cues.push(AudioCue::Drop),
      GameEvent::ItemEquipped { .. } | GameEvent::ItemUnequipped { .. } => {
        cues.push(AudioCue::Equip)
      }
      GameEvent::ItemUsed { .. } => cues.push(AudioCue::Use),
      GameEvent::WeaponReloaded { .. } => cues.push(AudioCue::Reload),
      GameEvent::LevelTransitioned { .. } => cues.push(AudioCue::LevelTransition),
      GameEvent::PlayerTeleported { .. } => cues.push(AudioCue::Teleport),
      GameEvent::ActorKnockedBack { .. } => cues.push(AudioCue::Knockback),
      GameEvent::TurnStarted { .. }
      | GameEvent::ActionCostPaid { .. }
      | GameEvent::DamageApplied { .. }
      | GameEvent::MedicalPowerarmorRepaired { .. }
      | GameEvent::SubtleKnifeInvoked { .. }
      | GameEvent::TrigunAltReloaded { .. }
      | GameEvent::GrammatonFireModeChanged { .. }
      | GameEvent::NukeActivated { .. }
      | GameEvent::LevelNuked { .. }
      | GameEvent::TurnEnded { .. } => {}
    }
  }
  cues
}

/// Returns the audio component name.
#[must_use]
pub fn audio_name() -> &'static str {
  "drl-audio"
}

/// Small deterministic frequency table used by generated browser cues.
#[must_use]
pub const fn cue_frequency(cue: AudioCue) -> f32 {
  match cue {
    AudioCue::Move => 220.0,
    AudioCue::Wait => 180.0,
    AudioCue::AttackMelee => 110.0,
    AudioCue::AttackRanged => 440.0,
    AudioCue::Hit => 330.0,
    AudioCue::Miss => 90.0,
    AudioCue::Death => 55.0,
    AudioCue::Pickup => 660.0,
    AudioCue::Drop => 300.0,
    AudioCue::Equip => 520.0,
    AudioCue::Use => 740.0,
    AudioCue::Reload => 260.0,
    AudioCue::Teleport => 880.0,
    AudioCue::LevelTransition => 130.0,
    AudioCue::Knockback => 150.0,
  }
}

/// User-controlled browser audio state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioSettings {
  pub muted: bool,
  pub volume: f32,
  pub unlocked: bool,
}

impl Default for AudioSettings {
  fn default() -> Self {
    Self {
      muted: false,
      volume: 0.65,
      unlocked: false,
    }
  }
}

/// Web Audio mixer. Construct and unlock this from a click/key handler so
/// browser autoplay policy cannot turn audio failure into gameplay failure.
#[cfg(target_arch = "wasm32")]
pub struct WebAudioMixer {
  context: web_sys::AudioContext,
  gain: web_sys::GainNode,
  settings: AudioSettings,
}

#[cfg(target_arch = "wasm32")]
impl WebAudioMixer {
  /// Creates a suspended Web Audio context and master gain node.
  pub fn new() -> Result<Self, wasm_bindgen::JsValue> {
    let context = web_sys::AudioContext::new()?;
    let gain = context.create_gain()?;
    gain.connect_with_audio_node(&context.destination())?;
    gain.gain().set_value(AudioSettings::default().volume);
    Ok(Self {
      context,
      gain,
      settings: AudioSettings::default(),
    })
  }

  /// Resumes the context after a trusted user gesture.
  pub async fn unlock(&mut self) -> Result<(), wasm_bindgen::JsValue> {
    wasm_bindgen_futures::JsFuture::from(self.context.resume()?).await?;
    self.settings.unlocked = true;
    Ok(())
  }

  /// Updates mute and volume without affecting simulation timing.
  pub fn set_settings(&mut self, muted: bool, volume: f32) {
    self.settings.muted = muted;
    self.settings.volume = volume.clamp(0.0, 1.0);
    let gain = if muted { 0.0 } else { self.settings.volume };
    self.gain.gain().set_value(gain);
  }

  /// Returns the current audio state for an accessible status message.
  #[must_use]
  pub fn settings(&self) -> AudioSettings {
    self.settings
  }

  /// Plays a short generated cue. Failure is returned to the presentation
  /// shell and never propagated into the simulation command path.
  pub fn play(&self, cue: AudioCue) -> Result<(), wasm_bindgen::JsValue> {
    if !self.settings.unlocked || self.settings.muted {
      return Ok(());
    }
    let oscillator = self.context.create_oscillator()?;
    oscillator.set_type(web_sys::OscillatorType::Sine);
    oscillator.frequency().set_value(cue_frequency(cue));
    oscillator.connect_with_audio_node(&self.gain)?;
    let now = self.context.current_time();
    oscillator.start()?;
    oscillator.stop_with_when(now + 0.08)?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn event_mapping_is_deterministic() {
    let events = [GameEvent::EntityWaited {
      entity_id: drl_protocol::EntityId::new(1),
      position: drl_protocol::Position::new(1, 1),
    }];
    assert_eq!(cues_for_events(&events), vec![AudioCue::Wait]);
  }
}
