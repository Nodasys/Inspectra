# Inspectra

Modern memory analysis, inspection and manipulation framework.

**Inspectra** is a professional toolset designed for low-level process introspection, memory scanning, pointer analysis, live patching and automation. It targets researchers, reverse engineers, QA engineers and developers who require a robust, extensible and production-quality memory analysis platform.

This repository contains the _public_ starter material for Inspectra: README, contributing guides, security policy, license and development scaffolding.

## Quick Links

- Specification / internal guidelines: kept privately at Nodasys and not included here.
- Public repo: `github.com/nodasys/inspectra`
- Contact: kevin.gregoire@nodasys.com

---

## Project Vision

Inspectra aims to combine high-performance native engine components with a modern, extensible UI and programmable automation. It seeks to provide:

- Accurate, fast memory scanning across architectures (x86/x64/ARM).
- Reliable pointer resolution and structure analysis.
- Safe, auditable code injection and patching workflows.
- A plugin SDK for extensions and integrations.
- Scriptable automation via Python/Lua bindings.
- Strong security posture for research and enterprise use.

---

## Getting started (developer)

### Prerequisites
- Windows (recommended) / Linux / macOS (limited functionality)
- C++17 or Rust toolchain (depending on core language decision)
- Python 3.10+ (for scripting integration and tooling)
- Node.js (optional, if web UI is used)
- CMake (native builds) or Cargo (for Rust)
- Visual Studio 2022 (Windows) or clang/gcc toolchain (Linux)

### Local dev workflow
1. Clone the repo:
```bash
git clone https://github.com/nodasys/inspectra.git
cd inspectra
```

2. Create branch:
```bash
git checkout -b feat/core-engine
```

3. Build core (example for C++/CMake):
```bash
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Debug
cmake --build .
```

4. Run unit tests:
```bash
# depends on chosen test framework
ctest --output-on-failure
```

5. Format and lint:
- C++: clang-format, clang-tidy
- Python: black, ruff, pytest
- JavaScript/TypeScript: eslint, prettier

---

## Repository structure

```
inspectra/
├── .github/
│   ├── ISSUE_TEMPLATE/
│   └── workflows/
├── build/                # build output
├── core/                 # native core engine (C++ or Rust)
├── bindings/             # python / lua bindings + SDK
├── ui/                   # frontend (Qt / Web)
├── plugins/              # plugin examples and SDK
├── scripts/              # helper scripts and dev tooling
├── docs/                 # public documentation
└── tests/                # unit & integration tests
```

---

## Roadmap (high level)
- v0.1 - Core attachment, read/write memory, basic scanner.
- v0.2 - Pointer scan, hex viewer, snapshot export.
- v0.5 - Script integration (Python), plugin system.
- v1.0 - Stable debugger integration, patching UX, plugin ecosystem.
- v2.0 - Remote agents, ML-assisted pointer detection, enterprise features.

---

## Support & Reporting

For private security reports and sensitive issues: kevin.gregoire@nodasys.com

For general issues, use GitHub Issues once the repository is public.

