//! Integration tests for Inspectra core

use inspectra_core::*;

#[test]
fn test_process_listing() {
    let manager = process::get_process_manager();
    let processes = manager.list_processes().unwrap();
    assert!(!processes.is_empty());
    println!("Found {} processes", processes.len());
}

#[test]
fn test_self_attach() {
    let manager = process::get_process_manager();
    let current_pid = std::process::id();
    
    let handle = manager.attach(current_pid).unwrap();
    assert!(handle.is_alive());
    assert_eq!(handle.pid(), current_pid);
}

#[test]
fn test_memory_regions() {
    let manager = process::get_process_manager();
    let current_pid = std::process::id();
    
    let handle = manager.attach(current_pid).unwrap();
    let memory = memory::create_memory(handle.as_ref()).unwrap();
    
    let regions = memory.query_regions().unwrap();
    assert!(!regions.is_empty());
    println!("Found {} memory regions", regions.len());
}

#[test]
fn test_memory_read_write() {
    let manager = process::get_process_manager();
    let current_pid = std::process::id();
    
    let handle = manager.attach(current_pid).unwrap();
    let memory = memory::create_memory(handle.as_ref()).unwrap();
    
    // Create a test variable
    let mut test_value: i32 = 12345;
    let address = &mut test_value as *mut i32 as usize;
    
    // Read the value
    let data = memory.read(address, 4).unwrap();
    let read_value = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    assert_eq!(read_value, 12345);
    
    // Write a new value
    let new_value = 54321i32;
    memory.write(address, &new_value.to_le_bytes()).unwrap();
    assert_eq!(test_value, 54321);
}

#[test]
fn test_scanner() {
    use types::DataType;
    use scanner::{Scanner, ScanConfig};
    
    let manager = process::get_process_manager();
    let current_pid = std::process::id();
    
    let handle = manager.attach(current_pid).unwrap();
    let memory = memory::create_memory(handle.as_ref()).unwrap();
    
    let mut config = ScanConfig::default();
    config.data_type = DataType::I32;
    
    let mut scanner = Scanner::new(memory, config);
    
    // Create test values
    let test_val1: i32 = 9999;
    let test_val2: i32 = 9999;
    
    // Scan for the value
    let results = scanner.scan(&9999i32.to_le_bytes()).unwrap();
    println!("Found {} results", results.len());
    
    // Should find at least our two test values
    assert!(results.len() >= 2);
}

#[test]
fn test_pattern_matching() {
    use scanner::Pattern;
    
    let pattern = Pattern::parse("48 8B ?? 50 FF").unwrap();
    let test_buffer = vec![0x48, 0x8B, 0x05, 0x50, 0xFF];
    
    assert!(pattern.matches(&test_buffer));
}

#[test]
fn test_pointer_chain() {
    use pointer::PointerChain;
    
    let mut chain = PointerChain::new(0x140000000, vec![0x18, 0x20, 0x8]);
    assert_eq!(chain.base_address, 0x140000000);
    assert_eq!(chain.offsets.len(), 3);
}
