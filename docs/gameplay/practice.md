# Use enemy-speed practice

Practice mode slows eligible attacks from the locked enemy. Wolf, the global
game clock, and deflect windows remain unchanged.

## Start practice

1. Keep Sekiro focused and the HUD visible. If you hid it, press **F8** to show it.
2. Press **F11** to arm practice for the current game session. The moon shows `ON`.
3. Press **Shift+F11** to cycle through **90% -> 80% -> 70% -> 60% -> 90%** enemy speed.
4. Lock onto an enemy and watch the moon during an eligible attack. Jade means
   the controller reports an applied speed override.
5. Press **F11** again to disable practice. The moon returns to gray `OFF`
   after cleanup, and attack hints remain available.

New settings select 90% speed. Practice starts off after every game restart.
The selected percentage persists.
Changing the selected speed does not turn practice on or off. Both practice
hotkeys require the game to be focused and the HUD to be visible.

## Recognize the moon and selected speed

![Enlarged comparison of ON 90%, ON 80%, ON 70%, and gray OFF 70% moon states](../images/0.12.4-practice-moon.png)

This is a renderer-generated illustration of the upper-right indicator,
enlarged so you can compare its shape and text. It is not a gameplay screenshot.
This illustration shows the original three presets. The thin moon represents
90% speed, the wider moon 80%, and the fullest moon 70%. The 60% preset shares
the fullest shape and displays 60%.
The sword remains inside the moon at each level.

The percentage is the enemy's selected animation speed relative to its original
speed. A lower percentage gives you more time to watch the attack:

| Selected speed | Slowdown | Time for one remaining second of animation |
| --- | --- | --- |
| 90% | 10% slower | About 1.11 real seconds. |
| 80% | 20% slower | About 1.25 real seconds. |
| 70% | 30% slower | About 1.43 real seconds. |
| 60% | 40% slower | About 1.67 real seconds. |

At 80%, a remaining 400 ms wind-up takes about 500 ms while the override is
applied. These are multiplier examples, not measurements from the recording.
Detection happens after an animation starts, so a whole move may take a
different amount of extra time. Changing presets during an attack restores the
original speed before applying the new multiplier. The reductions do not stack.

The moon stays visible while practice is off and when you have no locked target.
Its percentage always shows your selection. Use the color and the rail caption
to distinguish a saved selection from slowdown currently applied to an attack.

## Read the status during combat

| Moon | Meaning |
| --- | --- |
| Gray `OFF` | Practice is disabled. The selected speed is ready for next time. |
| Gold `ON` | Practice is armed and waiting for an eligible phase. |
| Jade `ON` | The controller reports an applied override on the current target. |
| Amber with `!` | Practice is unavailable, paused, unsupported, or waiting for cleanup. Open **F9** for the reason. |

### Gold means enabled and waiting

![Gold ON 70% moon in the upper-right corner with a neutral PRACTICE rail](../images/gameplay-2026-09-18-practice-ready.jpg)

At 00:40.000 in the recording, the upper-right moon is gold and reads `ON 70%`.
The top-center rail says `PRACTICE`, without a percentage. Practice is enabled
and waiting for an eligible attack. It does not continuously slow the enemy's
ordinary movement or recovery just because `ON` is visible.

### Jade means an override is applied

![Jade ON 70% moon with a green PARRY 70% rail during an eligible attack](../images/gameplay-2026-09-18-hero.jpg)

At 00:38.500, the moon is jade and the rail says `PARRY 70%`. Together these
report an applied override on the locked enemy. The rail still describes the
attack phase. You must read the enemy's movement and make the deflect yourself.

### Amber means check the reason in F9

![Amber ON 70% moon with an exclamation mark beside a PARRY rail without a percentage](../images/gameplay-2026-09-18-practice-attention.jpg)

At 00:47.000, the moon is amber with `!`, while the rail says `PARRY` without
an applied percentage. Attack hints can remain available when practice needs
attention. Press **F9** to read the controller status. The recording does not
show that panel, so this frame does not tell us which problem occurred.

An amber warning can remain after turning practice off if cleanup is still
pending. Check F9 rather than treating `OFF` alone as confirmation of restoration.

## Turn slowdown off or hide the HUD

Press **F11** to turn practice off while keeping the HUD visible. The controller
restores the enemy's saved speed, and the moon becomes gray with `OFF` and your
selected percentage. You can continue using attack hints at normal enemy speed.
The gray `OFF 70%` example in the illustration shows this state.

Press **F8** to hide both the attack rail and moon. This also disables practice
and requests restoration. Pressing **F8** again shows the HUD with practice still
off. Press **F11** separately if you want to arm it again.

Switching away from the game hides the HUD and releases the current override.
The practice toggle stays armed, so it can apply again to an eligible attack
after you return. Restarting Sekiro always starts practice off.

F6, F7, and F10 change the attack rail's placement. The moon keeps its separate
upper-right position and follows the HUD's scale and opacity settings.

## Know which attacks are eligible

Practice supports recognized PARRY, JUMP, and MIKIRI wind-up and active phases
on the locked enemy. It can stay applied between recognized hits in a combo and
restores the original speed after the last eligible phase.

Practice does not request slowdown for:

- DODGE grabs.
- NO PARRY or UNKNOWN phases.
- Normal movement and recovery.
- Another enemy.
- A stale or missing animation.
- A hidden or unfocused HUD.

Response-label settings do not change eligibility. For example, hiding PARRY
hints can produce a neutral `PRACTICE 80%` rail while the same attack remains
slowed.

## Understand the limits

Detection occurs after an animation starts, so a complete move does not always
last exactly as long as its selected multiplier suggests. Projectile flight,
AI timers, sound, and other independent systems are not rescaled. Paired grabs
are excluded because changing one participant could desynchronize the action.

Practice does not generate input or predict contact. It does not widen Wolf's
deflect window. The current gameplay recording demonstrates the HUD state, not
a measured animation-rate change.

The controller restores the saved original speed when you disable practice,
hide the HUD, lose focus, lose or switch targets, or leave an eligible phase. If another
mod changes the same speed value, practice pauses instead of overwriting the
external change. Toggle F11 off and on after resolving the conflict.

See [practice implementation and evidence](../research/practice-evidence.md) for
the memory field, ownership checks, audit log, and remaining live tests.
