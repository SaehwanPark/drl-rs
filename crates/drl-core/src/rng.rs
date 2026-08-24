//! Deterministic pseudo-random number generator for DRL-Rust simulation core.
//!
//! Provides reproducible, seedable randomness with no global or ambient state.
//! Uses SplitMix64 for state initialization and Xoshiro256++ for random generation.

/// Deterministic 64-bit PRNG wrapping Xoshiro256++.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRng {
  initial_seed: u64,
  state: [u64; 4],
}

/// Version of the deterministic sampling algorithms layered on raw PRNG output.
///
/// Replay metadata does not yet carry this identifier; it is exposed here so
/// the sampler contract has one explicit version to bind when replay semantics
/// versioning is added.
pub const RNG_SAMPLING_SEMANTICS_VERSION: u32 = 1;

impl GameRng {
  /// Creates a new RNG seeded deterministically from a 64-bit integer.
  #[must_use]
  pub fn from_seed(seed: u64) -> Self {
    let mut sm = SplitMix64(seed);
    let s0 = sm.next_u64();
    let s1 = sm.next_u64();
    let s2 = sm.next_u64();
    let s3 = sm.next_u64();
    Self {
      initial_seed: seed,
      state: [s0, s1, s2, s3],
    }
  }

  /// Returns the initial seed this RNG was constructed with.
  #[must_use]
  pub const fn initial_seed(&self) -> u64 {
    self.initial_seed
  }

  /// Returns the current internal 256-bit state.
  #[must_use]
  pub const fn state(&self) -> [u64; 4] {
    self.state
  }

  /// Generates the next random 64-bit unsigned integer.
  pub fn next_u64(&mut self) -> u64 {
    let result = (self.state[0].wrapping_add(self.state[3]))
      .rotate_left(23)
      .wrapping_add(self.state[0]);

    let t = self.state[1] << 17;

    self.state[2] ^= self.state[0];
    self.state[3] ^= self.state[1];
    self.state[1] ^= self.state[2];
    self.state[0] ^= self.state[3];

    self.state[2] ^= t;
    self.state[3] = self.state[3].rotate_left(45);

    result
  }

  /// Generates the next random 32-bit unsigned integer.
  pub fn next_u32(&mut self) -> u32 {
    (self.next_u64() >> 32) as u32
  }

  /// Generates a random integer within `[min, max)` (half-open range).
  ///
  /// Rejection sampling uses the complete `2^32` `u32` output domain, so every
  /// value in the requested span has equal probability. Rejected raw samples
  /// consume additional PRNG output by design and are part of the sampling
  /// semantics version.
  ///
  /// Panics if `range` is empty (`min >= max`).
  pub fn gen_range(&mut self, range: std::ops::Range<u32>) -> u32 {
    assert!(range.start < range.end, "empty range for gen_range");
    let span = u64::from(range.end - range.start);
    let domain_size = u64::from(u32::MAX) + 1;
    let acceptance_limit = domain_size - (domain_size % span);

    loop {
      let sample = u64::from(self.next_u32());
      if sample < acceptance_limit {
        return range.start + (sample % span) as u32;
      }
    }
  }

  /// Generates a random boolean with the specified probability in `[0.0, 1.0]`.
  pub fn gen_bool(&mut self, probability: f64) -> bool {
    if probability <= 0.0 {
      return false;
    }
    if probability >= 1.0 {
      return true;
    }
    let threshold = (probability * (u32::MAX as f64)) as u32;
    self.next_u32() < threshold
  }

  /// In-place Fisher-Yates shuffle on a mutable slice.
  pub fn shuffle<T>(&mut self, slice: &mut [T]) {
    let len = slice.len();
    if len <= 1 {
      return;
    }
    for i in (1..len).rev() {
      let j = self.gen_range(0..(i as u32 + 1)) as usize;
      slice.swap(i, j);
    }
  }
}

/// Helper SplitMix64 generator for seeding states.
struct SplitMix64(u64);

impl SplitMix64 {
  fn next_u64(&mut self) -> u64 {
    self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = self.0;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_rng_reproducibility() {
    let mut rng1 = GameRng::from_seed(12345);
    let mut rng2 = GameRng::from_seed(12345);

    for _ in 0..100 {
      assert_eq!(rng1.next_u64(), rng2.next_u64());
    }
  }

  #[test]
  fn test_different_seeds_produce_different_sequences() {
    let mut rng1 = GameRng::from_seed(1);
    let mut rng2 = GameRng::from_seed(2);

    let val1 = rng1.next_u64();
    let val2 = rng2.next_u64();
    assert_ne!(val1, val2);
  }

  #[test]
  fn test_gen_range_bounds() {
    let mut rng = GameRng::from_seed(999);
    for _ in 0..500 {
      let val = rng.gen_range(5..15);
      assert!((5..15).contains(&val));
    }
  }

  #[test]
  fn test_raw_rng_golden_vector() {
    let mut rng = GameRng::from_seed(0);
    let expected = [
      5_987_356_902_031_041_503,
      7_051_070_477_665_621_255,
      6_633_766_593_972_829_180,
      211_316_841_551_650_330,
      9_136_120_204_379_184_874,
    ];

    for value in expected {
      assert_eq!(rng.next_u64(), value);
    }
  }

  #[test]
  fn test_gen_range_golden_vectors_use_rejection_sampling() {
    let mut rng = GameRng::from_seed(0);
    assert_eq!(rng.gen_range(0..3), 2);
    assert_eq!(rng.gen_range(5..15), 12);
    assert_eq!(rng.gen_range(0..100), 45);

    let mut wide_rng = GameRng::from_seed(1);
    assert_eq!(wide_rng.gen_range(0..2_147_483_649), 430_144_855);
    assert_eq!(wide_rng.gen_range(0..2_147_483_649), 793_188_427);
  }

  #[test]
  fn test_gen_bool_golden_vectors() {
    let mut rng = GameRng::from_seed(0);
    assert!(!rng.gen_bool(0.0));
    assert!(!rng.gen_bool(0.25));
    assert!(rng.gen_bool(0.5));
    assert!(rng.gen_bool(0.75));
    assert!(rng.gen_bool(1.0));
  }

  #[test]
  fn test_shuffle_deterministic() {
    let mut rng1 = GameRng::from_seed(777);
    let mut rng2 = GameRng::from_seed(777);

    let mut list1 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut list2 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    rng1.shuffle(&mut list1);
    rng2.shuffle(&mut list2);

    assert_eq!(list1, list2);
  }
}
