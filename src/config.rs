// config.rs

//! Configuration-related utilities.

use std::env;
use std::ffi::OsStr;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::sync::OnceLock;

use thiserror::Error as ThisError;

use crate::collection::Uvec;
use crate::process::ExecType;

// Macros ---------------------------------------------------------------------------------------------------

/// Returns [`io::Result<()>`].
macro_rules! mod_debug {
  ($is_debug:expr, $($arg:tt)+) => {{
    if $is_debug {
      writeln!(io::stdout(), "[meadows::config] {}", format_args!($($arg)+))
    } else {
      Ok(())
    }
  }};
}

// `ConfigError` --------------------------------------------------------------------------------------------

/// Error type for the `find` functions.
#[derive(Debug, Eq, PartialEq, ThisError)]
pub enum ConfigError {
  /// File not found.
  #[error("File not found")]
  FileNotFound,
  /// Invalid file-name pattern.
  #[error("Invalid file-name pattern `{0}`")]
  InvalidFileNamePattern(String),
}

// `ConfigLevel` --------------------------------------------------------------------------------------------

/// Configuration levels, ordered from lowest (most general) to highest (most specific) priority.
///
/// In the following, `{name}` denotes the `name` value passed to a `find` function.
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
  /// this is `/etc/{name}`. On Windows, this is `%PROGRAMDATA%\{name}`, e.g. `C:\ProgramData\{name}`.
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
  /// Cargo-level configuration.
  ///
  /// Configuration files at Cargo level reside relative to the crate's manifest directory. For those to be
  /// found, the executable must be run via Cargo and the environment variable `CARGO_MANIFEST_DIR` must be
  /// set.
  Cargo,
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
/// use std::env;
///
/// use meadows::config;
/// use meadows::process::ExecType;
///
/// # fn run() -> anyhow::Result<()> {
/// let config_file = config::find_config_file(
///   ExecType::Binary,                  // `exec_type`
///   "{}config.toml",                   // `file_name_pattern`
///   true,                              // `is_debug`,
///   meadows::env::inv_name(),          // `name`
///   env::var_os("MY_PATH").as_deref(), // `paths`
///   true,                              // `set_env_vars`
/// )?;
/// #   Ok(())
/// # }
/// #
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
) -> anyhow::Result<(ConfigLevel, PathBuf)> {
  let files =
    find_config_files_impl(true, exec_type, file_name_pattern, is_debug, name, paths, set_env_vars)?;
  // If no error occurred, there must be at least one file, so `unwrap` is safe
  Ok(files.into_iter().next().unwrap())
}

/// Finds one or more configuration files, ordered from highest to lowest priority.
///
/// # Environment Variables
///
/// Oftentimes, configuration files support the usage of environment variables to be expanded when the file
/// is read. If `set_env_vars` is `true`, the function defines a few environment variables containing some
/// information about the currently running executable. This is done at most once per process.
///
/// The following variables are set for all executables:
///
/// | Name   | Description
/// | :----- | :-----------
/// | `dir`  | The canonical directory of the executable, as returned by [`dir`](crate::env::dir)
/// | `home` | The current user's home directory, as returned by [`dirs::home_dir`]
/// | `name` | The canonical name of the executable, as returned by [`name`](crate::env::name)
/// | `path` | The canonical path of the executable, as returned by [`path`](crate::env::path)
/// | `pid`  | The process ID (PID) of the executable
///
/// The following variables are set for binary executables only:
///
/// | Name       | Description
/// | :--------  | :----------
/// | `inv_dir`  | The invocation directory of the executable, as returned by [`inv_dir`](crate::env::inv_dir)
/// | `inv_name` | The invocation name of the executable, as returned by [`inv_name`](crate::env::inv_name)
/// | `inv_path` | The invocation path of the executable, as returned by [`inv_path`](crate::env::inv_path)
///
/// The following variables are set for test executables only:
///
/// | Name        | Description
/// | :---------- | :----------
/// | `test_name` | The canonical test name of the executable, as returned by [`test_name`](crate::env::test_name)
///
/// # Safety
///
/// If `set_env_vars` is `true`, some environment variables are defined using [`env::set_var`], which is not
/// thread-safe. For detailed information, read the "Safety" section for [`env::set_var`].
///
/// # File Search
///
/// If `is_debug` is `true`, the function outputs additional debug information on `stdout` that helps to
/// trace the file search.
///
/// As an example, let `file_name_pattern` be `"{}config.toml"`, let `name` be `"out"`.
///
/// XXX
///
/// # Errors
///
/// Returns
///
/// - [`ConfigError::FileNotFound`] if a configuration file cannot be found;
/// - [`ConfigError::InvalidFileNamePattern`] if `file_name_pattern` does not contain `"{}"`;
/// - [`io::Error`] if an I/O error occurs.
///
/// # Examples
///
/// ```
/// use std::env;
///
/// use meadows::config;
/// use meadows::process::ExecType;
///
/// # fn run() -> anyhow::Result<()> {
/// let config_files = config::find_config_files(
///   ExecType::Binary,                  // `exec_type`
///   "{}config.toml",                   // `file_name_pattern`
///   true,                              // `is_debug`,
///   meadows::env::inv_name(),          // `name`
///   env::var_os("MY_PATH").as_deref(), // `paths`
///   true,                              // `set_env_vars`
/// )?;
///
/// for config_file in config_files {
///   println!("{:?} | {:?}", config_file.0, config_file.1);
/// }
/// #   Ok(())
/// # }
/// #
/// # run();
/// ```
pub fn find_config_files(
  exec_type: ExecType,
  file_name_pattern: &str,
  is_debug: bool,
  name: &OsStr,
  paths: Option<&OsStr>,
  set_env_vars: bool,
) -> anyhow::Result<impl IntoIterator<Item = (ConfigLevel, PathBuf)>> {
  find_config_files_impl(false, exec_type, file_name_pattern, is_debug, name, paths, set_env_vars)
}

