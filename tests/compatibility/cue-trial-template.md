# Defensive cue trial record

Do not mark a move calibrated or gameplay validated until these observations
and links are present. Preserve failures, ambiguous outcomes and missing frames.

## Identity and setup (T1, R1, V2)

- Date/time, investigator, local session ID:
- Source commit plus `git status --short` / working-tree diff hash:
- Package/DLL version, loaded DLL path and SHA-256 (startup log):
- Game executable path/version and SHA-256:
- `attack_timings.rs` SHA-256, config file and profile evidence version:
- Loader/version/hash; other mods and gameplay modifiers:
- Encounter and exact form; model ID, animation ID, occurrence and phase ordinal:
- Exact phase activation/deactivation boundaries from the ledger:
- Classified response; capability evidence if applicable (Mikiri remains blocked):
- Windows/GPU/driver, refresh rate, frame rate/frame-time range, VSync/VRR:
- Surface/viewport size, UI scale, windowed/borderless/exclusive mode, letterboxing:
- Normal/overhead anchor, offsets/scale/reserved band/gap; screenshot of actual gap:
- Distance/height/angle/motion relative to target; approximate body radius:
- All config changes, display/input latency measurements and uncertainty:
- Log/video/input files with hashes and time-alignment method:

## Per-attempt timeline (T2, T3)

Use the shared monotonic epoch and record the uncertainty of conversions from
video/input clocks. Do not replace an unavailable observation with a draw timestamp.

| Trial | Capture/read time and sample age | Animation clock/rate | Estimated press interval | Draw submission | Visible presentation | Actual manual input | Contact | Independently confirmed outcome | Evidence / uncertainty |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Fill from observations | | | | | | | | | |

Outcome labels: successful deflect, block, damage, miss/out of range, cancelled,
ambiguous. Effect 105010 alone is not a confirmed deflect. Keep both successes
and failures and state sample size; do not manufacture percentages or confidence.
A contact spark alone does not distinguish every outcome or input timestamp.

## Calibration result

- Contact offset interval relative to activation (animation ms):
- Response early/late/preferred lead (real ms before contact), source and uncertainty:
- Total trials, successes, failures, ambiguous/excluded trials and exclusion reasons:
- Does evidence cover every runtime variant of the exact model/animation/phase?
  If not, record the named form and leave its runtime profile inactive.
- Exact compatible game/data hashes, phase boundaries, response and evidence links:
- Result: extracted / classified estimate / calibrated / gameplay validated /
  unsupported / unknown. State which conditions were actually checked.

## Lifecycle and HUD acceptance (H1, H2, C1)

Record lock-off observed time, first hidden submission, and first visibly hidden
frame separately. Report observation-to-hide delay and video frame uncertainty.
Repeat for target switch, interrupted combo, player/target death, loading, menu,
focus change, save reload and fresh reacquisition. No late/catch-up flash may occur.

Check actual posture-bar spacing at 1280x720, 1920x1080, 2560x1440, 3840x2160,
3440x1440 and 5120x1440 where available, plus letterboxing/UI scale/window modes.
Mark unavailable modes blocked, not passed. Synthetic fitted-viewport geometry
cannot establish the game's posture bar position or UI scaling.

Check first launch/missing config, malformed reload, every setting boundary,
F6/F7 persistence, F8/F9/F10, full restart, package installation/removal and save
reload. List next concrete action and required evidence for each open requirement.
