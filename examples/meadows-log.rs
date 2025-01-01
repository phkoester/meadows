// main.rs

//! An example program that shows how to set up `tracing` using [`meadows::tracing::config::init`].

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
  println!("This is meadows-log");
  info!("A log message");
  println!("Done.");
  Ok(())
}

// `main` ---------------------------------------------------------------------------------------------------

fn main() {
  // Init logging

  config::init(&Config::builder(ExecType::Example).build());

  // Run

  if let Err(err) = run() {
    process_error!("{err:#}");
    process::exit(1);
  }
}

// EOF
