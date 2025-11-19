//! Memory scanner module

use crate::error::Result;
use crate::memory::Memory;
use crate::types::{DataType, ScanResult, ScanType};
use std::sync::Arc;

mod engine;
mod patterns;

pub use engine::*;
pub use patterns::*;

/// Scanner configuration
#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub data_type: DataType,
    pub scan_type: ScanType,
    pub thread_count: usize,
    pub writable_only: bool,
    pub aligned: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            data_type: DataType::I32,
            scan_type: ScanType::Exact,
            thread_count: num_cpus::get(),
            writable_only: false,
            aligned: false,
        }
    }
}

/// Memory scanner for finding values in process memory
pub struct Scanner {
    memory: Arc<Box<dyn Memory>>,
    config: ScanConfig,
    results: Vec<ScanResult>,
}

impl Scanner {
    /// Create a new scanner
    pub fn new(memory: Box<dyn Memory>, config: ScanConfig) -> Self {
        Self {
            memory: Arc::new(memory),
            config,
            results: Vec::new(),
        }
    }

    /// Perform initial scan
    pub fn scan(&mut self, value: Option<&[u8]>, range: Option<(f64, f64)>) -> Result<Vec<ScanResult>> {
        let regions = self.memory.query_regions()?;
        let mut results = Vec::new();

        for region in regions {
            if self.config.writable_only && !region.protection.write {
                continue;
            }

            if let Ok(data) = self.memory.read(region.base_address, region.size) {
                let region_results = match self.config.scan_type {
                    ScanType::Unknown => {
                        // For unknown initial value, we scan all values
                        self.scan_buffer_unknown(&data, region.base_address)
                    }
                    ScanType::Exact => {
                        if let Some(val) = value {
                            self.scan_buffer(&data, region.base_address, val)
                        } else {
                            continue;
                        }
                    }
                    ScanType::Range => {
                        if let Some((min, max)) = range {
                            self.scan_buffer_range(&data, region.base_address, min, max)
                        } else {
                            continue;
                        }
                    }
                    ScanType::Bigger | ScanType::Smaller => {
                        if let Some(val) = value {
                            self.scan_buffer_comparison(&data, region.base_address, val)
                        } else {
                            continue;
                        }
                    }
                    _ => {
                        // For other types, we need a value to compare
                        if let Some(val) = value {
                            self.scan_buffer(&data, region.base_address, val)
                        } else {
                            continue;
                        }
                    }
                };
                results.extend(region_results);
            }
        }

        self.results = results.clone();
        Ok(results)
    }

    /// Rescan previous results
    pub fn rescan(&mut self, value: Option<&[u8]>, range: Option<(f64, f64)>) -> Result<Vec<ScanResult>> {
        let mut new_results = Vec::new();
        let value_size = value.map(|v| v.len()).unwrap_or_else(|| {
            match self.config.data_type {
                DataType::I8 | DataType::U8 => 1,
                DataType::I16 | DataType::U16 => 2,
                DataType::I32 | DataType::U32 | DataType::F32 => 4,
                DataType::I64 | DataType::U64 | DataType::F64 => 8,
                DataType::String | DataType::WString | DataType::Bytes => 16,
            }
        });

        for result in &self.results {
            if let Ok(data) = self.memory.read(result.address, value_size) {
                let matches = match self.config.scan_type {
                    ScanType::Exact => {
                        if let Some(val) = value {
                            self.compare_value(&data, val, None, None)
                        } else {
                            false
                        }
                    }
                    ScanType::Range => {
                        if let Some((min, max)) = range {
                            self.compare_value(&data, &[], Some(min), Some(max))
                        } else {
                            false
                        }
                    }
                    ScanType::Changed => {
                        self.compare_value(&data, &result.value, None, None)
                    }
                    ScanType::Unchanged => {
                        self.compare_value(&data, &result.value, None, None)
                    }
                    ScanType::Increased => {
                        self.compare_value(&data, &result.value, None, None)
                    }
                    ScanType::Decreased => {
                        self.compare_value(&data, &result.value, None, None)
                    }
                    ScanType::Bigger => {
                        if let Some(val) = value {
                            self.compare_value(&data, val, None, None)
                        } else {
                            false
                        }
                    }
                    ScanType::Smaller => {
                        if let Some(val) = value {
                            self.compare_value(&data, val, None, None)
                        } else {
                            false
                        }
                    }
                    ScanType::Unknown => true, // Unknown initial value accepts all
                };

                if matches {
                    new_results.push(ScanResult::new(
                        result.address,
                        data,
                        result.data_type,
                    ));
                }
            }
        }

        self.results = new_results.clone();
        Ok(new_results)
    }

