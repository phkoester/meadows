// meadows-env.rs

//! A program that dumps all environment variables.

use std::io;

use meadows::env;

fn main() -> io::Result<()> {
  env::dump()?;
  Ok(())
}

// EOF
