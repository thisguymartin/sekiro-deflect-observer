# Incoming response cue 0.6.0-preview

The slider follows Wolf's position and uses the locked enemy's current animation
to anticipate selected attacks. It never presses buttons or starts a timer from
guard input. Timing is estimated from attack activation, not confirmed contact.

## Display and controls

At the 1080p reference scale the track is 360 x 16 pixels, with a 10-pixel-radius
outlined diamond and a 26-pixel action label on a dark backing. READY gives a
legible lead-in before PARRY. The label stays above the track instead of moving
with the diamond. Label visibility is not prolonged past the estimated interval.

| Color / text | Meaning |
| --- | --- |
| Green PARRY | Selected melee phase; 150 ms of animation time before activation |
| Orange DODGE | Mapped grab; separate 300 ms advance estimate |
| Blue JUMP | Mapped low sweep; separate 300 ms advance estimate |
| Gray TIMING UNVERIFIED / SPECIAL UNVERIFIED | No confident response/timing guidance |
| LOCKED | Fresh target, no active timing cue |

The timer zooms to the final 0.65 seconds of wind-up and up to 0.25 seconds of
recovery. Earlier wind-up holds the diamond at the left. Animation playback
speed affects elapsed real time. These estimates do not change any game window.
DODGE does not predict a safe direction or establish invulnerability timing.

F6 lowers the track; F7 raises it by 8 reference pixels per fresh press, bounded
to +/-160 pixels. Adjustment resets when the DLL restarts. F8 toggles visibility;
F9 toggles diagnostics. All controls pass through to the game.

The default anchor is now player root +1.55 vertical world units, lowered from
2.15 after reviewing recording 02. It follows the player, not the enemy. This
is still a standing approximation: animated head-bone tracking is not implemented.
New alignment, crouches, jumps and acrobatics need gameplay verification.

## Extracted coverage

All 79 base c1000-c7999 animation archives in the local inventory were re-read
with additional event fields. The current table has 2,161 phases across 54 models,
450 green estimates across 39 models, 39 dodge phases and 58 jump phases.
There are 1,830 animations with special/projectile event indicators; many have
no timed response. These are data counts, not verified enemy or move counts.

| Model | Phases | Green estimates | Dodge phases | Jump phases |
| --- | ---: | ---: | ---: | ---: |
| Chained Ogre c5020 | 26 | 6 | 5 | 0 |
| Guardian Ape c5100 | 93 | 4 | 1 | 4 |

Animation variants can duplicate a move. Some Ogre grabs and Ape low sword sweeps
are mapped; other attacks and forms remain unverified. No claim covers every
enemy, miniboss or boss. See [per-model data](enemy-coverage.json) and
[response mappings with parameter IDs and source hashes](response-coverage.json).

Green coverage deliberately decreases from 819 phases in 0.5.0: overlapping
hitboxes are merged and stricter attack-parameter/event checks reject questionable
prompts. Separate combo activations are retained. The next phase's advance cue
can replace the previous phase's recovery display, avoiding a missed quick cue.

Shared full animation imports use their referenced timeline. Identical duplicate
definitions are coalesced; conflicting definitions and cyclic/missing references
are excluded. The table includes 54 imported phases, of which 3 with local events
remain uncertain; 3 import entries cannot resolve. Similar-looking model IDs do
not inherit another model's timings. No manual per-move timing measurements are
required to generate these data.

## Response inference and exclusions

The extractor now reads TAE attack type / behavior judge IDs, throw behavior,
action flags and aim type. Local game parameters supply 2,396 AtkParam_Npc rows
and 3,128 BehaviorParam rows. NpcParam and ThrowParam are also extracted for
research; they are not complete runtime response selectors.

The offline join uses NPC melee BehaviorParam entries grouped by model/variation
and judge ID. Every matching melee variation must reference an existing attack
whose reference name identifies that model, and variants must agree on response.
Nonstandard attack-type routing stays unverified. Actual runtime variation
selection has not been independently established.

An incoming throw flag maps to DODGE; grab-damage stages do not. JUMP requires
both deflection-disable flags plus an explicit low-sweep description in the
reference names. Ape attack rows 51000541 and 51000560 are explicit low-sweep
interpretations of the translated names. This is a source-based inference,
not a complete or gameplay-proven response classifier. Aim type 7 alone is not
enough to infer JUMP. Other nondeflectable or ambiguous attacks stay unverified.

