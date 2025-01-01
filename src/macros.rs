// macros.rs

//! Macros.

// Macros ---------------------------------------------------------------------------------------------------

/// Prints the process invocation name, an `"Error"` label, and a message to `stderr`.
/// 
/// **NOTE:** The macro requires the [`nu_ansi_term`] crate.
///
/// # Examples
///
/// ```
/// use meadows::process_error;
/// process_error!("Cannot start engine"); // -> "app: Error: Cannot start engine\n"
/// ```
#[macro_export]
macro_rules! process_error {
  ($($arg:tt)+) => {
    use std::io::IsTerminal;
    use std::io::Write;

    let name = $crate::process::inv_name().to_string_lossy();
    let mut write = std::io::stderr();
    let label = "Error";
    if write.is_terminal() {
      let label = nu_ansi_term::Color::Red.bold().paint(label);
      let _ = writeln!(write, "{}: {}: {}", name, label, format_args!($($arg)+));
    } else {
      let _ = writeln!(write, "{}: {}: {}", name, label, format_args!($($arg)+));
    }
  };
}

/// Prints the process invocation name and a message to , a `"Note"` label, and a message to `stdout`.
///
/// **NOTE:** The macro requires the [`nu_ansi_term`] crate.
/// 
/// # Examples
///
/// ```
/// use meadows::process_note;
/// process_note!("Engine started"); // -> "app: Note: Engine started\n"
/// ```
#[macro_export]
macro_rules! process_note {
  ($($arg:tt)+) => {
    use std::io::IsTerminal;
    use std::io::Write;

    let name = $crate::process::inv_name().to_string_lossy();
    let mut write = std::io::stdout();
    let label = "Note";
    if write.is_terminal() {
      let label = nu_ansi_term::Color::Green.bold().paint(label);
      let _ = writeln!(write, "{}: {}: {}", name, label, format_args!($($arg)+));
    } else {
      let _ = writeln!(write, "{}: {}: {}", name, label, format_args!($($arg)+));
    }
  };
}

/// Prints the process invocation name, a `"Warning"` label, and a warning message to `stderr`.
///
/// **NOTE:** The macro requires the [`nu_ansi_term`] crate.
/// 
/// # Examples
///
/// ```
/// use meadows::process_warn;
/// process_warn!("Engine overheating"); // -> "app: Warning: Engine overheating\n"
/// ```
#[macro_export]
macro_rules! process_warn {
  ($($arg:tt)+) => {
    use std::io::IsTerminal;
    use std::io::Write;

    let name = $crate::process::inv_name().to_string_lossy();
    let mut write = std::io::stderr();
    let label = "Warning";
    if write.is_terminal() {
      let label = nu_ansi_term::Color::Yellow.bold().paint(label);
      let _ = writeln!(write, "{}: {}: {}", name, label, format_args!($($arg)+));
    } else {
      let _ = writeln!(write, "{}: {}: {}", name, label, format_args!($($arg)+));
    }
  };
}

// EOF
