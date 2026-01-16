// uvec.rs

//! A [`Uvec`] is a vector containing unique elements.
//!
//! Internally, this vector is backed by a [`HashSet`] that guards the uniqueness of the elements. For each
//! element of type `V` in the vector, a key of type `K` needs to be generated. This is done by a
//! key-generating function that must be supplied when creating a [`Uvec`]. This function returns an
//! `Option<K>`---if the result is [`None`], no key is generated and the value is not inserted into the
//! vector.
//!
//! If the types `K` and `V` are identical, a [`Uvec`] may be created using the [`new`] function. In this
//! case, the key-generating function is automatically supplied and simply clones the values so they can be
//! used as keys.
//!
//! ```
//! use meadows::collections::Uvec;
//!
//! let mut uvec = Uvec::new();
//! assert_eq!(uvec.push("hello"), true);
//! assert_eq!(uvec.push("hello"), false); // Duplicate value: inserting fails
//! ```
//!
//! Otherwise, the key-generating function must be supplied. In the following example, only existing and
//! equivalent paths can be inserted into the [`Uvec`]:
//!
//! ```
//! # fn run() {
//! use std::path::PathBuf;
//!
//! use meadows::collections::Uvec;
//!
//! // If canonicalizing fails, no key is generated
//! let mut uvec = Uvec::with_key(&|val: &PathBuf| dunce::canonicalize(val).ok());
//! assert_eq!(uvec.push(PathBuf::from("beetlejuice")), false); // Path does not exist: inserting fails
//! assert_eq!(uvec.push(PathBuf::from(".")), true);
//! assert_eq!(uvec.push(PathBuf::from(".")), false); // Duplicate value: inserting fails
//! # }
//! # #[cfg(not(miri))]
//! # run();
//! ```
//!
//! [`new`]: Uvec::new

use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::Index;
use std::slice::SliceIndex;

// `Uvec` ---------------------------------------------------------------------------------------------------

/// A [`Uvec`] behaves very much like a [`Vec`], but it can only contain unique elements.
///
/// For some basic examples, see [the module documentation](crate::collections::uvec).
#[derive(Clone)]
pub struct Uvec<'a, K, V> {
  set: HashSet<K>,
  vec: Vec<V>,
  key: &'a dyn Fn(&V) -> Option<K>,
}

