# Listing text to review before publication

**Title:** Sekiro Deflect Observer - Experimental Beta

**Summary:** A visual timing slider above Wolf for selected incoming attacks.
Green PARRY, orange DODGE and blue JUMP cues. You press the buttons yourself.
Experimental coverage and timing. Windows x64 / Steam; drop-in installation.

## Description

Sekiro Deflect Observer follows Wolf with a timing slider. A moving diamond
approaches a colored zone to provide an advance response estimate for selected
locked-on enemy attacks. The visual design uses a glowing diamond, thin needle,
small pointer and English action labels.

- Green **PARRY**: an estimated deflect press cue.
- Orange **DODGE**: a mapped incoming grab; a safe direction is not predicted.
- Blue **JUMP**: a mapped low sweep.
- Gray **UNVERIFIED**: no confident response/timing guidance.
- **LOCKED**: the target is detected without an active timing cue.

This beta includes selected Chained Ogre, Guardian Ape and other enemy/boss
mappings. It does not cover every move, form or enemy. Timings come from attack
animation events and approximate reach checks; actual weapon contact and
successful-deflect timing remain unvalidated. It never presses buttons or
enlarges the game's real deflect window.

## Installation

1. Close Sekiro. In Steam, right-click Sekiro, choose Manage, then Browse local files.
2. Extract the ZIP. Copy `dinput8.dll` and `sekiro_deflect_observer.asi` beside `sekiro.exe`.
3. Launch normally through Steam, load a save and lock onto an enemy. F9 shows the version.

If `dinput8.dll` already exists, do not replace another mod's loader. Compatibility
needs to be resolved before installation. Do not also load the observer through
me3. The drop-in version does not require me3, Cheat Engine or development tools.

F6 lowers the bar, F7 raises it, F8 toggles visibility and F9 opens diagnostics.
The vertical adjustment resets each session. Close the game before removing
`sekiro_deflect_observer.asi`; remove the supplied loader only if no other mod
uses it. Keep the ZIP with its license notices when redistributing.

## Beta status and feedback

The 0.6.3 observer passed 35 Rust tests, formatting, Clippy, release build and
non-game-host startup checks. The new layout was checked offline. The drop-in
loader passed an isolated native startup/forwarding check; live Sekiro startup
through this route and a clean friend-PC installation still need testing.
Earlier gameplay images are labeled with their earlier version.

Report the enemy/move, observer version, what the cue displayed and a short
clip if possible. F9 and `%LOCALAPPDATA%\SekiroDeflectObserver` contain diagnostics.
An executable build the observer does not recognize disables its game-state reads.

## Credits and disclosure

Sekiro Deflect Observer contributors; hudhook/imgui and the dependencies listed
in the bundled notices; Ultimate ASI Loader 9.7.4 by ThirteenAG. Observer code
and loader use MIT licenses; dependency terms are included. The code and UI
implementation were developed with Codex-generated code under the author's
direction. This listing draft was also prepared with Codex.

Visual inspiration: https://www.youtube.com/watch?v=bejKX5vw2-Y
No video thumbnail or third-party promotional image is bundled with the mod.
This listing is a draft, not evidence of Nexus approval or a published release.
