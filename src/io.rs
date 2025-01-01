// io.rs

//! I/O-related utilities.

use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

/// Use this function to read lines from a file.
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

// EOF
