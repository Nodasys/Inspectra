# Inspectra Development Guide

## Getting Started

### Prerequisites

1. **Rust toolchain** (1.70+)
   ```powershell
   # Install rustup from https://rustup.rs
   rustup install stable
   rustup default stable
   ```

2. **Python** (3.8+) - for bindings
   ```powershell
   # Check Python version
   python --version
   ```

3. **Git**
   ```powershell
   git --version
   ```

### First Build

1. Clone the repository:
   ```powershell
   git clone https://github.com/nodasys/inspectra.git
   cd inspectra
   ```

2. Build the core:
   ```powershell
   cargo build
   ```

3. Run tests:
   ```powershell
   cargo test
   ```

4. Run examples:
   ```powershell
   cargo run --example list_processes
   ```

## Project Structure

```
inspectra/
├── core/                   # Core Rust library
│   ├── src/
│   │   ├── lib.rs         # Main library entry
│   │   ├── error.rs       # Error types
│   │   ├── types.rs       # Common types
│   │   ├── process/       # Process management
│   │   ├── memory/        # Memory operations
│   │   ├── scanner/       # Memory scanning
│   │   ├── pointer/       # Pointer analysis
│   │   ├── debugger/      # Debugging & patching
│   │   └── platform/      # Platform-specific code
│   └── examples/          # Usage examples
│
├── bindings/
│   └── python/            # Python bindings
│       ├── src/           # Rust FFI code
│       ├── inspectra/     # Python package
│       └── examples/      # Python examples
│
├── tests/                 # Integration tests
├── docs/                  # Documentation
├── scripts/               # Build scripts
└── .github/workflows/     # CI/CD
```

## Development Workflow

### 1. Create a Feature Branch

```powershell
git checkout -b feat/your-feature
```

### 2. Make Changes

Edit code in your IDE (VS Code recommended with rust-analyzer).

### 3. Format Code

```powershell
cargo fmt
```

### 4. Run Linter

```powershell
cargo clippy --all-targets --all-features -- -D warnings
```

### 5. Run Tests

```powershell
cargo test --all
```

### 6. Build

```powershell
# Debug build
cargo build

# Release build
cargo build --release
```

### 7. Test Examples

```powershell
cargo run --example list_processes
cargo run --example memory_scanner
cargo run --example pattern_scan
```

## Python Bindings Development

### Setup

```powershell
cd bindings/python
pip install maturin
```

### Development Build

```powershell
# Build and install in development mode
maturin develop

# Test
python examples/list_processes.py
```

### Release Build

```powershell
maturin build --release
```

## Testing

### Unit Tests

```powershell
# Run all tests
cargo test

# Run specific test
cargo test test_process_listing

# Run with output
cargo test -- --nocapture
```

### Integration Tests

```powershell
cargo test --test integration_tests
```

### Coverage

```powershell
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --verbose --all-features --workspace
```

## Debugging

### Rust

Use VS Code with rust-analyzer:
- Set breakpoints in code
- F5 to start debugging

### Logs

Enable logging:
```powershell
$env:RUST_LOG="debug"
cargo run --example list_processes
```

## Common Tasks

### Add a New Module

1. Create file in `core/src/your_module.rs`
2. Add `pub mod your_module;` to `lib.rs`
3. Export public items in `lib.rs`
4. Write tests
5. Update documentation

### Update Dependencies

```powershell
# Check for outdated dependencies
cargo outdated

# Update dependencies
cargo update
```

### Security Audit

```powershell
# Install cargo-audit
cargo install cargo-audit

# Run audit
cargo audit
```

## Performance Profiling

### Benchmarks

```powershell
cargo bench
```

### Profiling

Use tools like:
- **Windows**: Visual Studio Profiler, Windows Performance Analyzer
- **Linux**: perf, valgrind
- **Cross-platform**: cargo-flamegraph

## Documentation

### Generate Docs

```powershell
cargo doc --no-deps --open
```

### Write Docs

Use Rust doc comments:
```rust
/// This function does something
///
/// # Examples
///
/// ```
/// use inspectra_core::example;
/// example();
/// ```
pub fn example() {
    // ...
}
```

## CI/CD

GitHub Actions automatically:
- Runs tests on Windows, Linux, macOS
- Checks formatting
- Runs Clippy
- Generates coverage report

View results in the Actions tab on GitHub.

## Tips

1. **Use `cargo check`** for fast compilation checks
2. **Use `cargo watch`** for continuous compilation
3. **Enable all Clippy lints** during development
4. **Write tests first** (TDD)
5. **Document public APIs** thoroughly
6. **Handle errors properly** - don't panic!

## Troubleshooting

### Windows API Errors

Make sure to run with administrator privileges:
```powershell
# Run PowerShell as Administrator
Start-Process powershell -Verb runAs
```

### Linux Permission Denied

Use ptrace permissions:
```bash
echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope
```

### Python Binding Issues

Reinstall in development mode:
```powershell
cd bindings/python
pip uninstall inspectra
maturin develop --release
```

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [PyO3 Guide](https://pyo3.rs/)
- [Windows API Documentation](https://learn.microsoft.com/en-us/windows/win32/)
