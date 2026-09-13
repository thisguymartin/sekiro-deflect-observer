# Requested overhead parry cue

The user clarified the goal on 2026-09-12: an indicator above the enemy's head
that lights up when they should press deflect. This is the first usable product
requirement, replacing the earlier plan to ship a player-state display first.

The normal display should be one small cue that tracks the enemy's head and
lights green during a validated opportunity to press deflect. It must be driven
by the incoming attack, independently of whether the player has already pressed
guard. Diagnostic text belongs to the research display. Reaction-time scores,
window statistics, input automation, and changes to combat rules are not part of
the requested feature.

## Timing research

Sources inspected on 2026-09-12:

- [Animation analysis of Sekiro's standard deflect](https://www.nexusmods.com/stellarblade/articles/199)
  describes animation `a050_203000` applying effect `105010` immediately at guard
  input, with an ordinary duration of 0.2 seconds (12 frames at 60 fps). The
  article includes an animation-editor image and credits Igor's experiments.
  This is a published baseline, not a measurement of this executable or every
  guard/input condition.
- [Sekiro Resurrection's animation documentation](https://github.com/SekiroResurrection/modding-wiki/wiki/Animations)
  describes per-animation TAE event timelines and `InvokeAttackBehavior[1]`
  hitbox events. These provide a lead for mapping enemy attack phases.
- [SekiroTool](https://github.com/borgCode/SekiroTool) exposes locked-target
  options and hitbox/event views. Its source is a lead for target observation,
  not an implemented timing cue or proof of offsets on this local build.

Engineering inference: the deflect window describes how long a guard press can
accept a subsequent contact. It does not identify when an enemy will make
contact. Enemy attack events are a starting point; hitbox activation alone must
not be treated as the exact moment a blade reaches the player. Distance,
movement, animation playback, and display delay must be checked in gameplay.
Do not implement a fixed timer from any sword movement or from player effect
`105010` and label it a reliable parry prompt.

## Current implementation and recording evidence

The earlier 0.3.0-dev DLL reads only the local player's candidate effects. It has no
enemy target reader, enemy animation clock, attack timing map, head-position
reader, or camera projection. Moving its existing green label above an enemy
would not provide the requested prompt.

The supplied `Recording 2026-09-12 164102.mp4` is approximately 39.36 seconds at
30 recorded frames per second. Frame inspection shows the research panel during
combat and brief green candidate labels, including near 3.0, 9.8, and 31.8
seconds. These are visible player-effect observations, not independently
validated press opportunities. The recording contains no independent guard
input timestamps from which to establish exact press timing.

## Work required for a usable cue

1. Establish read-only, build-gated observations of the enemy target, current
   attack animation and playback time, player/target positions, and camera.
   Record source provenance and verify the layout on the local executable.
2. Derive attack phases from the installed game's TAE timelines and match them
   to the live enemy's current animation and playback time. Account for attack
   reach and player position before treating a hitbox event as imminent contact.
   Use representative gameplay checks to validate the calculation; the user
   should not manually catalogue each enemy's timings. Start with the soldier
   in the supplied recording, then check different attack types and speeds.
3. Project a validated head or target anchor through the camera to screen
   coordinates. Track camera movement and resolution changes. Hide cues for
   missing targets, invalid projection, loading, or stale observations.
4. Verify that the light precedes contact and guides a successful manual press,
   that each supported combo hit has its own opportunity, and that cancelled,
   out-of-range, or non-deflectable attacks do not produce a green prompt.

The [0.4.0 preview](cue-preview.md) now implements an overhead indicator and
an activation-based timing estimate. Exact contact prediction and gameplay
verification remain; the preview does not yet satisfy the validated press
opportunity requirement.

## Local game-file analysis

On 2026-09-12 the user supplied the installed game directory. A new read-only
archive inspector successfully indexed its character animation archives and
read actual attack timelines. See [the findings and reproduction steps](game-file-analysis.md).
This establishes that local attack event data can be obtained automatically.
Follow-up [live research](enemy-reader-research.md) now reads the locked enemy,
latest animation history frame, and normal camera pose. These reads have now
been integrated into the preview DLL. Verifying the overhead placement
visually and predicting contact still remain.
