// meadows-env.rs

//! A program that dumps all environment variables.

use meadows::env;

use std::io;
use std::io::prelude::*;

fn main() -> anyhow::Result<()> {
  writeln!(io::stdout().lock(), "This is meadows-env")?;
  env::dump()?;
  Ok(())
}

// EOF
