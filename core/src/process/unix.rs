//! Unix-specific process implementation

use super::{ProcessHandle, ProcessInfo, ProcessManager};
use crate::error::{InspectraError, Result};
use crate::types::{Architecture, Pid};
use std::fs;
use std::sync::Arc;

pub struct UnixProcessManager;

impl UnixProcessManager {
    pub fn new() -> Self {
        Self
    }
}

impl ProcessManager for UnixProcessManager {
    fn list_processes(&self) -> Result<Vec<ProcessInfo>> {
        let mut processes = Vec::new();

        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if let Ok(pid) = file_name.parse::<Pid>() {
                        if let Ok(name) = fs::read_to_string(format!("/proc/{}/comm", pid)) {
                            processes.push(ProcessInfo {
                                pid,
                                name: name.trim().to_string(),
                                path: String::new(),
                                architecture: Architecture::Unknown,
                                memory_usage: 0,
                                is_elevated: false,
                            });
                        }
                    }
                }
            }
        }

        Ok(processes)
    }

    fn attach(&self, pid: Pid) -> Result<Box<dyn ProcessHandle>> {
        // Check if process exists
        if !std::path::Path::new(&format!("/proc/{}", pid)).exists() {
            return Err(InspectraError::InvalidPid(pid));
        }

        Ok(Box::new(UnixProcessHandle { pid }))
    }

    fn find_by_name(&self, name: &str) -> Result<Vec<ProcessInfo>> {
        let all_processes = self.list_processes()?;
        Ok(all_processes
            .into_iter()
            .filter(|p| p.name.to_lowercase().contains(&name.to_lowercase()))
            .collect())
    }
}

pub struct UnixProcessHandle {
    pid: Pid,
}

impl ProcessHandle for UnixProcessHandle {
    fn pid(&self) -> Pid {
        self.pid
    }

    fn is_alive(&self) -> bool {
        std::path::Path::new(&format!("/proc/{}", self.pid)).exists()
    }

    fn info(&self) -> Result<ProcessInfo> {
        let name = fs::read_to_string(format!("/proc/{}/comm", self.pid))
            .unwrap_or_default()
            .trim()
            .to_string();

        Ok(ProcessInfo {
            pid: self.pid,
            name,
            path: String::new(),
            architecture: Architecture::Unknown,
            memory_usage: 0,
            is_elevated: false,
        })
    }

    fn terminate(&self) -> Result<()> {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid as NixPid;

        kill(NixPid::from_raw(self.pid as i32), Signal::SIGTERM)
            .map_err(|e| InspectraError::process(format!("Failed to terminate: {}", e)))?;

        Ok(())
    }
}
