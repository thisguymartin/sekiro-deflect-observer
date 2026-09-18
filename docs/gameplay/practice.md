# Use enemy-speed practice

Practice mode slows eligible attacks from the locked enemy. Wolf, the global
game clock, and deflect windows remain unchanged.

## Start practice

1. Press **F11** to arm practice for the current game session.
2. Press **Shift+F11** to cycle through 80%, 90%, and 70% enemy speed.
3. Watch the upper-right moon for the controller status.
4. Press **F11** again to disable practice.

Practice starts off after every game restart. The selected percentage persists.
At 70%, an eligible animation runs 30% slower than its original speed.

## Read the status

| Moon | Status |
| --- | --- |
| Gray | Off. |
| Gold | Armed and waiting for an eligible phase. |
| Jade | Applied to the current target. |
| Amber with `!` | Attention is required. Open F9 for the reason. |

The displayed percentage is the selected speed, even while practice is off or
waiting. An applied percentage also appears in the attack rail caption.

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
hide the HUD, lose focus, switch targets, or leave an eligible phase. If another
mod changes the same speed value, practice pauses instead of overwriting the
external change. Toggle F11 off and on after resolving the conflict.

See [practice implementation and evidence](../research/practice-evidence.md) for
the memory field, ownership checks, audit log, and remaining live tests.
