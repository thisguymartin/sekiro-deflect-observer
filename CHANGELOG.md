# Changelog

## Documentation refresh - 2026-09-18

- Replace the README's synthetic lead image with a gameplay frame of Wolf and
  the PARRY 70% HUD from the author's new recording; add a short normal-speed loop.
- Add a timestamped walkthrough of wind-up, active phase, armed practice and
  attention status, with source provenance and clear limits on what the clip proves.
- Align current guides and package instructions with 0.12.4, and separate older
  synthetic previews and installation drafts from current documentation.
- Documentation and media only; no runtime or packaged-binary change.

## 0.12.4-preview - 2026-09-18 (70% practice preset)

- Add 70% enemy speed to the saved Shift+F11 cycle: 80% → 90% → 70% → 80%.
  F11 remains on/off, and startup remains OFF with the last selected speed.
- Show OFF/ON 70% with a fuller crescent for the stronger slowdown.
- Extend persistence and speed-switch regressions through all three presets,
  including restoration of a non-1.0 baseline without stacking.

## 0.12.3-preview - 2026-09-18 (practice moon and two speed presets)

- Keep F11 as on/off and add Shift+F11 to switch and persist 80% / 90% speed
  without changing whether practice is armed. Speed changes restore the original
  baseline before applying the new multiplier.
- Replace the corner wind crest with a crescent moon and katana. Show OFF/ON
  plus the selected percentage even without a target or while disabled. The 90%
  moon is thinner; gray means off, gold armed, jade applied, amber ! attention.
- Keep F8 master visibility and focus behavior. Add shortcut, preset persistence
  and active-speed-switch regressions; extend mesh previews to both levels.

## 0.12.2-preview - 2026-09-18 (practice status crest)

- Add a small katana and wind crest in the playable viewport's upper-right
  corner whenever F11 practice is armed, including without an enemy lock.
- Gold `ON` means armed; jade with the applied percentage means a fresh matching
  enemy has a checked speed override. Amber `!` reports unavailable, paused,
  unsupported or pending-cleanup status; F9 shows the detailed reason.
- Keep the crest independent of attack hints and rail placement. F8 hides it,
  F11 disarms it, and focus loss hides it temporarily. Reduced-flash mode removes
  the active glow. Slowdown policy and memory writes are unchanged.

## 0.12.1-preview - 2026-09-17 (independent alerts and practice)

- Separate canonical attack kind/phase classification into `attack.rs`. Alert
  toggles, Mikiri hint fallback and incoming/legacy HUD mode only affect display;
  practice eligibility uses raw attack facts and capture freshness.
- Keep speed writes in a private Windows practice adapter. The shared reader is
  read-only, and HUD drawing reports applied status without choosing eligibility.
- Show PRACTICE 80% when a speed override is active but attack hints are disabled.
  F11 remains session-only; F8 hiding still disarms practice.
- Add regressions across all 32 hint/display-mode combinations for each of three
  eligible attack kinds. All 131 Rust tests pass; live gameplay remains unverified.
- Update feature boundaries, installation, controls, architecture and validation
  documentation for the 0.12.1 preview.

## 0.12.0-preview - 2026-09-17 (optional enemy-speed practice)

- Add F11 practice, initially off each process, with an 80% default multiplier
  for recognized locked-enemy parryable, thrust and sweep phases. Wolf's speed,
  global time and deflect windows are untouched. Grabs and unknown/no-parry
  phases remain excluded in this prototype; projectile flight is not rescaled.
- Save the original speed once and use ownership/value checks for restoration
  on attack/context loss or target switching. Pending cleanup blocks new writes;
  external speed changes pause practice until explicit rearming.
- Add applied-speed captions, F9 status, bounded practice-write audit logging,
  and regression tests. No game files or saves are edited; this optional mode
  does temporarily write enemy animation speed. Live reaction/cleanup checks
  remain pending. See `docs/enemy-speed-practice.md`.

## 0.11.0-preview - 2026-09-17 (enemy variants and incoming timing)

