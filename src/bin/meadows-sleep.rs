// meadows-sleep.rs

//! A program that sleeps for a given amount of seconds.
//! 
//! Usage: `meadows-sleep [N]`
//! 
//! | Argument | Description
//! | :------- | :----------
//! | `N`      | Number of seconds to sleep. Default: 5

use std::env;
use std::thread;
use std::time::Duration;

fn main() {
  let mut n = 5_u64;

  if let Some(arg) = env::args().skip(1).next() {
    n = arg.parse().expect("Invalid number");
  }

  if n > 0 {
    println!("Sleeping {n} seconds ...");
    thread::sleep(Duration::from_secs(n))
  }
}

// EOF
