//! Command Line Interface (CLI) Subsystem
//!
//! Strongly-typed command dispatching and argument parsing for Panasonic DIGA
//! forensic operations, hardware links, and documentation lookups.

pub mod args;
pub mod handlers;

pub use args::{Cli, Commands};
pub use handlers::{dispatch, launch_desktop_executable};
