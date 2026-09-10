# Gameplay test checklist

Run these cases with the [prototype testing guide](../../docs/testing.md). Record results in a [session report](../compatibility/session-template.md).

Each expected result is a requirement or a research question, not a claim about the existing prototype. Mark unavailable native-only controls `Not applicable` during prototype research. They remain required for the native release.

## State correctness rules

- `Active` requires a current, valid observation of the candidate effect on the local player.
- `Inactive` requires a current, complete, valid observation that excludes the candidate effect.
- `Unknown` covers unsupported builds, missing players, failed or incomplete reads, and samples beyond the freshness limit.
- Once the sample is stale, the indicator cannot retain its old active or inactive state. Measure against the documented freshness limit. If no limit exists, mark this check `Blocked`.
- Unknown intervals must remain visible as gaps or unavailable periods. Do not join active windows across them or include them in complete-window statistics.

## Research and lifecycle cases

| ID | Action and starting condition | Repeats | What to check and record |
|---|---|---|---|
| G01 | Stand idle in a safe area with valid player reads. | 30 seconds | Record whether the effect appears without input. Any appearance challenges the deflect-only hypothesis. Check that reads remain fresh. |
| G02 | Tap deflect once, then release. Wait at least two seconds and until the displayed state settles before the next tap. | 10 taps | Record each press, release, effect transition, and visible animation. Preserve missing activations and extra pulses. Do not assume one press produces one window. |
| G03 | Press and hold guard for five seconds, then release and wait five seconds. | 5 holds | Determine whether the effect is brief, sustained, refreshed, or absent. Record what changes at release. |
| G04 | Tap slowly for ten seconds, then rapidly for ten seconds after a rest. | 3 pairs | Record actual input intervals where measurable. Compare effect duration and gaps. Treat shortening or merging as observations to investigate. |
| G05 | Attempt a deflect against a repeatable enemy attack. | 10 clearly classified contacts, if feasible | Label successful deflects from game cues first. Compare effect timing afterward. Log attempts with ambiguous outcomes separately. |
| G06 | Hold guard well before the same attack to attempt a regular block. | 10 clearly classified contacts, if feasible | Confirm block outcome independently. Compare effect presence with G05. If the attempt deflects instead, keep the trial under its observed outcome. |
| G07 | Tap early or late against that attack. | 5 early and 5 late attempts | Record intended timing, actual input, contact, damage, and observed outcome. An active effect on a missed attempt is not automatically a detector bug. |
| G08 | Take an unguarded hit from the same attack, with no deflect input. | 5 hits | Check whether damage itself produces the effect. Record health change and any unavailable reads. |
| G09 | Die during normal play. Include an attempt with a tap shortly before death where feasible. | 2 deaths | Check sample validity throughout death. Require `Unknown` when valid player state is unavailable. Fresh reads of a dead player may still be valid; record the lifecycle semantics. |
| G10 | Resurrect, then make an isolated tap once control returns. | 2 resurrections | Check that reads belong to the current player. Do not reuse a stale pointer or continue a pre-death active interval through an unknown gap. |
| G11 | Rest at an idol, then return to play. | 3 rests | Record the effect and read validity through the transition. Check history separation and recovery. |
| G12 | Travel between areas and cross a zone boundary available in the test location. | 3 transitions | Record loading and player replacement. No stale green display during read loss. Fresh observations must resume before known state returns. |
| G13 | Open the pause menu, wait five seconds, then return. | 3 cycles | Record whether the game, effect, sampler, and history clock advance. A valid persistent effect during pause differs from a stale sample. |
| G14 | Quit to title, wait ten seconds, then load the save. | 2 cycles | Require `Unknown` when no valid local player exists. Check recovery without carrying an old active interval into the new session. |
| G15 | Quit Sekiro completely with the observer running, then restart. | 2 cycles | Check read loss, cleared process identity, and fresh version detection before new reads. Record manual reattachment if automatic reconnect is absent. |
| G16 | Start, stop, and start the observer using its documented controls. Close its window separately if supported. | 3 cycles | Check cleanup, duplicate windows or timers, error messages, and whether hiding a window stops sampling. Closing the observer must leave normal gameplay working. |

## Interaction and removal cases

| ID | Action and starting condition | Repeats | What to check and record |
|---|---|---|---|
| G17 | Exercise movement, camera, guard, attacks, and menus with the controller and keyboard while the observer runs. | 1 pass per device | Check input loss, changed bindings, accidental focus capture, and repeated activation. Test F8 only if it is actually implemented. |
| G18 | Run G01 and G02 in windowed, borderless, and fullscreen modes offered by the setup. | 1 pass per mode | Record visibility and capture visibility separately. An external window may fail fullscreen visibility. Do not report that as a native overlay pass. |
| G19 | Change focus away from the game and back. If settings exist, test position, scale, opacity, and every display mode. | 3 focus cycles; 1 pass per setting | Check input recovery, off-screen placement, timeline clipping, and whether state remains fresh. Mark missing settings `Not applicable` for the prototype. |
| G20 | Repeat a fixed 60-second scene with observation off, on, then off again. Follow with a 30-minute session. | 3 scene runs; 1 long run | Keep recording settings fixed. Record frame-time measurements if available, resource growth, freezes, and errors. Report observed overhead without inventing a performance budget. |
| G21 | Stop the observer, close Cheat Engine, and remove only project files listed in the installation manifest. Launch the game normally. | 1 clean relaunch | Confirm the original launch path, controls, saves, and gameplay work. Record any leftovers. Do not delete unrelated loader files or saves. |

## Developer-only failure cases

Use synthetic readers and fixture executable identities for [failure tests](../README.md). Do not corrupt live pointers or patch the executable to manufacture a fault.

| ID | Injected condition | Required result |
|---|---|---|
| F01 | Unknown executable identity | Disable version-sensitive reads. Show unsupported status and `Unknown`. |
| F02 | Null player, failed read, incomplete effect traversal, cycle, or node limit | Return `Unknown` with a reason. Never infer absence from a partial scan. |
| F03 | Valid active sample followed by stopped sampling | Expire to `Unknown` at the documented freshness limit. Mark the unfinished window incomplete. |
| F04 | Process replacement or stale callback from an earlier session | Reject old results. Identify the new executable before accepting state. |
| F05 | Active, then unknown, then active, then inactive | Keep the gap and incomplete intervals. Do not count a single complete window across unknown data. |

Passing synthetic cases does not establish that the real-game offsets or effect semantics are correct.
