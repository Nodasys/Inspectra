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
    pub fn scan(
        &mut self,
        value: Option<&[u8]>,
        range: Option<(f64, f64)>,
    ) -> Result<Vec<ScanResult>> {
        let regions = self.memory.query_regions()?;

        // Filter regions based on writable_only
        let regions_to_scan: Vec<_> = regions
            .into_iter()
            .filter(|r| !self.config.writable_only || r.protection.write)
            .collect();

        // Use threading for large scans
        if self.config.thread_count > 1 && regions_to_scan.len() > 1 {
            self.scan_parallel(&regions_to_scan, value, range)
        } else {
            self.scan_sequential(&regions_to_scan, value, range)
        }
    }

    /// Sequential scan (single-threaded)
    fn scan_sequential(
        &mut self,
        regions: &[crate::types::MemoryRegion],
        value: Option<&[u8]>,
        range: Option<(f64, f64)>,
    ) -> Result<Vec<ScanResult>> {
        let mut results = Vec::new();

        for region in regions {
            if let Ok(data) = self.memory.read(region.base_address, region.size) {
                let region_results =
                    self.scan_region_data(&data, region.base_address, value, range);
                results.extend(region_results);
            }
        }

        self.results = results.clone();
        Ok(results)
    }

    /// Parallel scan (multi-threaded)
    fn scan_parallel(
        &mut self,
        regions: &[crate::types::MemoryRegion],
        value: Option<&[u8]>,
        range: Option<(f64, f64)>,
    ) -> Result<Vec<ScanResult>> {
        use std::sync::mpsc;
        use std::thread;

        let (tx, rx) = mpsc::channel();
        let memory = Arc::clone(&self.memory);
        let config = self.config.clone();
        let value_clone = value.map(|v| v.to_vec());

        // Split regions into chunks for each thread
        let chunk_size = regions.len().div_ceil(self.config.thread_count);
        let mut handles = Vec::new();

        for chunk in regions.chunks(chunk_size) {
            let tx = tx.clone();
            let memory = Arc::clone(&memory);
            let config = config.clone();
            let value_clone = value_clone.clone();
            let chunk = chunk.to_vec();

            let handle = thread::spawn(move || {
                let mut chunk_results = Vec::new();

                for region in chunk {
                    if let Ok(data) = memory.read(region.base_address, region.size) {
                        let region_results = Self::scan_region_data_static(
                            &data,
                            region.base_address,
                            &config,
                            value_clone.as_deref(),
                            range,
                        );
                        chunk_results.extend(region_results);
                    }
                }

                tx.send(chunk_results).ok();
            });

            handles.push(handle);
        }

        drop(tx); // Close sender so receiver knows when done

        // Collect results from all threads
        let mut results = Vec::new();
        for handle in handles {
            handle.join().ok();
        }

        // Collect all results from channel
        while let Ok(chunk_results) = rx.recv() {
            results.extend(chunk_results);
        }

        self.results = results.clone();
        Ok(results)
    }

    /// Scan a single region's data
    fn scan_region_data(
        &self,
        data: &[u8],
        base_address: usize,
        value: Option<&[u8]>,
        range: Option<(f64, f64)>,
    ) -> Vec<ScanResult> {
        match self.config.scan_type {
            ScanType::Unknown => self.scan_buffer_unknown(data, base_address),
            ScanType::Exact => {
                if let Some(val) = value {
                    self.scan_buffer(data, base_address, val)
                } else {
                    Vec::new()
                }
            }
            ScanType::Range => {
                if let Some((min, max)) = range {
                    self.scan_buffer_range(data, base_address, min, max)
                } else {
                    Vec::new()
                }
            }
            ScanType::Bigger | ScanType::Smaller => {
                if let Some(val) = value {
                    self.scan_buffer_comparison(data, base_address, val)
                } else {
                    Vec::new()
                }
            }
            _ => {
                if let Some(val) = value {
                    self.scan_buffer(data, base_address, val)
                } else {
                    Vec::new()
                }
            }
        }
    }

    /// Static version for threading
    fn scan_region_data_static(
        data: &[u8],
        base_address: usize,
        config: &ScanConfig,
        value: Option<&[u8]>,
        range: Option<(f64, f64)>,
    ) -> Vec<ScanResult> {
        // Create a temporary scanner instance for scanning
        // This is a workaround since we can't easily pass &self to threads
        match config.scan_type {
            ScanType::Unknown => Self::scan_buffer_unknown_static(data, base_address, config),
            ScanType::Exact => {
                if let Some(val) = value {
                    Self::scan_buffer_static(data, base_address, val, config)
                } else {
                    Vec::new()
                }
            }
            ScanType::Range => {
                if let Some((min, max)) = range {
                    Self::scan_buffer_range_static(data, base_address, min, max, config)
                } else {
                    Vec::new()
                }
            }
            ScanType::Bigger | ScanType::Smaller => {
                if let Some(val) = value {
                    Self::scan_buffer_comparison_static(data, base_address, val, config)
                } else {
                    Vec::new()
                }
            }
            _ => {
                if let Some(val) = value {
                    Self::scan_buffer_static(data, base_address, val, config)
                } else {
                    Vec::new()
                }
            }
        }
    }

    /// Rescan previous results
    pub fn rescan(
        &mut self,
        value: Option<&[u8]>,
        range: Option<(f64, f64)>,
    ) -> Result<Vec<ScanResult>> {
        let mut new_results = Vec::new();
        let value_size = value
            .map(|v| v.len())
            .unwrap_or_else(|| match self.config.data_type {
                DataType::I8 | DataType::U8 => 1,
                DataType::I16 | DataType::U16 => 2,
                DataType::I32 | DataType::U32 | DataType::F32 => 4,
                DataType::I64 | DataType::U64 | DataType::F64 => 8,
                DataType::String | DataType::WString | DataType::Bytes => 16,
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
                    ScanType::Changed => self.compare_value(&data, &result.value, None, None),
                    ScanType::Unchanged => self.compare_value(&data, &result.value, None, None),
                    ScanType::Increased => self.compare_value(&data, &result.value, None, None),
                    ScanType::Decreased => self.compare_value(&data, &result.value, None, None),
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
                    new_results.push(ScanResult::new(result.address, data, result.data_type));
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
    fn scan_buffer_comparison(
        &self,
        buffer: &[u8],
        base_address: usize,
        target: &[u8],
    ) -> Vec<ScanResult> {
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
    fn scan_buffer_range(
        &self,
        buffer: &[u8],
        base_address: usize,
        min: f64,
        max: f64,
    ) -> Vec<ScanResult> {
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
    fn compare_value(
        &self,
        current: &[u8],
        target: &[u8],
        min: Option<f64>,
        max: Option<f64>,
    ) -> bool {
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
            DataType::I8 if !bytes.is_empty() => i8::from_le_bytes([bytes[0]]) as f64,
            DataType::U8 if !bytes.is_empty() => bytes[0] as f64,
            DataType::I16 if bytes.len() >= 2 => i16::from_le_bytes([bytes[0], bytes[1]]) as f64,
            DataType::U16 if bytes.len() >= 2 => u16::from_le_bytes([bytes[0], bytes[1]]) as f64,
            DataType::I32 if bytes.len() >= 4 => {
                i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64
            }
            DataType::U32 if bytes.len() >= 4 => {
                u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64
            }
            DataType::I64 if bytes.len() >= 8 => i64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]) as f64,
            DataType::U64 if bytes.len() >= 8 => u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]) as f64,
            DataType::F32 if bytes.len() >= 4 => {
                f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64
            }
            DataType::F64 if bytes.len() >= 8 => f64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]),
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

    // Static versions for threading

    fn scan_buffer_static(
        buffer: &[u8],
        base_address: usize,
        value: &[u8],
        config: &ScanConfig,
    ) -> Vec<ScanResult> {
        let mut results = Vec::new();
        let alignment = if config.aligned {
            match config.data_type {
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
                    config.data_type,
                ));
            }
            i += alignment;
        }

        results
    }

    fn scan_buffer_unknown_static(
        buffer: &[u8],
        base_address: usize,
        config: &ScanConfig,
    ) -> Vec<ScanResult> {
        let mut results = Vec::new();
        let size = match config.data_type {
            DataType::I8 | DataType::U8 => 1,
            DataType::I16 | DataType::U16 => 2,
            DataType::I32 | DataType::U32 | DataType::F32 => 4,
            DataType::I64 | DataType::U64 | DataType::F64 => 8,
            DataType::String | DataType::WString | DataType::Bytes => 16,
        };

        let alignment = if config.aligned {
            match config.data_type {
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
            results.push(ScanResult::new(base_address + i, value, config.data_type));
            i += alignment;
        }

        results
    }

    fn scan_buffer_comparison_static(
        buffer: &[u8],
        base_address: usize,
        target: &[u8],
        config: &ScanConfig,
    ) -> Vec<ScanResult> {
        let mut results = Vec::new();
        let size = match config.data_type {
            DataType::I8 | DataType::U8 => 1,
            DataType::I16 | DataType::U16 => 2,
            DataType::I32 | DataType::U32 | DataType::F32 => 4,
            DataType::I64 | DataType::U64 | DataType::F64 => 8,
            _ => return results,
        };

        let alignment = if config.aligned {
            match config.data_type {
                DataType::I16 | DataType::U16 => 2,
                DataType::I32 | DataType::U32 | DataType::F32 => 4,
                DataType::I64 | DataType::U64 | DataType::F64 => 8,
                _ => 1,
            }
        } else {
            1
        };

        let target_val = Self::bytes_to_f64(target, &config.data_type);
        let mut i = 0;
        while i + size <= buffer.len() {
            let value = buffer[i..i + size].to_vec();
            let current_val = Self::bytes_to_f64(&value, &config.data_type);

            let matches = match config.scan_type {
                ScanType::Bigger => current_val > target_val,
                ScanType::Smaller => current_val < target_val,
                ScanType::Increased => current_val > target_val,
                ScanType::Decreased => current_val < target_val,
                _ => false,
            };

            if matches {
                results.push(ScanResult::new(base_address + i, value, config.data_type));
            }
            i += alignment;
        }

        results
    }

    fn scan_buffer_range_static(
        buffer: &[u8],
        base_address: usize,
        min: f64,
        max: f64,
        config: &ScanConfig,
    ) -> Vec<ScanResult> {
        let mut results = Vec::new();
        let size = match config.data_type {
            DataType::I8 | DataType::U8 => 1,
            DataType::I16 | DataType::U16 => 2,
            DataType::I32 | DataType::U32 | DataType::F32 => 4,
            DataType::I64 | DataType::U64 | DataType::F64 => 8,
            _ => return results,
        };

        let alignment = if config.aligned {
            match config.data_type {
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
            let val = Self::bytes_to_f64(&value, &config.data_type);
            if val >= min && val <= max {
                results.push(ScanResult::new(base_address + i, value, config.data_type));
            }
            i += alignment;
        }

        results
    }

    fn bytes_to_f64(bytes: &[u8], data_type: &DataType) -> f64 {
        match data_type {
            DataType::I8 if !bytes.is_empty() => i8::from_le_bytes([bytes[0]]) as f64,
            DataType::U8 if !bytes.is_empty() => bytes[0] as f64,
            DataType::I16 if bytes.len() >= 2 => i16::from_le_bytes([bytes[0], bytes[1]]) as f64,
            DataType::U16 if bytes.len() >= 2 => u16::from_le_bytes([bytes[0], bytes[1]]) as f64,
            DataType::I32 if bytes.len() >= 4 => {
                i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64
            }
            DataType::U32 if bytes.len() >= 4 => {
                u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64
            }
            DataType::I64 if bytes.len() >= 8 => i64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]) as f64,
            DataType::U64 if bytes.len() >= 8 => u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]) as f64,
            DataType::F32 if bytes.len() >= 4 => {
                f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64
            }
            DataType::F64 if bytes.len() >= 8 => f64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]),
            _ => 0.0,
        }
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
    use crate::error::{InspectraError, Result};
    use crate::memory;
    use crate::memory::Memory;
    use crate::process;
    use crate::types::{MemoryRegion, Protection, RegionType};
    use std::sync::{Arc, Mutex};

    const BASE_ADDRESS: usize = 0x1000;

    struct MockMemory {
        data: Arc<Mutex<Vec<u8>>>,
        protection: Protection,
    }

    impl MockMemory {
        fn new(data: Vec<u8>, protection: Protection) -> Self {
            Self {
                data: Arc::new(Mutex::new(data)),
                protection,
            }
        }

        fn region(&self) -> MemoryRegion {
            MemoryRegion {
                base_address: BASE_ADDRESS,
                size: self.data.lock().unwrap().len(),
                protection: self.protection,
                region_type: RegionType::Private,
                module_name: Some("mock".to_string()),
            }
        }
    }

    impl Memory for MockMemory {
        fn read(&self, address: usize, size: usize) -> Result<Vec<u8>> {
            let data = self.data.lock().unwrap();
            let offset = address
                .checked_sub(BASE_ADDRESS)
                .ok_or(InspectraError::InvalidAddress(address))?;
            let end = offset + size;

            if end > data.len() {
                return Err(InspectraError::memory("Read exceeds mock memory"));
            }

            Ok(data[offset..end].to_vec())
        }

        fn write(&self, address: usize, bytes: &[u8]) -> Result<usize> {
            let mut data = self.data.lock().unwrap();
            let offset = address
                .checked_sub(BASE_ADDRESS)
                .ok_or(InspectraError::InvalidAddress(address))?;
            let end = offset + bytes.len();

            if end > data.len() {
                return Err(InspectraError::memory("Write exceeds mock memory"));
            }

            data[offset..end].copy_from_slice(bytes);
            Ok(bytes.len())
        }

        fn query_regions(&self) -> Result<Vec<MemoryRegion>> {
            Ok(vec![self.region()])
        }

        fn query_region(&self, address: usize) -> Result<MemoryRegion> {
            let region = self.region();
            if address >= region.base_address && address < region.base_address + region.size {
                Ok(region)
            } else {
                Err(InspectraError::InvalidAddress(address))
            }
        }

        fn protect(
            &self,
            _address: usize,
            _size: usize,
            _protection: Protection,
        ) -> Result<Protection> {
            Ok(self.protection)
        }

        fn allocate(&self, _size: usize, _protection: Protection) -> Result<usize> {
            Err(InspectraError::memory("Mock allocation is not supported"))
        }

        fn free(&self, _address: usize) -> Result<()> {
            Ok(())
        }
    }

    fn i32_buffer(values: &[i32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

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

    #[test]
    fn exact_i32_scan_finds_aligned_matches() {
        let mem = MockMemory::new(i32_buffer(&[7, 42, 13, 42]), Protection::read_write());
        let config = ScanConfig {
            data_type: DataType::I32,
            thread_count: 1,
            aligned: true,
            ..Default::default()
        };
        let mut scanner = Scanner::new(Box::new(mem), config);

        let needle = 42_i32.to_le_bytes();
        let results = scanner.scan(Some(&needle), None).unwrap();
        let addresses: Vec<_> = results.iter().map(|result| result.address).collect();

        assert_eq!(addresses, vec![BASE_ADDRESS + 4, BASE_ADDRESS + 12]);
    }

    #[test]
    fn range_i32_scan_filters_numeric_values() {
        let mem = MockMemory::new(i32_buffer(&[1, 5, 10, 15]), Protection::read_write());
        let config = ScanConfig {
            data_type: DataType::I32,
            scan_type: ScanType::Range,
            thread_count: 1,
            aligned: true,
            ..Default::default()
        };
        let mut scanner = Scanner::new(Box::new(mem), config);

        let results = scanner.scan(None, Some((4.0, 12.0))).unwrap();
        let values: Vec<_> = results
            .iter()
            .map(|result| i32::from_le_bytes(result.value.clone().try_into().unwrap()))
            .collect();

        assert_eq!(values, vec![5, 10]);
    }

    #[test]
    fn writable_only_scan_skips_read_only_regions() {
        let mem = MockMemory::new(i32_buffer(&[42]), Protection::read_only());
        let config = ScanConfig {
            data_type: DataType::I32,
            writable_only: true,
            thread_count: 1,
            aligned: true,
            ..Default::default()
        };
        let mut scanner = Scanner::new(Box::new(mem), config);

        let needle = 42_i32.to_le_bytes();
        let results = scanner.scan(Some(&needle), None).unwrap();

        assert!(results.is_empty());
    }
}
