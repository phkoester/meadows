// meadows-sleep.rs

//! A program that sleeps for a given amount of seconds.
//! 
//! Usage: `meadows-sleep [N]`
//! 
//! | Argument | Description
//! | :------- | :----------
//! | `N`      | Number of seconds to sleep. Default: 5

use std::env;
use std::io;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()>{
  let mut n = 5_u64; // Default: 5 seconds

  if let Some(arg) = env::args().nth(1) {
    n = arg.parse()?;
  }

  if n > 0 {
    let mut stdout = io::stdout();
    writeln!(stdout, "Sleeping {n} seconds ...")?;
    thread::sleep(Duration::from_secs(n));
  }

  Ok(())
}

// EOF
