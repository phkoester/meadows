// env.rs

//! Environment-related utilities.

use std::env;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use regex::Regex;

// Variables ------------------------------------------------------------------------------------------------

/// Thread-safe mutex for synchronizing environment-variable operations.
static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

// Functions ------------------------------------------------------------------------------------------------

/// Returns the canonical directory of the executable.
///
/// # Panics
///
/// Panics if the canonical path has no parent.
#[must_use]
pub fn dir() -> &'static PathBuf {
  static VAL: OnceLock<PathBuf> = OnceLock::new();
  VAL.get_or_init(|| {
    let path = path();
    path.parent().unwrap_or_else(|| panic!("Cannot obtain parent of path {path:?}")).to_owned()
  })
}

/// Prints the result of [`vars`] as key-value pairs to `stdout`.
///
/// # Safety
///
/// All environment-variable operations from this module are thread-safe as long as they are used
/// exclusively.
///
/// # Errors
///
/// Returns [`Err`] with [`std::io::Error`] if an I/O error occurs.
#[allow(clippy::missing_panics_doc)]
pub fn dump() -> io::Result<()> {
  let _guard = env_mutex().lock().unwrap();
  for (name, val) in vars() {
    writeln!(io::stdout(), "{name:?}={val:?}")?;
  }
  Ok(())
}

fn env_mutex() -> &'static Mutex<()> {
  ENV_MUTEX.get_or_init(|| Mutex::new(()))
}

/// A replacement for [`env::var_os`].
///
/// # Safety
///
/// All environment-variable operations from this module are thread-safe as long as they are used
/// exclusively.
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn get<K: AsRef<OsStr>>(key: K) -> Option<OsString> {
  let _guard = env_mutex().lock().unwrap();
  env::var_os(key)
}

/// Returns the invocation directory of the executable.
///
/// # Panics
///
/// Panics if the invocation path has no parent.
#[must_use]
pub fn inv_dir() -> &'static PathBuf {
  static VAL: OnceLock<PathBuf> = OnceLock::new();
  VAL.get_or_init(|| {
    let inv_path = inv_path();
    inv_path
      .parent()
      .unwrap_or_else(|| panic!("Cannot obtain parent of invocation path {inv_path:?}"))
      .to_owned()
  })
}

/// Returns the invocation name of the executable.
///
/// In Windows, this is the file stem only. In Unix, this is the file name.
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn inv_name() -> &'static OsString {
  static VAL: OnceLock<OsString> = OnceLock::new();
  VAL.get_or_init(|| {
    if cfg!(windows) {
      inv_path().file_stem().unwrap().into()
    } else {
      inv_path().file_name().unwrap().into()
    }
  })
}

/// Returns the invocation path of the executable.
///
/// | Executable Type        | Linux Example for `inv_path()`
/// | :--------------------- | :----------------------------
/// | `Binary`               | `target/debug/out`
/// | `Example`              | `target/debug/examples/out`
/// | `DocTest`              | `/tmp/rustdoctestDWexge/rust_out`
/// | `DocTest` (persistent) | `/home/alice/project/meadows/target/debug/deps/src_process_rs_123_0/rust_out`
/// | `UnitTest`             | `/home/alice/project/meadows/target/debug/deps/meadows-c905bc0db64270b7`
/// | `IntegTest`            | `/home/alice/project/meadows/target/debug/deps/test_std-df01c96339a9b446`
/// | `BenchTest`            | `/home/alice/project/meadows/target/release/deps/bench_std-f98027946d1328b1`
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn inv_path() -> &'static PathBuf {
  static VAL: OnceLock<PathBuf> = OnceLock::new();
  VAL.get_or_init(|| PathBuf::from(env::args_os().next().unwrap()))
}

/// Returns the canonical name of the executable.
///
/// In Windows, this is the file stem only. In Unix, this is the file name.
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn name() -> &'static OsString {
  static VAL: OnceLock<OsString> = OnceLock::new();
  VAL.get_or_init(|| {
    if cfg!(windows) { path().file_stem().unwrap().into() } else { path().file_name().unwrap().into() }
  })
}

