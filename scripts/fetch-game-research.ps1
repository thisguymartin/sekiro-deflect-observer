[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$outputRoot = Join-Path (Split-Path -Parent $PSScriptRoot) 'dist/game-analysis/references'
New-Item -ItemType Directory -Path $outputRoot -Force | Out-Null
$downloads = @{
    'paramdex-tree.json' = 'https://api.github.com/repos/soulsmods/Paramdex/git/trees/master?recursive=1'
    'SDT.AtkParam.xml' = 'https://raw.githubusercontent.com/soulsmods/Paramdex/master/SDT/Defs/AtkParam.xml'
    'SDT.BehaviorParam.xml' = 'https://raw.githubusercontent.com/soulsmods/Paramdex/master/SDT/Defs/BehaviorParam.xml'
    'SDT.NpcParam.xml' = 'https://raw.githubusercontent.com/soulsmods/Paramdex/master/SDT/Defs/NpcParam.xml'
    'SDT.ThrowParam.xml' = 'https://raw.githubusercontent.com/soulsmods/Paramdex/master/SDT/Defs/ThrowParam.xml'
    'PARAM.cs' = 'https://raw.githubusercontent.com/JKAnderson/SoulsFormats/master/SoulsFormats/Formats/PARAM/PARAM/PARAM.cs'
    'PARAM.Row.cs' = 'https://raw.githubusercontent.com/JKAnderson/SoulsFormats/master/SoulsFormats/Formats/PARAM/PARAM/Row.cs'
    'SDT.AtkParam.names.txt' = 'https://raw.githubusercontent.com/soulsmods/Paramdex/master/SDT/Names/AtkParam_Npc.txt'
    'TAE.Template.SDT.xml' = 'https://raw.githubusercontent.com/Meowmaritus/DSAnimStudio/master/DSAnimStudioNETCore/Res/TAE.Template.SDT.xml'
    'dsanimstudio-tree.json' = 'https://api.github.com/repos/Meowmaritus/DSAnimStudio/git/trees/master?recursive=1'
    'ArchiveKeys.cs' = 'https://raw.githubusercontent.com/Nordgaren/UXM-Selective-Unpack/master/UXM/ArchiveKeys.cs'
    'ArchiveUnpacker.cs' = 'https://raw.githubusercontent.com/Nordgaren/UXM-Selective-Unpack/master/UXM/ArchiveUnpacker.cs'
    'CryptographyUtility.cs' = 'https://raw.githubusercontent.com/Nordgaren/UXM-Selective-Unpack/master/UXM/CryptographyUtility.cs'
    'SekiroDictionary.txt' = 'https://raw.githubusercontent.com/Nordgaren/UXM-Selective-Unpack/master/UXM/res/SekiroDictionary.txt'
    'SekiroGameInfo.xml' = 'https://raw.githubusercontent.com/Nordgaren/UXM-Selective-Unpack/master/UXM/res/SekiroGameInfo.xml'
    'BHD5.cs' = 'https://raw.githubusercontent.com/JKAnderson/SoulsFormats/master/SoulsFormats/Formats/BHD5.cs'
    'soulsformats-tree.json' = 'https://api.github.com/repos/JKAnderson/SoulsFormats/git/trees/master?recursive=1'
    'TAE.cs' = 'https://raw.githubusercontent.com/soulsmods/SoulsFormatsNEXT/master/SoulsFormats/Formats/TAE/TAE.cs'
    'TAE.Animation.cs' = 'https://raw.githubusercontent.com/soulsmods/SoulsFormatsNEXT/master/SoulsFormats/Formats/TAE/Animation.cs'
    'TAE.Event.cs' = 'https://raw.githubusercontent.com/soulsmods/SoulsFormatsNEXT/master/SoulsFormats/Formats/TAE/Event.cs'
    'BND4.cs' = 'https://raw.githubusercontent.com/JKAnderson/SoulsFormats/master/SoulsFormats/Binder/BND4/BND4.cs'
    'Binder.cs' = 'https://raw.githubusercontent.com/JKAnderson/SoulsFormats/master/SoulsFormats/Binder/Binder.cs'
    'Oodle26.cs' = 'https://raw.githubusercontent.com/JKAnderson/SoulsFormats/master/SoulsFormats/Util/Oodle26.cs'
    'BinderFileHeader.cs' = 'https://raw.githubusercontent.com/JKAnderson/SoulsFormats/master/SoulsFormats/Binder/BinderFileHeader.cs'
    'SFUtil.cs' = 'https://raw.githubusercontent.com/JKAnderson/SoulsFormats/master/SoulsFormats/Util/SFUtil.cs'
    'DCX.cs' = 'https://raw.githubusercontent.com/JKAnderson/SoulsFormats/master/SoulsFormats/Formats/DCX.cs'
    'Offsets.cs' = 'https://raw.githubusercontent.com/borgCode/SekiroTool/189649781b00ba1f2fddb5d8bbfff7684ff5b647/SekiroTool/Memory/Offsets.cs'
    'sekirotool-tree.json' = 'https://api.github.com/repos/borgCode/SekiroTool/git/trees/189649781b00ba1f2fddb5d8bbfff7684ff5b647?recursive=1'
    'TargetService.cs' = 'https://raw.githubusercontent.com/borgCode/SekiroTool/189649781b00ba1f2fddb5d8bbfff7684ff5b647/SekiroTool/Services/TargetService.cs'
    'ChrInsService.cs' = 'https://raw.githubusercontent.com/borgCode/SekiroTool/189649781b00ba1f2fddb5d8bbfff7684ff5b647/SekiroTool/Services/ChrInsService.cs'
    'HookManager.cs' = 'https://raw.githubusercontent.com/borgCode/SekiroTool/189649781b00ba1f2fddb5d8bbfff7684ff5b647/SekiroTool/Memory/HookManager.cs'
}
foreach ($entry in $downloads.GetEnumerator()) {
    if (Test-Path -LiteralPath (Join-Path $outputRoot $entry.Key)) { continue }
    try {
        Invoke-WebRequest -Uri $entry.Value -OutFile (Join-Path $outputRoot $entry.Key) -TimeoutSec 30
        Write-Output "Downloaded $($entry.Key)"
    } catch {
        Write-Warning "Could not download $($entry.Key): $($_.Exception.Message)"
    }
}

# Downloads are reference text only. This script neither executes them nor
# reads/writes the game installation.
