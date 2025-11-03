//! Memory management and manipulation

use crate::error::Result;
use crate::process::ProcessHandle;
use crate::types::{Address, MemoryRegion, Protection, Size};

#[cfg(windows)]
mod windows;
#[cfg(unix)]
mod unix;

#[cfg(windows)]
pub use self::windows::*;
#[cfg(unix)]
pub use self::unix::*;

/// Memory reader/writer trait
pub trait Memory: Send + Sync {
    /// Read memory from process
    fn read(&self, address: Address, size: Size) -> Result<Vec<u8>>;

    /// Write memory to process
    fn write(&self, address: Address, data: &[u8]) -> Result<usize>;

    /// Query memory regions
    fn query_regions(&self) -> Result<Vec<MemoryRegion>>;

    /// Query specific memory region
    fn query_region(&self, address: Address) -> Result<MemoryRegion>;

    /// Change memory protection
    fn protect(&self, address: Address, size: Size, protection: Protection) -> Result<Protection>;

    /// Allocate memory in the target process
    fn allocate(&self, size: Size, protection: Protection) -> Result<Address>;

    /// Free allocated memory
    fn free(&self, address: Address) -> Result<()>;
}

/// Create a memory accessor for a process
pub fn create_memory(process: &dyn ProcessHandle) -> Result<Box<dyn Memory>> {
    #[cfg(windows)]
    return Ok(Box::new(WindowsMemory::new(process)?));

    #[cfg(unix)]
    return Ok(Box::new(UnixMemory::new(process)?));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process;

    #[test]
    fn test_memory_regions() {
        let manager = process::get_process_manager();
        let current_pid = std::process::id();
        
        if let Ok(handle) = manager.attach(current_pid) {
            if let Ok(memory) = create_memory(handle.as_ref()) {
                let regions = memory.query_regions().unwrap();
                assert!(!regions.is_empty());
            }
        }
    }
}