- Add optional, coherent NPC parameter identification for weapon-specific
  responses. Missing or mismatched identities retain the conservative fallback.
  Spear soldier Mikiri and Snake Eyes grab responses no longer depend on
  agreement with unrelated sword/gunner variants.
- Resolve verified dummy-0 warning carriers and payload-free visual bullets,
  restoring ninja Mikiri and Genichiro responses. Unknown or chained payloads
  remain unverified; no new event-dispatch hook or input automation.
- Exclude non-opponent hitboxes, throw damage and harmless Mikiri detection
  markers from incoming timing geometry. Separate spear combo hits and stop
  presenting Gyoubu's object-destruction contact as an incoming attack.
- Add a sparse alerts.csv alongside full-frame logs, with decision transitions,
  a one-second heartbeat, NPC/variation IDs and actual activation boundaries.
  Each file remains bounded at 16 MiB; the sparse stream outlasts frame logging.
- Preserve the top HUD and DX11 rendering path. Automated checks establish
  classification and timing behavior; live NPC identity/gameplay checks remain.

## 0.10.0-preview - 2026-09-17 (reference-style top HUD)

- Recreate the supplied screenshots with a slim tapered rail, green parry lead-in,
  chevron caps, moving diamond, LB/L1/RMB badge and original red vector strike art.
  Replace the large backing panel with local rail shadows and text outlines.
- Add the requested top anchor below the enemy posture bar, a 480-wide default
  and 160..640 width range. Existing explicit anchors remain configurable.
- Map wind-up progression to the center gate and use white/red for active
  parryable attack phases. This is not a contact/success indicator. Unknown and
  special responses remain distinct; reduced-flash suppresses white/red.
- Export the live vector emblem as an editable SVG and retain 0.9.1's DX11
  isolation fix. Add a phase-progress regression and verify shared render bounds.

## 0.9.1-preview - 2026-09-17 (DX11 rendering isolation)

- Fix a reproduced graphics-state leak in hudhook 0.9.2: draw on a private
  deferred context and execute with full host-state restoration. The old manual
  backup omitted render targets, NULL bindings and implicitly unbound resources.
- Skip empty draw frames and discard partial command lists on draw errors.
  Release private backbuffer bindings before a potential resize.
- Add a real offscreen D3D11 WARP regression for UNORM/sRGB targets, repeated and
  empty frames, host bindings and unchanged pixels outside the cue.
- Review the user's 2026-09-17 10-33-30 clip. LOCKED/PARRY and posture clearance
  are visible. The reported whole-scene tint still needs an in-game comparison
  with this candidate; no claim of confirmed color correction is made.

## 0.9.0-preview - 2026-09-17 (incoming attack responses)

- Default to incoming move identification without distance/facing or contact
  prediction. Show PARRY/DODGE/JUMP/MIKIRI/NO PARRY/UNKNOWN during wind-up,
  fill the lane during the attack phase, and retain LOCKED between attacks.
- Resolve generic effects, the harmless perilous-warning bullet route and explicit
  Mikiri dummy hitboxes from freshly extracted parameters. Require unanimous
  behavior-variant responses; keep conflicting or unknown routes unresolved.
- Classify 2,161 phases: 1,673 parry, 40 dodge, 56 jump, 63 Mikiri, 16 no-parry
  and 313 unknown. Preserve separate legacy timing tables and evidence.
- Add `incoming_cues` and `mikiri` preferences. Mikiri assumes an unlocked skill;
  disabling it falls back to PARRY for the classified deflectable thrusts.
- Keep the raised HUD and freshness/lifecycle protections. Add exact response
  provenance, classifier/runtime regressions and reproducible dispatch disassembly.
  No full decompilation, contact-result hook or new live gameplay trial is claimed.

## 0.8.1-preview - 2026-09-17 (local correction)

- Raise the fixed cue above the posture decoration measured in the 0.8.0 recording;
  enlarge the lane and label, and add a dark backing for contrast.
- Keep LOCKED visible for a freshly validated target when animation data fails
  or stalls. This neutral state has no press interval or pulse and expires after
  50 ms without a fresh lock read; losing lock still hides it immediately.
