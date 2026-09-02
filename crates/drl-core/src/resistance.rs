//! Deterministic percentage mitigation shared by typed actor damage paths.

/// Applies one percentage resistance to a positive damage amount.
///
/// The legacy rule rounds the remaining percentage to the nearest integer,
/// keeps nonzero damage at one point, and treats a full resistance as zero
/// damage. Integer arithmetic keeps the result independent of floating-point
/// behavior and consumes no simulation randomness.
#[must_use]
pub const fn apply_damage_resistance(raw_amount: u32, resistance_percent: u32) -> u32 {
  if raw_amount == 0 || resistance_percent == 0 {
    return raw_amount;
  }
  if resistance_percent >= 100 {
    return 0;
  }

  let remaining_percent = 100 - resistance_percent;
  let scaled = (raw_amount as u64) * (remaining_percent as u64);
  let rounded = ((scaled + 50) / 100) as u32;
  if rounded == 0 { 1 } else { rounded }
}

#[cfg(test)]
mod tests {
  use super::apply_damage_resistance;

  #[test]
  fn rounds_typed_resistance_without_float_math() {
    assert_eq!(apply_damage_resistance(10, 20), 8);
    assert_eq!(apply_damage_resistance(11, 20), 9);
    assert_eq!(apply_damage_resistance(10, 25), 8);
    assert_eq!(apply_damage_resistance(11, 25), 8);
  }

  #[test]
  fn preserves_zero_and_minimum_one_rules() {
    assert_eq!(apply_damage_resistance(0, 20), 0);
    assert_eq!(apply_damage_resistance(1, 20), 1);
    assert_eq!(apply_damage_resistance(10, 100), 0);
  }

  #[test]
  fn zero_resistance_preserves_the_original_amount() {
    assert_eq!(apply_damage_resistance(u32::MAX, 0), u32::MAX);
  }
}
