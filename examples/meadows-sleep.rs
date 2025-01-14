// meadows-sleep.rs

//! An example program that sleeps for a given amount of seconds.

use std::io;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;

use clap::Parser;

#[derive(Parser)]
#[command(about = "Sleeps for a given number of seconds", version)]
struct Args {
  #[arg(default_value = "5", help = "Number of seconds to sleep")]
  n: u64,
}

fn main() -> anyhow::Result<()> {
  let args = Args::parse();

  let n = args.n;

  if n > 0 {
    let mut stdout = io::stdout();
    let noun = if n == 1 { "second" } else { "seconds" };
    writeln!(stdout, "Sleeping {n} {noun} ...")?;
    thread::sleep(Duration::from_secs(n));
  }

  Ok(())
}

// EOF
