//! Unix memory implementation

use super::Memory;
use crate::error::{InspectraError, Result};
use crate::process::ProcessHandle;
use crate::types::{Address, MemoryRegion, Protection, RegionType, Size};
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};

pub struct UnixMemory {
    pid: u32,
}

impl UnixMemory {
    pub fn new(process: &dyn ProcessHandle) -> Result<Self> {
        Ok(Self { pid: process.pid() })
    }

    fn parse_maps_line(line: &str) -> Option<MemoryRegion> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let addr_range: Vec<&str> = parts[0].split('-').collect();
        if addr_range.len() != 2 {
            return None;
        }

        let base_address = usize::from_str_radix(addr_range[0], 16).ok()?;
        let end_address = usize::from_str_radix(addr_range[1], 16).ok()?;
        let size = end_address - base_address;

        let perms = parts.get(1)?;
        let protection = Protection {
            read: perms.contains('r'),
            write: perms.contains('w'),
            execute: perms.contains('x'),
        };

        let module_name = parts.get(5).map(|s| s.to_string());

        Some(MemoryRegion {
            base_address,
            size,
            protection,
            region_type: RegionType::Unknown,
            module_name,
        })
    }
}

impl Memory for UnixMemory {
    fn read(&self, address: Address, size: Size) -> Result<Vec<u8>> {
        let mem_path = format!("/proc/{}/mem", self.pid);
        let mut file = fs::OpenOptions::new()
            .read(true)
            .open(&mem_path)
            .map_err(|e| InspectraError::memory(format!("Cannot open mem file: {}", e)))?;

        file.seek(SeekFrom::Start(address as u64))
            .map_err(|e| InspectraError::memory(format!("Seek failed: {}", e)))?;

        let mut buffer = vec![0u8; size];
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|e| InspectraError::memory(format!("Read failed: {}", e)))?;

        buffer.truncate(bytes_read);
        Ok(buffer)
    }

    fn write(&self, address: Address, data: &[u8]) -> Result<usize> {
        let mem_path = format!("/proc/{}/mem", self.pid);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(&mem_path)
            .map_err(|e| InspectraError::memory(format!("Cannot open mem file: {}", e)))?;

        file.seek(SeekFrom::Start(address as u64))
            .map_err(|e| InspectraError::memory(format!("Seek failed: {}", e)))?;

        let bytes_written = file
            .write(data)
            .map_err(|e| InspectraError::memory(format!("Write failed: {}", e)))?;

        Ok(bytes_written)
    }

    fn query_regions(&self) -> Result<Vec<MemoryRegion>> {
        let maps_path = format!("/proc/{}/maps", self.pid);
        let content = fs::read_to_string(&maps_path)
            .map_err(|e| InspectraError::memory(format!("Cannot read maps: {}", e)))?;

        Ok(content.lines().filter_map(Self::parse_maps_line).collect())
    }

    fn query_region(&self, address: Address) -> Result<MemoryRegion> {
        let regions = self.query_regions()?;

        regions
            .into_iter()
            .find(|r| address >= r.base_address && address < r.base_address + r.size)
            .ok_or_else(|| InspectraError::InvalidAddress(address))
    }

    fn protect(
        &self,
        _address: Address,
        _size: Size,
        _protection: Protection,
    ) -> Result<Protection> {
        // mprotect would require ptrace or other system calls
        Err(InspectraError::platform(
            "Memory protection not yet implemented on Unix",
        ))
    }

    fn allocate(&self, _size: Size, _protection: Protection) -> Result<Address> {
        Err(InspectraError::platform(
            "Memory allocation not yet implemented on Unix",
        ))
    }

    fn free(&self, _address: Address) -> Result<()> {
        Err(InspectraError::platform(
            "Memory free not yet implemented on Unix",
        ))
    }
}
