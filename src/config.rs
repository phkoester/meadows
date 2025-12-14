// config.rs

//! Configuration-related utilities.

use std::ffi::OsStr;
use std::io;
use std::io::prelude::*;
use std::io::StdoutLock;
use std::path::PathBuf;
use std::process;
use std::sync::OnceLock;

use anstream::AutoStream;
use thiserror::Error as ThisError;

use crate::collections::Uvec;
use crate::process::ExecType;

// Macros ---------------------------------------------------------------------------------------------------

/// Evaluates to an [`io::Result<()>`].
macro_rules! debug {
  ($stream:expr, $($arg:tt)+) => {{
    if let Some(stream) = $stream {
      writeln!(stream, "[meadows::config] {}", format_args!($($arg)+))
    } else {
      Ok(())
    }
  }}
}

// Types ----------------------------------------------------------------------------------------------------

type AutoStreamStdoutLock = AutoStream<StdoutLock<'static>>;

// `ConfigLevel` --------------------------------------------------------------------------------------------

/// Configuration levels, ordered from lowest (most general) to highest (most specific) priority.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ConfigLevel {
  /// Executable-level configuration.
  ///
  /// Configuration files at executable level reside next to the executable. These are less common on Unix
  /// systems, but on Windows, configuration files are often placed there.
  Executable,
  /// System-level configuration.
  ///
  /// Configuration files at system level reside relative to a system-dependent directory. On Unix systems,
  /// this is `/etc`. On Windows, this is `%PROGRAMDATA%`, e.g. `C:\ProgramData`.
  System,
  /// User-level configuration.
  ///
  /// Configuration files at user level reside relative to a system-dependent directory as returned by
  /// [`dirs::config_dir`].
  User,
  /// Local-level configuration.
  ///
  /// Configuration files at local level reside relative to system-dependent directories as returned by
  /// [`dirs::home_dir`] and [`dirs::config_local_dir`].
  Local,
  /// Package-level configuration.
  ///
  /// Configuration files at package level reside relative to the package's manifest directory. For those to
  /// be found, the executable must be run via Cargo and the environment variable `CARGO_MANIFEST_DIR` must
  /// be set.
  Package,
  /// Instance-level configuration.
  ///
  /// Configuration files at instance level reside relative to the current working directory or any of its
  /// parent directories.
  Instance,
  /// Path-level configuration.
  ///
  /// Configuration files at path level reside at an explicitly specified path or relative to an explicitly
  /// specified directory.
  Path,
}

// `FindError` ----------------------------------------------------------------------------------------------

