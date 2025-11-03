# Inspectra API Documentation

## Core Modules

### Process Management

List and attach to processes:

```rust
use inspectra_core::process;

// Get process manager
let manager = process::get_process_manager();

// List all processes
let processes = manager.list_processes()?;

// Find by name
let chrome = manager.find_by_name("chrome")?;

// Attach to process
let handle = manager.attach(process_id)?;
```

### Memory Operations

Read and write process memory:

```rust
use inspectra_core::memory;

// Create memory accessor
let mem = memory::create_memory(&handle)?;

// Read memory
let data = mem.read(address, size)?;

// Write memory
mem.write(address, &data)?;

// Query memory regions
let regions = mem.query_regions()?;
```

### Memory Scanning

Scan for values in memory:

```rust
use inspectra_core::scanner::{Scanner, ScanConfig};
use inspectra_core::types::DataType;

// Configure scanner
let mut config = ScanConfig::default();
config.data_type = DataType::I32;

// Create scanner
let mut scanner = Scanner::new(memory, config);

// Initial scan
let value = 12345i32.to_le_bytes();
let results = scanner.scan(&value)?;

// Rescan
let new_value = 54321i32.to_le_bytes();
let results = scanner.rescan(&new_value)?;
```

### Pointer Scanning

Find pointer chains:

```rust
use inspectra_core::pointer::{PointerScanner, PointerScanConfig};

// Configure pointer scanner
let config = PointerScanConfig {
    max_level: 5,
    max_offset: 0x1000,
    max_results: 1000,
};

// Create scanner
let scanner = PointerScanner::new(config);

// Scan for pointers
let chains = scanner.scan(&memory, target_address)?;

// Resolve chain
for mut chain in chains {
    let address = chain.resolve(&memory)?;
    println!("Base: 0x{:X}, Offsets: {:?}", chain.base_address, chain.offsets);
}
```

### Code Patching

Modify code at runtime:

```rust
use inspectra_core::debugger::CodePatcher;

let patcher = CodePatcher::new(memory);

// NOP a region
let original = patcher.nop_region(address, 5)?;

// Write a jump
let original = patcher.write_jmp(from_address, to_address)?;

// Restore original
patcher.restore(address, &original)?;

// Inject shellcode
let shellcode = vec![0x90, 0x90, 0xC3]; // NOP NOP RET
let address = patcher.allocate_shellcode(&shellcode)?;
```

### Pattern Matching

Search for byte patterns:

```rust
use inspectra_core::scanner::Pattern;

// Parse pattern with wildcards
let pattern = Pattern::parse("48 8B ?? 50 FF")?;

// Match against buffer
if pattern.matches(&buffer) {
    println!("Pattern found!");
}

// Find all occurrences
let offsets = pattern.find_all(&buffer);
```

## Error Handling

All operations return `Result<T, InspectraError>`:

```rust
use inspectra_core::error::{InspectraError, Result};

match memory.read(address, size) {
    Ok(data) => println!("Read {} bytes", data.len()),
    Err(InspectraError::InvalidAddress(addr)) => {
        eprintln!("Invalid address: 0x{:X}", addr);
    }
    Err(InspectraError::PermissionDenied(msg)) => {
        eprintln!("Permission denied: {}", msg);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Python Bindings

Use Inspectra from Python:

```python
import inspectra

# Initialize
inspectra.init()

# List processes
manager = inspectra.ProcessManager()
processes = manager.list_processes()

for proc in processes:
    print(f"PID: {proc.pid}, Name: {proc.name}")

# Scan memory
scanner = inspectra.Scanner(pid=1234)
results = scanner.scan_i32(12345)
```

## Platform Support

- **Windows**: Full support
- **Linux**: Read/write memory via `/proc`
- **macOS**: Limited support (SIP restrictions)

## Security Considerations

- Always check privileges before operations
- Use sandboxed execution for untrusted scripts
- Verify plugin signatures
- Log sensitive operations
