use super::ParticleDecalInsertion;

/// Reports why a particle-decal request was not retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleDecalStorageError {
  /// The caller-provided request capacity has already been reached.
  CapacityExceeded { capacity: usize },
}

/// Caller-owned, deterministic storage for accepted particle-decal requests.
///
/// Requests are retained exactly in insertion order, including duplicates.
/// This store owns no map lookup, sprite selection, particle lifecycle,
/// rendering, or browser state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleDecalStore {
  capacity: usize,
  entries: Vec<ParticleDecalInsertion>,
}

impl ParticleDecalStore {
  /// Creates empty storage with the caller-provided request capacity.
  #[must_use]
  pub fn new(capacity: usize) -> Self {
    Self {
      capacity,
      entries: Vec::new(),
    }
  }

  /// Returns the maximum number of requests this store can retain.
  #[must_use]
  pub const fn capacity(&self) -> usize {
    self.capacity
  }

  /// Returns the number of retained requests.
  #[must_use]
  pub fn len(&self) -> usize {
    self.entries.len()
  }

  /// Returns whether no requests have been retained.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  /// Returns whether the caller-provided capacity has been reached.
  #[must_use]
  pub fn is_full(&self) -> bool {
    self.entries.len() >= self.capacity
  }

  /// Returns retained requests in their original insertion order.
  #[must_use]
  pub fn entries(&self) -> &[ParticleDecalInsertion] {
    &self.entries
  }

  /// Appends one request or reports that capacity has been reached.
  pub fn try_insert(
    &mut self,
    insertion: ParticleDecalInsertion,
  ) -> Result<(), ParticleDecalStorageError> {
    if self.is_full() {
      return Err(ParticleDecalStorageError::CapacityExceeded {
        capacity: self.capacity,
      });
    }
    self.entries.push(insertion);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn insertion(sprite_id: u32, cell: [i32; 2]) -> ParticleDecalInsertion {
    ParticleDecalInsertion {
      placement: super::super::ParticleDecalPlacement {
        cell,
        pixel: [cell[0] * 32, cell[1] * 32],
      },
      sprite_id,
    }
  }

  #[test]
  fn store_starts_empty_with_explicit_capacity() {
    let store = ParticleDecalStore::new(2);

    assert_eq!(store.capacity(), 2);
    assert_eq!(store.len(), 0);
    assert!(store.is_empty());
    assert!(!store.is_full());
    assert!(store.entries().is_empty());
  }

  #[test]
  fn store_appends_insertion_order() {
    let first = insertion(7, [1, 2]);
    let second = insertion(9, [3, 4]);
    let mut store = ParticleDecalStore::new(2);

    assert_eq!(store.try_insert(first), Ok(()));
    assert_eq!(store.try_insert(second), Ok(()));
    assert_eq!(store.entries(), &[first, second]);
    assert_eq!(store.len(), 2);
    assert!(store.is_full());
  }

  #[test]
  fn store_preserves_duplicate_requests() {
    let insertion = insertion(7, [1, 2]);
    let mut store = ParticleDecalStore::new(2);

    assert_eq!(store.try_insert(insertion), Ok(()));
    assert_eq!(store.try_insert(insertion), Ok(()));
    assert_eq!(store.entries(), &[insertion, insertion]);
  }

  #[test]
  fn store_reports_capacity_without_dropping_existing_requests() {
    let retained = insertion(7, [1, 2]);
    let rejected = insertion(9, [3, 4]);
    let mut store = ParticleDecalStore::new(1);

    assert_eq!(store.try_insert(retained), Ok(()));
    assert_eq!(
      store.try_insert(rejected),
      Err(ParticleDecalStorageError::CapacityExceeded { capacity: 1 })
    );
    assert_eq!(store.entries(), &[retained]);
    assert!(store.is_full());
  }

  #[test]
  fn zero_capacity_rejects_without_allocating_requests() {
    let mut store = ParticleDecalStore::new(0);

    assert_eq!(
      store.try_insert(insertion(7, [1, 2])),
      Err(ParticleDecalStorageError::CapacityExceeded { capacity: 0 })
    );
    assert!(store.is_empty());
  }
}
