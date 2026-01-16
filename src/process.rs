// process.rs

//! Process-related utilities.

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
  /// Checks if the executable type denotes a test executable.
  #[must_use]
  pub fn is_test(&self) -> bool { !matches!(self, Self::Binary | Self::Example) }
}

// EOF
