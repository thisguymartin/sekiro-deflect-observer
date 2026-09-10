[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if ($env:OS -ne 'Windows_NT') {
    throw 'Run this script on Windows x64. See docs/windows.md for prerequisites.'
}
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Cargo is missing. Install Rust with rustup, then open a new terminal.'
}

$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location $projectRoot
try {
    cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Formatting check failed.' }

    cargo fetch --locked
    if ($LASTEXITCODE -ne 0) { throw 'Cannot download the locked dependencies.' }

    cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Clippy failed. Check the build tools and compiler output.' }

    cargo test --locked --all-targets --target x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed.' }

    cargo build --release --locked --target x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'Windows DLL build failed.' }

    powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'test-dll-load.ps1')
    if ($LASTEXITCODE -ne 0) { throw 'DLL startup and host rejection check failed.' }

    & (Join-Path $PSScriptRoot 'package.ps1')
}
finally {
    Pop-Location
}
