// macros.rs

//! Macros.

// Macros ---------------------------------------------------------------------------------------------------

/// Prints the process invocation name, an `"Error"` label, and a message to `stderr`.
///
/// **NOTE:** The macro requires the [`nu_ansi_term`] crate.
///
/// # Errors
///
/// Returns [`Err`] with [`std::io::Error`] if an I/O error occurs.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate meadows;
/// process_error!("Cannot start engine")?; // -> "{inv_name}: Error: Cannot start engine\n"
/// # Ok::<(), anyhow::Error>(())
/// ```
#[macro_export]
macro_rules! process_error {
  ($($arg:tt)+) => {{
    use std::io::IsTerminal;
    use std::io::prelude::*;

    let name = $crate::env::inv_name().to_string_lossy();
    let mut write = std::io::stderr();
    let label = "Error";
    if write.is_terminal() {
      let label = nu_ansi_term::Color::Red.bold().paint(label);
      writeln!(write, "{}: {}: {}", name, label, format_args!($($arg)+))
    } else {
      writeln!(write, "{}: {}: {}", name, label, format_args!($($arg)+))
    }
  }};
}

/// Prints the process invocation name and a message to , a `"Note"` label, and a message to `stdout`.
///
/// **NOTE:** The macro requires the [`nu_ansi_term`] crate.
///
/// # Errors
///
/// Returns [`Err`] with [`std::io::Error`] if an I/O error occurs.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate meadows;
/// process_note!("Engine started")?; // -> "{inv_name}: Note: Engine started\n"
/// # Ok::<(), anyhow::Error>(())
/// ```
#[macro_export]
macro_rules! process_note {
  ($($arg:tt)+) => {{
    use std::io::IsTerminal;
    use std::io::prelude::*;

    let name = $crate::env::inv_name().to_string_lossy();
    let mut write = std::io::stdout();
    let label = "Note";
    if write.is_terminal() {
      let label = nu_ansi_term::Color::Green.bold().paint(label);
      writeln!(write, "{}: {}: {}", name, label, format_args!($($arg)+))
    } else {
      writeln!(write, "{}: {}: {}", name, label, format_args!($($arg)+))
    }
  }};
}

/// Prints the process invocation name, a `"Warning"` label, and a warning message to `stderr`.
///
/// **NOTE:** The macro requires the [`nu_ansi_term`] crate.
///
/// # Errors
///
/// Returns [`Err`] with [`std::io::Error`] if an I/O error occurs.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate meadows;
/// process_warn!("Engine overheating")?; // -> "{inv_name}: Warning: Engine overheating\n"
/// # Ok::<(), anyhow::Error>(())
/// ```
#[macro_export]
macro_rules! process_warn {
  ($($arg:tt)+) => {{
    use std::io::IsTerminal;
    use std::io::prelude::*;

    let name = $crate::env::inv_name().to_string_lossy();
    let mut write = std::io::stderr();
    let label = "Warning";
    if write.is_terminal() {
      let label = nu_ansi_term::Color::Yellow.bold().paint(label);
      writeln!(write, "{}: {}: {}", name, label, format_args!($($arg)+))
    } else {
      writeln!(write, "{}: {}: {}", name, label, format_args!($($arg)+))
    }
  }};
}

// EOF
