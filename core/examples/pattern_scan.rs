//! Example: Pattern scanning (AOB)

use inspectra_core::{memory, process, scanner::Pattern};
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    inspectra_core::init()?;

    println!("Inspectra Pattern Scanner (AOB) Example\n");

    // Get target process
    print!("Enter process PID: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let pid: u32 = input.trim().parse()?;

    let manager = process::get_process_manager();
    let handle = manager.attach(pid)?;

    println!("Attached to process: PID {}", handle.pid());

    let mem = memory::create_memory(handle.as_ref())?;

    // Get pattern
    println!("\nEnter pattern (hex bytes with ?? for wildcards):");
    println!("Example: 48 8B ?? 50 FF");
    print!("> ");
    io::stdout().flush()?;

    let mut pattern_input = String::new();
    io::stdin().read_line(&mut pattern_input)?;
    let pattern = Pattern::parse(pattern_input.trim())?;

    println!("Searching for pattern...");

    // Scan memory regions
    let regions = mem.query_regions()?;
    let mut total_matches = 0;

    for region in regions {
        if !region.protection.read {
            continue;
        }

        if let Ok(data) = mem.read(region.base_address, region.size) {
            let matches = pattern.find_all(&data);

            for offset in matches {
                let address = region.base_address + offset;
                println!("Match found at: 0x{:X}", address);
                total_matches += 1;

                if total_matches >= 100 {
                    println!("... (limiting output to 100 results)");
                    break;
                }
            }
        }

        if total_matches >= 100 {
            break;
        }
    }

    println!("\nTotal matches: {}", total_matches);

    Ok(())
}
