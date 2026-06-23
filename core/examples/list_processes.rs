//! Example: List all processes

use inspectra_core::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Inspectra
    inspectra_core::init()?;

    // Get process manager
    let manager = process::get_process_manager();

    // List all processes
    println!("Listing all processes...\n");
    let processes = manager.list_processes()?;

    // Sort by name
    let mut sorted_processes = processes;
    sorted_processes.sort_by(|a, b| a.name.cmp(&b.name));

    // Display
    println!("{:<10} {:<30} Path", "PID", "Name");
    println!("{}", "=".repeat(80));

    for proc in sorted_processes.iter().take(20) {
        println!("{:<10} {:<30} {}", proc.pid, proc.name, proc.path);
    }

    println!("\nTotal: {} processes", sorted_processes.len());

    Ok(())
}
