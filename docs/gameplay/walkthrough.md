# Follow one attack through the HUD

These images come from the September 18, 2026 gameplay recording. The recording
shows the 0.12.4-preview HUD design at 70% practice speed. It does not show F9,
so it does not identify the loaded DLL hash.

## Watch the sequence

![Gameplay sequence from wind-up through the return to PRACTICE](../images/gameplay-2026-09-18-demo.gif)

The clip covers 00:36.800 through 00:40.400 at normal playback speed.

## Read the incoming attack

![PARRY 70% rail and jade ON 70% moon](../images/gameplay-2026-09-18-hero.jpg)

At 00:38.500, the green rail identifies a parryable phase. The diamond moves
toward the center as the animation advances. `PARRY 70%` and the jade moon show
that practice reports an applied speed on the locked target.

## Recognize the active phase

![White rail and red strike emblem during the active phase](../images/gameplay-2026-09-18-active.jpg)

At 00:39.500, the rail turns white and the red strike emblem appears. The HUD
marks the classified active phase. It does not establish the exact contact time
or prove a successful deflect.

## Distinguish armed from applied

![PRACTICE rail and gold ON 70% moon](../images/gameplay-2026-09-18-practice-ready.jpg)

At 00:40.000, the rail returns to `PRACTICE`. The gold moon means practice is
armed and waiting. The selected 70% speed is not continuously applied.

## Check an attention state

![PARRY rail and amber ON 70% moon](../images/gameplay-2026-09-18-practice-attention.jpg)

At 00:47.000, the amber moon reports an attention state. The recording does not
show F9, so the exact cause is unknown. Attack hints continue to work because
their state is separate from practice.

## What the recording proves

The recording shows the rail, marker, active-phase emblem, and gold, jade, and
amber practice states during combat.

It does not prove the loaded build, measured enemy speed, unchanged player
speed, successful restoration after every lifecycle event, input-to-contact
timing, or compatibility across systems.

Read [the capture record](../research/gameplay-capture.md) for file properties,
hashes, extraction commands, and the limits of this evidence.
