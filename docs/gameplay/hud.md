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

| Appearance | Meaning |
| --- | --- |
| Gray OFF | Practice is disabled. The percentage is the selected speed. |
| Gold ON | Practice is armed and waiting. |
| Jade ON | The controller reports an applied override on the current target. |
| Amber ON with `!` | Practice is unavailable, paused, unsupported, or waiting for cleanup. Open F9. |

The moon remains visible without a lock. F8 hides both indicators and disables
practice. Focus loss hides the HUD until the game regains focus.

Read [the practice guide](practice.md) for eligibility and cleanup behavior.

## Change the presentation

The default position is below the enemy's top posture bar. It is configured
placement, not detection of Sekiro's native HUD. Use F6 and F7 to move the rail,
or edit [the configuration file](../users/configuration.md).

Set `reduced_flash = true` to retain labels and response colors without the
white rail and red emblem transition.

Set `incoming_cues = false` only if you need the
[legacy estimated timing mode](legacy-timing.md). That mode adds READY, PARRY
NOW, WATCH, and EXPIRED states.
