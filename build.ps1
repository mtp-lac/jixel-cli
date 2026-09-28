<#>
.SYNOPSIS
    Build script for jixel-cli on Windows with MSVC

.DESCRIPTION
    Builds the jixel-cli executable in release mode with optimizations.
    Requires the Rust MSVC toolchain and the Visual Studio C++ build tools.
    With -Test it also runs the full encode/decode verification suite.

.NOTES
    Run from Developer Command Prompt or Developer PowerShell for VS 2022+
#>

param(
    [switch]$Clean,
    [switch]$Test,
    [switch]$Install,
    [string]$InstallPath = "$env:USERPROFILE\.cargo\bin"
)

$ErrorActionPreference = "Stop"

Write-Host "=== jixel-cli Build Script ===" -ForegroundColor Cyan
Write-Host "Rust toolchain: $(rustc --version)" -ForegroundColor Gray
Write-Host "Cargo version:  $(cargo --version)" -ForegroundColor Gray
Write-Host "Target:         x86_64-pc-windows-msvc" -ForegroundColor Gray
Write-Host ""

$projectDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
Set-Location $projectDir

if ($Clean) {
    Write-Host "Cleaning previous build..." -ForegroundColor Yellow
    cargo clean
}

# Build release (dependencies come from crates.io on first build)
Write-Host "Building release..." -ForegroundColor Yellow
cargo build --release

$exePath = "target\release\jixel-cli.exe"
if (-not (Test-Path $exePath)) {
    Write-Host "Build failed - executable not found" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Build successful!" -ForegroundColor Green
Write-Host "Executable: $exePath" -ForegroundColor Cyan
$size = (Get-Item $exePath).Length
Write-Host ("Size: {0:N0} bytes ({1:N1} MB)" -f $size, ($size / 1MB)) -ForegroundColor Gray

if ($Test) {
    Write-Host ""
    Write-Host "Generating test images..." -ForegroundColor Yellow
    & "$projectDir\gen-test-images.ps1"
    cargo run --release --example gen16

    Write-Host ""
    Write-Host "Running encode matrix..." -ForegroundColor Yellow
    & "$projectDir\run-tests.ps1"

    Write-Host ""
    Write-Host "Running decode verification (jxl-oxide)..." -ForegroundColor Yellow
    cargo run --release --example verify
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Verification FAILED" -ForegroundColor Red
        exit 1
    }
}

if ($Install) {
    Write-Host ""
    Write-Host "Installing to $InstallPath..." -ForegroundColor Yellow
    if (-not (Test-Path $InstallPath)) {
        New-Item -ItemType Directory -Force -Path $InstallPath | Out-Null
    }
    Copy-Item -Force -Path $exePath -Destination $InstallPath
    Write-Host "Installed. Ensure $InstallPath is in your PATH." -ForegroundColor Green
}

Write-Host ""
Write-Host "Done!" -ForegroundColor Cyan
