//! Pure animation timing math for caller-supplied presentation elapsed time.

use drl_assets::SpriteAnimation;

/// Explicit policy for elapsed-time animation selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationPlayback {
  /// Repeat the descriptor's frame sequence.
  Loop,
  /// Hold the final frame after the sequence completes.
  Clamp,
}

/// Selects a frame from caller-supplied elapsed milliseconds.
///
/// This function owns no clock and does not decide which effect or sprite
/// supplies elapsed time. Zero metadata is rejected rather than guessed.
#[must_use]
pub fn animation_frame_index_at_elapsed(
  animation: SpriteAnimation,
  elapsed_ms: u64,
  playback: AnimationPlayback,
) -> Option<u16> {
  if animation.frame_count == 0 || animation.frame_time_ms == 0 {
    return None;
  }
  let frame_count = u64::from(animation.frame_count);
  let frame_time_ms = u64::from(animation.frame_time_ms);
  let frame_index = match playback {
    AnimationPlayback::Loop => {
      let cycle_ms = frame_count.checked_mul(frame_time_ms)?;
      (elapsed_ms % cycle_ms) / frame_time_ms
    }
    AnimationPlayback::Clamp => (elapsed_ms / frame_time_ms).min(frame_count - 1),
  };
  u16::try_from(frame_index).ok()
}

#[cfg(test)]
mod tests {
  use super::*;

  const TWO_FRAMES: SpriteAnimation = SpriteAnimation {
    frame_count: 2,
    frame_time_ms: 500,
  };

  #[test]
  fn elapsed_frame_selection_respects_boundaries_and_policy() {
    assert_eq!(
      animation_frame_index_at_elapsed(TWO_FRAMES, 0, AnimationPlayback::Loop),
      Some(0)
    );
    assert_eq!(
      animation_frame_index_at_elapsed(TWO_FRAMES, 499, AnimationPlayback::Loop),
      Some(0)
    );
    assert_eq!(
      animation_frame_index_at_elapsed(TWO_FRAMES, 500, AnimationPlayback::Loop),
      Some(1)
    );
    assert_eq!(
      animation_frame_index_at_elapsed(TWO_FRAMES, 1_000, AnimationPlayback::Loop),
      Some(0)
    );
    assert_eq!(
      animation_frame_index_at_elapsed(TWO_FRAMES, 1_000, AnimationPlayback::Clamp),
      Some(1)
    );
    let max_elapsed_frame =
      animation_frame_index_at_elapsed(TWO_FRAMES, u64::MAX, AnimationPlayback::Loop);
    assert_eq!(max_elapsed_frame, Some(1));
    assert_eq!(
      max_elapsed_frame,
      animation_frame_index_at_elapsed(TWO_FRAMES, u64::MAX, AnimationPlayback::Loop)
    );
    assert_eq!(
      animation_frame_index_at_elapsed(TWO_FRAMES, u64::MAX, AnimationPlayback::Clamp),
      Some(1)
    );
  }

  #[test]
  fn elapsed_frame_selection_rejects_zero_metadata() {
    assert_eq!(
      animation_frame_index_at_elapsed(
        SpriteAnimation {
          frame_count: 0,
          frame_time_ms: 500,
        },
        0,
        AnimationPlayback::Loop,
      ),
      None
    );
    assert_eq!(
      animation_frame_index_at_elapsed(
        SpriteAnimation {
          frame_count: 2,
          frame_time_ms: 0,
        },
        0,
        AnimationPlayback::Clamp,
      ),
      None
    );
  }
}