- Show READY during mapped wind-up before coarse reach passes. Action prompts
  still require valid timing and reach. Show neutral WATCH for unverified phases.
- Migrate untouched 0.8.0 layout defaults atomically while preserving customized
  settings. Add placement, lock feedback, reach and migration regression checks.
- Windows tests and synthetic render checks do not establish gameplay timing.

## 0.8.0-preview - 2026-09-16 (local research candidate)

- Preserve original capture age and use stable animation progression with bounded
  render-time projection; raw ring sequence no longer defines progression or a new attack.
- Separate READY, defensive press intervals, expiry and evidence-backed per-hit pulses.
  Latency shifts intervals earlier without widening them. No preferred timing is invented.
- Default to a compact configured HUD above Wolf's posture bar, with safe full bounds;
  retain optional overhead mode and independent camera validity.
- Add bounded local TOML configuration, atomic persistence, reload, F6/F7 saves and F10 reset.
- Clear timing on lock/owner/clock invalidation, hide on lost focus, preserve Ogre batch
  selection and response exclusions; reject mixed projectile response classification.
- Add exact phase coverage ledger, source/DLL identity diagnostics and synthetic regressions.
  Tables remain 2,161 phases / 54 models / 450 parry / 39 dodge / 58 jump estimates.
- Contact calibration, Mikiri capability/classification, complete menu/playability reads,
  actual posture alignment and gameplay outcome validation remain open. No combat rules change.


## 0.7.0-preview

- Observe completed animation batches immediately before the engine advances the ring boundary at researched RVA 0xb5bef0. Require the supported executable hash and exact loaded function hash before installing the native detour.
- Drive the locked target's animation from captured batches; invalidate on stale captures, target changes, empty batches or competing attacks. The callback uses fixed-size reads and try-locks and preserves the original call.
- Add ordered activation/deactivation timeline-crossing and track-change logs, plus hook status and capture counters in F9.
- Retain existing response classifications. This does not yet implement resolved behavior-variant, hitbox-dispatch or contact-result hooks, and is not a verified parry-timing release.

## 0.6.4-preview

- Fit the camera viewport into the current display instead of rejecting a differing aspect ratio. Center pillarboxed and letterboxed views and resize cue geometry with the viewport.
- Show placement failures and camera/surface dimensions in F9; render logs distinguish invalid input, depth rejection and an offscreen anchor.
- Add projection regression coverage for ultrawide, 16:9, 16:10, 4:3, portrait and small window sizes. Live alignment and contact timing remain unverified.

## 0.6.3-preview

- Refine the reference-inspired design with a larger ivory diamond, longer needle, moving magenta pointer, brighter chevrons and tapered translucent ribbon wings.
- Replace the action label's rounded box with outlined English text and provide clearance above the pointer. Preserve green PARRY, orange DODGE and blue JUMP.
- Keep the functional timing lane at 360 x 12 reference pixels. Decorative wings do not widen its timing zone or change the estimates, detection, player anchor or game rules.

## 0.6.2-preview

- Match the supplied video's thumbnail more closely with a filled ivory diamond, thin vertical needle, soft glow, tapered timing segment and slimmer 360 x 12 track.
- Retain English action labels and green/orange/blue response colors. Move labels clear of the taller needle.
- Keep the 150 ms parry estimate, 300 ms dodge/jump estimate, Ogre batch reader and actual combat rules unchanged.
- Allow the offline layout tools to write to a chosen folder, preserving earlier visual evidence.

## 0.6.1-preview

- Fix the Chained Ogre's neutral-only bar observed during the 0.6.0 gameplay trial: its auxiliary animation 40000 was hiding the attack in the last-entry reader.
- Select a mapped attack from the engine's current animation submission batch. Preserve the ten-entry ring and 0x14 stride; never search previous batches for an attack. Competing attack tracks, changing bytes/boundaries and invalid reads suppress guidance.
- Add batch/wrap/cancellation/ambiguity regression checks (35 Rust tests total) and a bounded read-only animation-track capture tool.
- Preserve 0.6.0 recordings and package as evidence of the discovered failure. The corrected DLL requires a full game restart and a new timing trial.

