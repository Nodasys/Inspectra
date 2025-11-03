# Inspectra Architecture

## High-Level Overview

```
┌─────────────────────────────────────────┐
│         User Interface Layer            │
│  (Tauri/Qt - Future Implementation)     │
└────────────┬────────────────────────────┘
             │
┌────────────▼────────────────────────────┐
│         Scripting Layer                 │
│    (Python/Lua Bindings - PyO3)        │
└────────────┬────────────────────────────┘
             │
┌────────────▼────────────────────────────┐
│          Core Engine (Rust)             │
│  ┌────────────────────────────────┐    │
│  │  Process Management            │    │
│  │  - List/Attach/Query           │    │
│  └────────────────────────────────┘    │
│  ┌────────────────────────────────┐    │
│  │  Memory Operations             │    │
│  │  - Read/Write/Query Regions    │    │
│  │  - Protection Management       │    │
│  └────────────────────────────────┘    │
│  ┌────────────────────────────────┐    │
│  │  Scanner Engine                │    │
│  │  - Value Scanning              │    │
│  │  - Pattern Matching (AOB)      │    │
│  │  - Multi-threaded Search       │    │
│  └────────────────────────────────┘    │
│  ┌────────────────────────────────┐    │
│  │  Pointer Analysis              │    │
│  │  - Chain Discovery             │    │
│  │  - Path Resolution             │    │
│  └────────────────────────────────┘    │
│  ┌────────────────────────────────┐    │
│  │  Debugger/Patcher              │    │
│  │  - Breakpoints                 │    │
│  │  - Code Injection              │    │
│  │  - Assembly Patching           │    │
│  └────────────────────────────────┘    │
└────────────┬────────────────────────────┘
             │
┌────────────▼────────────────────────────┐
│       Platform Abstraction              │
│  ┌──────────┐  ┌──────────┐            │
│  │ Windows  │  │  Linux   │            │
│  │ Win32 API│  │ /proc    │            │
│  └──────────┘  └──────────┘            │
└─────────────────────────────────────────┘
```

## Module Dependencies

- **core**: Main engine (no dependencies on other modules)
- **bindings/python**: Depends on core, provides Python interface
- **ui** (future): Depends on core and bindings
- **plugins** (future): Plugin API based on core

## Data Flow

1. **Process Attachment**: User → ProcessManager → OS API → Handle
2. **Memory Read**: Handle → Memory → OS API → Data
3. **Scanning**: Data → Scanner → Results
4. **Pointer Resolution**: Results → PointerScanner → Chains

## Platform Support

| Platform | Status | Implementation |
|----------|--------|----------------|
| Windows  | ✅ Full | Win32 API |
| Linux    | ✅ Partial | /proc, ptrace |
| macOS    | ⚠️ Limited | SIP restrictions |

## Technology Stack

- **Language**: Rust 2021 edition
- **Build**: Cargo workspace
- **Python**: PyO3 + Maturin
- **Testing**: cargo test + integration tests
- **CI/CD**: GitHub Actions

## Security Model

- Privilege checks before operations
- Safe memory access patterns
- No unsafe code where possible
- Plugin sandboxing (future)
