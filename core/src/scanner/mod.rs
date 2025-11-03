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
    pub fn scan(&mut self, value: &[u8]) -> Result<Vec<ScanResult>> {
        let regions = self.memory.query_regions()?;
        let mut results = Vec::new();

        for region in regions {
            if self.config.writable_only && !region.protection.write {
                continue;
            }

            if let Ok(data) = self.memory.read(region.base_address, region.size) {
                let region_results = self.scan_buffer(&data, region.base_address, value);
                results.extend(region_results);
            }
        }

        self.results = results.clone();
        Ok(results)
    }

    /// Rescan previous results
    pub fn rescan(&mut self, value: &[u8]) -> Result<Vec<ScanResult>> {
        let mut new_results = Vec::new();

        for result in &self.results {
            if let Ok(data) = self
                .memory
                .read(result.address, value.len())
            {
                if self.compare_value(&data, value) {
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

    /// Compare values based on scan type
    fn compare_value(&self, current: &[u8], target: &[u8]) -> bool {
        match self.config.scan_type {
            ScanType::Exact => current == target,
            ScanType::Changed => current != target,
            ScanType::Unchanged => current == target,
            _ => false, // Other scan types need more context
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
}

// Add num_cpus dependency
use std::num::NonZeroUsize;

fn num_cpus() -> NonZeroUsize {
    std::thread::available_parallelism().unwrap_or(NonZeroUsize::new(4).unwrap())
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