impl<'a, K, V> Uvec<'a, K, V>
where
  K: Eq + Hash,
{
  /// Extracts a slice containing the entire vector.
  #[inline]
  #[must_use]
  pub fn as_slice(&self) -> &[V] { self.vec.as_slice() }

  /// Clears the vector, removing all elements.
  pub fn clear(&mut self) {
    self.set.clear();
    self.vec.clear();
  }

  /// Inserts a value at position `index` within the vector, shifting all elements after it to the right.
  ///
  /// Returns whether the operation succeeds.
  ///
  /// # Panics
  ///
  /// Panics if `index` is out of bounds.
  ///
  /// # Examples
  ///
  /// ```
  /// use meadows::collections::Uvec;
  ///
  /// let mut uvec = Uvec::new();
  /// uvec.insert(0, 3);
  /// uvec.insert(0, 2);
  /// uvec.insert(0, 1);
  /// assert_eq!(uvec, Uvec::from([1, 2, 3]));
  /// ```
  pub fn insert(&mut self, index: usize, val: V) -> bool {
    let len = self.len();
    assert!(index <= len, "`index` ({index}) > `len` ({len})");

    let key = (self.key)(&val);
    if let Some(key) = key && self.set.insert(key) {
      self.vec.insert(index, val);
      return true;
    }
    false
  }

  /// Checks if the vector contains no elements.
  #[inline]
  #[must_use]
  pub fn is_empty(&self) -> bool { self.vec.is_empty() }

  /// Returns the number of elements in the vector, also referred to as its "length".
  #[inline]
  #[must_use]
  pub fn len(&self) -> usize { self.vec.len() }

  /// Removes the last element from a vector and returns it, or [`None`] if it is empty.
  ///
  /// # Examples
  ///
  /// ```
  /// use meadows::collections::Uvec;
  ///
  /// let mut uvec = Uvec::from([1, 2, 3, 2, 1]);
  /// assert_eq!(uvec.pop(), Some(3));
  /// assert_eq!(uvec.pop(), Some(2));
  /// assert_eq!(uvec.pop(), Some(1));
  /// assert_eq!(uvec.pop(), None);
  /// ```
  pub fn pop(&mut self) -> Option<V> {
    if let Some(val) = self.vec.pop() {
      self.remove_from_set(&val);
      Some(val)
    } else {
      None
    }
  }

  /// Appends a value to the back of the vector.
  ///
  /// Returns whether the operation succeeds.
  pub fn push(&mut self, val: V) -> bool {
    let key = (self.key)(&val);
    if let Some(key) = key && self.set.insert(key) {
      self.vec.push(val);
      return true;
    }
    false
  }

  /// Removes and returns the element at position `index` within the vector, shifting all elements after it
  /// to the left.
  ///
  /// # Panics
  ///
  /// Panics if `index` is out of bounds.
  pub fn remove(&mut self, index: usize) -> V {
    let ret = self.vec.remove(index);
    self.remove_from_set(&ret);
    ret
  }

  fn remove_from_set(&mut self, val: &V) {
    let key = (self.key)(val);
    let result = self.set.remove(&key.unwrap());
    debug_assert!(result);
  }

  /// Creates a new [`Uvec`] with a key-generating function.
  ///
  /// # Examples
  ///
  /// ```
  /// # fn run() {
  /// use std::path::PathBuf;
  ///
  /// use meadows::collections::Uvec;
  ///
  /// // If canonicalizing fails, no key is generated
  /// let mut uvec = Uvec::with_key(&|val: &PathBuf| dunce::canonicalize(val).ok());
  /// assert_eq!(uvec.push(PathBuf::from("beetlejuice")), false); // Path does not exist: inserting fails
  /// assert_eq!(uvec.push(PathBuf::from(".")), true);
  /// assert_eq!(uvec.push(PathBuf::from(".")), false); // Duplicate value: inserting fails
  /// # }
  /// # #[cfg(not(miri))]
  /// # run();
  /// ```
  #[inline]
  #[must_use]
  pub fn with_key(key: &'a dyn Fn(&V) -> Option<K>) -> Self {
    Self { set: HashSet::new(), vec: Vec::new(), key }
  }
}

/// If the types `K` and `V` are identical, a [`Uvec`] may be created using the [`new`](Uvec::new) function.
#[allow(clippy::mismatching_type_param_order)]
impl<V> Uvec<'_, V, V>
where
  V: Clone,
{
  /// Creates a new [`Uvec`], automatically supplying a key-generating function.
  ///
  /// # Examples
  ///
  /// ```
  /// use meadows::collections::Uvec;
  ///
  /// let mut uvec = Uvec::new();
  /// uvec.push(42);
  /// ```
  ///
  /// This is equivalent to
  ///
  /// ```
  /// use meadows::collections::Uvec;
  ///
  /// let mut uvec = Uvec::with_key(&|val: &i32| Some(val.clone()));
  /// uvec.push(42);
  /// ```
  #[inline]
  #[must_use]
  pub fn new() -> Self { Self { set: HashSet::new(), vec: Vec::new(), key: &|val: &V| Some(val.clone()) } }
}

impl<K, V> AsRef<[V]> for Uvec<'_, K, V> {
  #[inline]
  fn as_ref(&self) -> &[V] { &self.vec }
}

impl<K, V> AsRef<Vec<V>> for Uvec<'_, K, V> {
  #[inline]
  fn as_ref(&self) -> &Vec<V> { &self.vec }
}

impl<K, V> Debug for Uvec<'_, K, V>
where
  V: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { self.vec.fmt(f) }
}

/// A [`Uvec`] implements [`Default`] if the types `K` and `V` are identical.
#[allow(clippy::mismatching_type_param_order)]
impl<V> Default for Uvec<'_, V, V>
where
  V: Clone,
{
  #[inline]
  fn default() -> Self { Self::new() }
}

