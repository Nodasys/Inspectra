//! # Inspectra Core Engine
//!
//! High-performance memory analysis and manipulation framework.
//!
//! This crate provides the core functionality for process introspection,
//! memory scanning, pointer analysis, and runtime manipulation.

pub mod debugger;
pub mod error;
pub mod memory;
pub mod platform;
pub mod pointer;
pub mod process;
pub mod scanner;
pub mod types;

pub use error::{InspectraError, Result};

/// Initialize the Inspectra engine with default configuration
pub fn init() -> Result<()> {
    env_logger::init();
    log::info!("Inspectra Core Engine initialized");
    Ok(())
}

/// Get version information
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
