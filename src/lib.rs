// lib.rs

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! # Meadows
//!
//! Meadows is an experimental library written in Rust.
//!
//! ## Crate Features
//!
//! - **`tracing_config`** (disabled by default) - When enabled, the `crate::tracing::config` module is
//!   available.
//!
//! ## Logging
//!
//! Internally, Meadows uses the [`tracing`](::tracing) crate for logging.
//!
//! ## Colored Terminal Output
//!
//! For styled/colored output, Meadows uses [`anstream::stdout`] and [`anstream::stderr`], which in turn
//! call [`anstream::AutoStream::choice`] to configure the streams. The following envionment variables
//! are read:
//!
//! | Environment Variable | Description
//! | :------------------- | :-----------
//! | `CLICOLOR`           | Set it to `0` to disable colored output
//! | `CLICOLOR_FORCE`     | Set it to `1` to enforce colored output. This overrides `CLICOLOR`
//! | `NO_COLOR`           | Set it to `1` to disable colored output. This overrides `CLICOLOR_FORCE`

pub mod collections;
pub mod config;
pub mod env;
pub mod io;
pub mod macros;
pub mod math;
pub mod prelude;
pub mod process;
pub mod str;
pub mod tracing;
pub mod vec;

// Constants ------------------------------------------------------------------------------------------------

/// A general formatting hint.
///
/// This may be the assumed minimum column width of a terminal or editor, including line breaks. Lines may be
/// wrapped if they exceed `TEXT_WIDTH` - 1 columns.
pub const TEXT_WIDTH: usize = 110;

/// The crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// EOF
