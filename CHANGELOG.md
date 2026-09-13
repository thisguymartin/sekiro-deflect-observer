# Changelog

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
