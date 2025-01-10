// vec.rs

//! Utilities related to [`Vec`].

use std::collections::HashSet;
use std::hash::Hash;

// `VecExt` -------------------------------------------------------------------------------------------------

/// An extension trait for vectors.
///
/// This is included in the crate's [prelude](crate::prelude).
pub trait VecExt<T> {
  /// Removes *all* duplicates from a vector based on a key function, retaining the order of the elements.
  ///
  /// This is different from the vector's `dedup` methods, which only remove *consecutive* duplicates. The
  /// key function returns an `Option<K>`---if the result is [`None`], no key is generated and the element is
  /// not retained in the vector.
  ///
  /// # Examples
  ///
  /// ```
  /// use meadows::prelude::*;
  ///
  /// let mut vec = vec![1, 2, 3, 2, 1];
  /// vec.dedup_all_by_key(|&x| Some(x));
  /// assert_eq!(vec, vec![1, 2, 3])
  /// ```
  fn dedup_all_by_key<K, F>(&mut self, f: F)
  where
    K: Eq + Hash,
    F: FnMut(&T) -> Option<K>;
}

impl<T> VecExt<T> for Vec<T> {
  fn dedup_all_by_key<K, F>(&mut self, mut f: F)
  where
    K: Eq + Hash,
    F: FnMut(&T) -> Option<K>, {
    let mut set = HashSet::new();
    self.retain(|val| if let Some(key) = f(val) { set.insert(key) } else { false });
  }
}

// EOF
