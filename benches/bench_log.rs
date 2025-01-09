// bench_log.rs

//! Benchmarks measuring [`log::info`] with [`tracing::info`].

#![feature(test)]

use std::sync::Once;

use meadows::process::ExecType;
use meadows::tracing::config;
use meadows::tracing::config::Config;

// Functions ------------------------------------------------------------------------------------------------

fn set_up() {
  static ONCE: Once = Once::new();
  ONCE.call_once(|| {
    // Initialize `tracing`
    config::init(&Config { log_start: false, ..Config::new(ExecType::BenchTest) });
    // Initialize `log`
    tracing_log::LogTracer::init().unwrap()
  });
}

// Tests ----------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
  extern crate test;

  use test::Bencher;

  use super::*;

  #[bench]
  fn bench_log_info(b: &mut Bencher) {
    set_up();
    b.iter(|| log::info!("message"))
  }

  #[bench]
  fn bench_tracing_info(b: &mut Bencher) {
    set_up();
    b.iter(|| tracing::info!("message"))
  }
}

// EOF
