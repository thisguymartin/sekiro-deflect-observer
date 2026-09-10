# Test session report

Blank template. Replace `UNRECORDED` fields with observed values or a reason they cannot be measured. Leave unexecuted cases as `Not run`. This file is not a completed test.

## Session identity

| Field | Value |
|---|---|
| Run ID and UTC date | UNRECORDED |
| Tester alias | UNRECORDED |
| Artifact type | UNRECORDED: Cheat Engine prototype or native build |
| Source revision and local changes | UNRECORDED |
| Artifact filename and SHA-256 | UNRECORDED |
| Companion files and SHA-256 values | UNRECORDED |
| Sekiro displayed version | UNRECORDED |
| Executable version metadata | UNRECORDED |
| sekiro.exe SHA-256 | UNRECORDED |
| Build-profile and offset provenance | UNRECORDED |
| Automatic version detection result | UNRECORDED: distinguish manual identification |
| OS edition and build | UNRECORDED |
| CPU, GPU, driver, and RAM | UNRECORDED |
| Display mode, resolution, refresh rate, and scaling | UNRECORDED |
| Game frame rate and timing modifications | UNRECORDED |
| Input device and bindings | UNRECORDED |
| Cheat Engine version or native loader version | UNRECORDED |
| Other mods, overlays, and remaining dependencies | UNRECORDED |
| Location, enemy, attack, and player conditions | UNRECORDED |
| Activation action and stop action | UNRECORDED |
| Files installed and exact destinations | UNRECORDED |

## Measurement setup

| Field | Value |
|---|---|
| Recording tool and version | UNRECORDED |
| Capture FPS and constant or variable frame rate | UNRECORDED |
| Dropped or duplicate frames | UNRECORDED |
| Input evidence source | UNRECORDED |
| Alignment marker, clock origins, drift, and uncertainty | UNRECORDED |
| Poll interval and observed sample gaps | UNRECORDED |
| Sample timestamp meaning and read duration | UNRECORDED |
| Freshness limit | UNRECORDED |
| History duration and clock behavior during pause | UNRECORDED |
| Actual indicator labels and state mapping | UNRECORDED |
| Available read diagnostics and missing fields | UNRECORDED |

## Case results

Use [the result definitions](README.md). Add F01 through F05 if a synthetic failure run is part of this session.

| Case | Result | Trials completed | Evidence reference | Failure or limitation |
|---|---|---|---|---|
| G01 idle | Not run | 0 | None | |
| G02 single tap | Not run | 0 | None | |
| G03 held guard | Not run | 0 | None | |
| G04 repeated taps | Not run | 0 | None | |
| G05 deflect contact | Not run | 0 | None | |
| G06 regular block | Not run | 0 | None | |
| G07 early and late attempts | Not run | 0 | None | |
| G08 damage without guard | Not run | 0 | None | |
| G09 death | Not run | 0 | None | |
| G10 resurrection | Not run | 0 | None | |
| G11 rest | Not run | 0 | None | |
| G12 loading and zones | Not run | 0 | None | |
| G13 pause | Not run | 0 | None | |
| G14 title and reload | Not run | 0 | None | |
| G15 process restart | Not run | 0 | None | |
| G16 observer lifecycle | Not run | 0 | None | |
| G17 input | Not run | 0 | None | |
| G18 display modes | Not run | 0 | None | |
| G19 focus and settings | Not run | 0 | None | |
| G20 performance and long session | Not run | 0 | None | |
| G21 removal | Not run | 0 | None | |

## Trial observation

Duplicate this section for each measured trial. Use frame indices or timestamps with units. Label an unavailable measurement rather than filling in a guessed value.

| Field | Value |
|---|---|
| Case ID and trial number | UNRECORDED |
| Clip filename or evidence URL, SHA-256, and time range | UNRECORDED |
| Starting conditions and intended action | UNRECORDED |
| Input press and release | UNRECORDED |
| First active and first non-active displayed frames | UNRECORDED |
| Last inactive, first active, last active, next inactive sample times | UNRECORDED |
| Enemy contact time and classification cues | UNRECORDED |
| Observed outcome | UNRECORDED: deflect, block, damage, no contact, or ambiguous |
| Read validity and unknown gaps | UNRECORDED |
| Displayed duration, sample bounds, and units | UNRECORDED |
| Boundary uncertainty and incomplete-interval status | UNRECORDED |
| Observation before interpretation | UNRECORDED |
| Contradiction or follow-up | UNRECORDED |

## Performance and removal

- Observer-off, observer-on, and observer-off-again measurements: UNRECORDED.
- Measurement tool, units, recording overhead, and comparison limits: UNRECORDED.
- Long-session resource growth, errors, or freezes: UNRECORDED.
- Stop behavior and evidence that sampling stopped: UNRECORDED.
- Files removed, files preserved, and launch settings restored: UNRECORDED.
- Vanilla relaunch, input, and save-load result: UNRECORDED.

## Proton details

For a Windows-only run, mark this section `Not applicable`.

- Linux distribution, kernel, and desktop session: UNRECORDED.
- Steam client, exact Proton build, and runtime version: UNRECORDED.
- Prefix setup, loader version, overrides, and launch command: UNRECORDED.
- GPU driver, translation-layer versions, and logs if available: UNRECORDED.
- Desktop Mode or Game Mode: UNRECORDED.
- Clean-prefix installation and removal results: UNRECORDED.

## Findings and decision

- Reader correctness supported by this run: UNRECORDED.
- Narrow claim supported about effect 105010: UNRECORDED.
- Complete, incomplete, ambiguous, and contradictory trial counts: UNRECORDED.
- Confidence and evidence basis: UNRECORDED. Do not invent a percentage.
- Unresolved explanations and missing measurements: UNRECORDED.
- Failures, reproduction steps, and linked follow-up work: UNRECORDED.
- Compatibility claim justified for this exact setup: UNRECORDED.
- Release-gate decision and evidence reviewer: UNRECORDED.
