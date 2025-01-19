// macros.rs

//! Macros.

// Macros ---------------------------------------------------------------------------------------------------

/// Prints the process invocation name, an error label, and a message to a stream.
///
/// The macro evaluates to a [`std::io::Result<()>`], just like [`writeln`] does.
///
/// **NOTE:** The macro requires the crate [`owo-colors`].
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate meadows;
/// let mut stderr = meadows::io::stderr().lock();
/// process_error!(stderr, "Cannot start engine")?; // -> "${inv_name}: error: Cannot start engine\n"
/// # Ok::<(), anyhow::Error>(())
/// ```
#[macro_export]
macro_rules! process_error {
  ($stream:expr, $($arg:tt)+) => {{
    use ::std::io::prelude::*;
    use ::owo_colors::OwoColorize;

    let name = $crate::env::inv_name().to_string_lossy();
    writeln!($stream, "{}: {}: {}", name, "error".bold().red(), format_args!($($arg)+))
  }};
}

/// Prints the process invocation name and a message to , a note label, and a message to a stream.
///
/// The macro evaluates to a [`std::io::Result<()>`], just like [`writeln`] does.
///
/// **NOTE:** The macro requires the crate [`owo-colors`].
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate meadows;
/// let mut stdout = meadows::io::stdout().lock();
/// process_note!(stdout, "Engine started")?; // -> "${inv_name}: note: Engine started\n"
/// # Ok::<(), anyhow::Error>(())
/// ```
#[macro_export]
macro_rules! process_note {
  ($stream:expr, $($arg:tt)+) => {{
    use ::std::io::prelude::*;
    use ::owo_colors::OwoColorize;

    let name = $crate::env::inv_name().to_string_lossy();
    writeln!($stream, "{}: {}: {}", name, "note".bold().green(), format_args!($($arg)+))
  }};
}

/// Prints the process invocation name, a warning label, and a message to a stream.
///
/// The macro evaluates to a [`std::io::Result<()>`], just like [`writeln`] does.
///
/// **NOTE:** The macro requires the crate [`owo-colors`].
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate meadows;
/// let mut stderr = meadows::io::stderr().lock();
/// process_warn!(stderr, "Engine overheating")?; // -> "${inv_name}: warning: Engine overheating\n"
/// # Ok::<(), anyhow::Error>(())
/// ```
#[macro_export]
macro_rules! process_warn {
  ($stream:expr, $($arg:tt)+) => {{
    use ::std::io::prelude::*;
    use ::owo_colors::OwoColorize;

    let name = $crate::env::inv_name().to_string_lossy();
    writeln!($stream, "{}: {}: {}", name, "warning".bold().yellow(), format_args!($($arg)+))
  }};
}

// EOF