    /// Scan a buffer for matching values
    fn scan_buffer(&self, buffer: &[u8], base_address: usize, value: &[u8]) -> Vec<ScanResult> {
        let mut results = Vec::new();
        let alignment = if self.config.aligned {
            match self.config.data_type {
                DataType::I16 | DataType::U16 => 2,
                DataType::I32 | DataType::U32 | DataType::F32 => 4,
                DataType::I64 | DataType::U64 | DataType::F64 => 8,
                _ => 1,
            }
        } else {
            1
        };

        let mut i = 0;
        while i + value.len() <= buffer.len() {
            if buffer[i..i + value.len()] == *value {
                results.push(ScanResult::new(
                    base_address + i,
                    value.to_vec(),
                    self.config.data_type,
                ));
            }
            i += alignment;
        }

        results
    }

    /// Scan buffer for unknown initial values (captures all values)
    fn scan_buffer_unknown(&self, buffer: &[u8], base_address: usize) -> Vec<ScanResult> {
        let mut results = Vec::new();
        let size = match self.config.data_type {
            DataType::I8 | DataType::U8 => 1,
            DataType::I16 | DataType::U16 => 2,
            DataType::I32 | DataType::U32 | DataType::F32 => 4,
            DataType::I64 | DataType::U64 | DataType::F64 => 8,
            DataType::String | DataType::WString | DataType::Bytes => 16,
        };
        
        let alignment = if self.config.aligned {
            match self.config.data_type {
                DataType::I16 | DataType::U16 => 2,
                DataType::I32 | DataType::U32 | DataType::F32 => 4,
                DataType::I64 | DataType::U64 | DataType::F64 => 8,
                _ => 1,
            }
        } else {
            1
        };

        let mut i = 0;
        while i + size <= buffer.len() {
            let value = buffer[i..i + size].to_vec();
            results.push(ScanResult::new(
                base_address + i,
                value,
                self.config.data_type,
            ));
            i += alignment;
        }

        results
    }

    /// Scan buffer for comparison (bigger/smaller)
    fn scan_buffer_comparison(&self, buffer: &[u8], base_address: usize, target: &[u8]) -> Vec<ScanResult> {
        let mut results = Vec::new();
        let size = target.len();
        
        let alignment = if self.config.aligned {
            match self.config.data_type {
                DataType::I16 | DataType::U16 => 2,
                DataType::I32 | DataType::U32 | DataType::F32 => 4,
                DataType::I64 | DataType::U64 | DataType::F64 => 8,
                _ => 1,
            }
        } else {
            1
        };

        let mut i = 0;
        while i + size <= buffer.len() {
            let value = buffer[i..i + size].to_vec();
            if self.compare_value(&value, target, None, None) {
                results.push(ScanResult::new(
                    base_address + i,
                    value,
                    self.config.data_type,
                ));
            }
            i += alignment;
        }

        results
    }

    /// Scan buffer for range values
    fn scan_buffer_range(&self, buffer: &[u8], base_address: usize, min: f64, max: f64) -> Vec<ScanResult> {
        let mut results = Vec::new();
        let size = match self.config.data_type {
            DataType::I8 | DataType::U8 => 1,
            DataType::I16 | DataType::U16 => 2,
            DataType::I32 | DataType::U32 | DataType::F32 => 4,
            DataType::I64 | DataType::U64 | DataType::F64 => 8,
            _ => return results,
        };
        
        let alignment = if self.config.aligned {
            match self.config.data_type {
                DataType::I16 | DataType::U16 => 2,
                DataType::I32 | DataType::U32 | DataType::F32 => 4,
                DataType::I64 | DataType::U64 | DataType::F64 => 8,
                _ => 1,
            }
        } else {
            1
        };

        let mut i = 0;
        while i + size <= buffer.len() {
            let value = buffer[i..i + size].to_vec();
            if self.compare_value(&value, &[], Some(min), Some(max)) {
                results.push(ScanResult::new(
                    base_address + i,
                    value,
                    self.config.data_type,
                ));
            }
            i += alignment;
        }

        results
    }

