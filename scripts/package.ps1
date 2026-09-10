[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$projectRoot = Split-Path -Parent $PSScriptRoot
$dll = Join-Path $projectRoot 'target/x86_64-pc-windows-msvc/release/sekiro_deflect_observer.dll'
if (-not (Test-Path -LiteralPath $dll -PathType Leaf)) {
    throw 'The Windows DLL is missing. Run scripts/build.ps1 first.'
}

Push-Location $projectRoot
$staging = $null
try {
    $metadataJson = cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read the locked dependency metadata.' }
    $metadata = ($metadataJson -join "`n") | ConvertFrom-Json
    $project = @($metadata.packages | Where-Object { $_.name -eq 'sekiro-deflect-observer' })
    if ($project.Count -ne 1) { throw 'Cannot identify the observer package.' }

    $packageName = "SekiroDeflectObserver-$($project[0].version)-windows-x64"
    $dist = Join-Path $projectRoot 'dist'
    New-Item -ItemType Directory -Path $dist -Force | Out-Null
    $staging = Join-Path $dist ([Guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Path $staging | Out-Null
    Copy-Item -LiteralPath $dll -Destination $staging
    foreach ($name in @('observer.me3', 'launch-observer.cmd', 'README.txt')) {
        Copy-Item -LiteralPath (Join-Path $projectRoot "packaging/$name") -Destination $staging
    }
    Copy-Item -LiteralPath (Join-Path $projectRoot 'LICENSE') -Destination $staging

    $notice = [System.Text.StringBuilder]::new()
    [void]$notice.AppendLine('Dependency licenses for this locked Windows build, including build-time tools.')
    [void]$notice.AppendLine('me3 is installed separately and is not included in this package.')
    foreach ($dependency in ($metadata.packages | Sort-Object name, version)) {
        if ($dependency.id -eq $project[0].id) { continue }
        $dependencyRoot = Split-Path -Parent $dependency.manifest_path
        [void]$notice.AppendLine("`n=== $($dependency.name) $($dependency.version) ===")
        [void]$notice.AppendLine("License expression: $($dependency.license)")
        [void]$notice.AppendLine("Source: $($dependency.repository)")
        $licenseFiles = @(Get-ChildItem -LiteralPath $dependencyRoot -File -Recurse |
            Where-Object { $_.Name -match '^(LICENSE|LICENCE|COPYING|NOTICE)([._-].*)?$' } |
            Sort-Object FullName)
        if ($dependency.license_file) {
            $licenseFiles += Get-Item -LiteralPath (Join-Path $dependencyRoot $dependency.license_file)
        }
        $licenseFiles = @($licenseFiles | Sort-Object FullName -Unique)
        if ($licenseFiles.Count -eq 0) {
            throw "No license text found for $($dependency.name) $($dependency.version). Review it before packaging."
        }
        foreach ($licenseFile in $licenseFiles) {
            $relativeName = $licenseFile.FullName.Substring($dependencyRoot.Length).TrimStart('\', '/')
            [void]$notice.AppendLine("--- $relativeName ---")
            [void]$notice.AppendLine([System.IO.File]::ReadAllText($licenseFile.FullName))
        }
    }
    $utf8 = [System.Text.UTF8Encoding]::new($false)
    [System.IO.File]::WriteAllText((Join-Path $staging 'THIRD-PARTY-NOTICES.txt'), $notice.ToString(), $utf8)

    $hashLines = @(Get-ChildItem -LiteralPath $staging -File | Sort-Object Name | ForEach-Object {
        $hash = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        "$hash  $($_.Name)"
    })
    [System.IO.File]::WriteAllLines((Join-Path $staging 'SHA256SUMS.txt'), $hashLines, $utf8)
    $archive = Join-Path $dist "$packageName.zip"
    Compress-Archive -Path (Join-Path $staging '*') -DestinationPath $archive -Force
    $archiveHash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    [System.IO.File]::WriteAllText("$archive.sha256", "$archiveHash  $packageName.zip`n", $utf8)
    Write-Host "Package created: $archive"
    Write-Host "SHA-256: $archiveHash"
}
finally {
    if ($staging -and (Test-Path -LiteralPath $staging)) {
        Remove-Item -LiteralPath $staging -Recurse -Force
    }
    Pop-Location
}
