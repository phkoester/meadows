// io.rs

//! I/O-related utilities.

use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

/// Reads lines from a file.
///
/// # Errors
///
/// See [`File::open`].
pub fn read_lines<P>(path: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
  P: AsRef<Path>, {
  let file = File::open(path)?;
  Ok(io::BufReader::new(file).lines())
}

/// Returns a configured ANSI-aware stream for `stderr`.
///
/// See [`anstream::stderr`].
#[inline]
#[must_use]
pub fn stderr() -> anstream::Stderr { anstream::stderr() }

/// Returns a configured ANSI-aware stream for `stdout`.
///
/// See [`anstream::stdout`].
#[inline]
#[must_use]
pub fn stdout() -> anstream::Stdout { anstream::stdout() }

// EOF
