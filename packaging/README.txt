Sekiro Deflect Observer 0.6.0-preview
Windows x64 incoming-attack cue preview

The timing slider follows Wolf and anticipates selected locked-enemy attacks.
You press the buttons yourself. Timing and response classification remain
experimental; attack activation is not confirmed contact with the player.

INSTALL AND RUN

1. Close Sekiro fully. Keep Steam running.
2. Keep this package's extracted files together in their own observer folder.
3. Double-click observer.me3, or run launch-observer.cmd.
   me3 must be installed: https://github.com/garyttierney/me3/releases
4. Load a save and lock onto an enemy. Look just above Wolf for the slider.
5. Follow the diamond toward the colored section and its English action label:
   GREEN / PARRY: estimated deflect press timing.
   ORANGE / DODGE: mapped incoming grab; dodge/evade, direction is not predicted.
   BLUE / JUMP: mapped low sweep; estimated jump cue.
   GRAY / UNVERIFIED: no reliable response or timing guidance for this attack.
   LOCKED: target is visible to the observer, with no active timing cue.

F6 lowers the bar; F7 raises it, by 8 reference pixels per press (session only).
F8 hides/shows the indicator. F9 shows/hides optional diagnostics.
Close Sekiro to stop the DLL. Start normally through Steam for an ordinary run.
A running game keeps the previously loaded version until it exits completely.

WHAT CHANGED

The bar is lower, the diamond is larger, and the action text is easier to read.
The 360 x 16 track scales with resolution. Its anchor follows Wolf's position
plus standing height; it does not yet follow the animated head during crouches
or acrobatics. F6/F7 allow placement adjustment without editing files.

Parry guidance remains 150 ms of animation time before eligible activation.
Dodge and jump use a separate experimental 300 ms advance cue. Neither changes
the game's acceptance window. Playback speed changes affect real-time duration.
The label expires at activation; it is not extended to imply a longer window.
Guard input does not start the bar. Mapped combo phases get separate cycles;
overlapping hitboxes are combined and upcoming cues take priority over recovery.

COVERAGE AND LIMITS

2,161 phases across 54 models; 450 green estimates across 39 models, plus
39 dodge phases and 58 jump phases. These are extracted data counts, not
verified support for all enemies. Stricter checks remove questionable green
prompts from the older version. Unsupported and ambiguous attacks stay gray.

Chained Ogre has 5 mapped grab phases and 6 green estimates. Guardian Ape has
1 mapped grab phase, 4 jump phases and 4 green estimates, including animation
variants. Other moves/forms remain unverified. Some bosses have only partial
data coverage; pure projectiles and a complete perilous-attack classifier are
not supported. DODGE does not guarantee a safe direction or dodge timing.

A fresh neutral LOCKED bar survives idle or unavailable animation state.
Timing requires a progressing clock. Lost locks, invalid dependent reads,
stale data, dead actors and invalid projections suppress guidance.
Approximate reach/facing checks are not weapon collision geometry.

Recording 02 of the older build shows a soldier cue before contact sparks.
The new layout has an offline render check and automated tests; new gameplay
placement, boss responses, successful deflects and precise contact timing still
need validation. Normal camera at 16:9 or wider matching display is supported.

RUN REQUIREMENTS AND LOGS

No Rust, Visual Studio, Python, or Cheat Engine is needed to run this package.
The DLL observes game state read-only, uses a DirectX 11 render hook, and makes
no network requests or automated gameplay inputs. The supported executable
SHA-256 is 637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856.

Logs: %LOCALAPPDATA%\SekiroDeflectObserver
observer-PID.cue.csv: observations, read stages, animation and press estimates.
observer-PID.render.csv: frame drawing decisions and submitted action labels.
observer-PID.samples.csv: separate candidate-effect research diagnostics.
Each CSV stops at 16 MiB. F9 reports log state and dropped render records.
A render submission is not proof of presentation, contact or successful input.
An estimated_press cue-log row is not proof that the bar was drawn.

Source and detailed evidence:
https://github.com/thisguymartin/sekiro-deflect-observer
See docs/cue-preview.md and docs/validation-0.6.md in the source checkout.
See LICENSE and THIRD-PARTY-NOTICES.txt for license information.

