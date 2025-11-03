//! Common types used throughout Inspectra

use serde::{Deserialize, Serialize};

/// Process ID type
pub type Pid = u32;

/// Memory address type
pub type Address = usize;

/// Memory size type
pub type Size = usize;

/// Data types supported by the scanner
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    /// 8-bit signed integer
    I8,
    /// 8-bit unsigned integer
    U8,
    /// 16-bit signed integer
    I16,
    /// 16-bit unsigned integer
    U16,
    /// 32-bit signed integer
    I32,
    /// 32-bit unsigned integer
    U32,
    /// 64-bit signed integer
    I64,
    /// 64-bit unsigned integer
    U64,
    /// 32-bit floating point
    F32,
    /// 64-bit floating point
    F64,
    /// UTF-8 string
    String,
    /// UTF-16 string
    WString,
    /// Array of bytes
    Bytes,
}

/// Memory protection flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Protection {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl Protection {
    pub fn new(read: bool, write: bool, execute: bool) -> Self {
        Self { read, write, execute }
    }

    pub fn read_only() -> Self {
        Self::new(true, false, false)
    }

    pub fn read_write() -> Self {
        Self::new(true, true, false)
    }

    pub fn read_execute() -> Self {
        Self::new(true, false, true)
    }

    pub fn all() -> Self {
        Self::new(true, true, true)
    }
}

/// Memory region information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegion {
    pub base_address: Address,
    pub size: Size,
    pub protection: Protection,
    pub region_type: RegionType,
    pub module_name: Option<String>,
}

/// Type of memory region
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegionType {
    Private,
    Mapped,
    Image,
    Unknown,
}

/// Architecture type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Architecture {
    X86,
    X64,
    ARM,
    ARM64,
    Unknown,
}

/// Scan comparison type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanType {
    Exact,
    Range,
    Increased,
    Decreased,
    Changed,
    Unchanged,
    Unknown,
}

/// Scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub address: Address,
    pub value: Vec<u8>,
    pub data_type: DataType,
}

impl ScanResult {
    pub fn new(address: Address, value: Vec<u8>, data_type: DataType) -> Self {
        Self {
            address,
            value,
            data_type,
        }
    }
}
