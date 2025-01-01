// test_std.rs

//! Integration tests related to the standard library.

use std::mem;
use std::num::NonZero;

#[test]
fn test_div_i32() {
  assert_eq!(1 / 2, 0);
  assert_eq!(3 / 2, 1);
}

#[test]
fn test_std_mem_size_of() {
  assert_eq!(mem::size_of::<i32>(), 4);
  assert_eq!(mem::size_of::<Option<i32>>(), 8);
  assert_eq!(mem::size_of::<Option<NonZero<i32>>>(), 4);

  let size_of_usize = mem::size_of::<usize>();

  assert_eq!(mem::size_of::<Box<i32>>(), size_of_usize);
  assert_eq!(mem::size_of::<Option<Box<i32>>>(), size_of_usize);
}

// EOF
