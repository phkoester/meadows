// meadows-log.rs

//! An example program that shows how to set up `tracing` using [`meadows::tracing::config::init`].

use std::io;
use std::io::Write;
use std::process;

use meadows::process::ExecType;
use meadows::process_error;
use meadows::tracing::config;
use meadows::tracing::config::Config;
use tracing::info;
use tracing::instrument;

// Functions ------------------------------------------------------------------------------------------------

#[instrument(ret)]
fn run() -> anyhow::Result<()> {
  let mut stdout = io::stdout();
  writeln!(stdout, "This is meadows-log")?;
  info!("A log message");
  writeln!(stdout, "Done.")?;
  Ok(())
}

// `main` ---------------------------------------------------------------------------------------------------

fn main() -> io::Result<()> {
  // Init logging

  config::init(&Config::new(ExecType::Example));

  // Run

  if let Err(err) = run() {
    process_error!("{err:#}")?;
    process::exit(1);
  }
  Ok(())
}

// EOF
