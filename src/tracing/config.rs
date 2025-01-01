// config.rs

//! Initialize `tracing` using a TOML file.
//!
//! This module requires the `tracing_config` feature. It is a wrapper around the [`tracing_config`] crate,
//! using Meadows's configuration-file search from [`crate::config`].
//!
//! For binary executables, use the [`try_init`] function. For example and test executables, use the [`init`]
//! function.
//!
//! Unless the values are explicitly set in [`Config`], the following environment variables are used:
//!
//! | Name                   | Description
//! | :--------------------- | :----------
//! | `tracing_config`       | One or more paths, separated by the system-dependent path separator. Each path may point to a file or a directory
//! | `tracing_config_debug` | If set to `true`, the debug mode is enabled

use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process;
use std::sync::OnceLock;

use tracing::info;
use tracing_config;
use tracing_config::config::ArcMutexGuard;
use tracing_config::TracingConfigError;

use crate::prelude::*;
use crate::process::ExecType;
use crate::process_error;
use crate::process_note;

// `Config` -------------------------------------------------------------------------------------------------

/// This struct holds the initialization configuration.
///
/// You create a [`Config`] using the [`Config::builder`] function and then pass it to either [`try_init`] or
/// [`init`].
#[derive(Debug)]
pub struct Config {
  exec_type: ExecType,
  is_debug: bool,
  log_start: bool,
  paths: Option<OsString>,
  print_path: bool,
  text_width: usize,
}

impl Config {
  /// Returns a builder that creates a [`Config`].
  #[must_use]
  pub fn builder(exec_type: ExecType) -> ConfigBuilder { ConfigBuilder::new(exec_type) }
}

// `ConfigBuilder` ------------------------------------------------------------------------------------------

/// A builder for [`Config`].
#[derive(Debug)]
pub struct ConfigBuilder {
  exec_type: ExecType,
  is_debug: Option<bool>,
  log_start: bool,
  paths: Option<OsString>,
  print_path: bool,
  text_width: usize,
}

impl ConfigBuilder {
  /// Creates a [`Config`], based on the current settings.
  ///
  /// If [`debug`](ConfigBuilder::debug) was not called with a specific value, an attempt is made to read the
  /// `tracing_config_debug` environment variable. If it is set to `"true"`, debug mode is enabled.
  ///
  /// If [`paths`](ConfigBuilder::paths) was not called with a specific value, an attempt is made to read the
  /// `tracing_config` environment variable. This may contain one or more paths, separated by the
  /// system-dependent path separator, where each path may point to a file or a directory.
  #[must_use]
  pub fn build(self) -> Config {
    let is_debug = self.is_debug.or_else(get_env_debug).unwrap_or(false);
    let paths = self.paths.or_else(get_env);
    Config {
      exec_type: self.exec_type,
      is_debug,
      log_start: self.log_start,
      paths,
      print_path: self.print_path,
      text_width: self.text_width,
    }
  }

  /// Sets the `debug` value.
  ///
  /// If `true`, debug mode is enabled. The default value is that of the environment variable
  /// `tracing_config_debug`, `false` otherwise.
  #[must_use]
  pub fn debug(mut self, val: bool) -> Self {
    self.is_debug = Some(val);
    self
  }

  /// Sets the `log_start` value.
  ///
  /// If `true`, a message is logged when the process starts. The default value is `false`.
  #[must_use]
  pub fn log_start(mut self, val: bool) -> Self {
    self.log_start = val;
    self
  }

  /// Creates a [`ConfigBuilder`] with the given `exec_type`.
  #[must_use]
  fn new(exec_type: ExecType) -> Self {
    Self {
      exec_type,
      is_debug: None,
      log_start: true,
      paths: None,
      print_path: true,
      text_width: crate::TEXT_WIDTH,
    }
  }

  /// Sets the `paths` value.
  ///
  /// This may contain one or more paths, separated by the system-dependent path separator, where each path
  /// may point to a file or a directory. The default value is that of the environment variable
  /// `tracing_config`, [`None`] otherwise.
  #[must_use]
  pub fn paths(mut self, val: OsString) -> Self {
    self.paths = Some(val);
    self
  }

  /// Sets the `print_path` value.
  ///
  /// If `true`, the path and the title of the loaded configuration file are printed to `stdout`. The default
  /// value is `true`.
  #[must_use]
  pub fn print_path(mut self, val: bool) -> Self {
    self.print_path = val;
    self
  }

  /// Sets the `text_width` value.
  ///
  /// This is used to format the start message if `log_start` is `true`. The default value is
  /// [`crate::TEXT_WIDTH`].
  ///
  /// # Panics
  ///
  /// Panics if `val` is `0`.
  #[must_use]
  pub fn text_width(mut self, val: usize) -> Self {
    assert!(val != 0);
    self.text_width = val;
    self
  }
}

// Functions ------------------------------------------------------------------------------------------------

fn get_env() -> Option<OsString> { env::var_os("tracing_config") }

fn get_env_debug() -> Option<bool> { env::var_os("tracing_config_debug").map(|val| val == "true") }

fn init_file(config: &Config, file: &Path) -> Result<ArcMutexGuard, TracingConfigError> {
  // Read configuration

  let tracing_config =
    tracing_config::config::read_config(file, tracing_config::config::RESOLVE_FROM_ENV_DEPTH)?;

  if config.print_path {
    process_note!(
            "Loaded configuration file `{}` titled \"{}\"",
            file.display(),
            tracing_config.title
        );
  }

  // Apply configuration

  tracing_config::config::init_config(config.is_debug, &tracing_config)
}

