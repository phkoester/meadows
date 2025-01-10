// bench_std.rs

//! Benchmarks measuring various functions from the standard library.

#![feature(test)]

// Functions ------------------------------------------------------------------------------------------------

#[inline]
fn use_string_from(v: &str) -> String { String::from(v) }

#[inline]
fn use_str_to_owned(v: &str) -> String { v.to_owned() }

#[inline]
fn use_str_to_string(v: &str) -> String { v.to_string() }

// Tests ----------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
  extern crate test;

  use test::Bencher;

  use super::*;

  #[bench]
  fn bench_str_to_owned(b: &mut Bencher) { b.iter(|| format!("{}", use_str_to_owned("hello"))); }

  #[bench]
  fn bench_str_to_string(b: &mut Bencher) { b.iter(|| format!("{}", use_str_to_string("hello"))); }

  #[bench]
  fn bench_string_from(b: &mut Bencher) { b.iter(|| format!("{}", use_string_from("hello"))); }
}

// EOF