    /// Compare values based on scan type
    fn compare_value(&self, current: &[u8], target: &[u8], min: Option<f64>, max: Option<f64>) -> bool {
        match self.config.scan_type {
            ScanType::Exact => {
                if target.is_empty() {
                    return false;
                }
                current == target
            }
            ScanType::Range => {
                if let (Some(min_val), Some(max_val)) = (min, max) {
                    let current_val = self.bytes_to_numeric(current);
                    current_val >= min_val && current_val <= max_val
                } else {
                    false
                }
            }
            ScanType::Changed => {
                if target.is_empty() {
                    return true;
                }
                current != target
            }
            ScanType::Unchanged => {
                if target.is_empty() {
                    return false;
                }
                current == target
            }
            ScanType::Increased => {
                if target.is_empty() {
                    return false;
                }
                let current_val = self.bytes_to_numeric(current);
                let target_val = self.bytes_to_numeric(target);
                current_val > target_val
            }
            ScanType::Decreased => {
                if target.is_empty() {
                    return false;
                }
                let current_val = self.bytes_to_numeric(current);
                let target_val = self.bytes_to_numeric(target);
                current_val < target_val
            }
            ScanType::Bigger => {
                if target.is_empty() {
                    return false;
                }
                let current_val = self.bytes_to_numeric(current);
                let target_val = self.bytes_to_numeric(target);
                current_val > target_val
            }
            ScanType::Smaller => {
                if target.is_empty() {
                    return false;
                }
                let current_val = self.bytes_to_numeric(current);
                let target_val = self.bytes_to_numeric(target);
                current_val < target_val
            }
            ScanType::Unknown => true,
        }
    }

    /// Convert bytes to numeric value for comparison
    fn bytes_to_numeric(&self, bytes: &[u8]) -> f64 {
        match self.config.data_type {
            DataType::I8 if bytes.len() >= 1 => i8::from_le_bytes([bytes[0]]) as f64,
            DataType::U8 if bytes.len() >= 1 => bytes[0] as f64,
            DataType::I16 if bytes.len() >= 2 => {
                i16::from_le_bytes([bytes[0], bytes[1]]) as f64
            }
            DataType::U16 if bytes.len() >= 2 => {
                u16::from_le_bytes([bytes[0], bytes[1]]) as f64
            }
            DataType::I32 if bytes.len() >= 4 => {
                i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64
            }
            DataType::U32 if bytes.len() >= 4 => {
                u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64
            }
            DataType::I64 if bytes.len() >= 8 => {
                i64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ]) as f64
            }
            DataType::U64 if bytes.len() >= 8 => {
                u64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ]) as f64
            }
            DataType::F32 if bytes.len() >= 4 => {
                f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64
            }
            DataType::F64 if bytes.len() >= 8 => {
                f64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ])
            }
            _ => 0.0,
        }
    }

    /// Get current results
    pub fn results(&self) -> &[ScanResult] {
        &self.results
    }

    /// Clear results
    pub fn clear(&mut self) {
        self.results.clear();
    }

    /// Set results (for restoring previous scan state)
    pub fn set_results(&mut self, results: Vec<ScanResult>) {
        self.results = results;
    }
}

mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory;
    use crate::process;

    #[test]
    fn test_scanner_creation() {
        let manager = process::get_process_manager();
        let current_pid = std::process::id();
        
        if let Ok(handle) = manager.attach(current_pid) {
            if let Ok(mem) = memory::create_memory(handle.as_ref()) {
                let scanner = Scanner::new(mem, ScanConfig::default());
                assert_eq!(scanner.results().len(), 0);
            }
        }
    }
}