/// Initializes `tracing` for an example or test executable with the given configuration.
///
/// The function can be called multiple times, but internally, it configures `tracing` exactly once per
/// process. Because it stores the guard in a static variable, its result may usually be dismissed.
///
/// For detailed information about the usage of the environment and the file search, see
/// [`crate::config::find_config_file`].
///
/// # Panics
///
/// Panics if
///
/// - `config.exec_type` is [`ExecType::Binary`];
/// - the initialization fails.
///
/// # Examples
///
/// ```
/// use meadows::process;
/// use meadows::process_error;
/// use meadows::tracing::config;
///
/// // This function can be called from every test. `init` may be called multiple times, but internally, it
/// // configures `tracing` exactly once per process
/// fn set_up() { config::init(&config::Config::builder(process::ExecType::UnitTest).build()); }
///
/// #[test]
/// fn test_1() {
///   set_up();
///   // ...
/// }
///
/// #[test]
/// fn test_2() {
///   set_up();
///   // ...
/// }
/// ```
#[allow(clippy::test_attr_in_doctest)]
pub fn init(config: &Config) -> &'static ArcMutexGuard {
  static VAL: OnceLock<ArcMutexGuard> = OnceLock::new();
  VAL.get_or_init(|| {
    assert!(config.exec_type != ExecType::Binary);
    match try_init_impl(config) {
      Ok(guard) => guard,
      Err(err) => {
        process_error!("{:#}", err.context("Cannot initialize logging"));
        process::exit(2);
      }
    }
  })
}

fn start_message(config: &Config, config_path: &Path) -> String {
  let mut ret = String::new();

  // "Process started"

  let inv_name = crate::process::inv_name().to_string_lossy();
  let current_dir_str = match env::current_dir() {
    Ok(dir) => format!("{dir:?}"),
    Err(_) => String::from("-"),
  };
  let inv_path = crate::process::inv_path();
  let path = crate::process::path();

  ret.push_str(&format!(
        "\
Process started: {inv_name}

Log-configuration file: {config_path:?}

Current directory: {current_dir_str}
Invocation path  : {inv_path:?}
Path             : {path:?}
"
    ));

  // Arguments, if any

  let args: Vec<String> = env::args().skip(1).collect();
  if !args.is_empty() {
    ret.push_str("\nArguments:\n\n");
    for arg in args {
      let line = format!("- {arg:?}\n");
      ret.push_str(&line);
    }
  }

  ret.pop(); // Strip trailing '\n'
  ret.fence('#', config.text_width)
}

/// Initializes `tracing` for a binary executable with the given configuration.
///
/// This function should be called as early as possible on process startup. Its result contains a guard
/// that must be held as long as possible, preferably until the end of `main`. If an error is returned, that
/// error is typically printed, but the process should continue to run.
///
/// For detailed information about the usage of the environment and the file search, see
/// [`crate::config::find_config_file`].
///
/// # Errors
///
/// Returns
///
/// - [`crate::config::ConfigError`] if searching the configuration file fails;
/// - [`tracing_config::TracingConfigError`] if the underlying initialization of [`tracing_config`] fails.
///
/// # Panics
///
/// Panics if `config.exec_type` is not [`ExecType::Binary`].
///
/// # Examples
///
/// ```
/// use meadows::process_error;
/// use meadows::process;
/// use meadows::tracing::config;
///
/// fn main() {
///   // Call `try_init` in `main`, as early as possible, hold the result
///   let init_result = config::try_init(&config::Config::builder(process::ExecType::Binary).build());
///   if let Err(err) = init_result {
///     // Print the error, but continue running
///     process_error!("{:#}", err.context("Cannot initialize logging"));
///   }
///
///   // ...
/// }
#[allow(clippy::needless_doctest_main)]
pub fn try_init(config: &Config) -> anyhow::Result<ArcMutexGuard> {
  assert!(config.exec_type == ExecType::Binary);
  try_init_impl(config)
}

fn try_init_impl(config: &Config) -> anyhow::Result<ArcMutexGuard> {
  // Look for configuration file

  let config_file = crate::config::find_config_file(
    config.exec_type,
    "{}tracing.toml", // `file_name_pattern`
    config.is_debug,
    config.paths.as_deref(),
    true, // `set_env_vars`
  )?;

  // Load configuration file

  let guard = init_file(config, &config_file)?;
  if config.log_start {
    info!("\n{}", start_message(config, &config_file));
  }
  Ok(guard)
}

// Tests ====================================================================================================

#[cfg(test)]
mod tests {
  use std::thread;
  use std::time::Duration;

  use tracing::info;

  use super::*;

  fn set_up() { init(&Config::builder(ExecType::UnitTest).build()); }

  // Functions ----------------------------------------------------------------------------------------------

  #[test]
  fn test_init_1() {
    set_up();
    for i in 0..4 {
      info!(i, "test_init_1");
      thread::sleep(Duration::from_millis(1));
    }
  }

  #[test]
  fn test_init_2() {
    set_up();
    for i in 0..4 {
      info!(i, "test_init_2");
      thread::sleep(Duration::from_millis(1));
    }
  }
}

// EOF
