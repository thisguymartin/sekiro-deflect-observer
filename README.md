# Sekiro Deflect Observer

**Read the attack. Learn the rhythm. Make the deflect yourself.**

![Wolf facing a sword enemy on a snowy bridge, with the green PARRY 70% rail and jade practice moon visible](docs/images/gameplay-2026-09-18-hero.jpg)

*Actual gameplay from the September 18 recording, at 00:38.500. The black side
margins have been cropped; the game and HUD are unchanged.*

A Windows overlay for Sekiro that identifies a locked enemy's incoming attack
and shows its animation progress. Optional practice mode slows eligible enemy
attacks to **90%, 80% or 70% speed** while leaving Wolf's speed unchanged.
You control every block, deflect, dodge and counter.

**Current source: 0.12.4-preview.**
[Install](#install-and-run-on-windows) · [Controls](#controls) ·
[Gameplay walkthrough](docs/screenshots.md) · [Configuration](docs/configuration.md) ·
[All docs](docs/README.md)

## How it works

1. **Lock onto a living enemy.** The top-center rail follows that target's
   captured attack animation. Between recognized attacks it shows `LOCKED`, or
   `PRACTICE` while practice is armed and waiting.
2. **Read the response and moving diamond.** For a parryable wind-up, the rail
   lights green and the diamond approaches the center. The `LB` badge names the
   configured parry button; it can also display `L1` or `RMB`.
3. **Watch the attack phase.** The white rail and red strike emblem mark the
   active parryable phase. Use the enemy's movement to time your own input.

![Gameplay loop showing the diamond approaching the center, the white active-phase cue, and the return to PRACTICE](docs/images/gameplay-2026-09-18-demo.gif)

*00:36.800–00:40.400 from the same recording, at normal playback speed; reduced
to 640 × 360 and 15 fps for the README. [Full-size stills and explanations](docs/screenshots.md).*

The cue describes the enemy's animation phase. **It does not confirm contact,
detect a successful deflect, or give an exact “press now” instruction.** Default
incoming mode can show an attack even when the enemy is out of reach.

| Cue | Meaning |
| --- | --- |
| **PARRY** | The classified attack permits deflection. |
| **DODGE** | A classified grab; no dodge direction is chosen. |
| **JUMP** | A classified low sweep. |
| **MIKIRI** | A thrust with a supported Mikiri counter route. |
| **NO PARRY** | Deflection is disabled; no specific alternative is established. |
| **UNKNOWN** | The attack is known but its response is unresolved. |
| **LOCKED** | A fresh target is available without a current attack cue. |

Mikiri hints assume you have unlocked the skill. Set `mikiri = false` if you
have not; supported deflectable thrusts then show PARRY. These response types
come from the implementation; the new recording illustrates the PARRY sequence.
See [classification and coverage](docs/incoming-attacks.md) for the other types.

## Practice at your pace

Press **F11** to enable practice for this session. **Shift+F11** cycles
**80% → 90% → 70% → 80%** and saves your selected speed. Practice starts **off**
each time you launch the game; the speed selection is remembered.

The crescent moon in the upper-right corner shows `OFF` or `ON` and the selected
percentage, even without a target. Its color reports the controller's status:

| Moon | Status |
| --- | --- |
| Gray | Practice is off. |
| Gold | Armed and waiting for an eligible attack. |
| Jade | The controller reports an applied override on the current target. |
| Amber **!** | Attention needed; open F9 for the reason. |

In the main image, **PARRY 70%** and the jade moon report an applied speed
override. That means 70% of the enemy's original animation speed, or 30% slower.
The percentage next to the moon alone is the selected preset, even while waiting.

Practice affects eligible parryable, thrust and sweep attack phases on your
locked target. Grabs, unknown/no-parry moves and other enemies are excluded.
Wolf's speed and deflect windows are unchanged. Turning off attack hints does
not disable practice; **F8 hides the HUD and disarms it**. See
[practice behavior and limits](docs/enemy-speed-practice.md).

## Install and run on Windows

1. Close Sekiro completely. Extract
   `SekiroDeflectObserver-0.12.4-preview-windows-x64.zip` into its own folder.
   Use a built package supplied by the author, or [build from source](#build-and-check).
   GitHub's **Code → Download ZIP** contains source, not the ready-to-run mod.
2. Install [me3](https://github.com/garyttierney/me3/releases), keep Steam running,
   then double-click `observer.me3` or run `launch-observer.cmd` from the package.
3. Load a save, lock onto a living enemy, and look for the top-center rail.
   **F9** shows the loaded version and diagnostics.
4. Press **F11** if you want optional enemy-speed practice.

Players need no Rust, Python, Visual Studio or Cheat Engine. Use one observer
loading method per session. The historical 0.6.3 drop-in package does not contain
the current features; do not overwrite another mod's `dinput8.dll`.

Settings live in `%LOCALAPPDATA%/SekiroDeflectObserver/cue.toml`. The file is
created on first use and reloaded once per second. For an older layout, use
`anchor = "top"`, `width = 480` and F10 to reset offsets. See
[all settings](docs/configuration.md) and [Windows troubleshooting](docs/windows.md#troubleshoot).

## Controls

| Key | Action |
| --- | --- |
| F6 / F7 | Lower / raise the rail by 8 reference pixels; save the offset. |
| F8 | Show / hide the gameplay HUD; hiding also disarms practice. |
| F9 | Show / hide the diagnostics panel and loaded version. |
| F10 | Reset horizontal and vertical offsets. |
| F11 | Toggle enemy-speed practice for this session. |
| Shift+F11 | Switch and save 80% / 90% / 70% speed without changing on/off. |

Hotkeys require a fresh press while the game is focused and pass through to the
game. Combat input is never captured or automated. Lost, dead, switched or stale
targets clear attack guidance; the observation freshness ceiling is 50 ms.

To uninstall, close the game and stop using the observer's me3 profile. Remove
its extracted folder if desired, keep shared loader files, and launch normally
through Steam. A rebuilt DLL requires a full game restart. Removing the local
config while the game is closed resets settings.

## Status and coverage

The [new recording](docs/screenshots.md) shows the live PARRY rail, active-phase
emblem and 70% practice status. It provides a visual demonstration, not a
controlled measurement of slowdown or deflect success. The clip does not show
F9 or identify the loaded DLL hash. Exact timing, cleanup across game transitions,
boss/form coverage and the earlier scene-tint report still need dedicated checks.
See [current validation](docs/validation-0.12.4.md).

The fallback data classifies **2,112 phases across 53 models**, including **293
unknown** phases. Another 3,730 entries describe 78 NPC behavior variations;
these overlap the fallback data and are not additional unique moves or validated
successes. [Detailed evidence](docs/incoming-attacks.md) and
[per-phase coverage](docs/incoming-coverage.json) document the limits.

The overlay uses no telemetry or account. F11 explicitly enables temporary
enemy animation-speed writes; it starts disabled. Game files, saves, Wolf's
speed and deflect windows are not modified. Optional `incoming_cues = false`
selects the separate [legacy estimated timing mode](docs/parry-cue.md).

## Build and check

Install the pinned Rust toolchain through rustup and Visual Studio Build Tools
with Desktop C++ and the Windows SDK. From Developer PowerShell:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build.ps1
```

The script checks formatting, Clippy and tests, builds the MSVC release DLL,
checks non-game host rejection, and packages configuration, licenses and
checksums. It replaces a same-version ZIP; preserve previous candidates first.
Generated tables are included, so building does not require game archives.

See [Windows build instructions](docs/windows.md),
[release checks](docs/release-testing.md), [architecture](docs/architecture.md)
and [feature ownership](docs/feature-boundaries.md).
