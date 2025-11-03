# Inspectra

Modern memory analysis, inspection and manipulation framework.

**Inspectra** is a professional toolset designed for low-level process introspection, memory scanning, pointer analysis, live patching and automation. It targets researchers, reverse engineers, QA engineers and developers who require a robust, extensible and production-quality memory analysis platform.

## 🚀 Features

- **Process Management**: List, attach, and inspect running processes
- **Memory Operations**: Read, write, and query process memory with fine-grained control
- **Advanced Scanning**: Multi-threaded memory scanner with support for all data types
- **Pattern Matching**: AOB (Array of Bytes) scanning with wildcard support
- **Pointer Analysis**: Multi-level pointer chain discovery and resolution
- **Code Patching**: Runtime code modification, NOP patching, and shellcode injection
- **Cross-Platform**: Windows (full support), Linux (via /proc), macOS (limited)
- **Python Bindings**: Full Python API via PyO3
- **Safe & Fast**: Written in Rust for memory safety and performance

## 📦 Installation

### Rust

Add to your `Cargo.toml`:
```toml
[dependencies]
inspectra-core = "0.1.0"
```

### Python

```bash
pip install inspectra
```

## 🎯 Quick Start

### Rust

```rust
use inspectra_core::*;

fn main() -> Result<()> {
    init()?;
    
    // List all processes
    let manager = process::get_process_manager();
    let processes = manager.list_processes()?;
    
    // Find and attach to a process
    let target = manager.find_by_name("target_app")?;
    let handle = manager.attach(target[0].pid)?;
    
    // Create memory accessor
    let mem = memory::create_memory(&handle)?;
    
    // Scan for a value
    let mut scanner = scanner::Scanner::new(
        mem,
        scanner::ScanConfig::default()
    );
    
    let results = scanner.scan(&42i32.to_le_bytes())?;
    println!("Found {} results", results.len());
    
    Ok(())
}
```

### Python

```python
import inspectra

# Initialize
inspectra.init()

# List processes
manager = inspectra.ProcessManager()
processes = manager.list_processes()

for proc in processes:
    print(f"{proc.pid}: {proc.name}")

# Find and scan
chrome = manager.find_by_name("chrome")
if chrome:
    scanner = inspectra.Scanner(chrome[0].pid)
    results = scanner.scan_i32(12345)
    print(f"Found {len(results)} matches")
```

## 📚 Documentation

- **[API Documentation](docs/API.md)**: Complete API reference
- **[Architecture](docs/ARCHITECTURE.md)**: System design and architecture
- **[Development Guide](docs/DEVELOPMENT.md)**: Contributing and development workflow
- **[Roadmap](docs/ROADMAP.md)**: Future plans and features

## 🏗️ Project Structure

```
inspectra/
├── core/                 # Rust core engine
│   ├── src/             # Source code
│   └── examples/        # Usage examples
├── bindings/
│   └── python/          # Python bindings
├── tests/               # Integration tests
├── docs/                # Documentation
└── scripts/             # Build & utility scripts
```

## 🛠️ Building from Source

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- Python 3.8+ (for bindings)
- Git

### Build

```powershell
# Clone repository
git clone https://github.com/nodasys/inspectra.git
cd inspectra

# Build core
cargo build --release

# Run tests
cargo test

# Run examples
cargo run --example list_processes

# Build Python bindings
cd bindings/python
pip install maturin
maturin develop
```

## 🧪 Examples

### List Processes
```powershell
cargo run --example list_processes
```

### Memory Scanner
```powershell
cargo run --example memory_scanner
```

### Pattern Scanning
```powershell
cargo run --example pattern_scan
```

### Python Examples
```bash
python bindings/python/examples/list_processes.py
python bindings/python/examples/memory_scanner.py
```

## 🎨 Architecture

Inspectra uses a modular architecture:

1. **Core Engine** (Rust): High-performance memory operations
2. **Scripting Layer** (Python/Lua): Automation and extensibility  
3. **Platform Layer**: OS-specific implementations
4. **UI Layer** (Future): Cross-platform interface

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for details.

## 🗺️ Roadmap

- **v0.1**: Core engine, basic scanning ✅
- **v0.2**: Enhanced scanning with SIMD
- **v0.3**: Advanced pointer analysis
- **v0.4**: Full debugger integration
- **v0.5**: Scripting engine (Python/Lua)
- **v0.6**: GUI (Tauri-based)
- **v1.0**: Production-ready release

See [ROADMAP.md](docs/ROADMAP.md) for complete roadmap.

## 🤝 Contributing

We welcome contributions! Please read our [Contributing Guide](CONTRIBUTING.md) for details on:

- Code style and standards
- Testing requirements
- Pull request process
- Security considerations

## 📋 Requirements

### Runtime
- Windows 10+ / Linux (kernel 3.10+) / macOS 10.15+
- Administrator/root privileges for memory operations

### Development
- Rust 1.70+
- Python 3.8+ (for bindings)
- CMake or Cargo

## 🔒 Security

Inspectra is a powerful tool that requires elevated privileges. Use responsibly and only on systems you own or have permission to analyze.

- See [SECURITY.md](SECURITY.md) for security policy
- Report vulnerabilities privately to: kevin.gregoire@nodasys.com

## 📄 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

## 🏢 About

**Inspectra** is developed by [Nodasys](https://nodasys.com) as a next-generation memory analysis framework.

- **Repository**: [github.com/nodasys/inspectra](https://github.com/nodasys/inspectra)
- **Contact**: kevin.gregoire@nodasys.com
- **Website**: https://nodasys.com

## ⭐ Show Your Support

If you find Inspectra useful, please consider:
- ⭐ Starring the repository
- 🐛 Reporting bugs and issues
- 💡 Suggesting features
- 🤝 Contributing code

## 📈 Status

![Build Status](https://github.com/nodasys/inspectra/workflows/CI/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
![Python](https://img.shields.io/badge/python-3.8%2B-blue.svg)

---

**Disclaimer**: This tool is for research, development, and authorized security testing only. Users are responsible for complying with applicable laws and regulations.

For private security reports and sensitive issues: kevin.gregoire@nodasys.com

For general issues, use GitHub Issues once the repository is public.

