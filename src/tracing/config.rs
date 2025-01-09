// config.rs

//! Initialize `tracing` using a TOML file.
//!
//! This module requires the `tracing_config` feature. It is a wrapper around the [`tracing_config`] crate,
//! using Meadows's configuration-file search from [`crate::config`].
//!
//! For binary executables, use the [`try_init`] function. For example and test executables, use the [`init`]
//! function.

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

/// This structs holds the configuration used to initialize `tracing`.
#[derive(Debug)]
pub struct Config {
  /// The executable type.
  pub exec_type: ExecType,
  /// If `true`, debug mode is enabled.
  pub is_debug: bool,
  /// If `true`, a process-start message is logged.
  pub log_start: bool,
  /// The name to search `{}tracing.toml` with.
  pub name: OsString,
  /// One or more paths, separated by the system-dependent path separator. Each path may point to a file or
  /// directory.
  pub paths: Option<OsString>,
  /// If `true`, the path of the loaded log-configuration file is printed to `stdout`.
  pub print_path: bool,
  /// This hint is used to format the process-start message.
  pub text_width: usize,
}

impl Config {
  /// Returns a new `Config` with default settings suitable for the `exec_type`.
  ///
  /// | Field        | Default Value
  /// | :----------- | :------------
  /// | `is_debug`   | `true` if environment variable `tracing_config_debug` is set to to `"true"`, `otherwise `false`
  /// | `log_start`  | `true`
  /// | `name`       | Depends on `exec_type`
  /// | `paths`      | The value of the environment variable `tracing_config`, otherwise [`None`]
  /// | `print_path` | `true`
  /// | `text_width` | [`crate::TEXT_WIDTH`]
  ///
  /// # Safety
  ///
  /// The function reads the environment, which is not thread-safe. For detailed information, read the
  /// "Safety" section for [`env::set_var`].
  #[must_use]
  pub fn new(exec_type: ExecType) -> Config {
    use ExecType::*;

    let is_debug = get_env_debug().unwrap_or(false);
    let name = match exec_type {
      Binary => crate::process::inv_name(),
      Example => crate::process::name(),
      DocTest | UnitTest | IntegTest | BenchTest => crate::process::test_name(),
    };
    let paths = get_env();
    Config {
      exec_type,
      is_debug,
      log_start: true,
      name: name.clone(),
      paths,
      print_path: true,
      text_width: crate::TEXT_WIDTH,
    }
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
    process_note!("Loaded configuration file `{}` titled `{}`", file.display(), tracing_config.title);
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
/// use meadows::process::ExecType;
/// use meadows::process_error;
/// use meadows::tracing::config;
///
/// // This function can be called from every test. `init` may be called multiple times, but internally, it
/// // configures `tracing` exactly once per process
/// fn set_up() { config::init(&config::Config::new(ExecType::UnitTest)); }
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
/// use meadows::config::ConfigError;
/// use meadows::process_error;
/// use meadows::process;
/// use meadows::process::ExecType;
/// use meadows::tracing::config;
///
/// fn main() {
///   // Call `try_init` in `main`, as early as possible, hold the result
///   if let Err(err) = config::try_init(&config::Config::new(ExecType::Binary)) {
///     if !matches!(err.downcast_ref::<ConfigError>(), Some(ConfigError::FileNotFound)) {
///       process_error!("{:#}", err.context("Cannot initialize logging"));
///     }
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
    &config.name,
    config.paths.as_deref(),
    true, // `set_env_vars`
  )?;

  // Load configuration file

  let guard = init_file(config, &config_file.1)?;
  if config.log_start {
    info!("\n{}", start_message(config, &config_file.1));
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

  fn set_up() { init(&Config::new(ExecType::UnitTest)); }

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
