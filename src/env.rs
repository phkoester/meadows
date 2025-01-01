// env.rs

//! Environment-related utilities.

use std::env;

// Functions ------------------------------------------------------------------------------------------------

/// Prints the result of [`env::vars`] as key-value pairs to `stdout`.
pub fn dump() {
  for (name, val) in env::vars() {
    println!("{name}={val}");
  }
}

/// Prints the result of [`env::vars_os`] as key-value pairs to `stdout`.
pub fn dump_os() {
  for (name, val) in env::vars_os() {
    println!("{name:?}={val:?}");
  }
}

// EOF
