//! Presentation-clock contracts; timing never advances gameplay.

use super::*;

#[test]
fn animation_elapsed_ms_is_monotonic_bounded_and_clock_free() {
  assert_eq!(animation_elapsed_ms(100.0, 100.0), Some(0));
  assert_eq!(animation_elapsed_ms(100.0, 100.9), Some(0));
  assert_eq!(animation_elapsed_ms(100.0, 101.1), Some(1));
  assert_eq!(animation_elapsed_ms(100.0, 99.0), None);
  assert_eq!(animation_elapsed_ms(f64::NAN, 100.0), None);
  assert_eq!(animation_elapsed_ms(100.0, f64::INFINITY), None);
  assert_eq!(animation_elapsed_ms(0.0, u64::MAX as f64), Some(u64::MAX));
}

#[test]
fn animation_clock_rebases_after_hidden_frames() {
  let mut clock = AnimationClock::default();
  assert_eq!(clock.elapsed_ms(false, 100.0), Some(0));
  assert_eq!(clock.elapsed_ms(false, 101.0), Some(1));
  assert_eq!(clock.elapsed_ms(true, 500.0), None);
  assert_eq!(clock.elapsed_ms(false, 501.0), Some(0));
  assert_eq!(clock.elapsed_ms(false, 502.0), Some(1));
  clock.reset();
  assert_eq!(clock.elapsed_ms(false, 900.0), Some(0));
}

#[test]
fn animation_clock_rebases_on_visibility_lifecycle_change() {
  let mut clock = AnimationClock::default();
  assert_eq!(clock.elapsed_ms(false, 100.0), Some(0));
  assert_eq!(clock.elapsed_ms(false, 101.0), Some(1));
  clock.visibility_changed();
  assert_eq!(clock.elapsed_ms(false, 500.0), Some(0));
  clock.visibility_changed();
  assert_eq!(clock.elapsed_ms(false, 900.0), Some(0));
}
