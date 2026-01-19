// bench_std.rs

//! Benchmarks measuring various functions from the standard library.

#![feature(test)]

// Functions ------------------------------------------------------------------------------------------------

#[inline]
fn use_string_from(val: &str) -> String { String::from(val) }

#[inline]
fn use_str_to_owned(val: &str) -> String { val.to_owned() }

#[inline]
fn use_str_to_string(val: &str) -> String { val.to_string() }

// Tests ----------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
  extern crate test;

  use test::Bencher;

  use super::*;

  #[bench]
  fn bench_str_to_owned(b: &mut Bencher) { b.iter(|| use_str_to_owned("hello")); }

  #[bench]
  fn bench_str_to_string(b: &mut Bencher) { b.iter(|| use_str_to_string("hello")); }

  #[bench]
  fn bench_string_from(b: &mut Bencher) { b.iter(|| use_string_from("hello")); }
}

// EOF
