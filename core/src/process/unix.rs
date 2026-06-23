//! Unix-specific process implementation

use super::{ProcessHandle, ProcessInfo, ProcessManager};
use crate::error::{InspectraError, Result};
use crate::types::{Architecture, Pid};
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct UnixProcessManager;

impl UnixProcessManager {
    pub fn new() -> Self {
        Self
    }

    fn list_procfs_processes() -> Vec<ProcessInfo> {
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

        processes
    }

    fn list_ps_processes() -> Result<Vec<ProcessInfo>> {
        let output = Command::new("ps")
            .args(["-axo", "pid=,comm="])
            .output()
            .map_err(|e| InspectraError::process(format!("Failed to run ps: {}", e)))?;

        if !output.status.success() {
            return Err(InspectraError::process(format!(
                "ps exited with status {}",
                output.status
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                let mut parts = line.trim().splitn(2, char::is_whitespace);
                let pid = parts.next()?.parse::<Pid>().ok()?;
                let path = parts.next().unwrap_or_default().trim().to_string();
                let name = Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path)
                    .to_string();

                Some(ProcessInfo {
                    pid,
                    name,
                    path,
                    architecture: Architecture::Unknown,
                    memory_usage: 0,
                    is_elevated: false,
                })
            })
            .collect())
    }

    fn process_exists(pid: Pid) -> bool {
        if Path::new("/proc").exists() {
            return Path::new(&format!("/proc/{}", pid)).exists();
        }

        let result = unsafe { libc::kill(pid as i32, 0) };
        if result == 0 {
            return true;
        }

        std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
    }

    fn process_name(pid: Pid) -> String {
        if Path::new("/proc").exists() {
            return fs::read_to_string(format!("/proc/{}/comm", pid))
                .unwrap_or_default()
                .trim()
                .to_string();
        }

        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output();

        output
            .ok()
            .filter(|out| out.status.success())
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .unwrap_or_default()
    }
}

impl Default for UnixProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessManager for UnixProcessManager {
    fn list_processes(&self) -> Result<Vec<ProcessInfo>> {
        if Path::new("/proc").exists() {
            return Ok(Self::list_procfs_processes());
        }

        Self::list_ps_processes()
    }

    fn attach(&self, pid: Pid) -> Result<Box<dyn ProcessHandle>> {
        if !Self::process_exists(pid) {
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
        UnixProcessManager::process_exists(self.pid)
    }

    fn info(&self) -> Result<ProcessInfo> {
        let name = UnixProcessManager::process_name(self.pid);

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
