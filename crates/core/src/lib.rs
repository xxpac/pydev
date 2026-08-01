//! pydev-core: shared engine for the pydev installer.
//!
//! All privileged work (downloading, running installers, editing PATH, probing
//! the network) lives here and is deliberately Tauri-agnostic so it can be
//! driven by both the GUI (`src-tauri`) and the CLI (`crates/cli`). Routines are
//! blocking and stream progress through a [`Reporter`].

pub mod config;
pub mod download;
pub mod error;
pub mod fsutil;
pub mod installers;
pub mod netcheck;
pub mod orchestrate;
pub mod pathenv;
pub mod platform;
pub mod progress;
pub mod runner;
pub mod status;

pub use config::Config;
pub use error::{Error, Result};
pub use progress::{LogLevel, LogLine, NullReporter, Reporter, Stage};
