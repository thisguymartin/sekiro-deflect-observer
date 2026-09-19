Sekiro Deflect Observer 0.12.4-preview
Incoming attack HUD and optional enemy-speed practice for Windows x64

Follow a locked enemy's attack animation with a top-center rail and response
label. You control every defensive input. Optional F11 practice slows eligible
enemy attacks to 90%, 80%, 70% or 60% of their original animation speed.

INSTALL
1. Close Sekiro completely and extract this package into its own folder.
2. Install me3 separately: https://github.com/garyttierney/me3/releases
3. Keep Steam running and open observer.me3 or launch-observer.cmd.
4. Load a save, lock onto a living enemy and look for the top-center rail.
   F9 shows the loaded version and diagnostics. F11 enables optional practice.

Use only one observer loading method. No Rust, Python or Cheat Engine is needed.
Do not overwrite another mod's dinput8.dll. An older drop-in observer must not
also load alongside this profile. A rebuilt DLL needs a full process restart.

HOW TO READ THE HUD
PARRY: the classified phase permits deflection.
DODGE: a classified grab; no safe dodge direction is chosen.
JUMP: a classified low sweep.
MIKIRI: a thrust with an explicit supported counter route.
NO PARRY: deflection disabled; no specific alternative established.
UNKNOWN: an attack is known but its response is unresolved.
LOCKED: fresh target without a current attack cue.
PRACTICE: practice is armed and waiting for an eligible phase.

A green segment and approaching diamond show parryable wind-up. A white
segment and red strike emblem mark its active phase. This is not an exact
press-time instruction, confirmed weapon contact or successful-deflect signal.
Default incoming mode can show an attack even when the enemy is out of reach.

PRACTICE
F11 starts OFF every process. New settings select 90% speed.
Shift+F11 cycles 90% -> 80% -> 70% -> 60% -> 90% and saves the level without
changing on/off. 70% speed means 30% slower animation.
The upper-right crescent shows OFF/ON plus the selected percentage:
  Gray = off. Gold = armed/waiting. Jade = applied. Amber ! = check F9.
The moon remains visible without a target and while off. It grows fuller at
lower speeds. Its percentage alone is a selection, not proof of active slowdown.
An applied target speed is also appended to the rail, for example PARRY 70%.

Eligible locked-target parry/thrust/sweep attack phases can be slowed. Other
enemies, grabs, unknowns and no-parry moves are excluded. Wolf's speed and
deflect windows are untouched. Projectile flight and independent timers are
not rescaled. Disabling attack hints does not disable practice.
F11 off releases the override. F8 hides the HUD and disarms practice.
Focus/lock loss, death and attack ending attempt ownership-checked cleanup.
Read/write failures retry; external speed changes pause until explicitly rearmed.

CONTROLS (fresh press while the game is focused)
F6 / F7: lower / raise the rail by 8 reference pixels and save.
F8: show / hide the HUD; hiding also disarms practice.
F9: show / hide diagnostics and loaded version.
F10: reset horizontal and vertical offsets.
F11: toggle practice for this session.
Shift+F11: switch and save the selected enemy speed.
Hotkeys pass through. The observer never captures or generates combat input.

SETTINGS
%LOCALAPPDATA%/SekiroDeflectObserver/cue.toml
Created on first use, reloaded once per second. Malformed reloads retain the
last valid configuration. The bundled cue.toml documents defaults. Remove the
local config while closed to reset everything.
For an older layout: anchor = "top", width = 480, and F10 to clear offsets.
parry_button = "LB" selects the badge; "L1" and "RMB" are supported.
mikiri = false falls back to PARRY for supported deflectable thrusts if you
have not unlocked Mikiri. The observer does not read skill-unlock state.
reduced_flash = true suppresses the white/red active-phase transition.
practice_speed accepts 0.5..1.0; 1.0 requests no slowdown.
incoming_cues = false selects the older estimated press-window mode.

EVIDENCE AND LIMITS
The September 18 recording shows the live PARRY rail, active-phase emblem and
70% practice states. It does not show F9/build identity or measure slowdown,
input timing, successful deflects or cleanup across every game transition.
This remains an experimental preview. See the timestamped gameplay walkthrough:
https://github.com/thisguymartin/sekiro-deflect-observer/blob/main/docs/gameplay/walkthrough.md

Fallback data classifies 2,112 phases across 53 models; 293 remain unknown.
NPC behavior variants add weapon-aware choices. These are classifications,
not unique moves or proven gameplay successes. Full boss/form coverage is
not claimed. Invalid, dead, lost, switched or stale targets clear guidance.
The exact supported executable SHA-256 is:
637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856

F11 enables temporary enemy animation-speed writes. The original value is
restored only while the same owner/value remains valid. Game files and saves
are not edited. No account, telemetry or automatic retaliation is provided.

LOGS AND REMOVAL
Logs: %LOCALAPPDATA%/SekiroDeflectObserver
Startup records actual DLL/game hashes. Local bounded CSVs record observations,
alerts and practice transitions; each stops at 16 MiB. F9 shows status/errors.
A logged draw or checked speed write does not prove contact or measured slowdown.
The earlier scene-tint report still needs a controlled live comparison.

Close Sekiro to unload. Stop using this me3 profile and remove its extracted
folder if desired. Keep loader files shared by other mods and launch normally
through Steam. An independently installed ASI observer needs separate removal.
F8 hides the HUD and disarms practice; it does not unload the hook.

Source, configuration and current validation:
https://github.com/thisguymartin/sekiro-deflect-observer
See docs/archive/validation/validation-0.12.4.md,
docs/users/configuration.md, docs/research/attack-classification.md,
LICENSE and THIRD-PARTY-NOTICES.txt.
