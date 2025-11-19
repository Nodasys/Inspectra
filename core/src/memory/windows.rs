//! Windows memory implementation

use super::Memory;
use crate::error::{InspectraError, Result};
use crate::process::ProcessHandle;
use crate::types::{Address, MemoryRegion, Protection, RegionType, Size, Pid};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, VirtualProtectEx, VirtualQueryEx, MEMORY_BASIC_INFORMATION,
    MEM_COMMIT, MEM_IMAGE, MEM_MAPPED, MEM_PRIVATE, MEM_RELEASE, MEM_RESERVE,
    PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_NOACCESS, PAGE_READONLY,
    PAGE_READWRITE, PAGE_PROTECTION_FLAGS,
};
use windows::Win32::Foundation::HANDLE;
use std::sync::Mutex;

pub struct WindowsMemory {
    pid: Pid,
    handle: Mutex<HANDLE>,
}

impl WindowsMemory {
    pub fn new(process: &dyn ProcessHandle) -> Result<Self> {
        // Get the raw handle from the process
        let raw_handle = process.as_raw_handle()
            .ok_or_else(|| InspectraError::memory("Failed to get process handle"))?;
        
        let pid = process.pid();
        
        // Use the handle directly - we'll recreate it if needed
        Ok(Self {
            pid,
            handle: Mutex::new(unsafe { HANDLE(raw_handle as isize) }),
        })
    }
    
    fn ensure_handle(&self) -> Result<HANDLE> {
        // Check if handle is still valid, if not recreate it
        let mut handle = self.handle.lock().unwrap();
        unsafe {
            use windows::Win32::System::Threading::OpenProcess;
            use windows::Win32::System::Threading::{PROCESS_VM_READ, PROCESS_VM_WRITE, PROCESS_VM_OPERATION, PROCESS_QUERY_INFORMATION};
            
            // Try to open process again if handle is invalid
            if handle.is_invalid() {
                let new_handle = OpenProcess(
                    PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_QUERY_INFORMATION,
                    false,
                    self.pid,
                ).map_err(|e| InspectraError::memory(format!("Failed to reopen process: {}", e)))?;
                *handle = new_handle;
            }
        }
        Ok(*handle)
    }

    fn convert_protection(prot: Protection) -> PAGE_PROTECTION_FLAGS {
        match (prot.read, prot.write, prot.execute) {
            (false, false, false) => PAGE_NOACCESS,
            (true, false, false) => PAGE_READONLY,
            (true, true, false) => PAGE_READWRITE,
            (true, false, true) => PAGE_EXECUTE_READ,
            (true, true, true) => PAGE_EXECUTE_READWRITE,
            (false, false, true) => PAGE_EXECUTE,
            _ => PAGE_NOACCESS,
        }
    }

    fn parse_protection(prot: PAGE_PROTECTION_FLAGS) -> Protection {
        let read = prot == PAGE_READONLY
            || prot == PAGE_READWRITE
            || prot == PAGE_EXECUTE_READ
            || prot == PAGE_EXECUTE_READWRITE;
        
        let write = prot == PAGE_READWRITE || prot == PAGE_EXECUTE_READWRITE;
        
        let execute = prot == PAGE_EXECUTE
            || prot == PAGE_EXECUTE_READ
            || prot == PAGE_EXECUTE_READWRITE;

        Protection { read, write, execute }
    }
}

impl Memory for WindowsMemory {
    fn read(&self, address: Address, size: Size) -> Result<Vec<u8>> {
        let handle = self.ensure_handle()?;
        
        unsafe {
            let mut buffer = vec![0u8; size];
            let mut bytes_read = 0;

            ReadProcessMemory(
                handle,
                address as *const _,
                buffer.as_mut_ptr() as *mut _,
                size,
                Some(&mut bytes_read),
            )
            .map_err(|e| InspectraError::memory(format!("Read failed: {}", e)))?;

            buffer.truncate(bytes_read);
            Ok(buffer)
        }
    }

