// meadows-std.rs

//! A minimal program using [`std`].

use std::io;
use std::io::prelude::*;

fn main() -> io::Result<()> {
  writeln!(io::stdout(), "This is meadows-std")?;
  Ok(())
}

// EOF
