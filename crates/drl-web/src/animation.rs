//! Presentation-clock helpers that turn an external timestamp source into
//! bounded elapsed milliseconds without owning scheduling policy.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

/// Converts a browser animation timestamp into bounded elapsed milliseconds.
///
/// The timestamp source and scheduling policy remain outside this pure helper.
#[must_use]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) fn animation_elapsed_ms(start_ms: f64, timestamp_ms: f64) -> Option<u64> {
  if !start_ms.is_finite() || !timestamp_ms.is_finite() || timestamp_ms < start_ms {
    return None;
  }
  let elapsed_ms = (timestamp_ms - start_ms).floor();
  if elapsed_ms >= u64::MAX as f64 {
    Some(u64::MAX)
  } else {
    Some(elapsed_ms.max(0.0) as u64)
  }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) struct AnimationClock {
  pub(crate) start_ms: Option<f64>,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
impl AnimationClock {
  pub(crate) fn reset(&mut self) {
    self.start_ms = None;
  }

  pub(crate) fn visibility_changed(&mut self) {
    self.reset();
  }

  pub(crate) fn elapsed_ms(&mut self, hidden: bool, timestamp_ms: f64) -> Option<u64> {
    if hidden {
      self.reset();
      return None;
    }
    let start_ms = *self.start_ms.get_or_insert(timestamp_ms);
    animation_elapsed_ms(start_ms, timestamp_ms)
  }
}
