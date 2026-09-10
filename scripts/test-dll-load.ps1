[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if ($env:OS -ne 'Windows_NT' -or [IntPtr]::Size -ne 8) {
    throw 'Run the DLL load check in a 64-bit Windows PowerShell process.'
}

$projectRoot = Split-Path -Parent $PSScriptRoot
$dll = Join-Path $projectRoot 'target\x86_64-pc-windows-msvc\release\sekiro_deflect_observer.dll'
if (-not (Test-Path -LiteralPath $dll)) { throw 'Build the release DLL before running this check.' }

Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class ObserverLoadCheck {
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr LoadLibraryW(string fileName);
}
'@

$handle = [ObserverLoadCheck]::LoadLibraryW($dll)
if ($handle -eq [IntPtr]::Zero) {
    $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
    throw "LoadLibraryW failed with Windows error $code."
}

$log = Join-Path $env:LOCALAPPDATA "SekiroDeflectObserver\observer-$PID.log"
$deadline = [DateTime]::UtcNow.AddSeconds(10)
do {
    if (Test-Path -LiteralPath $log) {
        $content = Get-Content -LiteralPath $log -Raw
        if ($content -match 'Hooks installed') {
            throw 'The DLL installed hooks in the wrong host.'
        }
        if ($content -match 'host is not sekiro.exe; rendering hooks were not installed') {
            Write-Host 'DLL startup passed: the non-Sekiro host was rejected before hooking.'
            exit 0
        }
    }
    Start-Sleep -Milliseconds 50
} while ([DateTime]::UtcNow -lt $deadline)

throw "The DLL loaded but did not record host rejection. Inspect $log."