fn find_config_files_impl(
  find_one: bool,
  exec_type: ExecType,
  file_name_pattern: &str,
  is_debug: bool,
  name: &OsStr,
  paths: Option<&OsStr>,
  set_env_vars: bool,
) -> anyhow::Result<impl IntoIterator<Item = (ConfigLevel, PathBuf)>> {
  use ConfigLevel::*;
  use ExecType::*;

  // Some introductory debug info

  mod_debug!(is_debug, "Checking paths for {} executable", match exec_type {
    Binary => "binary",
    Example => "example",
    DocTest => "doc-test",
    UnitTest => "unit-test",
    IntegTest => "integration-test",
    BenchTest => "benchmark-test",
  })?;

  mod_debug!(is_debug, "Current directory: {}", {
    match env::current_dir() {
      Ok(dir) => format!("{dir:?}"),
      Err(_) => String::from("-"),
    }
  })?;

  // If requested, set env vars. This is executed only once

  if set_env_vars {
    if let Err(err) = self::set_env_vars(exec_type, is_debug) {
      return Err(err.into());
    }
  }

  // Collect paths to probe, ordered from highest to lowest priority

  let name = name.to_string_lossy();

  // `config.toml`
  let bare_file_name = replace_in_pattern(file_name_pattern, "")?;
  // `{name}.config.toml`
  let file_name = replace_in_pattern(file_name_pattern, &name)?;
  // `.{name}/config.toml`
  let hidden_relative_file = PathBuf::from(format!(".{name}")).join(&bare_file_name);
  // `{name}/config.toml`
  let relative_file = PathBuf::from(name.as_ref()).join(&bare_file_name);

  let mut file_paths = Vec::new();

  // Closure returns a `Some` if the outer function should return quickly
  let mut add_file_path =
    |level: ConfigLevel, path: PathBuf| -> io::Result<Option<(ConfigLevel, PathBuf)>> {
      file_paths.push((level, path.clone()));
      if is_debug {
        let level_str = format!("{level:?}");
        mod_debug!(is_debug, "{level_str:<10} | Adding path {path:?}")?;
        Ok(None)
      } else if find_one && path.is_file() {
        Ok(Some((level, path.clone())))
      } else {
        Ok(None)
      }
    };

  // Level `Path`
  if let Some(paths) = paths {
    for path in env::split_paths(paths) {
      if path.is_file() {
        if let Some(val) = add_file_path(Path, path)? {
          return Ok(vec![val].into_iter());
        }
      } else {
        if let Some(val) = add_file_path(Path, path.join(&file_name))? {
          return Ok(vec![val].into_iter());
        }
        if let Some(val) = add_file_path(Path, path.join(&hidden_relative_file))? {
          return Ok(vec![val].into_iter());
        }
      }
    }
  }

  // Level `Instance`
  if exec_type == Binary {
    let mut dir = env::current_dir().ok();
    while let Some(val) = dir {
      if let Some(val) = add_file_path(Instance, val.join(&file_name))? {
        return Ok(vec![val].into_iter());
      }
      if let Some(val) = add_file_path(Instance, val.join(&hidden_relative_file))? {
        return Ok(vec![val].into_iter());
      }
      dir = val.parent().map(PathBuf::from);
    }
  }

  // Level `Cargo`
  let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from);
  if let Some(dir) = manifest_dir {
    match exec_type {
      Binary => {
        if let Some(val) = add_file_path(Cargo, dir.join("src").join(&file_name))? {
          return Ok(vec![val].into_iter());
        }
        if let Some(val) = add_file_path(Cargo, dir.join("src").join("bin").join(&file_name))? {
          return Ok(vec![val].into_iter());
        }
      }
      Example => {
        if let Some(val) = add_file_path(Cargo, dir.join("examples").join(&file_name))? {
          return Ok(vec![val].into_iter());
        }
        if let Some(val) = add_file_path(Cargo, dir.join("examples").join(&bare_file_name))? {
          return Ok(vec![val].into_iter());
        }
      }
      DocTest | UnitTest => {
        if let Some(val) = add_file_path(Cargo, dir.join("src").join(&file_name))? {
          return Ok(vec![val].into_iter());
        }
        if let Some(val) = add_file_path(Cargo, dir.join("src").join(&bare_file_name))? {
          return Ok(vec![val].into_iter());
        }
      }
      IntegTest => {
        if let Some(val) = add_file_path(Cargo, dir.join("tests").join(&file_name))? {
          return Ok(vec![val].into_iter());
        }
        if let Some(val) = add_file_path(Cargo, dir.join("tests").join(&bare_file_name))? {
          return Ok(vec![val].into_iter());
        }
      }
      BenchTest => {
        if let Some(val) = add_file_path(Cargo, dir.join("benches").join(&file_name))? {
          return Ok(vec![val].into_iter());
        }
        if let Some(val) = add_file_path(Cargo, dir.join("benches").join(&bare_file_name))? {
          return Ok(vec![val].into_iter());
        }
      }
    }
  }

  // Level `Local`
  if exec_type == Binary {
    if let Some(dir) = dirs::home_dir() {
      if let Some(val) = add_file_path(Local, dir.join(&file_name))? {
        return Ok(vec![val].into_iter());
      }
      if let Some(val) = add_file_path(Local, dir.join(&hidden_relative_file))? {
        return Ok(vec![val].into_iter());
      }
    }
    if let Some(dir) = dirs::config_local_dir() {
      if let Some(val) = add_file_path(Local, dir.join(&relative_file))? {
        return Ok(vec![val].into_iter());
      }
    }
  }

  // Level `User`
  if exec_type == Binary {
    if let Some(dir) = dirs::config_dir() {
      if let Some(val) = add_file_path(User, dir.join(&relative_file))? {
        return Ok(vec![val].into_iter());
      }
    }
  }

  // Level `System`
  if exec_type == Binary {
    if let Some(dir) = crate::env::system_config_dir() {
      if let Some(val) = add_file_path(System, dir.join(&file_name))? {
        return Ok(vec![val].into_iter());
      }
      if let Some(val) = add_file_path(System, dir.join(&relative_file))? {
        return Ok(vec![val].into_iter());
      }
    }
  }

  // Level `Executable`
  if exec_type == Binary {
    if let Some(val) = add_file_path(Executable, crate::env::inv_dir().join(&file_name))? {
      return Ok(vec![val].into_iter());
    }
  }

  // Collect existing files

  // No canonical duplicates, only existing files
  let mut files = Uvec::with_key(&|val: &(ConfigLevel, PathBuf)| dunce::canonicalize(&val.1).ok());
  files.extend(file_paths);
  if files.is_empty() {
    Err(ConfigError::FileNotFound.into())
  } else {
    Ok(files.into_iter())
  }
}

