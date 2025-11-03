//! Windows-specific process implementation

use super::{ProcessHandle, ProcessInfo, ProcessManager};
use crate::error::{InspectraError, Result};
use crate::types::{Architecture, Pid};
use std::sync::Arc;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
    TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, TerminateProcess, PROCESS_QUERY_INFORMATION,
    PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
};

pub struct WindowsProcessManager;

impl WindowsProcessManager {
    pub fn new() -> Self {
        Self
    }
}

impl ProcessManager for WindowsProcessManager {
    fn list_processes(&self) -> Result<Vec<ProcessInfo>> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
                .map_err(|e| InspectraError::process(format!("Failed to create snapshot: {}", e)))?;

            let mut processes = Vec::new();
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    let name = String::from_utf16_lossy(
                        &entry.szExeFile[..entry
                            .szExeFile
                            .iter()
                            .position(|&c| c == 0)
                            .unwrap_or(entry.szExeFile.len())],
                    );

                    processes.push(ProcessInfo {
                        pid: entry.th32ProcessID,
                        name: name.clone(),
                        path: String::new(), // Would need additional API call
                        architecture: Architecture::Unknown,
                        memory_usage: 0,
                        is_elevated: false,
                    });

                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }

            let _ = CloseHandle(snapshot);
            Ok(processes)
        }
    }

    fn attach(&self, pid: Pid) -> Result<Box<dyn ProcessHandle>> {
        unsafe {
            let handle = OpenProcess(
                PROCESS_VM_READ
                    | PROCESS_VM_WRITE
                    | PROCESS_VM_OPERATION
                    | PROCESS_QUERY_INFORMATION,
                false,
                pid,
            )
            .map_err(|e| InspectraError::process(format!("Failed to open process: {}", e)))?;

            Ok(Box::new(WindowsProcessHandle {
                pid,
                handle: Arc::new(SafeHandle(handle)),
            }))
        }
    }

    fn find_by_name(&self, name: &str) -> Result<Vec<ProcessInfo>> {
        let all_processes = self.list_processes()?;
        Ok(all_processes
            .into_iter()
            .filter(|p| p.name.to_lowercase().contains(&name.to_lowercase()))
            .collect())
    }
}

struct SafeHandle(HANDLE);

impl Drop for SafeHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

unsafe impl Send for SafeHandle {}
unsafe impl Sync for SafeHandle {}

pub struct WindowsProcessHandle {
    pid: Pid,
    handle: Arc<SafeHandle>,
}

impl ProcessHandle for WindowsProcessHandle {
    fn pid(&self) -> Pid {
        self.pid
    }

    fn is_alive(&self) -> bool {
        // Check if handle is still valid
        self.handle.0.is_invalid() == false
    }

    fn info(&self) -> Result<ProcessInfo> {
        Ok(ProcessInfo {
            pid: self.pid,
            name: String::new(),
            path: String::new(),
            architecture: Architecture::Unknown,
            memory_usage: 0,
            is_elevated: false,
        })
    }

    fn terminate(&self) -> Result<()> {
        unsafe {
            TerminateProcess(self.handle.0, 1)
                .map_err(|e| InspectraError::process(format!("Failed to terminate: {}", e)))?;
            Ok(())
        }
    }
}
