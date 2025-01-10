// test_tracing_config.rs

//! Integration tests for [`meadows::tracing::config::init`].

use std::thread;
use std::time::Duration;

use meadows::process::ExecType;
use meadows::tracing::config;
use meadows::tracing::config::Config;
use tracing::info;

fn set_up() { config::init(&Config::new(ExecType::IntegTest)); }

#[cfg_attr(miri, ignore)]
#[test]
fn test_tracing_config_init_1() {
  set_up();
  for i in 0..4 {
    info!(i, "test_tracing_config_init_1");
    thread::sleep(Duration::from_millis(1));
  }
}

#[cfg_attr(miri, ignore)]
#[test]
fn test_tracing_config_init_2() {
  set_up();
  for i in 0..4 {
    info!(i, "test_tracing_config_init_2");
    thread::sleep(Duration::from_millis(1));
  }
}

// EOF
