// main.rs

#![allow(missing_docs)]

use std::process;

use meadows::process::ExecType;
use meadows::process_error;
use meadows::tracing::config;
use meadows::tracing::config::Config;
use tracing::instrument;

// Functions ------------------------------------------------------------------------------------------------

#[instrument(ret)]
fn run() -> anyhow::Result<()> {
  println!("This is meadows-log");
  println!("Done.");
  Ok(())
}

// `main` ---------------------------------------------------------------------------------------------------

fn main() {
  // Init logging

  let init_result = config::try_init(&Config::builder(ExecType::Standard).build());
  if let Err(err) = init_result {
    process_error!("{:#}", err.context("Cannot initialize logging"));
  }

  // Run

  if let Err(err) = run() {
    process_error!("{err:#}");
    process::exit(1);
  }
}

// EOF
