[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$output = Join-Path $projectRoot 'dist/drop-in-research/smoke'
New-Item -ItemType Directory -Path $output -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $projectRoot 'dist/drop-in-research/loader/dinput8.dll') -Destination (Join-Path $output 'dinput8.dll')
Copy-Item -LiteralPath (Join-Path $projectRoot 'Mods/SekiroDeflectObserver-0.6.3-preview/sekiro_deflect_observer.dll') -Destination (Join-Path $output 'sekiro_deflect_observer.asi')
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
