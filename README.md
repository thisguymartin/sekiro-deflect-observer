# Sekiro Deflect Observer

![Current 0.12.1 HUD and practice states, not gameplay](docs/images/0.12.1-practice-gallery.png)

**0.12.1-preview separates enemy-speed practice from alert preferences.**
Turning off response hints or switching HUD mode no longer changes slowdown
eligibility. F11 practice remains initially off; F8 hiding still disarms it.
Eligible locked-enemy attacks run at 80% of their existing animation speed;
Wolf stays at normal speed. Grabs and unknown/no-parry moves remain unchanged.
This new mode still needs live verification. See [controls and limits](docs/enemy-speed-practice.md)
and [build checks](docs/validation-0.12.1.md).
The [feature boundaries](docs/feature-boundaries.md) document where future
animation, alert, HUD and speed changes belong.

The preserved 0.11.0 baseline improves enemy-specific alerts and combo timing. It identifies
NPC variants for spear Mikiri and grabs, resolves more boss visual effects, and
keeps a sparse alert log for longer fights. The top HUD retains its tapered rail,
green parry lead-in, white diamond and red strike emblem. The diamond travels
toward the center during wind-up; white/red marks the active parryable attack
phase, not confirmed sword contact or a successful deflect. DODGE, JUMP, MIKIRI,
NO PARRY and UNKNOWN retain distinct colors and labels.

The default mode does not require the enemy to be within reach and does not
predict contact or tell you the exact instant to press. It uses the existing
animation-batch hook and responses extracted from your game parameters.
LOCKED stays visible while the target is fresh but attack timing is unavailable.

Mikiri hints assume the skill is unlocked. Set `mikiri = false` in the config
if it is unavailable; supported thrusts then show PARRY. Unknown responses stay
explicitly UNKNOWN. See [classification evidence and limits](docs/incoming-attacks.md)
and [current validation](docs/validation-0.12.1.md).

0.9.1 fixes a reproduced DX11 graphics-state leak in the overlay renderer.
Drawing uses a separate command list with full host-state restoration. The
reported scene tint still requires an in-game comparison after a full restart.

## Install and run on Windows

1. Close Sekiro completely. Extract the newly built
   `SekiroDeflectObserver-0.12.1-preview-windows-x64.zip` into its own folder.
2. Install [me3](https://github.com/garyttierney/me3/releases), keep Steam running,
   then double-click `observer.me3` or run `launch-observer.cmd`.
3. Load a save, lock onto a living enemy, and look below the enemy's top posture bar.
   F9 displays the loaded version and research diagnostics.
4. Edit `%LOCALAPPDATA%/SekiroDeflectObserver/cue.toml` for placement, appearance
   or calibration. The file is created on first use and reloaded once per second.
   See [all defaults, bounds and reset behavior](docs/configuration.md).

For an existing configuration, use `anchor = "top"` and `width = 480`.
The button badge defaults to `parry_button = "LB"`; `L1` and `RMB` are supported.
The original scalable [strike emblem](assets/ui/strike-emblem.svg) is exported
from the same vector geometry used by the live HUD; no texture loading is needed.

Players need no Rust, Python, Visual Studio or Cheat Engine. Use one observer
loading method per session. The older **0.6.3 drop-in ZIP** described in
[the sharing guide](docs/sharing-beta.md) remains a historical package and
contains none of these changes. Do not overwrite another mod's `dinput8.dll`.
The current build is a local research candidate, not a gameplay-validated release.

| Key | Action (focused game, one fresh press) |
| --- | --- |
| F6 / F7 | Lower / raise by 8 reference pixels; persist the offset |
| F8 | Toggle gameplay HUD; persist visibility; hiding disarms practice |
| F9 | Toggle separate research panel; persist its visibility |
| F10 | Reset horizontal and vertical offsets to zero |
| F11 | Toggle enemy speed practice for this session; starts off |

All hotkeys pass through; ordinary combat input is never captured. F9 may show
research without a target but cannot enable an unlocked gameplay cue. Invalid,
dead, lost, switched or stale target observations clear timing and pulses.
The freshness ceiling remains 50 ms. Optional legacy timing mode uses bounded
projection; default incoming mode has no press/contact window.

To remove a me3 installation, close the game and stop using the observer profile;
remove its extracted folder if desired. Start normally through Steam. Keep
shared loader files used by other mods. A full process restart is required to
load a rebuilt DLL. Removing the config while closed resets all settings.

## Build and check

Install pinned Rust via rustup and Visual Studio Build Tools with Desktop C++
and the Windows SDK. From Developer PowerShell in this checkout:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build.ps1
```

The script checks formatting, Clippy, tests, builds the MSVC release DLL, tests
non-game host rejection and packages it with `cue.toml`, licenses and checksums.
It replaces a same-version ZIP; preserve previous candidates before rebuilding.
The generated tables suffice to compile; no game archives are needed to run.

```powershell
cargo fmt --all -- --check
cargo test --locked --offline --target x86_64-pc-windows-msvc
python scripts/test-attack-timings.py
python scripts/update-move-coverage.py --check
git diff --check
```

See [Windows instructions](docs/windows.md) and
[release checks](docs/release-testing.md).

## Evidence and limits

The fallback table covers **2,112 phases across 53 models**: 1,632 parry, 41 dodge,
59 jump, 66 Mikiri, 21 no-parry and 293 unknown. A further 3,730 entries describe
78 specific NPC behavior variations. These overlap the fallback entries; they
are not additional unique moves or gameplay successes. Live variant identity
and responses still need an in-game check; complete boss/form coverage is not claimed.

The generator resolves harmless warning effects and explicit Mikiri detection
hitboxes without treating them as conflicting damaging attacks. Conflicting
behavior variants stay UNKNOWN when NPC identity is unavailable. Unresolved
projectile routes remain UNKNOWN. Exact sources
and per-phase parameter IDs are in [incoming coverage](docs/incoming-coverage.json).

The DLL does not press buttons or require telemetry/accounts. Optional F11
practice temporarily writes eligible enemy animation speed; default-off behavior
does not change gameplay state. Deflect windows and Wolf's speed are untouched.
The current synthetic previews show the intended LOCKED/PARRY cues. Live HUD
comparison after the rendering fix remains outstanding. Optional `incoming_cues = false` retains
the older estimated press-window mode and its separate [timing ledger](docs/boss-move-coverage.md).
