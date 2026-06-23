//! Example: Memory scanner

use inspectra_core::{memory, process, scanner, types};
use scanner::{ScanConfig, Scanner};
use std::io::{self, Write};
use types::DataType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    inspectra_core::init()?;

    println!("Inspectra Memory Scanner Example\n");

    // Get target process
    print!("Enter process name or PID: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    let manager = process::get_process_manager();

    // Try to parse as PID, otherwise search by name
    let handle = if let Ok(pid) = input.parse::<u32>() {
        manager.attach(pid)?
    } else {
        let procs = manager.find_by_name(input)?;
        if procs.is_empty() {
            eprintln!("No process found with name: {}", input);
            return Ok(());
        }
        println!(
            "Found {} processes, using first: {} (PID: {})",
            procs.len(),
            procs[0].name,
            procs[0].pid
        );
        manager.attach(procs[0].pid)?
    };

    println!("Attached to process: PID {}", handle.pid());

    // Create memory accessor
    let mem = memory::create_memory(handle.as_ref())?;

    // Configure scanner
    let config = ScanConfig {
        data_type: DataType::I32,
        ..Default::default()
    };

    let mut scanner = Scanner::new(mem, config);

    // Scan loop
    loop {
        print!("\nEnter value to scan for (or 'quit'): ");
        io::stdout().flush()?;

        let mut value_input = String::new();
        io::stdin().read_line(&mut value_input)?;
        let value_input = value_input.trim();

        if value_input.eq_ignore_ascii_case("quit") {
            break;
        }

        if let Ok(value) = value_input.parse::<i32>() {
            println!("Scanning for value: {}", value);

            let bytes = value.to_le_bytes();
            let results = scanner.scan(Some(&bytes), None)?;

            println!("Found {} results", results.len());

            // Display first 10 results
            for (i, result) in results.iter().take(10).enumerate() {
                println!("  [{}] Address: 0x{:X}", i, result.address);
            }

            if results.len() > 10 {
                println!("  ... and {} more", results.len() - 10);
            }
        } else {
            println!("Invalid value");
        }
    }

    println!("Exiting...");
    Ok(())
}
