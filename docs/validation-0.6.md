# 0.6.0-preview validation record

Date: 2026-09-12. This is an experimental local preview, not proof of precise
incoming-hit timing or universal enemy support. Existing work and older packages
were preserved. No gameplay inputs or changes to combat rules were introduced.

## Gameplay evidence reviewed before changes

Reviewed the supplied reference screenshot and the frame sheet/detail images
from `dist/game-analysis/gameplay-review-02.mp4` and its written review. Recording
02 is a 30-second, 30-fps, 1920 x 1050 soldier-combat recording. The earlier black
recording 01 was not used as gameplay evidence.

The older overlay renders during combat. Around 21 seconds the diamond enters
green and PARRY appears before visible contact sparks. The bar is excessively
high above Wolf and its press label is small. These observations motivate the
lower standing anchor and larger marker/text. They do not establish exact hit
time, manual input time, successful deflection, delayed attacks or boss accuracy.

The active game initially used PID 2508 and the 0.5.0-preview DLL from its existing
Mods folder; PID 25496 was historical. A later process check found no running
Sekiro process. No new 0.6 gameplay has been captured in this implementation pass.

## Detection investigation

The existing logs were read without modification. Full-session sample counts:

| Cue log | Rows | Valid target | Generic invalid address | No target | Changed observation | Camera error |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| observer-25496.cue.csv | 100,394 | 31,685 | 3,964 | 64,739 | 6 | 0 |
| observer-2508.cue.csv | 137,829 | 56,024 | 8,697 | 73,104 | 3 | 1 |

Both sessions include successful c5020 Chained Ogre reads (18,836 and 4,461
samples respectively). This establishes some detection, not correct attack cues.
There are 8 and 6 short gaps respectively, bounded by the same target handle
within 250 ms. They include changed observations, no-target rows and one camera
error. These are not a measured lock-on failure rate: continuous lock state and
per-frame rendering were not recorded in these old CSVs. Generic invalid-address
rows cannot be attributed to one pointer or subsystem.

The old arbitrary 256-slot actor-bucket cap was replaced with the 14-bit handle
bound, retaining selected-slot and ownership checks. The live research probe
reported a bucket-limit error, but a later direct bucket snapshot showed a
maximum of 144; a bucket above 256 was not captured. Therefore this change is
a bounds correction, not a demonstrated explanation of the reported dropout.

Other changes address plausible failure paths without concealing invalid reads:
one bounded retry for changed ownership/animation observations, neutral LOCKED
for animation-only failures, shorter render mutex scope, and staged read reasons.
New render.csv records drawing decisions separately from sampled estimates.
The historical intermittent disappearance remains unconfirmed as fixed.

Local reproducible analysis and evidence: `dist/review-0.6/analyze_logs.py`,
`log-analysis.json`, and `live-buckets.json` in the same folder. These are ignored
local artifacts, not bundled runtime dependencies.

## Data and automated checks

All 79 base enemy/NPC animation archives were re-extracted into the separate
`dist/game-analysis-v2` folder. Original extractions remain intact. Local attack
parameters and additional event fields supply response inferences. Sources,
interpretation rules and limits are in [cue scope](cue-preview.md),
[model coverage](enemy-coverage.json) and [response evidence](response-coverage.json).

The result contains 2,161 phases across 54 models, 450 green estimates across
39 models, 39 dodge phases and 58 jump phases. Smaller green coverage than 0.5.0
is intentional: overlapping hitboxes and uncertain responses no longer produce
separate or questionable parry prompts. None of these counts is gameplay proof.

Checks completed for this implementation:

- 33 Rust tests passed, including latest ring entry/wrap, every required-byte
  read failures, ownership changes, neutral idle state, fresh/stale behavior,
  projection, bucket bound, next-combo priority, and Ogre/Ape response separation.
- 8 Python checks passed for imports, overlap grouping, malformed timing,
  response exclusions and parameter bitfield/schema handling.
- Formatting check and Clippy for all Windows targets passed with warnings denied.
- The optimized Windows x64 release DLL built with the locked offline dependencies.
- DLL startup check passed: an unrelated PowerShell host was rejected without
  installing rendering hooks. This is not a Sekiro hook/render integration test.
- The shared live ImGui drawing code rendered READY, PARRY, DODGE and JUMP in the
  offline example; the resulting layout was visually inspected. It checks text,
  marker, colors and geometry, not camera placement or DX11 presentation in game.

The visual check is `dist/review-0.6/layout/cue-layout.png`, generated with
`cargo run --locked --offline --example cue-layout --target x86_64-pc-windows-msvc`
and `python scripts/render-cue-layout.py`. Its software texture filtering differs
from the live renderer. It is explicitly labeled as synthetic, not gameplay.

## Required next gameplay validation

Restart with the new package and confirm 0.6.0-preview in F9. Record the exact
PID, new cue/render CSVs, capture start time, display size and video frame rate.
For each tested move, retain the model/animation/phase and compare first submitted
action cue, actual visible contact and the player's input. A 30-fps recording alone
cannot support millisecond-precise claims. Sparks or candidate effect 105010 alone
do not establish successful deflection.

Priorities are ordinary soldier combos and delayed swings, Ogre grabs, then Ape
grab/sweep/sword combinations. Check both mapped and unverified phases; do not
generalize one move to its entire boss or another form. Compare cancellations,
distance/facing, camera turns, lock changes, paused clocks, death and loads.
Confirm F6/F7 adjustment and the lowered bar in standing, crouching and jumping.

Still unimplemented or unverified: animated head tracking, exact weapon/player
collision timing, complete response classification, runtime behavior variation,
all enemy/form coverage, projectile prediction, successful dodge direction, and
new live placement and timing. The overlay never automates the player's response.