## 0.6.0-preview

- Lower Wolf's standing anchor from 2.15 to 1.55 world units; add F6/F7 vertical adjustment. Animated head tracking remains pending.
- Enlarge the diamond and action labels on a 360 x 16 reference track. Keep the parry estimate at 150 ms of animation time; combat rules are unchanged.
- Add orange DODGE for mapped grabs and blue JUMP for mapped sweeps, including selected Chained Ogre and Guardian Ape attacks. These use a separate experimental 300 ms advance cue.
- Read shared BehaviorParam/AtkParam data and more TAE event fields. Merge overlapping hitboxes and prioritize an upcoming combo cue over previous recovery.
- Generate 2,161 phases across 54 models, with 450 green estimates across 39 models, 39 dodge phases and 58 jump phases. Stricter filters intentionally reduce green coverage; unknown attacks stay unverified.
- Preserve a neutral lock when only animation reads fail; retry changed observations within the existing read budget and use the handle's actual bucket-index bound.
- Add read-stage and render-submission CSV diagnostics, shorten the render mutex scope, and expose log status in F9.
- Add response, import, bitfield, read-failure and combo regression checks plus a shared-renderer visual check. New live timing, placement and universal enemy support remain unverified; see docs/validation-0.6.md.

## 0.5.0-preview

- Extracted all 79 base enemy/NPC archives; 2,341 melee phases across 54 models,
  with 819 green-eligible phases across 44 models. Uncertain phases stay gray.
- Resolve unambiguous local animation imports and publish per-model coverage.
- Keep a fresh neutral locked-target bar visible when an enemy idles.
- Increase bar size, zoom the timeline, and expand the estimate to 150 ms.
- Include body radius in approximate boss reach and distinguish camera/animation errors.
- Full move coverage and precise contact/deflectability validation remain incomplete.


## 0.4.2-preview

- Matched the supplied video reference with a thin dark track, green timing zone,
  white diamond marker, and end chevrons above Sekiro.
- Removed routine countdown/status text; retain PARRY during the estimate.
- Preserved incoming-attack timing; the reference video describes post-input
  visualization and does not validate advance press timing.

## 0.4.1-preview

- Replaced the enemy dot with a timing slider above Sekiro.
- Show wind-up, green press interval, recovery, and successive combo-hit cycles.
- Drive the bar with enemy animation time, independently of guard input.
- Preserve unsupported-attack, range, stale-read and frozen-clock suppression.
- User confirmed the 0.4.0 enemy circle was visible; new bar placement and precise timing remain to be checked.

## 0.4.0-preview

- Added an overhead ring and experimental green press estimate driven by live
  locked-enemy animation and automatically extracted soldier/general timelines.
- Added bounded target/camera reads, stale/frozen clock suppression, and cue logs.
- Hidden the statistics panel by default; F9 toggles optional diagnostics.
- Exact contact timing, deflectability, and visual alignment remain unvalidated.

## 0.3.0-dev

Candidate-effect diagnostic reader; live layout and deflect semantics pending validation.

- Add exact executable-hash gating and source-documented local-player effect traversal.
- Reject incomplete, cyclic, over-limit, changing and unreadable observations.
- Sample through ReadProcessMemory with a read budget and monotonic timestamps.
- Show candidate presence/absence, unknown reasons, freshness, and bounded transitions.
- Record capped per-process sample CSVs for gameplay research.
- Add synthetic reader/history tests and document sources and outstanding trials.
- Correct the unconfirmed handover claim about an existing Cheat Engine prototype.

## 0.2.0-dev

Windows x64 overlay proof of concept. Not validated as a deflect observer.

- Add a native DirectX 11 DLL and explicit unavailable-reader status.
- Add an F8 visibility toggle and local startup diagnostics.
- Add Windows build checks, ZIP packaging, checksums, and dependency notices.
- Add a me3 launch profile and Windows run instructions.

Deflect detection, timing history, incoming-attack prediction, and verified platform compatibility are not included in this milestone.