impl<K, V> Deref for Uvec<'_, K, V>
where
  K: Eq + Hash,
{
  type Target = [V];

  #[inline]
  fn deref(&self) -> &Self::Target { self.as_slice() }
}

impl<K, V> Eq for Uvec<'_, K, V> where V: Eq {}

impl<K, V> Extend<V> for Uvec<'_, K, V>
where
  K: Eq + Hash,
{
  #[inline]
  fn extend<I: IntoIterator<Item = V>>(&mut self, iter: I) {
    for item in iter {
      self.push(item);
    }
  }
}

#[allow(clippy::mismatching_type_param_order)]
impl<V, const N: usize> From<[V; N]> for Uvec<'_, V, V>
where
  V: Clone + Eq + Hash,
{
  fn from(s: [V; N]) -> Self {
    let mut ret = Uvec::new();
    for item in s {
      ret.push(item);
    }
    ret
  }
}

/// Collects an iterator into a [`Uvec`], commonly called via [`Iterator::collect`].
#[allow(clippy::mismatching_type_param_order)]
impl<V> FromIterator<V> for Uvec<'_, V, V>
where
  V: Clone + Eq + Hash,
{
  fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
    let mut ret = Uvec::new();
    for item in iter {
      ret.push(item);
    }
    ret
  }
}

/// [`Uvec`] supports indexing just like [`Vec`] does.
impl<K, V, I> Index<I> for Uvec<'_, K, V>
where
  I: SliceIndex<[V]>,
{
  type Output = I::Output;

  #[inline]
  fn index(&self, index: I) -> &Self::Output { self.vec.index(index) }
}

// `IntoIterator` for `Uvec`
impl<K, V> IntoIterator for Uvec<'_, K, V>
where
  K: Eq + Hash,
{
  type IntoIter = <Vec<V> as IntoIterator>::IntoIter;
  type Item = <Vec<V> as IntoIterator>::Item;

  #[inline]
  fn into_iter(self) -> Self::IntoIter { self.vec.into_iter() }
}

// `IntoIterator` for `&Uvec`
#[allow(clippy::into_iter_without_iter)]
impl<'a, K, V> IntoIterator for &'a Uvec<'a, K, V> {
  type IntoIter = <&'a Vec<V> as IntoIterator>::IntoIter;
  type Item = <&'a Vec<V> as IntoIterator>::Item;

  #[inline]
  fn into_iter(self) -> Self::IntoIter { self.vec.iter() }
}

impl<K, V> Ord for Uvec<'_, K, V>
where
  V: Ord,
{
  #[inline]
  fn cmp(&self, rhs: &Self) -> Ordering { self.vec.cmp(&rhs.vec) }
}

