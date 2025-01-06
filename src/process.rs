// process.rs

//! Process-related utilities.

use std::env;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::anyhow;
use regex::Regex;

// `ExecType` -----------------------------------------------------------------------------------------------

/// An enum for the type of the Rust executable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecType {
  /// A standard binary executable.
  Binary,
  /// An example executable.
  Example,
  /// A doc-test executable.
  DocTest,
  /// A unit-test executable.
  UnitTest,
  /// An integration-test executable.
  IntegTest,
  /// A benchmark-test executable.
  BenchTest,
}

impl ExecType {
  /// Returns `true` if the executable type denotes a test executable.
  #[must_use]
  pub fn is_test(&self) -> bool { !matches!(self, Self::Binary | Self::Example) }
}

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
    path
      .parent()
      .unwrap_or_else(|| {
        let err = anyhow!("Cannot obtain parent of path {:?}", path);
        panic!("{err:?}");
      })
      .to_owned()
  })
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
      .unwrap_or_else(|| {
        let err = anyhow!("Cannot obtain parent of invocation path {:?}", inv_path);
        panic!("{err:?}");
      })
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
/// 
/// | Executable Type        | Linux Example for `inv_path()`
/// | :--------------------- | :----------------------------
/// | `Binary`               | `target/debug/a`
/// | `Example`              | `target/debug/examples/a`
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
///
/// For test executables, the canonical name may end with a `-`, followed by a hash code. To strip the
/// suffix, use [`test_name`].
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn name() -> &'static OsString {
  static VAL: OnceLock<OsString> = OnceLock::new();
  VAL.get_or_init(|| {
    if cfg!(windows) {
      path().file_stem().unwrap().into()
    } else {
      path().file_name().unwrap().into()
    }
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
      let err =
        anyhow::Error::from(err).context(format!("Cannot canonicalize invocation path {inv_path:?}"));
      panic!("{err:?}")
    })
  })
}

/// Returns the canonical test name of the executable.
///
/// This is the canonical name as returned by [`name`], stripped of a trailing `-` and hexadecimal number, if
/// any.
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

// Tests ====================================================================================================

#[cfg(test)]
mod tests {
  use super::*;

  // Functions ----------------------------------------------------------------------------------------------

  #[test]
  fn test_test_name_impl() {
    assert_eq!(test_name_impl(OsStr::new("rust_out")), "rust_out");
    assert_eq!(test_name_impl(OsStr::new("a-b-0123456789abcdef")), "a-b");
  }

  #[test]
  #[should_panic(expected="`a` is not a valid test-executable name")]
  fn test_test_name_impl_fail_1() {
    test_name_impl(OsStr::new("a"));
  }

  #[test]
  #[should_panic(expected="`a-01234567` is not a valid test-executable name")]
  fn test_test_name_impl_fail_2() {
    test_name_impl(OsStr::new("a-01234567"));
  }
}

// EOF
