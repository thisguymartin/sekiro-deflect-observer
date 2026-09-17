[CmdletBinding(SupportsShouldProcess)]
param(
    [string]$ConfigPath = (Join-Path $env:LOCALAPPDATA 'SekiroDeflectObserver/cue.toml'),
    [ValidateSet('LB','L1','RMB')][string]$ParryButton = 'LB'
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$path = [IO.Path]::GetFullPath($ConfigPath)
$original = [IO.File]::ReadAllText($path)
$updated = $original
$changes = [ordered]@{ anchor='"top"'; width='480.0'; offset_x='0.0'; offset_y='0.0'; parry_button=('"'+$ParryButton+'"') }
foreach ($key in $changes.Keys) {
    $pattern = '(?m)^' + [regex]::Escape($key) + '\s*=\s*[^\r\n]*'
    $matchesForKey = [regex]::Matches($updated, $pattern)
    if ($matchesForKey.Count -gt 1) { throw "Multiple $key entries; configuration was not changed." }
    if ($matchesForKey.Count -eq 1) {
        $updated = [regex]::Replace($updated, $pattern, ($key+' = '+$changes[$key]))
    } else {
        $updated = $key+' = '+$changes[$key]+"`r`n"+$updated
    }
}
if ($updated -eq $original) { Write-Host 'Reference UI placement is already selected.'; exit 0 }
if (-not $PSCmdlet.ShouldProcess($path, 'Select top reference HUD; preserve a backup of all previous settings')) { exit 0 }
$temporary = $path + '.reference-' + [guid]::NewGuid().ToString('N') + '.tmp'
$backup = $path + '.before-reference-' + [DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fffffff') + '.bak'
try {
    [IO.File]::WriteAllText($temporary, $updated, [Text.UTF8Encoding]::new($false))
    if ([IO.File]::ReadAllText($path) -cne $original) { throw 'Settings changed during preparation; repeat after the game/editor finishes saving.' }
    [IO.File]::Replace($temporary, $path, $backup)
    Write-Host "Selected top anchor, 480-wide rail, zero offsets and $ParryButton badge."
    Write-Host "Previous configuration: $backup"
} finally {
    if ([IO.File]::Exists($temporary)) { [IO.File]::Delete($temporary) }
}