/// Returns the canonical path of the executable.
///
/// # Panics
///
/// Panics if canonicalizing the invocation path fails.
#[must_use]
pub fn path() -> &'static PathBuf {
  static VAL: OnceLock<PathBuf> = OnceLock::new();
  VAL.get_or_init(|| {
    let inv_path = inv_path();
    dunce::canonicalize(inv_path).unwrap_or_else(|err| {
      panic!(
        "{:?}",
        anyhow::Error::from(err).context(format!("Cannot canonicalize invocation path {inv_path:?}"))
      )
    })
  })
}

/// A replacement for [`env::set_var`] and [`env::remove_var`].
///
/// If `value` is [`Some`], the environment variable `key` is set to the given value. If `value` is [`None`],
/// the environment variable `key`is removed.
///
/// # Safety
///
/// All environment-variable operations from this module are thread-safe as long as they are used
/// exclusively.
///
/// # Examples
///
/// ```
/// use meadows::env;
///
/// // Set an environment variable
/// env::set("MY_VAR", Some("my_value".into()));
///
/// // Remove an environment variable
/// env::set("MY_VAR", None);
/// ```
#[allow(clippy::missing_panics_doc)]
pub unsafe fn set<K: AsRef<OsStr>, V: AsRef<OsStr>>(key: K, value: Option<V>) {
  let _guard = env_mutex().lock().unwrap();
  match value {
    Some(val) => unsafe {
      env::set_var(key, val);
    },
    None => unsafe {
      env::remove_var(key);
    },
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

#[cfg(not(windows))]
fn system_config_dir_impl() -> Option<PathBuf> {
  let ret = PathBuf::from("/etc");
  if ret.is_dir() {
    return Some(ret);
  }
  None
}

#[cfg(windows)]
fn system_config_dir_impl() -> Option<PathBuf> {
  let dir = get("PROGRAMDATA").map(PathBuf::from);
  if let Some(dir) = dir && dir.is_dir() {
    return Some(dir);
  }
  None
}

/// Returns the canonical test name of the executable.
///
/// This is the canonical name as returned by [`name`], stripped of a trailing `-` and 16-digit hexadecimal
/// number, if any.
///
/// # Panics
///
/// Panics if the canonical name is not a valid test-executable name.
#[must_use]
pub fn test_name() -> &'static OsString {
  static VAL: OnceLock<OsString> = OnceLock::new();
  VAL.get_or_init(|| test_name_impl(name()))
}

fn test_name_impl(name: &OsStr) -> OsString {
  // Special case: doc test
  if name == "rust_out" {
    return name.to_owned();
  }

  // Strip trailing `-` and 16-digit hex number
  let re = Regex::new("-[0-9a-f]{16}$").unwrap();
  let name = name.to_string_lossy();
  assert!(re.is_match(name.as_ref()), "`{name}` is not a valid test-executable name");
  name[0..name.len() - 17].into()
}

/// A replacement for [`env::vars_os`].
///
/// # Safety
///
/// All environment-variable operations from this module are thread-safe as long as they are used
/// exclusively.
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn vars() -> env::VarsOs {
  let _guard = env_mutex().lock().unwrap();
  env::vars_os()
}

// Tests ====================================================================================================

#[cfg(test)]
mod tests {
  use super::*;

  // Functions ----------------------------------------------------------------------------------------------

  #[test]
  fn test_test_name_impl() {
    assert_eq!(test_name_impl(OsStr::new("rust_out")), "rust_out");
    assert_eq!(test_name_impl(OsStr::new("ab-cd-0123456789abcdef")), "ab-cd");
  }

  #[test]
  #[should_panic(expected = "`out` is not a valid test-executable name")]
  fn test_test_name_impl_fail_1() { test_name_impl(OsStr::new("out")); }

  #[test]
  #[should_panic(expected = "`a-0123456789` is not a valid test-executable name")]
  fn test_test_name_impl_fail_2() { test_name_impl(OsStr::new("a-0123456789")); }
}

// EOF
