Sekiro Deflect Observer 0.12.4-preview
Native Windows x64 incoming attack response HUD

NEW: F11 toggles practice on/off; Shift+F11 selects and saves 70%, 80% or 90% speed.
A crescent moon and katana stay in the upper-right playable corner, including
while OFF and without a target. OFF/ON plus 70%, 80% or 90% shows the selected mode.
Gray = off, gold = armed, jade = applied, amber ! = attention (F9 gives details).
The 90% crescent is thin; 80% is wider; 70% is fullest. F8 hides and disarms.
Shift+F11 cycles 80% -> 90% -> 70% -> 80%. 70% means 30% slower animation.

NEW: F11 toggles optional enemy-speed practice. Starts OFF every process.
Eligible locked-enemy parry/thrust/sweep attacks run at the selected 70%, 80% or 90%
of existing animation speed; Wolf stays normal. The main attack caption appends
the applied percentage after a checked speed write.
PRACTICE means armed and waiting. F9 shows current speed-control status.
Grabs, unknowns and no-parry moves stay normal. Projectile flight is not rescaled.
Set practice_speed = 0.8 (allowed 0.5..1.0) in your local cue.toml.
F11 off or F8 hide releases the speed override; hiding disarms practice.
Focus/lock loss, death and attack ending also attempt ownership-checked cleanup.
Read/write failures retry; external speed changes pause practice until rearmed.
This is a gameplay-changing prototype and still needs live verification.
In 0.12.1, response toggles, Mikiri hints and HUD mode affect alerts only.
Practice uses raw attack classification independently. With hints disabled,
PRACTICE 80% still reports the applied speed. F8 remains a master shutdown.

This build fixes a reproduced DX11 graphics-state leak by drawing through a
private deferred context with full host-state restoration. The reported scene
tint requires an in-game comparison; automated checks are not gameplay evidence.
Fully restart to load this DLL. F8 hides the cue but does not unload the hook.

INSTALL
Close Sekiro fully. Extract this package into its own folder, keep Steam running,
and open observer.me3 or launch-observer.cmd. Install me3 separately from:
https://github.com/garyttierney/me3/releases
Use only one observer loading method. No Rust, Python or Cheat Engine is needed.

Load a save, lock onto a living enemy and look below the enemy's top posture bar.
For an existing config, set anchor = "top", width = 480 and zero the offsets.
parry_button = "LB" selects the badge; "L1" and "RMB" are also supported.
PARRY: the resolved attack permits deflection.
DODGE: a grab. JUMP: a classified low sweep.
MIKIRI: a thrust with explicit Mikiri detection and a supported counter route.
NO PARRY: deflection is disabled; no specific alternative is established.
UNKNOWN: an attack is known but its response is unresolved.
LOCKED: target found; no current attack phase or usable animation data.

The green segment and approaching diamond show parryable wind-up. A white
segment and red strike emblem mark its active phase, NOT confirmed sword contact
or successful deflection. Other responses retain distinct colors and captions.
reduced_flash = true suppresses the white/red transition.
The default incoming mode does not wait for distance or facing checks, predict
contact with Wolf, or issue an automatic input.

Mikiri hints assume the skill is unlocked. Set mikiri = false if needed; those
thrusts then show PARRY. The mod does not read your skill-unlock state.
Set incoming_cues = false only to use the older estimated press-window mode.

F6/F7: lower/raise 8 reference pixels and save. F8: toggle cue. F9: research panel.
F10: reset offsets. F11: session-only practice (initially off).
Shift+F11: switch 80% / 90% / 70%; remembers the level without enabling practice.
All hotkeys require focus, ignore repeats and pass through.
Settings: %LOCALAPPDATA%/SekiroDeflectObserver/cue.toml
The file is created with safe defaults and reloaded once per second. The included
cue.toml is a documented copy of those defaults. Malformed reloads retain the
last valid configuration. Remove the local config while closed to reset all.
Appearance, safe-area placement, response enables and bounded latency settings
can change without rebuilding. Lock/hash/freshness/uncertainty gates cannot.

CLASSIFICATION AND LIMITS
The fallback table has 2,112 phases on 53 models; 293 remain unknown. Specific
NPC behavior variations add weapon-aware choices, including soldier Mikiri and
Snake Eyes grabs. Missing/mismatched NPC identity keeps the fallback table.
These are data classifications, not unique moves or validated success counts.
Some Guardian Ape event types and real/unresolved projectiles remain unknown.
No automatic retaliation is provided. This build still needs live verification.

The new top-center rail uses local shadows with no large enclosing panel.
Untouched 0.8.0 layout defaults migrate atomically. Custom layouts are preserved.
Lost/changed targets, focus loss, hidden visibility and stale observations clear
old guidance. Animation failure retains only neutral LOCKED for a fresh target.

The observer never presses buttons or changes Wolf's speed/deflect windows.
F11 explicitly enables temporary enemy animation-speed writes; ordinary overlay
operation starts with those writes off. Original speed is saved and restored
only while the same owner/value remains valid. No game files or saves are edited.
The supported executable SHA256 is:
637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856

LOGS AND REMOVAL
Logs are under %LOCALAPPDATA%/SekiroDeflectObserver. Startup identifies actual DLL
and executable hashes; bounded CSVs record observations, animation crossings and
draw decisions. Each CSV stops at 16 MiB. The additional alerts.csv records received decision
changes plus a one-second heartbeat, continuing after full-frame logging fills.
It includes NPC/variation IDs and activation boundaries for later diagnosis.
F9 shows logging status, dropped records and config errors. Draw submission and candidate effect 105010 do not prove presentation,
actual input, contact or successful deflection. Supply those in a trial record.

Close the game to unload. A running process keeps its old DLL until full restart.
Stop using the me3 profile and remove only this extracted folder to uninstall.
Keep shared loader files used by other mods. Normal Steam launch omits this me3
profile; an independently installed ASI observer would still need removal.

Source, configuration details, coverage and open acceptance checks:
https://github.com/thisguymartin/sekiro-deflect-observer
See docs/validation-0.12.1.md, docs/configuration.md, docs/incoming-attacks.md and
LICENSE / THIRD-PARTY-NOTICES.txt. This package is not a validated gameplay release.
