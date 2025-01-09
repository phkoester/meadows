// meadows-bare.rs

//! A bare program that consists of a single [`writeln`] statement only.

use std::io;
use std::io::Write;

use anyhow::Ok;

fn main() -> anyhow::Result<()> {
  writeln!(io::stdout(), "This is meadows-bare")?;
  Ok(())
}

// EOF
