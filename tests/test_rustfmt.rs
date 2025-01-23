// test_rustfmt.rs

#![allow(missing_docs, clippy::must_use_candidate, clippy::needless_bool)]

//! Code snippets testing the current Rustfmt configuration.
//!
//! The code in this file is not supposed to run or actually test anything except how it is formatted by
//! Rustfmt.

// imports_granularity

use std::fmt;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

// blank_lines_upper_bound, fn_single_line

pub fn a() {}

pub fn b() { a(); }

pub struct MyStruct {
  pub n: i32,
}

impl MyStruct {
  pub fn method(&self) -> &'static str { "result" }
}

// brace_style

pub fn lorem<T>(_: T)
where
  T: Add + Div + Mul + Sub, {
}

pub struct Wrapper<T>(T);

impl<T> fmt::Display for Wrapper<Vec<T>>
where
  T: fmt::Display,
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "[...]") }
}

pub fn c() -> bool {
  if 3 > 2 {
    true
  } else {
    false
  }
}

pub fn some_function() {
  // array_width

  let _ = [1, 2, 3];

  let _ = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1,
  ];

  // binop_separator, max_width

  let _ = 1_000_000
    + 1_000_000
    + 1_000_000
    + 1_000_000
    + 1_000_000
    + 1_000_000
    + 1_000_000
    + 1_000_000
    + 1_000_000;

  #[rustfmt::skip]
  let _ = 1_000_000 + 1_000_000 + 1_000_000 + 1_000_000 + 1_000_000 + 1_000_000 + 1_000_000 + 1_000_000 + 1_000_000;
}

// comment_width, wrap_comments

// eueiueiwe weiwe wiuewi uewi ewiuewiuewiuew ieuwi eu wieuwi euwi euwi eiwu eiweuwieu wieuwieuw ieu wieuwi
// eu wiewiue EOF

// condense_wildcard_suffices

fn _d() { let (_a, ..) = (1, 2, 3); }

// format_code_in_doc_comments

/// Adds one to the number given.
///
/// # Examples
///
/// ```
/// let five = 5;
///
/// assert_eq!(6, add_one(5));
/// ```
pub fn add_one(x: i32) -> i32 { x + 1 }

// hex_literal_case

pub const I: i32 = 0xabcd;

// skip_macro_invocations

pub fn g() {
  let _ = format!("{}{}{}{}{}{}{}{}", 1_000_000, 1_000_000, 1_000_000, 1_000_000, 1_000_000, 1_000_000, 1_000_000, 1_000_000);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Foo {}

// EOF
