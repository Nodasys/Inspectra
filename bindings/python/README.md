# Inspectra Python Bindings

Python bindings for the Inspectra memory analysis framework.

## Installation

```bash
pip install maturin
cd bindings/python
maturin develop
```

## Usage

```python
import inspectra

# Initialize
inspectra.init()

# List processes
manager = inspectra.ProcessManager()
processes = manager.list_processes()

for proc in processes:
    print(f"PID: {proc.pid}, Name: {proc.name}")

# Find specific process
chrome_procs = manager.find_by_name("chrome")

# Scan memory
scanner = inspectra.Scanner(pid=1234)
results = scanner.scan_i32(12345)
print(f"Found {len(results)} matches")
```

## Building

Build the wheel:
```bash
maturin build --release
```

## Development

Build and install in development mode:
```bash
maturin develop
```
