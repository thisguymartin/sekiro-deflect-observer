[CmdletBinding()]
param([string]$ObserverDll, [string]$OutputDirectory)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
if (-not $ObserverDll) { $ObserverDll = Join-Path $projectRoot 'target/x86_64-pc-windows-msvc/release/sekiro_deflect_observer.dll' }
if (-not $OutputDirectory) { $OutputDirectory = Join-Path $projectRoot 'dist/review-0.9.1/asi-smoke' }
$output = [System.IO.Path]::GetFullPath($OutputDirectory)
$distRoot = [System.IO.Path]::GetFullPath((Join-Path $projectRoot 'dist')) + [System.IO.Path]::DirectorySeparatorChar
if (-not $output.StartsWith($distRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'Loader smoke output must stay inside this workspace dist directory.'
}
if (-not (Test-Path -LiteralPath $ObserverDll -PathType Leaf)) { throw 'Build the current DLL or pass -ObserverDll.' }
Write-Host ('Observer SHA256: ' + (Get-FileHash -LiteralPath $ObserverDll -Algorithm SHA256).Hash.ToLowerInvariant())
New-Item -ItemType Directory -Path $output -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $projectRoot 'dist/drop-in-research/loader/dinput8.dll') -Destination (Join-Path $output 'dinput8.dll')
Copy-Item -LiteralPath $ObserverDll -Destination (Join-Path $output 'sekiro_deflect_observer.asi')
rustc --edition 2021 --target x86_64-pc-windows-msvc -C target-feature=+crt-static (Join-Path $PSScriptRoot 'test-fixtures/asi-loader-smoke.rs') -o (Join-Path $output 'observer-loader-smoke.exe')
if ($LASTEXITCODE -ne 0) { throw 'Could not build the loader smoke host.' }
$originalLocalAppData = $env:LOCALAPPDATA
try {
    # Keep the synthetic host's logs inside its isolated test folder.
    $env:LOCALAPPDATA = Join-Path $output 'local-app-data'
    & (Join-Path $output 'observer-loader-smoke.exe')
    if ($LASTEXITCODE -ne 0) { throw 'ASI loader smoke check failed.' }
} finally {
    $env:LOCALAPPDATA = $originalLocalAppData
}