impl<'a, K, V> PartialEq<Uvec<'a, K, V>> for Uvec<'a, K, V>
where
  V: PartialEq,
{
  #[inline]
  fn eq(&self, rhs: &Uvec<'a, K, V>) -> bool { self.vec.eq(&rhs.vec) }
}

impl<'a, K, V> PartialOrd<Uvec<'a, K, V>> for Uvec<'a, K, V>
where
  V: PartialOrd,
{
  #[inline]
  fn partial_cmp(&self, rhs: &Uvec<'a, K, V>) -> Option<Ordering> { self.vec.partial_cmp(&rhs.vec) }
}

// Tests ====================================================================================================

#[cfg(test)]
mod tests {
  use std::env;
  use std::path::PathBuf;

  use super::*;

  // `Uvec` -------------------------------------------------------------------------------------------------

  #[test]
  fn test_uvec_clear() {
    let mut uvec = Uvec::from([1, 2, 3, 2, 1]);
    assert_eq!(uvec.len(), 3);
    uvec.clear();
    assert_eq!(uvec.len(), 0);
    assert_eq!(uvec.set.len(), 0);
    assert_eq!(uvec.vec.len(), 0);
  }

  #[test]
  fn test_uvec_is_empty() {
    let mut uvec = Uvec::from([1, 2, 3, 2, 1]);
    assert!(!uvec.is_empty());
    uvec.clear();
    assert!(uvec.is_empty());
  }

  #[test]
  fn test_uvec_new() {
    let mut uvec = Uvec::new();
    assert!(uvec.push(1));
    assert!(uvec.push(2));
    assert!(uvec.push(3));
    assert!(!uvec.push(2));
    assert!(!uvec.push(1));

    assert_eq!(uvec.set, HashSet::from([1, 2, 3]));
    assert_eq!(uvec.vec, vec![1, 2, 3]);
  }

  #[test]
  fn test_uvec_with_key_to_string() {
    let mut uvec = Uvec::with_key(&|val: &i32| Some(val.to_string()));
    assert!(uvec.push(1));
    assert!(uvec.push(2));
    assert!(uvec.push(3));
    assert!(!uvec.push(2));
    assert!(!uvec.push(1));

    assert_eq!(uvec.set, HashSet::from(["1".to_string(), "2".to_string(), "3".to_string()]));
    assert_eq!(uvec.vec, vec![1, 2, 3]);
  }

  #[cfg_attr(miri, ignore)]
  #[test]
  fn test_uvec_with_key_canonicalize() {
    let current_dir = env::current_dir().unwrap();
    let dir_name = current_dir.file_name().unwrap().to_string_lossy();

    // Use `unwrap` in the key function to ensure canonicalizing succeeds
    let mut uvec = Uvec::with_key(&|val: &PathBuf| Some(dunce::canonicalize(val).unwrap()));
    assert!(uvec.push(PathBuf::from(".")));
    // `../dir_name` must be equivalent to `.`
    assert!(!uvec.push(PathBuf::from(format!("../{}", dir_name))));
    assert!(uvec.push(PathBuf::from("..")));

    assert_eq!(uvec.set.len(), 2);
    assert_eq!(uvec.vec, vec![PathBuf::from("."), PathBuf::from("..")]);
  }

  #[test]
  fn test_as_ref_slice_for_uvec() {
    let uvec = Uvec::from([1, 2, 3, 2, 1]);
    let other: &[i32] = uvec.as_ref();
    assert_eq!(other, &[1, 2, 3]);
  }

  #[test]
  fn test_as_ref_vec_for_uvec() {
    let uvec = Uvec::from([1, 2, 3, 2, 1]);
    let other: &Vec<i32> = uvec.as_ref();
    assert_eq!(other, &[1, 2, 3]);
  }

  #[test]
  fn test_debug_for_uvec() {
    let uvec = Uvec::from([1, 2, 3, 2, 1]);
    assert_eq!(format!("{:?}", uvec), "[1, 2, 3]");
  }

  #[test]
  fn test_deref_for_uvec() {
    let uvec = Uvec::from([1, 2, 3, 2, 1]);
    let other: &[i32] = &uvec;
    assert_eq!(other, &[1, 2, 3]);
  }

  #[test]
  fn test_from_iter_for_uvec() {
    let uvec: Uvec<_, _> = [1, 2, 3, 2, 1].into_iter().collect();
    assert_eq!(uvec.vec, vec![1, 2, 3]);
  }

  #[test]
  fn test_index_for_uvec() {
    let uvec = Uvec::from([1, 2, 3, 2, 1]);
    assert_eq!(uvec[0], 1);
    assert_eq!(uvec[0..3], [1, 2, 3]);
  }

  #[allow(clippy::explicit_counter_loop)]
  #[test]
  fn test_into_iter_for_uvec() {
    let uvec = Uvec::from([1, 2, 3, 2, 1]);
    assert_eq!(uvec.len(), 3);
    let mut n = 0;
    for item in uvec {
      match n {
        0 => assert_eq!(item, 1),
        1 => assert_eq!(item, 2),
        _ => assert_eq!(item, 3),
      }
      n += 1;
    }
  }

  #[allow(clippy::explicit_counter_loop)]
  #[test]
  fn test_into_iter_for_ref_uvec() {
    let uvec = Uvec::from([1, 2, 3, 2, 1]);
    assert_eq!(uvec.len(), 3);
    let mut n = 0;
    for item in &uvec {
      match n {
        0 => assert_eq!(item, &1),
        1 => assert_eq!(item, &2),
        _ => assert_eq!(item, &3),
      }
      n += 1;
    }
  }
}

// EOF
