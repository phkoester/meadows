// lib.rs

//! # Meadows
//!
//! Meadows is an experimental library written in Rust.
//!
//! ## Crate Features
//!
//! - `tracing_config` - When enabled, the `crate::tracing::config` module is available. This is **disabled
//!   by default.**

// Constants ------------------------------------------------------------------------------------------------

/// A general formatting hint.
///
/// This may be the assumed column width of a terminal or editor. Lines may be wrapped if they exceed
/// `TEXT_WIDTH` - 1 columns.
pub const TEXT_WIDTH: usize = 110;

/// The crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Modules --------------------------------------------------------------------------------------------------

pub mod collection;
pub mod config;
pub mod env;
pub mod io;
pub mod macros;
pub mod prelude;
pub mod process;
pub mod str;
pub mod tracing;
pub mod vec;

// EOF