fn replace_in_pattern(pattern: &str, to: &str) -> Result<String, ConfigError> {
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
    Err(ConfigError::InvalidFileNamePattern(pattern.to_owned()))
  }
}

/// Defines a few general-purpose environment variables that may be used from within configuration files.
/// This calls [`set_env_vars_impl`] exactly once per process.
fn set_env_vars(exec_type: ExecType, is_debug: bool) -> &'static io::Result<()> {
  static VAL: OnceLock<io::Result<()>> = OnceLock::new();
  VAL.get_or_init(|| set_env_vars_impl(exec_type, is_debug))
}

fn set_env_vars_impl(exec_type: ExecType, is_debug: bool) -> io::Result<()> {
  let set_env_var = |name: &str, val: &OsStr| -> io::Result<()> {
    mod_debug!(is_debug, "Setting `{}` to {:?}", name, val)?;
    env::set_var(name, val);
    Ok(())
  };

  set_env_var("dir", crate::env::dir().as_ref())?;
  if let Some(dir) = dirs::home_dir() {
    set_env_var("home", dir.as_ref())?;
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
  fn test_replace_in_pattern() -> Result<(), ConfigError> {
    assert_eq!(
      replace_in_pattern("", "name").unwrap_err(),
      ConfigError::InvalidFileNamePattern("".to_owned())
    );
    assert_eq!(
      replace_in_pattern("begend", "name").unwrap_err(),
      ConfigError::InvalidFileNamePattern("begend".to_owned())
    );

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