    fn write(&self, address: Address, data: &[u8]) -> Result<usize> {
        let handle = self.ensure_handle()?;
        
        unsafe {
            let mut bytes_written = 0;

            WriteProcessMemory(
                handle,
                address as *const _,
                data.as_ptr() as *const _,
                data.len(),
                Some(&mut bytes_written),
            )
            .map_err(|e| InspectraError::memory(format!("Write failed: {}", e)))?;

            Ok(bytes_written)
        }
    }

    fn query_regions(&self) -> Result<Vec<MemoryRegion>> {
        let handle = self.ensure_handle()?;
        
        let mut regions = Vec::new();
        let mut address: usize = 0;

        unsafe {
            loop {
                let mut mbi = MEMORY_BASIC_INFORMATION::default();
                let result = VirtualQueryEx(
                    handle,
                    Some(address as *const _),
                    &mut mbi,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                );

                if result == 0 {
                    break;
                }

                if mbi.State == MEM_COMMIT {
                    let region_type = if mbi.Type == MEM_PRIVATE {
                        RegionType::Private
                    } else if mbi.Type == MEM_MAPPED {
                        RegionType::Mapped
                    } else if mbi.Type == MEM_IMAGE {
                        RegionType::Image
                    } else {
                        RegionType::Unknown
                    };

                    regions.push(MemoryRegion {
                        base_address: mbi.BaseAddress as usize,
                        size: mbi.RegionSize,
                        protection: Self::parse_protection(mbi.Protect),
                        region_type,
                        module_name: None,
                    });
                }

                address = (mbi.BaseAddress as usize) + mbi.RegionSize;
                if address == 0 {
                    break;
                }
            }
        }

        Ok(regions)
    }

    fn query_region(&self, address: Address) -> Result<MemoryRegion> {
        let handle = self.ensure_handle()?;
        
        unsafe {
            let mut mbi = MEMORY_BASIC_INFORMATION::default();
            let result = VirtualQueryEx(
                handle,
                Some(address as *const _),
                &mut mbi,
                std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            );

            if result == 0 {
                return Err(InspectraError::InvalidAddress(address));
            }

            let region_type = if mbi.Type == MEM_PRIVATE {
                RegionType::Private
            } else if mbi.Type == MEM_MAPPED {
                RegionType::Mapped
            } else if mbi.Type == MEM_IMAGE {
                RegionType::Image
            } else {
                RegionType::Unknown
            };

            Ok(MemoryRegion {
                base_address: mbi.BaseAddress as usize,
                size: mbi.RegionSize,
                protection: Self::parse_protection(mbi.Protect),
                region_type,
                module_name: None,
            })
        }
    }

    fn protect(&self, address: Address, size: Size, protection: Protection) -> Result<Protection> {
        let handle = self.ensure_handle()?;
        
        unsafe {
            let new_protect = Self::convert_protection(protection);
            let mut old_protect = PAGE_PROTECTION_FLAGS::default();

            VirtualProtectEx(
                handle,
                address as *const _,
                size,
                new_protect,
                &mut old_protect,
            )
            .map_err(|e| InspectraError::memory(format!("Protection change failed: {}", e)))?;

            Ok(Self::parse_protection(old_protect))
        }
    }

    fn allocate(&self, size: Size, protection: Protection) -> Result<Address> {
        let handle = self.ensure_handle()?;
        
        unsafe {
            let protect = Self::convert_protection(protection);
            let address = VirtualAllocEx(
                handle,
                None,
                size,
                MEM_COMMIT | MEM_RESERVE,
                protect,
            );

            if address.is_null() {
                return Err(InspectraError::memory("Allocation failed"));
            }

            Ok(address as Address)
        }
    }

    fn free(&self, address: Address) -> Result<()> {
        let handle = self.ensure_handle()?;
        
        unsafe {
            VirtualFreeEx(handle, address as *mut _, 0, MEM_RELEASE)
                .map_err(|e| InspectraError::memory(format!("Free failed: {}", e)))?;
            Ok(())
        }
    }
}
