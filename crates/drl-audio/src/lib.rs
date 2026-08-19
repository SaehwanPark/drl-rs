//! Audio playback and cue mapping layer for DRL-Rust.
//!
//! Handles sound effects and music triggered by simulation events.

/// Returns the audio component name.
#[must_use]
pub fn audio_name() -> &'static str {
  "drl-audio"
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_audio_name() {
    assert_eq!(audio_name(), "drl-audio");
  }
}
