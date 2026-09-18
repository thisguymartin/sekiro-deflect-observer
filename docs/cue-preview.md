# Current HUD preview

The 0.12.1 preview uses a compact 480 x 18 reference-pixel rail at `(0.5, 0.16)`
of the playable viewport. This places the rail below the enemy's top posture bar.
The gallery uses the same drawing code as the live overlay with synthetic attack
and practice states. Normal play shows one rail at a time.

![Current 0.12.1 HUD and practice states](images/0.12.1-practice-gallery.png)

The diamond approaches the center gate during wind-up. A green segment marks a
parryable wind-up. A white segment and the red strike emblem mark the active
parryable phase. They do not prove sword contact or a successful deflect. DODGE,
JUMP, MIKIRI, NO PARRY, and UNKNOWN use separate labels and colors.

F11 arms enemy-speed practice for the current process. `PRACTICE` means armed and
waiting. `PRACTICE 80%` means the controller read back an applied 80% speed value.
F8 hides the HUD and disarms practice. Alert settings do not change practice
eligibility.

![Practice remains visible when attack hints are disabled](images/0.12.1-practice-no-hints.png)

The editable [strike emblem](../assets/ui/strike-emblem.svg) uses the same vector
geometry as the live HUD. The observer loads no texture for it. Reduced-flash
mode keeps the response color and suppresses the white and red transition.

## States

| State | Meaning |
| --- | --- |
| PARRY | The classified phase permits deflection. |
| DODGE | The classified phase is a grab. The observer does not choose a direction. |
| JUMP | The classified phase is a low sweep. |
| MIKIRI | The classified thrust has an explicit supported counter route. |
| NO PARRY | Deflection is disabled and no specific alternative is established. |
| UNKNOWN | The attack is known but its response is unresolved. |
| LOCKED | The target is fresh but no current attack phase is available. |
| PRACTICE | Practice is armed and waiting for an eligible phase. |
| PRACTICE 80% | The observer applied and read back the configured speed. |

Set `incoming_cues = false` to use the legacy estimated timing mode. That mode
adds READY, PARRY NOW, WATCH, and EXPIRED states. See
[legacy timing](parry-cue.md) and [configuration](configuration.md).

## Reproduce the previews

Run these commands from Developer PowerShell:

```powershell
cargo run --locked --offline --example cue-layout --target x86_64-pc-windows-msvc -- dist/review-0.12.1/layout/practice --incoming-gallery --practice
cargo run --locked --offline --example cue-layout --target x86_64-pc-windows-msvc -- dist/review-0.12.1/layout/no-hints --state incoming-parry --practice --no-hints
python scripts/render-cue-layout.py dist/review-0.12.1/layout/practice
python scripts/render-cue-layout.py dist/review-0.12.1/layout/no-hints
```

The previews verify renderer output and layout bounds. They do not verify live
placement, enemy slowdown, input timing, contact, or a successful defensive
action. See [current validation](validation-0.12.1.md) and the
[gameplay checklist](../tests/manual/gameplay-checklist.md).
