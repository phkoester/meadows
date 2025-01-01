// test_tracing_config.rs

#![allow(missing_docs)]

use std::thread;
use std::time::Duration;

use meadows::process::ExecType;
use meadows::tracing::config;
use meadows::tracing::config::Config;
use tracing::info;

fn set_up() { config::init_test(&Config::builder(ExecType::IntegTest).build()); }

#[test]
fn test_tracing_config_init_test_1() {
  set_up();
  for i in 0..4 {
    info!(i, "test_tracing_config_init_test_1");
    thread::sleep(Duration::from_millis(1));
  }
}

#[test]
fn test_tracing_config_init_test_2() {
  set_up();
  for i in 0..4 {
    info!(i, "test_tracing_config_init_test_2");
    thread::sleep(Duration::from_millis(1));
  }
}

// EOF
