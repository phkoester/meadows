// meadows-bare.rs

//! A bare program that consists of a single [`writeln`] statement only.

use std::io;
use std::io::prelude::*;

fn main() -> anyhow::Result<()> {
  writeln!(io::stdout(), "This is meadows-bare")?;
  Ok(())
}

// EOF
