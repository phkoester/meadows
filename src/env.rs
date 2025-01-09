// env.rs

//! Environment-related utilities.

use std::env;
use std::path::PathBuf;

// Functions ------------------------------------------------------------------------------------------------

/// Prints the result of [`env::vars`] as key-value pairs to `stdout`.
pub fn dump() {
  for (name, val) in env::vars() {
    println!("{name}={val}");
  }
}

/// Prints the result of [`env::vars_os`] as key-value pairs to `stdout`.
pub fn dump_os() {
  for (name, val) in env::vars_os() {
    println!("{name:?}={val:?}");
  }
}

/// Returns the path to the system's configuration directory.
///
/// The returned value depends on the operating system and is either a [`Some`], containing a value from the
/// following table, or a [`None`].
///
/// | Platform | Value           | Example
/// | :------- | :-------------- | :------
/// | Unix     | `/etc`          | `/etc`                      |
/// | Windows  | `%PROGRAMDATA%` | `C:\ProgramData`
#[must_use]
pub fn system_config_dir() -> Option<PathBuf> { system_config_dir_impl() }

#[cfg(unix)]
fn system_config_dir_impl() -> Option<PathBuf> {
  let ret = PathBuf::from("/etc");
  if ret.is_dir() {
    Some(ret)
  } else {
    None
  }
}

#[cfg(windows)]
fn system_config_dir_impl() -> Option<PathBuf> {
  let dir = env::var_os("PROGRAMDATA").map(PathBuf::from);
  if let Some(dir) = dir {
    if dir.is_dir() {
      Some(dir)
    } else {
      None
    }
  } else {
    None
  }
}

// EOF
