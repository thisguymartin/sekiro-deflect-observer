# Read the HUD

The top-center rail reports the locked enemy's attack phase. The upper-right
moon reports enemy-speed practice. The two indicators can show different states
because attack hints and practice have separate controls.

![Live PARRY rail and jade practice moon above Wolf](../images/gameplay-2026-09-18-hero.jpg)

## Read the attack rail

During a parryable wind-up, the rail is green and the diamond approaches the
center. A white rail and red strike emblem mark the classified active phase.

| Caption | Meaning |
| --- | --- |
| PARRY | The classified phase permits deflection. |
| DODGE | The classified phase is a grab. No safe direction is predicted. |
| JUMP | The classified phase is a low sweep. |
| MIKIRI | The thrust has a supported Mikiri counter route. |
| NO PARRY | Deflection is disabled. No specific alternative is established. |
| UNKNOWN | The attack is known, but its response is unresolved. |
| LOCKED | The target is fresh without a current attack phase. |
| PRACTICE | Practice is armed and waiting for an eligible phase. |
| PARRY 70% | The rail reports an applied practice speed during a PARRY phase. |
| PRACTICE 70% | Practice is applied while attack hints are neutral or hidden. |

The rail follows animation progress. It does not identify weapon contact,
confirm a successful deflect, or provide an exact press time. Incoming mode can
show a response while the enemy is out of reach.

Mikiri hints assume that you unlocked the skill. Set `mikiri = false` to show
PARRY for supported deflectable thrusts instead.

## Read the practice moon

![Enlarged moon illustration comparing ON 90%, ON 80%, ON 70%, and OFF 70%](../images/0.12.4-practice-moon.png)

This renderer-generated illustration shows the indicator in the upper-right
corner. The moon grows fuller from 90% to 80% to 70% enemy speed. These presets
mean 10%, 20%, and 30% slower animation, respectively. The gray example shows
practice turned off with 70% still selected. This is a visual reference, not a
gameplay capture.

Press **F11** to turn practice on or off. Press **Shift+F11** to select
**80% -> 90% -> 70% -> 80%** without changing the on/off state. The selected
speed survives a restart, but practice always starts off.

| Appearance | Meaning |
| --- | --- |
| Gray OFF | Practice is disabled. The percentage is the selected speed. |
| Gold ON | Practice is armed and waiting. |
| Jade ON | The controller reports an applied override on the current target. |
| Amber with `!` | Practice is unavailable, paused, unsupported, or waiting for cleanup. Open F9. Cleanup can remain pending after switching OFF. |

The moon remains visible without a lock and while practice is off. `ON 70%`
alone means practice is armed with 70% selected. A jade moon and an applied
percentage in the rail, such as `PARRY 70%`, report slowdown on the current target.

F11 off keeps the gray moon and attack hints visible. F8 hides both indicators
and disables practice. Showing the HUD again does not re-enable practice.
Focus loss hides the HUD and releases the override until the game regains focus,
but keeps practice armed. Practice hotkeys work with the game focused and HUD visible.

The moon follows the playable viewport and HUD scale and opacity. Moving the
attack rail with F6 or F7 does not move the moon. Reduced-flash mode removes
the moon's active glow.

Read [the illustrated practice guide](practice.md) for gameplay screenshots of
gold, jade, and amber states, plus eligibility and cleanup behavior.

## Change the presentation

The default position is below the enemy's top posture bar. It is configured
placement, not detection of Sekiro's native HUD. Use F6 and F7 to move the rail,
or edit [the configuration file](../users/configuration.md).

Set `reduced_flash = true` to retain labels and response colors without the
white rail and red emblem transition.

Set `incoming_cues = false` only if you need the
[legacy estimated timing mode](legacy-timing.md). That mode adds READY, PARRY
NOW, WATCH, and EXPIRED states.
