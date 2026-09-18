# Sekiro Deflect Observer

**Read the attack. Learn the rhythm. Make the deflect yourself.**

![Wolf facing a sword enemy with the PARRY rail and practice moon visible](docs/images/gameplay-2026-09-18-hero.jpg)

Sekiro Deflect Observer is a Windows overlay for Sekiro. It identifies a locked
enemy's incoming attack and shows the current animation phase. Optional practice
mode slows eligible enemy attacks while leaving Wolf at normal speed.

Current source: **0.12.4-preview**.

## Install

1. Close Sekiro.
2. Extract `SekiroDeflectObserver-0.12.4-preview-windows-x64.zip` into its own
   folder. GitHub's **Code > Download ZIP** contains source code, not the mod.
3. Install [me3](https://github.com/garyttierney/me3/releases).
4. Keep Steam running. Open `observer.me3` or run `launch-observer.cmd`.
5. Load a save, lock onto a living enemy, and look for the top-center rail.

Players do not need Rust, Python, Visual Studio, or Cheat Engine. Use only one
observer loading method in a game session. Read the full
[installation guide](docs/users/install.md) before switching from an older
drop-in version.

## Read the HUD

The rail follows the locked enemy's attack animation. The moving diamond shows
progress toward the active phase. A white rail and red strike emblem mark the
classified active phase.

| Cue | Meaning |
| --- | --- |
| **PARRY** | The classified attack permits deflection. |
| **DODGE** | The attack is a classified grab. The observer does not choose a direction. |
| **JUMP** | The attack is a classified low sweep. |
| **MIKIRI** | The thrust has a supported Mikiri route. |
| **NO PARRY** | Deflection is disabled. No specific alternative is established. |
| **UNKNOWN** | The attack is known, but its response is unresolved. |
| **LOCKED** | A fresh target is available without a current attack cue. |

The cue describes an animation phase. It does not detect weapon contact,
confirm a successful deflect, or tell you the exact instant to press a button.
Default incoming mode can show an attack while the enemy is out of reach.

See the [gameplay walkthrough](docs/gameplay/walkthrough.md) and
[HUD guide](docs/gameplay/hud.md) for every state.

## Use practice mode

Press **F11** to enable practice for the current session. Press **Shift+F11**
to cycle through 80%, 90%, and 70% enemy animation speed. Practice starts off
after every game restart.

The moon in the upper-right corner reports practice status. Gray means off,
gold means armed, jade means applied, and amber means that the controller needs
attention. Press **F9** to see the reason for an amber state.

Practice affects eligible attacks from the locked enemy. It does not slow Wolf,
change deflect windows, or generate input. Read the
[practice guide](docs/gameplay/practice.md) before using it with bosses or other
mods that change animation speed.

## Controls

| Key | Action |
| --- | --- |
| F6 and F7 | Move the rail down or up and save the offset. |
| F8 | Show or hide the HUD. Hiding it also disables practice. |
| F9 | Show or hide diagnostics and the loaded version. |
| F10 | Reset the horizontal and vertical offsets. |
| F11 | Toggle practice for this session. |
| Shift+F11 | Change and save the practice speed. |

Hotkeys require a fresh press while the game is focused. They pass through to
the game and never capture combat input.

## Documentation

- [Install and troubleshoot](docs/users/install.md)
- [Configure the observer](docs/users/configuration.md)
- [Understand the gameplay cues](docs/gameplay/hud.md)
- [See how the observer works](docs/how-it-works.md)
- [Build and develop](docs/development/README.md)
- [Review research and validation evidence](docs/research/README.md)

The overlay uses no telemetry or online account. It reads game state, draws a
DX11 HUD, and writes enemy animation speed only to apply or restore practice
mode after you enable it. Game files and save files are not modified.
