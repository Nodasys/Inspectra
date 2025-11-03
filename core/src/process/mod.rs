//! Process management and introspection

use crate::error::Result;
use crate::types::{Architecture, Pid};
use serde::{Deserialize, Serialize};

#[cfg(windows)]
mod windows;
#[cfg(unix)]
mod unix;

#[cfg(windows)]
pub use self::windows::*;
#[cfg(unix)]
pub use self::unix::*;

/// Process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub path: String,
    pub architecture: Architecture,
    pub memory_usage: u64,
    pub is_elevated: bool,
}

/// Process handle for memory operations
pub trait ProcessHandle: Send + Sync {
    /// Get the process ID
    fn pid(&self) -> Pid;

    /// Check if the process is still running
    fn is_alive(&self) -> bool;

    /// Get process information
    fn info(&self) -> Result<ProcessInfo>;

    /// Terminate the process (requires elevated privileges)
    fn terminate(&self) -> Result<()>;
}

/// Process manager for listing and attaching to processes
pub trait ProcessManager {
    /// List all running processes
    fn list_processes(&self) -> Result<Vec<ProcessInfo>>;

    /// Attach to a process by PID
    fn attach(&self, pid: Pid) -> Result<Box<dyn ProcessHandle>>;

    /// Find processes by name
    fn find_by_name(&self, name: &str) -> Result<Vec<ProcessInfo>>;
}

/// Get the platform-specific process manager
pub fn get_process_manager() -> Box<dyn ProcessManager> {
    #[cfg(windows)]
    return Box::new(WindowsProcessManager::new());

    #[cfg(unix)]
    return Box::new(UnixProcessManager::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_processes() {
        let manager = get_process_manager();
        let processes = manager.list_processes().unwrap();
        assert!(!processes.is_empty());
    }
}
