// meadows-env.rs

//! A program that dumps all environment variables.

use meadows::env;

fn main() -> anyhow::Result<()> {
  env::dump()?;
  Ok(())
}

// EOF
