//! Pointer scanning and analysis module

use crate::error::Result;
use crate::memory::Memory;
use crate::types::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pointer chain representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointerChain {
    pub base_address: Address,
    pub offsets: Vec<isize>,
    pub final_address: Address,
}

impl PointerChain {
    /// Create a new pointer chain
    pub fn new(base_address: Address, offsets: Vec<isize>) -> Self {
        Self {
            base_address,
            offsets,
            final_address: 0,
        }
    }

    /// Resolve the pointer chain
    pub fn resolve(&mut self, memory: &dyn Memory) -> Result<Address> {
        let mut current_address = self.base_address;

        for (i, offset) in self.offsets.iter().enumerate() {
            if i < self.offsets.len() - 1 {
                // Read pointer value
                let bytes = memory.read(current_address, std::mem::size_of::<usize>())?;
                current_address = usize::from_le_bytes(bytes.try_into().unwrap());
            }

            // Apply offset
            current_address = (current_address as isize + offset) as usize;
        }

        self.final_address = current_address;
        Ok(current_address)
    }

    /// Check if pointer chain is still valid
    pub fn is_valid(&self, memory: &dyn Memory, expected_address: Address) -> bool {
        let mut chain = self.clone();
        if let Ok(address) = chain.resolve(memory) {
            address == expected_address
        } else {
            false
        }
    }
}

/// Pointer scanner configuration
#[derive(Debug, Clone)]
pub struct PointerScanConfig {
    pub max_level: usize,
    pub max_offset: isize,
    pub max_results: usize,
}

impl Default for PointerScanConfig {
    fn default() -> Self {
        Self {
            max_level: 5,
            max_offset: 0x1000,
            max_results: 10000,
        }
    }
}

/// Pointer scanner
pub struct PointerScanner {
    config: PointerScanConfig,
}

impl PointerScanner {
    /// Create a new pointer scanner
    pub fn new(config: PointerScanConfig) -> Self {
        Self { config }
    }

    /// Scan for pointers to a target address
    pub fn scan(&self, memory: &dyn Memory, target_address: Address) -> Result<Vec<PointerChain>> {
        let mut results = Vec::new();
        let regions = memory.query_regions()?;

        // Find all potential pointers
        let mut pointer_map: HashMap<Address, Vec<Address>> = HashMap::new();

        for region in &regions {
            if !region.protection.read {
                continue;
            }

            if let Ok(data) = memory.read(region.base_address, region.size) {
                for i in (0..data.len()).step_by(std::mem::size_of::<usize>()) {
                    if i + std::mem::size_of::<usize>() > data.len() {
                        break;
                    }

                    let bytes: [u8; std::mem::size_of::<usize>()] = data
                        [i..i + std::mem::size_of::<usize>()]
                        .try_into()
                        .unwrap();
                    let value = usize::from_le_bytes(bytes);
                    let pointer_address = region.base_address + i;

                    pointer_map.entry(value).or_default().push(pointer_address);
                }
            }
        }

        // Build pointer chains
        self.build_chains(&pointer_map, target_address, Vec::new(), 0, &mut results);

        Ok(results)
    }

    /// Recursively build pointer chains
    fn build_chains(
        &self,
        pointer_map: &HashMap<Address, Vec<Address>>,
        current_target: Address,
        current_offsets: Vec<isize>,
        level: usize,
        results: &mut Vec<PointerChain>,
    ) {
        if level >= self.config.max_level || results.len() >= self.config.max_results {
            return;
        }

        // Try different offsets
        for offset in -self.config.max_offset..=self.config.max_offset {
            let search_address = (current_target as isize - offset) as usize;

            if let Some(pointers) = pointer_map.get(&search_address) {
                for &pointer_addr in pointers {
                    let mut offsets = current_offsets.clone();
                    offsets.push(offset);

                    if level == 0 {
                        // Found a valid chain
                        results.push(PointerChain::new(pointer_addr, offsets.clone()));
                    } else {
                        // Continue searching
                        self.build_chains(pointer_map, pointer_addr, offsets, level + 1, results);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pointer_chain_creation() {
        let chain = PointerChain::new(0x1000, vec![0x10, 0x20]);
        assert_eq!(chain.base_address, 0x1000);
        assert_eq!(chain.offsets.len(), 2);
    }

    #[test]
    fn test_pointer_scan_config() {
        let config = PointerScanConfig::default();
        assert_eq!(config.max_level, 5);
    }
}