/// Error type for the `find` functions.
#[derive(Debug, ThisError)]
pub enum FindError {
  /// File not found.
  #[error("File not found")]
  FileNotFound,
  /// Invalid file-name pattern.
  #[error("Invalid file-name pattern `{0}`")]
  InvalidFileNamePattern(String),
  /// [`io::Error`].
  #[error("I/O error")]
  Io(#[from] io::Error),
}

impl FindError {
  /// Returns `true` if the error should be printed.
  #[must_use]
  pub fn should_print(&self) -> bool { !matches!(self, Self::FileNotFound) }
}

// Functions ------------------------------------------------------------------------------------------------

/// Finds a configuration file.
///
/// Unlike [`find_config_files`], the function looks for a single configuration file only. If an existing
/// file is found, the one with the highest priority is returned.
///
/// See [`find_config_files`].
///
/// # Errors
///
/// See [`find_config_files`].
///
/// # Examples
///
/// ```
/// # fn run() -> anyhow::Result<()> {
/// use meadows::config;
/// use meadows::env;
/// use meadows::process::ExecType;
///
/// let config_file = config::find_config_file(
///   ExecType::Binary,               // `exec_type`
///   "{}config.toml",                // `file_name_pattern`
///   true,                           // `is_debug`,
///   env::inv_name(),                // `name`
///   env::get("MY_PATH").as_deref(), // `paths`
///   true,                           // `set_env_vars`
/// )?;
/// #   Ok(())
/// # }
/// # #[cfg(not(miri))]
/// # run();
/// ```
#[allow(clippy::missing_panics_doc)]
pub fn find_config_file(
  exec_type: ExecType,
  file_name_pattern: &str,
  is_debug: bool,
  name: &OsStr,
  paths: Option<&OsStr>,
  set_env_vars: bool,
) -> Result<(ConfigLevel, PathBuf), FindError> {
  let files =
    find_config_files_impl(true, exec_type, file_name_pattern, is_debug, name, paths, set_env_vars)?;
  // If no error occurred, there must be at least one file, so `unwrap` is safe
  Ok(files.into_iter().next().unwrap())
}

/// Finds one or more configuration files suitable for a given `exec_type`, ordered from highest to lowest
/// priority.
///
/// # Environment Variables
///
/// Oftentimes, configuration files support the usage of environment variables to be expanded when the file
/// is read. If `set_env_vars` is `true`, the function defines a few environment variables containing some
/// information about the currently running executable. This is done at most once per process.
///
/// The following variables are set:
///
/// | Name        | `exec_type`      | Description
/// | :---------- | :--------------- | :-----------
/// | `dir`       | Any              | The canonical directory of the executable, as returned by [`dir`]
/// | `home_dir`  | Any              | The current user's home directory, as returned by [`dirs::home_dir`]
/// | `name`      | Any              | The canonical name of the executable, as returned by [`name`]
/// | `path`      | Any              | The canonical path of the executable, as returned by [`path`]
/// | `pid`       | Any              | The process ID (PID) of the executable
/// | `inv_dir`   | [`Binary`]       | The invocation directory of the executable, as returned by [`inv_dir`]
/// | `inv_name`  | [`Binary`]       | The invocation name of the executable, as returned by [`inv_name`]
/// | `inv_path`  | [`Binary`]       | The invocation path of the executable, as returned by [`inv_path`]
/// | `test_name` | Test executables | The canonical test name of the executable, as returned by [`test_name`]
///
/// # File Search
///
/// If `is_debug` is `true`, the function outputs additional debug information on `stdout` that helps to
/// trace the file search.
///
/// As an example, let `file_name_pattern` be `"{}config.toml"`. Let the current working directory be
/// `/home/alice`.
///
/// In the following, these placeholders are used:
///
/// | Placeholder            | Description
/// | :--------------------- | :----------
/// | `${config_dir}`        | A system-dependent directory as returned by [`dirs::config_dir`]
/// | `${config_local_dir}`  | A system-dependent directory as returned by [`dirs::config_local_dir`]
/// | `${home_dir}`          | The user's home directory as returned by [`dirs::home_dir`], e.g. `/home/alice`
/// | `${inv_dir}`           | The invocation directory as returned by [`inv_dir`]
/// | `${manifest_dir}`      | The package's manifest directory. This applies only if the executable is run via Cargo
/// | `${name}`              | `name`
/// | `${path}`              | Each path from `paths`, which is separated by the system-dependent path separator. Each path may point to a file or directory. This applies only if `paths` is a [`Some`]
/// | `${system_config_dir}` | A system-dependent directory as returned by [`system_config_dir`]
///
/// The function probes the following paths, from highest to lowest priority, in the exact order shown, if
/// they point to existing files:
///
/// | Configuration Level | `exec_type`               | Path
/// | :------------------ | :------------------------ | :---
/// | [`Path`]            | Any                       | `${path}`
/// | [`Path`]            | Any                       | `${path}/.${name}.config.toml`
/// | [`Path`]            | Any                       | `${path}/.${name}/config.toml`
/// | [`Instance`]        | [`Binary`]                | `/home/alice/.${name}.config.toml`
/// | [`Instance`]        | [`Binary`]                | `/home/alice/.${name}/config.toml`
/// | [`Instance`]        | [`Binary`]                | `/home/.${name}.config.toml`
/// | [`Instance`]        | [`Binary`]                | `/home/.${name}/config.toml`
/// | [`Instance`]        | [`Binary`]                | `/.${name}.config.toml`
/// | [`Instance`]        | [`Binary`]                | `/.${name}/config.toml`
/// | [`Package`]         | [`Binary`]                | `${manifest_dir}/src/${name}.config.toml`
/// | [`Package`]         | [`Binary`]                | `${manifest_dir}/src/bin/${name}.config.toml`
/// | [`Package`]         | [`Example`]               | `${manifest_dir}/examples/${name}.config.toml`
/// | [`Package`]         | [`Example`]               | `${manifest_dir}/examples/config.toml`
/// | [`Package`]         | [`DocTest`], [`UnitTest`] | `${manifest_dir}/src/${name}.config.toml`
/// | [`Package`]         | [`DocTest`], [`UnitTest`] | `${manifest_dir}/src/config.toml`
/// | [`Package`]         | [`IntegTest`]             | `${manifest_dir}/tests/${name}.config.toml`
/// | [`Package`]         | [`IntegTest`]             | `${manifest_dir}/tests/config.toml`
/// | [`Package`]         | [`BenchTest`]             | `${manifest_dir}/benches/${name}.config.toml`
/// | [`Package`]         | [`BenchTest`]             | `${manifest_dir}/benches/config.toml`
/// | [`Local`]           | [`Binary`]                | `${home_dir}/.${name}.config.toml`
/// | [`Local`]           | [`Binary`]                | `${home_dir}/.${name}/config.toml`
/// | [`Local`]           | [`Binary`]                | `${config_local_dir}/${name}/config.toml`
/// | [`User`]            | [`Binary`]                | `${config_dir}/${name}/config.toml`
/// | [`System`]          | [`Binary`]                | `${system_config_dir}/${name}.config.toml`
/// | [`System`]          | [`Binary`]                | `${system_config_dir}/${name}/config.toml`
/// | [`Executable`]      | [`Binary`]                | `${inv_dir}/${name}.config.toml`
///
/// The function returns an [`IntoIterator`] that produces pairs of [`ConfigLevel`]s and [`PathBuf`]s for
/// existing files. How multiple configuration files are combined into a specific configuration, is left
/// entirely to the program. The general idea is that settings from a configuration file override settings
/// from a preceding configuration file.
///
/// # Errors
///
/// Returns [`Err`] with
///
/// - [`FindError::FileNotFound`] if a configuration file cannot be found
/// - [`FindError::InvalidFileNamePattern`] if `file_name_pattern` does not contain `"{}"`
/// - [`FindError::Io`] if an [`io::Error`] occurs
///
/// # Examples
///
/// ```
/// # fn run() -> anyhow::Result<()> {
/// use meadows::config;
/// use meadows::env;
/// use meadows::process::ExecType;
///
/// let config_files = config::find_config_files(
///   ExecType::Binary,    // `exec_type`
///   "{}config.toml",     // `file_name_pattern`
///   true,                // `is_debug`
///   env::inv_name(),     // `name`
///   env::get("MY_PATH"), // `paths`
///   true                 // `set_env_vars`
/// )?;
///
/// for config_file in config_files {
///   println!("{:?} | {:?}", config_file.0, config_file.1);
/// }
/// #   Ok(())
/// # }
/// # #[cfg(not(miri))]
/// # run();
/// ```
///
/// [`dir`]: crate::env::dir
/// [`inv_dir`]: crate::env::inv_dir
/// [`inv_name`]: crate::env::inv_name
/// [`inv_path`]: crate::env::inv_path
/// [`name`]: crate::env::name
/// [`path`]: crate::env::path
/// [`system_config_dir`]: crate::env::system_config_dir
/// [`test_name`]: crate::env::test_name
///
/// [`Path`]: ConfigLevel::Path
/// [`Instance`]: ConfigLevel::Instance
/// [`Package`]: ConfigLevel::Package
/// [`Local`]: ConfigLevel::Local
/// [`User`]: ConfigLevel::User
/// [`System`]: ConfigLevel::System
/// [`Executable`]: ConfigLevel::Executable
///
/// [`Binary`]: ExecType::Binary
/// [`Example`]: ExecType::Example
/// [`DocTest`]: ExecType::DocTest
/// [`UnitTest`]: ExecType::UnitTest
/// [`IntegTest`]: ExecType::IntegTest
/// [`BenchTest`]: ExecType::BenchTest
pub fn find_config_files<Paths: AsRef<OsStr>>(
  exec_type: ExecType,
  file_name_pattern: &str,
  is_debug: bool,
  name: &OsStr,
  paths: Option<Paths>,
  set_env_vars: bool,
) -> Result<impl IntoIterator<Item = (ConfigLevel, PathBuf)>, FindError> {
  find_config_files_impl(false, exec_type, file_name_pattern, is_debug, name, paths, set_env_vars)
}

fn find_config_files_impl<Paths: AsRef<OsStr>>(
  find_one: bool,
  exec_type: ExecType,
  file_name_pattern: &str,
  is_debug: bool,
  name: &OsStr,
  paths: Option<Paths>,
  set_env_vars: bool,
) -> Result<impl IntoIterator<Item = (ConfigLevel, PathBuf)>, FindError> {
  use ConfigLevel::*;
  use ExecType::*;

  // Some introductory debug info

  let mut stdout = if is_debug { Some(crate::io::stdout().lock()) } else { None };
  let stdout = &mut stdout;

  debug!(stdout, "Checking paths for {} executable", match exec_type {
    Binary => "binary",
    Example => "example",
    DocTest => "doc-test",
    UnitTest => "unit-test",
    IntegTest => "integration-test",
    BenchTest => "benchmark-test",
  })?;

  debug!(stdout, "Current directory: {}", {
    match std::env::current_dir() {
      Ok(dir) => format!("{dir:?}"),
      Err(_) => String::from("-"),
    }
  })?;

  // If requested, set env vars. This is executed only once

  if set_env_vars {
    self::set_env_vars(stdout, exec_type)?;
  }

  // Define a few names and relative paths

  let name = name.to_string_lossy();
  // `config.toml`
  let bare_file_name = replace_in_pattern(file_name_pattern, "")?;
  // `{name}.config.toml`
  let file_name = replace_in_pattern(file_name_pattern, &name)?;
  // `.{name}.config.toml`
  let hidden_file_name = format!(".{file_name}");
  // `{name}/config.toml`
  let relative_file = PathBuf::from(name.as_ref()).join(&bare_file_name);
  // `.{name}/config.toml`
  let hidden_relative_file = PathBuf::from(format!(".{name}")).join(&bare_file_name);

  // Collect paths to probe, ordered from highest to lowest priority

  let mut file_paths = Vec::new();

  // Closure returns a `Some` if the outer function should return quickly
  let mut add_file_path =
    |level: ConfigLevel, path: PathBuf| -> io::Result<Option<(ConfigLevel, PathBuf)>> {
      file_paths.push((level, path.clone()));
      if is_debug {
        let level_str = format!("{level:?}");
        let bullet = if path.is_file() { "*" } else { "" };
        debug!(stdout, "{level_str:<10} | {bullet:<1} {path:?}")?;
        // In debug mode, we don't return quickly
        Ok(None)
      } else if find_one && path.is_file() {
        Ok(Some((level, path)))
      } else {
        Ok(None)
      }
    };

  // Macro returns from outer function if closure returns a `Some`
  macro_rules! add {
    ($level:expr, $path:expr) => {{
      if let Some(val) = add_file_path($level, $path)? {
        return Ok(vec![val].into_iter());
      }
    }};
  }

  // Level `Path`
  if let Some(paths) = &paths {
    for path in std::env::split_paths(paths) {
      if path.is_file() {
        add!(Path, path);
      } else {
        add!(Path, path.join(&hidden_file_name));
        add!(Path, path.join(&hidden_relative_file));
      }
    }
  }

  // Level `Instance`
  if exec_type == Binary {
    let mut dir = std::env::current_dir().ok();
    while let Some(val) = dir {
      add!(Instance, val.join(&hidden_file_name));
      add!(Instance, val.join(&hidden_relative_file));
      dir = val.parent().map(PathBuf::from);
    }
  }

  // Level `Package`
  let manifest_dir = crate::env::get("CARGO_MANIFEST_DIR").map(PathBuf::from);
  if let Some(dir) = manifest_dir {
    match exec_type {
      Binary => {
        add!(Package, dir.join("src").join(&file_name));
        add!(Package, dir.join("src").join("bin").join(&file_name));
      }
      Example => {
        add!(Package, dir.join("examples").join(&file_name));
        add!(Package, dir.join("examples").join(&bare_file_name));
      }
      DocTest | UnitTest => {
        add!(Package, dir.join("src").join(&file_name));
        add!(Package, dir.join("src").join(&bare_file_name));
      }
      IntegTest => {
        add!(Package, dir.join("tests").join(&file_name));
        add!(Package, dir.join("tests").join(&bare_file_name));
      }
      BenchTest => {
        add!(Package, dir.join("benches").join(&file_name));
        add!(Package, dir.join("benches").join(&bare_file_name));
      }
    }
  }

  // Level `Local`
  if exec_type == Binary {
    if let Some(dir) = dirs::home_dir() {
      add!(Local, dir.join(&hidden_file_name));
      add!(Local, dir.join(&hidden_relative_file));
    }
    if let Some(dir) = dirs::config_local_dir() {
      add!(Local, dir.join(&relative_file));
    }
  }

  // Level `User`
  if exec_type == Binary && let Some(dir) = dirs::config_dir() {
    add!(User, dir.join(&relative_file));
  }

  // Level `System`
  if exec_type == Binary && let Some(dir) = crate::env::system_config_dir() {
    add!(System, dir.join(&file_name));
    add!(System, dir.join(&relative_file));
  }

  // Level `Executable`
  if exec_type == Binary {
    add!(Executable, crate::env::inv_dir().join(&file_name));
  }

  // Collect existing files

  // No canonical duplicates, only existing files
  let mut files = Uvec::with_key(&|val: &(ConfigLevel, PathBuf)| dunce::canonicalize(&val.1).ok());
  files.extend(file_paths);
  if files.is_empty() {
    Err(FindError::FileNotFound)
  } else {
    Ok(files.into_iter())
  }
}

fn replace_in_pattern(pattern: &str, to: &str) -> Result<String, FindError> {
  let from = "{}";
  if let Some(index) = pattern.find(from) {
    let ldot = index > 0;
    let ldot_str = if ldot && !to.is_empty() { "." } else { "" };
    let rdot = index < pattern.len() - from.len();
    let rdot_str = if rdot && !to.is_empty() { "." } else { "" };
    //
    let to = format!("{ldot_str}{to}{rdot_str}");
    Ok(pattern.replacen(from, &to, 1))
  } else {
    Err(FindError::InvalidFileNamePattern(pattern.to_owned()))
  }
}

/// Defines a few general-purpose environment variables that may be used from within configuration files.
/// This calls [`set_env_vars_impl`] exactly once per process.
fn set_env_vars(stdout: &mut Option<AutoStreamStdoutLock>, exec_type: ExecType) -> io::Result<()> {
  static VAL: OnceLock<io::Result<()>> = OnceLock::new();
  match VAL.get_or_init(|| set_env_vars_impl(stdout, exec_type)) {
    Ok(()) => Ok(()),
    Err(err) => Err(io::Error::new(err.kind(), err.to_string())),
  }
}

fn set_env_vars_impl(stdout: &mut Option<AutoStreamStdoutLock>, exec_type: ExecType) -> io::Result<()> {
  let mut set_env_var = |name: &str, val: &OsStr| -> io::Result<()> {
    debug!(stdout, "Setting `{name}` to {val:?}")?;
    crate::env::set(name, Some(val));
    Ok(())
  };

  set_env_var("dir", crate::env::dir().as_ref())?;
  if let Some(dir) = dirs::home_dir() {
    set_env_var("home_dir", dir.as_ref())?;
  }
  set_env_var("name", crate::env::name())?;
  set_env_var("path", crate::env::path().as_ref())?;
  set_env_var("pid", process::id().to_string().as_ref())?;

  if exec_type == ExecType::Binary {
    set_env_var("inv_dir", crate::env::inv_dir().as_ref())?;
    set_env_var("inv_name", crate::env::inv_name())?;
    set_env_var("inv_path", crate::env::inv_path().as_ref())?;
  }

  if exec_type.is_test() {
    set_env_var("test_name", crate::env::test_name())?;
  }

  Ok(())
}

// Tests ====================================================================================================

#[cfg(test)]
mod tests {
  use super::*;

  // Functions ----------------------------------------------------------------------------------------------

  #[test]
  fn test_replace_in_pattern() -> Result<(), FindError> {
    assert!(matches!(replace_in_pattern("", "name"), Err(FindError::InvalidFileNamePattern(_))));
    assert!(matches!(replace_in_pattern("begend", "name"), Err(FindError::InvalidFileNamePattern(_))));

    assert_eq!(replace_in_pattern("{}", "name")?, "name");
    assert_eq!(replace_in_pattern("{}", "")?, "");

    assert_eq!(replace_in_pattern("beg{}", "name")?, "beg.name");
    assert_eq!(replace_in_pattern("beg{}", "")?, "beg");

    assert_eq!(replace_in_pattern("{}end", "name")?, "name.end");
    assert_eq!(replace_in_pattern("{}end", "")?, "end");

    assert_eq!(replace_in_pattern("beg{}end", "name")?, "beg.name.end");
    assert_eq!(replace_in_pattern("beg{}end", "")?, "begend");

    Ok(())
  }
}

// EOF
