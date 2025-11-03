//! Error types for Inspectra

use thiserror::Error;

/// Result type alias for Inspectra operations
pub type Result<T> = std::result::Result<T, InspectraError>;

/// Main error type for Inspectra operations
#[derive(Error, Debug)]
pub enum InspectraError {
    #[error("Process error: {0}")]
    Process(String),

    #[error("Memory error: {0}")]
    Memory(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid address: {0:#x}")]
    InvalidAddress(usize),

    #[error("Invalid process ID: {0}")]
    InvalidPid(u32),

    #[error("Scanner error: {0}")]
    Scanner(String),

    #[error("Pointer error: {0}")]
    Pointer(String),

    #[error("Debugger error: {0}")]
    Debugger(String),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl InspectraError {
    /// Create a process error
    pub fn process(msg: impl Into<String>) -> Self {
        Self::Process(msg.into())
    }

    /// Create a memory error
    pub fn memory(msg: impl Into<String>) -> Self {
        Self::Memory(msg.into())
    }

    /// Create a permission denied error
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }
}