Green additionally excludes projectile/mixed animations, throw/action/aim
indicators, common-behavior or effect-add events, uncertain imports, nonstandard
attack routing, lead-ins under 0.1 seconds and continuous hitboxes over 0.4
seconds. Attack names and parameters do not independently prove that a hit will
reach Wolf or that pressing within the displayed interval succeeds.

Primary format references:

- [DSAnimStudio Sekiro TAE template](https://github.com/Meowmaritus/DSAnimStudio/blob/master/DSAnimStudioNETCore/Res/TAE.Template.SDT.xml)
- [Paramdex attack definition](https://github.com/soulsmods/Paramdex/blob/master/SDT/Defs/AtkParam.xml) and [behavior definition](https://github.com/soulsmods/Paramdex/blob/master/SDT/Defs/BehaviorParam.xml)
- [Paramdex attack names](https://github.com/soulsmods/Paramdex/blob/master/SDT/Names/AtkParam_Npc.txt)
- [SoulsFormats PARAM reader](https://github.com/JKAnderson/SoulsFormats/blob/master/SoulsFormats/Formats/PARAM/PARAM/PARAM.cs)

## Detection and diagnostics

The latest entry in the ten-entry animation history ring retains its 0x14 stride.
A fresh valid lock remains neutral while idle or when only the animation read
fails. Failed animations are not reused. Changed observations get one retry
within the existing read budget; ownership is rechecked. Stale observations
still expire at 50 ms. Lost lock, dead actors and invalid dependent reads hide
the bar. The bucket bound now follows the handle's 14-bit index instead of an
arbitrary 256-entry cap; this was not confirmed as the historical dropout cause.

The render callback takes a short snapshot lock, then draws without holding it.
New cue CSV fields distinguish lock-disabled, selected-point, ownership,
animation, camera and other stages. A separate bounded nonblocking queue records
render callback decisions in observer-PID.render.csv. F9 shows live version,
read status, log availability and dropped render-record count.

Logs live in %LOCALAPPDATA%\SekiroDeflectObserver; each CSV stops at 16 MiB.
The cue/render headers record version and epoch for recording alignment.
estimated_press=true in cue.csv is a calculation only. press_submitted or
response_submitted in render.csv records an ImGui draw submission, not proof
of presentation, contact or successful input. Video and input evidence remain
necessary. The samples.csv candidate-effect reader is a separate research aid.

## Remaining validation

Recording 02 shows the old bar rendering in soldier combat, high above Wolf,
with a brief PARRY before visible sparks around 21 seconds. It does not prove
exact contact timing or a successful manual deflect. The 0.6 layout is checked
offline using the actual shared drawing code; new gameplay is still required.
See [the current evidence and outstanding trials](validation-0.6.md).

Reach uses distance, approximate body bounds, height and facing, not weapon
collision geometry. Blends, delayed strikes, projectile timing, unusually large
attacks and runtime behavior selection remain limitations. Lock-on is required.
Normal camera aspect must match the display and be 16:9 or wider; debug cameras
and invalid/offscreen projections suppress the bar.

## Reproduce data and visual checks

From the project root, after obtaining the documented public references:

```powershell
python scripts/inspect-game-archives.py 'C:\Program Files (x86)\Steam\steamapps\common\Sekiro' --all-enemies --output-directory dist/game-analysis-v2
python scripts/inspect-attack-params.py 'C:\Program Files (x86)\Steam\steamapps\common\Sekiro'
python scripts/generate-attack-timings.py
python scripts/test-attack-timings.py
cargo fmt --all
cargo test --locked --offline --target x86_64-pc-windows-msvc
cargo run --locked --offline --example cue-layout --target x86_64-pc-windows-msvc
python scripts/render-cue-layout.py
```

The last command uses the Pillow already installed under dist/video-tools.
The layout output is synthetic, not gameplay. Runtime packages require neither
Python nor extracted archives. Public references and local game data remain in
ignored dist folders; source hashes and derived coverage are recorded in docs.

