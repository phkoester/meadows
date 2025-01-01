// process.rs

//! Process-related utilities.

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::anyhow;

// `ExecType` -----------------------------------------------------------------------------------------------

/// An enum for the type of the Rust executable.
///
/// Any `ExecType` other than [`Standard`](ExecType::Standard) identifies a test executable.
#[derive(Clone, Copy, Debug)]
pub enum ExecType {
  /// A standard executable.
  Standard,
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
  /// 
  /// This is true for all variants except [`Standard`](ExecType::Standard).
  #[must_use]
  pub fn is_test(&self) -> bool { !matches!(self, Self::Standard) }
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
/// In Windows, this is the file stem only, the file name otherwise.
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
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn inv_path() -> &'static PathBuf {
  static VAL: OnceLock<PathBuf> = OnceLock::new();
  VAL.get_or_init(|| PathBuf::from(env::args_os().next().unwrap()))
}

/// Returns the canonical name of the executable.
///
/// In Windows, this is the file stem only, the file name otherwise.
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
/// This is the canonical name as returned by [`name`], stripped of a trailing `-` and hash code, if any.
#[must_use]
pub fn test_name() -> &'static OsString {
  static VAL: OnceLock<OsString> = OnceLock::new();
  VAL.get_or_init(|| {
    let name = name().to_string_lossy();
    if let Some(index) = name.rfind('-') {
      OsString::from(&name[0..index])
    } else {
      name.as_ref().into()
    }
  })
}

// EOF
