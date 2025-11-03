#!/usr/bin/env pwsh
# Build script for Inspectra

param(
    [Parameter()]
    [ValidateSet('debug', 'release')]
    [string]$Configuration = 'debug',
    
    [Parameter()]
    [switch]$Test,
    
    [Parameter()]
    [switch]$Clippy,
    
    [Parameter()]
    [switch]$Format,
    
    [Parameter()]
    [switch]$Python
)

Write-Host "Building Inspectra..." -ForegroundColor Cyan
Write-Host "Configuration: $Configuration" -ForegroundColor Gray
Write-Host ""

# Format check
if ($Format) {
    Write-Host "Checking code formatting..." -ForegroundColor Yellow
    cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Formatting check failed! Run 'cargo fmt' to fix." -ForegroundColor Red
        exit 1
    }
    Write-Host "✓ Formatting OK" -ForegroundColor Green
}

# Clippy
if ($Clippy) {
    Write-Host "Running Clippy..." -ForegroundColor Yellow
    cargo clippy --all-targets --all-features -- -D warnings
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Clippy found issues!" -ForegroundColor Red
        exit 1
    }
    Write-Host "✓ Clippy OK" -ForegroundColor Green
}

# Build core
Write-Host "Building core..." -ForegroundColor Yellow
if ($Configuration -eq 'release') {
    cargo build --release
} else {
    cargo build
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Build OK" -ForegroundColor Green

# Run tests
if ($Test) {
    Write-Host "Running tests..." -ForegroundColor Yellow
    cargo test --all
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Tests failed!" -ForegroundColor Red
        exit 1
    }
    Write-Host "✓ Tests OK" -ForegroundColor Green
}

# Build Python bindings
if ($Python) {
    Write-Host "Building Python bindings..." -ForegroundColor Yellow
    Push-Location bindings/python
    
    # Check if maturin is installed
    if (-not (Get-Command maturin -ErrorAction SilentlyContinue)) {
        Write-Host "Installing maturin..." -ForegroundColor Yellow
        pip install maturin
    }
    
    if ($Configuration -eq 'release') {
        maturin build --release
    } else {
        maturin develop
    }
    
    Pop-Location
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Python bindings build failed!" -ForegroundColor Red
        exit 1
    }
    Write-Host "✓ Python bindings OK" -ForegroundColor Green
}

Write-Host ""
Write-Host "✓ All done!" -ForegroundColor Green
