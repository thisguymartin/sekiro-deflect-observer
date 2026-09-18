# Sekiro Deflect Observer

**Read the attack. Learn the rhythm. Make the deflect yourself.**

![Wolf facing a sword enemy with the PARRY rail and practice moon visible](docs/images/gameplay-2026-09-18-hero.jpg)

Actual gameplay at 00:38.500 in the September 18 recording. The top-center
rail shows `PARRY 70%`. The jade moon in the upper-right corner shows `ON 70%`.
Both report that practice is applied to the locked enemy in this frame.

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

![Gameplay sequence showing the green wind-up rail, white active phase, and return to PRACTICE](docs/images/gameplay-2026-09-18-demo.gif)

Watch the diamond approach the center during the wind-up. The rail then turns
white and shows a red strike emblem during the active phase. Between eligible
attacks, it returns to `PRACTICE` while practice is armed, or `LOCKED` with
practice off. The `LB` badge names your configured parry button; it can also
display `L1` or `RMB`. You still time and press the button yourself.

The clip covers 00:36.800 through 00:40.400 at normal playback speed.
[Follow each frame in the illustrated walkthrough](docs/gameplay/walkthrough.md).

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
to cycle through **80% -> 90% -> 70% -> 80%** enemy animation speed. Press
**F11** again to turn practice off. Changing the speed keeps the current on/off
state. Practice starts off after every game restart, but remembers your speed.

![Enlarged moon comparison showing ON at 90%, 80%, and 70%, followed by OFF at 70%](docs/images/0.12.4-practice-moon.png)

This enlarged renderer-generated illustration shows the upper-right moon's
presets and off state. The moon grows fuller as you select more slowdown.
90% means 10% slower, 80% means 20% slower, and 70% means 30% slower.
This illustration is separate from the real gameplay images above.

The moon shows `OFF` or `ON` and your selected speed, even without a target.
Read its color to tell whether slowdown is currently applied:

| Moon | What it tells you |
| --- | --- |
| Gray `OFF` | Practice is disabled. The percentage is saved for next time. |
| Gold `ON` | Practice is armed and waiting for an eligible attack. |
| Jade `ON` | The controller reports an applied speed override on the locked enemy. |
| Amber with `!` | Practice needs attention. Press **F9** for the reason. |

For example, a gold `ON 70%` moon means 70% is selected and practice is waiting.
A jade moon together with `PARRY 70%` on the rail reports an applied override.
The percentage beside the moon alone does not mean the enemy is being slowed.

Turning practice off with **F11** keeps the gray moon and attack hints visible.
**F8** hides the entire HUD and disables practice. After showing the HUD again,
press **F11** if you want to re-enable slowdown.

Practice affects eligible attacks from the locked enemy. It does not slow Wolf,
change deflect windows, or generate input. Recognized parryable attacks, sweeps,
and supported thrusts are eligible. Grabs, unknown moves, ordinary movement,
and other enemies keep their normal speed. Read the
[illustrated practice guide](docs/gameplay/practice.md) for the live moon states,
speed examples, and what happens when you switch targets or leave the game window.

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
