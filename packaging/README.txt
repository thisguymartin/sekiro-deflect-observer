Sekiro Deflect Observer 0.10.0-preview
Native Windows x64 incoming attack response HUD

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
F10: reset offsets. All hotkeys require focus, ignore repeats and pass through.
Settings: %LOCALAPPDATA%/SekiroDeflectObserver/cue.toml
The file is created with safe defaults and reloaded once per second. The included
cue.toml is a documented copy of those defaults. Malformed reloads retain the
last valid configuration. Remove the local config while closed to reset all.
Appearance, safe-area placement, response enables and bounded latency settings
can change without rebuilding. Lock/hash/freshness/uncertainty gates cannot.

CLASSIFICATION AND LIMITS
Incoming mode covers 2,161 phases across 54 models: 1,673 parry, 40 dodge,
56 jump, 63 Mikiri, 16 no-parry and 313 unknown. These are data classifications,
not unique moves or gameplay-validated success counts. Unknown variants and
unresolved projectiles remain unknown. No automatic retaliation is provided.

The new top-center rail uses local shadows with no large enclosing panel.
Untouched 0.8.0 layout defaults migrate atomically. Custom layouts are preserved.
Lost/changed targets, focus loss, hidden visibility and stale observations clear
old guidance. Animation failure retains only neutral LOCKED for a fresh target.

The observer never presses buttons or changes attack speed, deflect windows or
gameplay state. The supported executable SHA256 is:
637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856

LOGS AND REMOVAL
Logs are under %LOCALAPPDATA%/SekiroDeflectObserver. Startup identifies actual DLL
and executable hashes; bounded CSVs record observations, animation crossings and
draw decisions. Each CSV stops at 16 MiB. F9 shows dropped records and config
errors. Draw submission and candidate effect 105010 do not prove presentation,
actual input, contact or successful deflection. Supply those in a trial record.

Close the game to unload. A running process keeps its old DLL until full restart.
Stop using the me3 profile and remove only this extracted folder to uninstall.
Keep shared loader files used by other mods. Normal Steam launch omits this me3
profile; an independently installed ASI observer would still need removal.

Source, configuration details, coverage and open acceptance checks:
https://github.com/thisguymartin/sekiro-deflect-observer
See docs/WORK-STATUS.md, docs/configuration.md, docs/incoming-attacks.md and
LICENSE / THIRD-PARTY-NOTICES.txt. This package is not a validated gameplay release.
