// config.rs

//! Initialize `tracing` using a TOML file.
//!
//! It is a wrapper around the [`tracing_config`] crate, using Meadows's configuration-file search from
//! [`crate::config`].
//!
//! For binary executables, use the [`try_init`] function. For example and test executables, use the [`init`]
//! function.

use std::ffi::OsString;
use std::fmt::Write;
use std::io;
use std::path::Path;
use std::sync::OnceLock;

use thiserror::Error as ThisError;
use tracing::info;
use tracing_config;
use tracing_config::config::ArcMutexGuard;
use tracing_config::TracingConfigError;

use crate::config::FindError;
use crate::prelude::*;
use crate::process::ExecType;
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
  /// Returns a new [`Config`] with default settings suitable for the `exec_type`.
  ///
  /// | Field        | Default Value
  /// | :----------- | :------------
  /// | `is_debug`   | `true` if environment variable `tracing_config_debug` is set to to `true`
  /// | `log_start`  | `true`
  /// | `name`       | Depends on `exec_type`
  /// | `paths`      | The value of the environment variable `tracing_config`, otherwise [`None`]
  /// | `print_path` | `true`
  /// | `text_width` | [`crate::TEXT_WIDTH`]
  #[must_use]
  pub fn new(exec_type: ExecType) -> Config {
    use ExecType::*;

    let is_debug = get_env_debug().unwrap_or(false);
    let name = match exec_type {
      Binary => crate::env::inv_name(),
      Example => crate::env::name(),
      DocTest | UnitTest | IntegTest | BenchTest => crate::env::test_name(),
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

// `InitError` ----------------------------------------------------------------------------------------------

/// Error type for [`init`]  and [`try_init`].
#[derive(Debug, ThisError)]
pub enum InitError {
  /// [`FindError`]
  #[error("Cannot find configuration file")]
  Find(#[from] FindError),
  /// [`io::Error`].
  #[error("I/O error")]
  Io(#[from] io::Error),
  /// [`TracingConfigError`].
  #[error("Cannot configure `tracing`")]
  TracingConfig(#[from] TracingConfigError),
}

impl InitError {
  /// Returns `true` if the error should be printed.
  #[must_use]
  pub fn should_print(&self) -> bool {
    match self {
      InitError::Find(err) => err.should_print(),
      _ => true,
    }
  }
}

// Functions ------------------------------------------------------------------------------------------------

fn get_env() -> Option<OsString> { crate::env::get("tracing_config") }

fn get_env_debug() -> Option<bool> { crate::env::get("tracing_config_debug").map(|val| val == "true") }

fn init_file(config: &Config, file: &Path) -> Result<ArcMutexGuard, InitError> {
  // Read configuration

  let tracing_config =
    tracing_config::config::read_config(file, tracing_config::config::RESOLVE_FROM_ENV_DEPTH)?;

  if config.print_path {
    process_note!(crate::io::stdout(), "Loaded configuration file `{}` titled {:?}", file.display(), tracing_config.title)?;
  }

  // Apply configuration

  match tracing_config::config::init_config(config.is_debug, &tracing_config) {
    Ok(guard) => {
      if config.log_start {
        info!("\n{}", start_message(config, file));
      }
      Ok(guard)
    }
    Err(err) => Err(err.into()),
  }
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
      Err(err) => panic!("{:?}", anyhow::Error::from(err).context("Cannot initialize logging")),
    }
  })
}

fn start_message(config: &Config, config_path: &Path) -> String {
  let mut ret = String::new();

  // `Process started`

  let inv_name = crate::env::inv_name().to_string_lossy();
  let current_dir_str = match std::env::current_dir() {
    Ok(dir) => format!("{dir:?}"),
    Err(_) => String::from("N/A"),
  };
  let inv_path = crate::env::inv_path();
  let path = crate::env::path();

  write!(ret, "\
Process started: {inv_name}

Log-configuration file: {config_path:?}

Current directory: {current_dir_str}
Invocation path  : {inv_path:?}
Path             : {path:?}
").unwrap();

  // Arguments, if any

  let args: Vec<String> = std::env::args().skip(1).collect();
  if !args.is_empty() {
    ret.push_str("\nArguments:\n\n");
    for arg in args {
      writeln!(ret, "- {arg:?}").unwrap();
    }
  }

  ret.pop(); // Strip trailing '\n'
  ret.fence('#', config.text_width)
}

/// Initializes `tracing` for a binary executable with the given configuration.
///
/// This function should be called as early as possible on process startup. Its result contains a guard
/// that must be held as long as possible, preferably until the end of `main`. If an error is returned, that
/// error should be printed if [`InitError::should_print`]  returns `true`, but the process should continue
/// to run.
///
/// For detailed information about the usage of the environment and the file search, see
/// [`crate::config::find_config_file`].
///
/// # Errors
///
/// Returns [`Err`] with
///
/// - [`InitError::Find`] if a [`FindError`] occurs
/// - [`InitError::Io`] if an [`io::Error`] occurs
/// - [`InitError::TracingConfig`] if a [`TracingConfigError`] occurs
///
/// # Panics
///
/// Panics if `config.exec_type` is not [`ExecType::Binary`].
///
/// # Examples
///
/// ```
/// use meadows::io;
/// use meadows::process_error;
/// use meadows::process;
/// use meadows::process::ExecType;
/// use meadows::tracing::config;
///
/// fn main() -> anyhow::Result<()> {
/// # fn run() -> anyhow::Result<()> {
///   // Call `try_init` in `main`, as early as possible, hold the result
///   let init_result = config::try_init(&config::Config::new(ExecType::Binary));
///   if let Err(err) = init_result {
///     if err.should_print() {
///       let mut stderr = io::stderr().lock();
///       process_error!(stderr, "{:#}", anyhow::Error::from(err).context("Cannot initialize logging"))?;
///     }
///   }
/// #   Ok(())
/// # }
/// # #[cfg(not(miri))]
/// # run();
///
///   // ...
///
///   Ok(())
/// }
#[allow(clippy::needless_doctest_main)]
pub fn try_init(config: &Config) -> Result<ArcMutexGuard, InitError> {
  assert!(config.exec_type == ExecType::Binary);
  try_init_impl(config)
}

fn try_init_impl(config: &Config) -> Result<ArcMutexGuard, InitError> {
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

  init_file(config, &config_file.1)
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

  #[cfg_attr(miri, ignore)]
  #[test]
  fn test_init_1() {
    set_up();
    for i in 0..4 {
      info!(i, "test_init_1");
      thread::sleep(Duration::from_millis(1));
    }
  }

  #[cfg_attr(miri, ignore)]
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
