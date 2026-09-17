# Work status - defensive cue and posture HUD

This file is the local handoff record. Append dated session entries and preserve
historical evidence. Acceptance IDs are from the defensive-cue request.

**Current checkpoint: the 0.11.0 baseline is committed and pushed as `4c0df9a`
on `codex/defensive-cue-posture-hud`. New enemy-speed practice work is isolated
on `feat/enemy-attack-slowdown` (0.12.1-preview, feature separation follow-up).** The user explicitly requested
committing/pushing the current work before implementing optional 80% attack speed.
The original attack-response scope remains; no contact prediction is promised.
See [practice behavior and limits](enemy-speed-practice.md) and
[0.12.1 validation](validation-0.12.1.md). Existing releases, Mods folders and
historical validation records are preserved.

- [Approved design](superpowers/specs/2026-09-16-defensive-cue-design.md)
- [Executed implementation plan](superpowers/plans/2026-09-16-defensive-cue.md)
- [Timing investigation and regression evidence](timing-investigation-2026-09-16.md)
- [0.8.0 exact verification and artifact hashes](validation-0.8.0.md)
- [Coverage and form limits](boss-move-coverage.md) / [exact phase ledger](boss-move-phases.csv)
- [Configuration defaults and bounds](configuration.md)

## Requirement state

| ID | Implemented and automated evidence | Live or other completion still needed |
| --- | --- | --- |
| T1 | Three labeled synthetic timing defects reproduced RED then fixed; original capture age, progression and projected interval tests pass | Compare exact loaded 0.8.0 observations with video/input/contact; synthetic results are not gameplay timing validation |
| T2 | Portable preparation/contact/press/preferred/pulse concepts; units, evidence and latency-sign tests | All shipped timing remains activation-based; no measured contact or preferred profile ships |
| T3 | Occurrence/phase pulse identity; independent combo/repeat, cancellation, pause/rewind and skipped-interval tests | Preferred pulses require compatible evidence; validate each real supported combo after calibration |
| R1 | Conservative response exclusions and every enum case tested; explicit mixed-projectile regression fixed; counts unchanged | Mikiri needs confirmed thrust mapping and capability read; special responses and runtime form/behavior applicability unresolved |
| H1 | Fixed configured posture anchor and optional checked overhead mode; shared-renderer geometry matrix passes | Actual Wolf posture gap, UI scale, windowed/borderless/fullscreen and arbitrary crop layouts unverified |
| H2 | Atomic invalidation before publication; render checks, owner generation, lock/switch/reacquisition and F8 tests | Measure new observation-to-hide/presentation delay; menus/loading/save reload and full game lifecycle unverified |
| C1 | Bounded worker-owned TOML load/reload, safe defaults, atomic persistence/reset, invalid reload retention, exact compatible profile validation; 19 config tests | Full game restart/hotkey/reload checks; finish external edits before hotkey writes due to remaining noncooperating-editor race |
| V1 | 91 Windows MSVC Rust tests, 11 Python tests, fmt, Clippy, native DLL/ASI fixtures, package manifest and synthetic renderer pass | Plain GNU-target command lacks dlltool; python3 alias absent; working MSVC/python alternatives recorded |
| V2 | Exact candidate built/hashed/staged separately; live old 0.7.0 process identified and preserved | No 0.8.0 gameplay trials. Complete manual restart, installation/removal and soldier -> Ogre -> Ape/combos trials |
| D1 | Coverage generator and 2,161-row ledger current; present behavior docs, manual checklist, trial template and CHANGELOG updated | Expanded per-parry parameter provenance and per-form gameplay evidence remain absent |
| D2 | Dated investigation and execution entries, exact checks/hashes, implemented/synthetic/live split, next actions below | Every later execution must append its own evidence and update these states |

## Session 2026-09-16 — investigation and design checkpoint

### Completed work

1. **Baseline preservation (all IDs).** Initial Git status was clean. Compared
   HEAD with the specified audit commit and read all requested existing files.
   No applicable repository/ancestor AGENTS.md was found. WORK-STATUS and
   boss-move-coverage were absent, so this session creates them. Runtime sources,
   lockfile, package version, release DLLs, game settings and game data were not
   modified. Existing debug build outputs were refreshed by baseline tests.
2. **Skill workflow.** Located actual Superpowers skills under
   `C:/Users/mpati/.codex/.tmp/plugins/plugins/superpowers/skills/` after searching
   the initial catalog/cache locations. Read using-superpowers, systematic-debugging,
   brainstorming, writing-plans, test-driven-development and
   verification-before-completion, plus the Codex adaptation. Applied discovery,
   debugging/design and verification stages. No Context7 tool is exposed.
3. **T1/H2 investigation.** Added `docs/timing-investigation-2026-09-16.md` with
   exact code path, clock domains, source-age loss, regression specifications,
   artifact/log hashes, and measured read-completion-to-hidden-submission delays.
   The six available lock-loss transitions range 0.376–16.599 ms. They do not
   measure presentation. Sequence-only freeze was not seen in 41,509 same-track
   pairs; that case remains synthetic. Current-session logs have 83 parry draw
   submissions, not 83 successful deflects. The cue log is capped at 16 MiB.
4. **T2/T3/R1/H1/H2/C1 design.** Added
   `docs/superpowers/specs/2026-09-16-defensive-cue-design.md`: alternatives,
   portable state model, source timestamps, bounded progression/extrapolation,
   evidence/latency units, per-hit pulses, compact fixed HUD, conservative response
   blockers, configuration defaults/bounds and staged verification. This is an
   unapproved proposal, not an implementation plan or a completed feature.
5. **D1 coverage audit.** Added `docs/boss-move-coverage.md` and
   `docs/boss-move-phases.csv`. Parsed all generated rows, preserved exact phase
   boundary strings and model/animation identity, joined response/source evidence,
   and checked per-model totals against `enemy-coverage.json`. Counts remain
   2,161 phases / 54 models / 450 parry estimates / 39 dodge / 58 jump /
   1,614 unverified. Zero complete per-move calibration/validation records found.
   All 79 source archives are listed, including unresolved zero-phase variants.
6. **V1 baseline checks.** Results below are fresh checks of unchanged runtime
   code, not validation of the proposed features. The existing native detour
   fixture ran as part of the Windows unit tests; no game was running.

### Exact checks and environment

Git commands use a per-command `-c safe.directory=C:/Users/mpati/workspace/sekiro-deflect-observer`
because the sandbox account differs from the checkout owner. No global Git
configuration was changed.

Before Cargo checks, set this process-local PATH:

```powershell
$env:PATH = 'C:/Users/mpati/.cargo/bin;' + $env:PATH
```

This selects pinned rustup Cargo/rustc 1.94.0; the original PATH selected 1.87.0.
Rustup's active host is `1.94.0-x86_64-pc-windows-gnu`; its MSVC target is installed.

| Command | Actual output/result | Evidence |
| --- | --- | --- |
| `cargo fmt --all -- --check` | exit 0 | `dist/review-2026-09-16-design/fmt.txt` |
| `cargo test --locked --offline` | exit 101; missing `dlltool.exe` for default GNU target | `dist/review-2026-09-16-design/cargo-test-default.txt` |
| `cargo test --locked --offline --target x86_64-pc-windows-msvc` | 44 passed, 0 failed; 0 doctests; exit 0 | `dist/review-2026-09-16-design/cargo-test-msvc.txt` |
| `python3 scripts/test-attack-timings.py` | unavailable: `python3` not recognized; failed before log file creation | captured tool output; Windows command name is `python` |
| `python scripts/test-attack-timings.py` | 8 tests, OK; exit 0 | `dist/review-2026-09-16-design/python-test.txt` |
| `git -c safe.directory=C:/Users/mpati/workspace/sekiro-deflect-observer diff --check` | exit 0 | checked before edits and at documentation handoff |
| Full generated-data audit | 2,161 exact phase records; all per-model and response counts match | `docs/boss-move-phases.csv`, `docs/enemy-coverage.json`, `docs/response-coverage.json` |

Final documentation verification also passed: all 2,161 ledger rows match exact
Rust model/animation/boundary strings, phase ordinals, response joins and source
hashes; all five new files have clean whitespace and valid relative Markdown
links. Final Git status contains only these new documents. No new runtime
regression was executed or represented as passing.

`scripts/build.ps1`, release packaging, standalone DLL/ASI loader checks and
shared-renderer previews were read but not rerun for this design-only checkpoint.
The build script can replace a same-version release ZIP, so implementation needs
an isolated artifact directory or a new version before running it. No fresh
build, installed-candidate or gameplay-validation claim is made.

### Exact artifacts and remaining evidence limits

The current disk game SHA-256 is
`637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856`.
Existing 0.7.0 release and Mods DLLs both hash to
`8f90adcbb8004a85d6f72bca2330bce26ff33843a87742756ef8eff6d46b15d6`.
The old process-2524 startup log does not record its loaded DLL hash; do not
silently assign this current disk hash to that trial. Exact log hashes and an
identifiable render row are in the investigation. Current graphics configuration
is fullscreen 5120x1440; this does not establish an old trial's presentation mode.

There is no validated posture anchor, menu-state read, Mikiri-capability read,
weapon collision model, preferred press calibration or independent successful-
deflect timing evidence in the inspected code/records. Exact contact and live
acceptance remain open. Three historical images were inspected; they do not
validate the new posture-HUD placement.

Working-tree state at handoff: only the five new audit/design/status documents
listed above; no commit made. Historical validation records remain unchanged.
The phase ledger was generated by a one-time read-only table audit; a checked-in
regeneration/check command remains part of D1 implementation work.

### Remaining work at the design checkpoint (superseded by execution below)

- **T1 / design checkpoint:** Review and approve the proposed design, then write
  the implementation plan with the installed writing-plans skill. Run T1-A/B/C
  synthetic cases against current code and save failing output before fixing
  timing. Dependency: approval under brainstorming's explicit gate. Completion:
  accepted spec/plan plus reproducible red/green evidence.
- **T2–T3:** Implement original-capture timestamps, explicit occurrence/hit state,
  stable animation progression, half-open press intervals and optional calibrated
  preferred pulses. Current runtime still uses sampled activation estimates and
  has no per-hit pulse. Completion: deterministic boundaries, latency-sign,
  speed/stop/rewind/repeat/combo/stall tests proving decisions and pulse counts.
- **R1:** Preserve conservative classifications and audit every response exclusion.
  Mikiri needs exact eligible-thrust mappings and a validated capability source;
  neither is implemented. Keep it unsupported until both are established. Record
  contact/response uncertainty per move and never infer offensive retaliation.
- **H1–H2:** Implement the compact fixed HUD, full-bounds posture gap and invalidation
  of pending flashes/copied snapshots. Keep Ogre selection, owner checks and the
  50 ms ceiling. Research menu/playability evidence without guessed offsets.
  Completion: shared-renderer layout matrix, lifecycle tests, and separately
  recorded live gap/lock-switch/death/loading/menu and visible-hide trials.
- **C1:** Add bounded local config, worker-only reload/persistence, reset and exact
  game/data-bound calibration metadata. Verify official upstream APIs because
  Context7 is unavailable. Completion: invalid initial/reload, NaN/infinity,
  colors/intervals, atomic-write, concurrency, restart and invariant tests.
- **V1:** After implementation run fmt, locked offline MSVC tests, Python data
  checks, diff check, Clippy, preserved-artifact Windows build and actual DLL/ASI
  loader/shared-renderer checks. Plain default-target tests currently fail for
  missing GNU dlltool; `python3` is absent, while `python` runs all 8 checks.
- **V2:** Arrange manual gameplay on the exact new DLL; no game was running here.
  Start with soldier, then Ogre, Ape and supported combos. Record loaded hashes,
  occurrence/phase, capture/render/presentation/input/contact/outcome separately,
  trial successes and failures, distance/angle, frame rate, display mode and
  modifiers. Complete first launch/restart/install/removal/focus/resize/save-load
  trials; no synthetic result may close gameplay acceptance.
- **D1–D2:** Add reproducible coverage-ledger regeneration and expanded provenance,
  update user-facing behavior docs/manual checklist/CHANGELOG as changes land,
  preserve historical records and append a dated status entry after each job.
  Completion: data/docs consistency checks and detailed next actions for every
  remaining acceptance ID.

## Session 2026-09-16 - approved implementation and Windows verification

### Completed implementation

1. **T1-T3 / timing and occurrence model.** `src/timing.rs`,
   `tests/timing-regressions.rs`, `src/cue.rs`, `src/event_hook.rs` and
   `src/attack_events.rs` carry original capture age and validated player/module/
   target identity. Three distinct captures establish two agreeing animation
   rates; progression stops or resets on pause, rewind, switch or bad data.
   Projection is bounded to 25 ms and freshness stays 50 ms. READY, half-open
   press interval, contact evidence, optional preferred instant and bounded
   per-hit pulse are separate. Positive real-ms display/input latency moves the
   interval earlier without widening it. Legacy estimated animation leads remain
   150/300/300 ms. No preferred time was invented for activation-only data.
2. **H2 / lifecycle and provenance.** `src/lifecycle.rs`,
   `tests/lifecycle-regressions.rs`, `src/windows/diagnostics.rs`, `src/windows.rs`
   and `src/input.rs` invalidate lock/owner/visibility changes before worker
   publication and check generation before draw submission. F6/F7/F8/F9/F10 use
   focused fresh presses and pass through. F9 stays separate from gameplay lock
   gating. Reads and cue publication precede configuration/log I/O. Original
   capture, read, publication and render IDs/timestamps, source age, response/
   phase/occurrence, calibration references, layout and hidden-submission delay
   are logged locally. Actual DLL hash is measured at startup. Missing capture
   times remain empty; failed hook captures retain original identity/time.
3. **H1 / fixed HUD.** `src/layout.rs`, `src/windows/cue_draw.rs`,
   `tests/layout-regressions.rs`, `examples/cue-layout.rs` and
   `scripts/render-cue-layout.py` share one renderer consuming portable decisions.
   Default normalized center `(0.5,0.87)` is fitted to the camera viewport, or an
   explicit 16:9 fallback. Full label/lane/glow bounds preserve the configured
   posture-band gap. READY is hollow; PARRY NOW/DODGE/JUMP are filled; EXPIRED is
   neutral. Labels have independent contrast. Unsupported small fits suppress;
   overhead mode retains checked camera projection. The synthetic image and
   matrix are documented in `docs/cue-preview.md`; no actual HUD detection claim.
4. **C1 / persistent configuration.** `src/config.rs`, `packaging/cue.toml`,
   `Cargo.toml`/`Cargo.lock`, `docs/configuration.md` implement pinned serde/TOML,
   64 KiB bounded whole-file validation, safe first-use defaults, one-second
   worker reload, last-valid retention, queued hotkey persistence and offset
   reset. Atomic same-directory writes include a final source-fingerprint
   recheck. Profiles require exact executable/data/model/animation/phase/response
   identity and evidence metadata. Named forms remain inactive without a
   runtime form observation; explicit all-variant exact-key evidence is required
   for `form = "model-animation-phase"`. Config cannot disable correctness gates.
5. **R1/D1 / classifier and coverage.** `scripts/attack_responses.py` rejects
   explicit mixed type-4 projectile events; RED/GREEN Python tests cover all
   response cases. Broad generic markers were audited rather than assumed to
   invalidate the existing parameter-backed Ogre/Ape responses.
   `scripts/update-move-coverage.py` generates/checks the exact coverage summary
   and ledger. Regeneration left `src/attack_timings.rs`, `docs/enemy-coverage.json`
   and `docs/response-coverage.json` byte-identical to HEAD: 2,161 phases, 54 models,
   450 parry estimates, 39 dodge, 58 jump, 1,614 unverified. These are not unique
   moves. Zero complete calibrated/gameplay-validated per-move records exist.
6. **V1/D1 / native candidate and documentation.** Version is 0.8.0-preview.
   `scripts/package.ps1` includes defaults; `scripts/test-asi-loader.ps1` exercises
   the current DLL in an isolated bounded output folder. README, parry-cue,
   architecture, cue-preview, configuration, Windows/release instructions,
   `tests/manual/gameplay-checklist.md`, `tests/compatibility/cue-trial-template.md`,
   CHANGELOG and `docs/validation-0.8.0.md` reflect current behavior and limitations.
   Historical validation files were not rewritten.
7. **Workflow and review.** The installed Superpowers skills were read and the
   approved design/plan executed with TDD and scoped independent reviews.
   Review corrections include polling-pause action, loss-before-publication,
   rapid hide/show, event owner continuity, invalid calibration intervals,
   external-save conflicts, and failed-capture/CSV attribution. Final review
   closed its findings. Context7 was unavailable; locked crate sources and
   official upstream Microsoft API documentation were used. No invented tools,
   telemetry, automated game input or gameplay-state writes were introduced.

### Tests, synthetic evidence and artifacts

The final source passed **70 library + 5 layout + 2 lifecycle + 14 timing = 91
Rust tests**, 0 failures; 0 doctests. Native hook call/return and empty-batch
fixtures run in that suite. Config contributes 19 library tests. Python has
11 passing classifier/data tests. These are automated tests, not game outcomes.

All exact commands, outputs and blockers are in
[validation-0.8.0](validation-0.8.0.md#environment-and-exact-checks). Summary:

| Exact command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo test --locked --offline` | exit 101: missing GNU `dlltool.exe` |
| `cargo test --locked --offline --target x86_64-pc-windows-msvc` | exit 0; 91 passed, 0 failed |
| `python3 scripts/test-attack-timings.py` | unavailable; command not recognized, recorded exit 127 |
| `python scripts/test-attack-timings.py` | exit 0; 11 tests OK |
| `python scripts/update-move-coverage.py --check` | exit 0; exact generated inventory current |
| `cargo clippy --locked --offline --all-targets --target x86_64-pc-windows-msvc -- -D warnings` | exit 0 |
| `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1` | exit 0; fmt/Clippy/all-target tests/release DLL/startup/package |
| `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-asi-loader.ps1` | exit 0; forwarding/ASI loading/host rejection |
| `cargo run --locked --offline --target x86_64-pc-windows-msvc --example cue-layout -- dist/review-0.8.0/layout/1080-gallery --gallery --display 1920 1080` | exit 0; shared-renderer vertex containment |
| `python scripts/render-cue-layout.py dist/review-0.8.0/layout/1080-gallery` | exit 0; synthetic image inspected |
| `git -c safe.directory=C:/Users/mpati/workspace/sekiro-deflect-observer diff --check` | exit 0 |

The synthetic layout matrix also covers 720p, 1440p, 4K, 3440/5120 ultrawide,
letterboxing, scale extremes, alternate camera aspect and missing-camera
fallback. Portable tests cover inset/max bounds and unsupported-fit suppression.
This is shared ImGui geometry evidence, not actual posture-bar/UI-scale evidence.

Raw RED/GREEN, test, build and preview records are under `dist/review-0.8.0/`.
The DLL's SHA-256 is
`60756a07633b232c36c1ea33aa89006ff42343cf3efb3f20e798e0b4924d39a8`.
The ZIP's SHA-256 is
`33d4b296768653045bf7a4b865194e8549a1a93dfc5bb7ead9bad6cfb1e37253`.
All seven packaged content hashes were independently verified, as were DLL/config
byte equality. Package: `dist/SekiroDeflectObserver-0.8.0-preview-windows-x64.zip`.
Separate staging: `Mods/SekiroDeflectObserver-0.8.0-preview/`.
The ASI fixture's startup log reports that exact DLL and rejects its non-game
host before rendering hooks. It is not a Sekiro swapchain test.

### Live evidence and handoff limits

Sekiro PID 11400 was observed loading the previous
`Mods/SekiroDeflectObserver-0.7.0-preview/sekiro_deflect_observer.dll` through me3.
The game was left running and untouched; the new folder was not launched.
**The blocker is a full manual restart and gameplay trials of the candidate,
not an unavailable Windows host or absent game.** Older logs cannot validate the
new DLL. No new successful deflect, contact instant or visible presentation was
recorded. The historical n=6 observation-to-hidden-submission result remains
historical, and is not a 0.8.0 latency claim.

Work remains uncommitted on the named feature branch at the original source
HEAD. No merge, push, cleanup or replacement of a running DLL was performed.

## Remaining work

- **T1 / live timing comparison.** Current behavior fixes reproducible synthetic
  sample-age, projection and sequence defects. Missing: matched live evidence
  showing actual input/contact/outcome under 0.8.0. Next: restart manually with
  the exact candidate, record soldier model/animation/phase, capture age/clock,
  render timestamp, display mode and hashes, then correlate a video/input trace.
  Completion evidence: trial records separating estimated press, submitted draw,
  visible cue, actual input, contact and confirmed success/failure.
- **T2-T3 / contact calibration and per-hit pulse trials.** The model and synthetic
  combo/repeat/pause/rewind/stall tests pass; shipped moves still use activation
  estimates and have no preferred pulse. Dependency: measured contact intervals
  and response-specific press evidence, independent display/input latency and
  exact game/data identity. Next: collect both successes and failures, add only
  compatible evidence profiles, and validate each combo hit's single timely
  pulse. Never widen estimates or infer contact from activation/effect 105010.
- **R1 / conservative response evidence.** Parry/dodge/jump remain classified
  estimates. Mikiri is explicitly unsupported: exact eligible-thrust mappings
  and a validated player-capability read are missing. Runtime form/behavior
  variation, special responses and weapon collision geometry are unresolved.
  Next: obtain read-only evidence and mapping/capability regressions before adding
  a response. Keep uncertain moves unverified; body distance is not collision.
- **H1 / actual posture spacing.** Configured fixed placement and synthetic
  full-bound checks pass. Missing: visible gap over Wolf's actual bar/labels/glow
  in 720p, 1080p, 1440p, 4K, ultrawide/letterbox, UI scaling and
  windowed/borderless/fullscreen modes. Next: capture those layouts with the
  candidate, including jumps/crouches/camera/boss proximity and resize. Record
  supported settings and suppress/explicitly bound unsupported layouts; do not
  describe normalized placement as automatic HUD detection.
- **H2 / gameplay lifecycle and hide delay.** Atomic invalidation and deterministic
  lock/switch/F8/reacquisition tests pass. Missing: new measured observation-to-
  hide delay and live death/loading/menu/save-reload/focus evidence. No validated
  immediate menu/playability source exists. Next: perform manual lifecycle
  trials, correlate logged invalidation and hidden submissions with video, and
  research safe read-only playability observations if existing gates fail.
  Already submitted GPU commands cannot be revoked; presentation stays separate.
- **C1 / persistence in the game.** Parse/bounds/invalid reload/atomic save/reset
  tests pass. Missing: full process restart and focused F6-F10/reload trials with
  the candidate. Next: test valid edits, malformed edits, offsets/reset and reload
  after restart; confirm no gameplay input capture and no render-thread I/O.
  Finish external editor saves before hotkey saves: fingerprint checks narrow
  but cannot eliminate a noncooperating editor's recheck-to-rename race. Keep
  malformed/conflicting files and the last valid runtime config intact.
- **V1 / default command environment.** Explicit MSVC offline tests and native
  build/fixtures pass. The literal default-target test command still lacks GNU
  `dlltool.exe`; `python3` is absent while `python` passes. Next, only if those
  exact command spellings are required: prepare the GNU toolchain/python3 alias
  and rerun them. Do not misreport these environment failures as green checks.
- **V2 / candidate installation and gameplay acceptance.** The package is built,
  hashed and staged; the running game uses old 0.7.0. Next: manually close Sekiro
  fully, launch `Mods/SekiroDeflectObserver-0.8.0-preview/observer.me3`, confirm
  0.8.0 and its hash in startup logs, then follow the manual checklist for first
  launch, restart, installation/removal, focus, resize, lock/unlock, switch,
  death/loading/save reload. Validate ordinary soldier first, Ogre auxiliary
  track second, selected Ape responses and boss combos third. Record encounter/
  form, animation/phase, distance/angle, FPS/modifiers, trial count, successes and
  failures and log/video links with uncertainty. Zero complete per-move gameplay
  validations currently exist; do not mark V2 complete from fixture tests.
- **D1 / provenance and evidence ledger.** Current table counts and user-facing
  docs match implemented estimates. Expanded per-parry parameter provenance and
  complete encounter/form trial records are missing. Next: extend the extraction
  evidence join without guessing move names or shared-form support, regenerate
  with `scripts/update-move-coverage.py`, and promote only evidence-backed records.
  Preserve the historical 0.6/0.7 evidence and unresolved requested-form rows.
- **D2 / continuation record.** At every later execution, append a dated session
  with exact commands, changed paths, source/artifact hashes, implemented versus
  tested status and updated concrete next actions. Keep synthetic and gameplay
  evidence separate, and print this remaining-work section before ending.

## Session 2026-09-17 - finish 0.8.1 posture and lock feedback correction

User reported an inactive/empty 0.8.0 cue and native posture overlap, supplied
`C:/Users/mpati/Videos/Captures/Sekiro 2026-09-16 11-34-12.mp4` as the older
working reference, and requested the cue a little above Wolf's posture bar.
The checkout already held uncommitted partial 0.8.1 changes, a built ZIP and
saved 0.8.0 recording/log review. These were preserved and completed, not reset.

### Completed and verified

- Inspected frames from the supplied older clip, the saved 0.8.0 overlap frame
  and synthetic raised placement. Independently counted saved render statuses:
  7,244/11,248 target submissions were outside coarse reach; 1,386 target
  submissions hid for missing/stale animation observation. The last startup log
  inspected still identifies 0.8.0; no game process was running.
- Retained the partial fix's raised anchor (0.81), reserved band (0.85), larger
  320x14 lane, 30-pixel label, dark backing, READY-before-reach, neutral WATCH,
  and atomic migration of untouched 0.8.0 layout defaults.
- Fixed the remaining fresh-lock/animation-failure regression in `src/cue.rs`,
  `src/timing.rs` and `src/windows.rs`. Lock freshness is independent of animation
  capture freshness for neutral feedback only. Full clear/loss/visibility/focus
  still hide; no press/contact/pulse survives unavailable animation. Added the
  integrated lifecycle/recovery regression to `tests/timing-regressions.rs`.
- Updated README, CHANGELOG, configuration, architecture, timing, preview,
  Windows and packaged instructions. Fixed the missing validation link and
  created [validation-0.8.1](validation-0.8.1.md) with exact evidence and limits.
- Fresh fmt, Clippy, 97 Rust tests, 11 Python tests, coverage consistency, offline
  MSVC release build, native DLL/ASI loader fixtures and shared-renderer gallery
  passed. Gallery visually inspected; full mesh containment passed. These are
  automated/synthetic checks, not gameplay outcomes.
- Built and hash-verified every packaged/staged file. Staged separately at
  `Mods/SekiroDeflectObserver-0.8.1-preview/`. DLL SHA256
  `70a2bbbb973edfce9492732b518f8e89bac3605a1590a063342fceefa4957744`;
  ZIP SHA256 `cdc3e3186605e1a8f29c4f7fec465092c292e8f33b3de4537d3408a4e29d338a`.
  Prior partial 0.8.1 archive is preserved under
  `dist/review-0.8.1-completion/previous-candidate/`; earlier releases/Mods remain.
- Logs/check outputs and extracted reference frames are under
  `dist/review-0.8.1-completion/`. No user configuration, game installation or
  gameplay state was edited; no game was launched. No commit or push was made.

### Current remaining work

- **Immediate H1/H2/V2 check:** Fully restart through the new 0.8.1 profile,
  confirm version/hash, verify visible LOCKED, posture gap, moving READY and
  unlock hiding. Check ordinary sword soldier, then Ogre and selected bosses.
  Record the exact loaded candidate; older logs do not validate this DLL.
- **Timing/coverage:** All move timing remains an activation estimate. Complete
  contact/input/outcome trials, combo calibration, runtime-form and special
  response research still require independent evidence as listed above.
- **Configuration/lifecycle:** The actual untouched 0.8.0 tuple is eligible for
  migration; the migration/restart, hotkeys, menus, focus, death and loading
  still need game trials. Placement across other UI scales remains unverified.


## Session 2026-09-17 - incoming attack responses and local game-data research

The user authorized inspecting the installed game and suggested decompilation,
then explicitly clarified that predicting whether an attack reaches Wolf is
unnecessary: identify the incoming attack and parry/dodge/Mikiri response.
That clarification supersedes contact prediction as an acceptance requirement
for the default HUD. Earlier contact-calibration work remains optional legacy
research rather than a blocker for this request.

### Completed

- Verified the installed executable hash and freshly extracted all 79 base enemy
  animation archives plus 2,396 NPC attack, 3,128 behavior, 1,531 bullet, 1,279
  NPC, 3,096 effect and 2,414 throw rows. Installation/saves were read only.
- Traced the previously captured loaded animation consumer at RVA 0xb5c730. It
  looks up the animation and calls 0xb58530 with timeline and animation times.
  Disk bytes differ at that address. Saved reproducible disassembly and report;
  no full decompilation or new dispatch/contact hook was performed.
- Identified general-swing false exclusions from generic effect/aim events,
  harmless danger-warning bullet 8/effects 211000-211001, and explicit Mikiri
  detection dummy hitboxes. The incoming classifier requires unanimous resolved
  variants; Mikiri additionally requires thrust fields, a simultaneous marker
  and a player-to-model counter route. Damaging/unknown mixed routes stay unknown.
- Added incoming_responses.py, generate-incoming-attacks.py, incoming.rs and
  incoming_attacks.rs with exact provenance in incoming-coverage.json. Counts:
  2,161 phases / 54 models / 1,673 parry / 40 dodge / 56 jump / 63 Mikiri /
  16 no-parry / 313 unknown. Legacy timing tables/coverage remain byte-identical.
- Default incoming_cues=true removes reach/facing, rate and press/contact-window
  requirements. Immediate wind-up/active response labels retain the raised HUD,
  fresh lock feedback and cancellation/expiry gates. Mikiri is a user preference
  (default true), not a detected skill; false falls back to deflectable PARRY.
- Updated docs/defaults/package and built 0.9.0. 105 all-target Rust tests passed,
  followed by all 9 incoming tests with the added combo regression (106 current
  tests total); 11 legacy + 9 incoming Python tests passed, both generators check,
  fmt/Clippy/build/native DLL/ASI fixtures and shared-renderer gallery passed.
- Staged Mods/SekiroDeflectObserver-0.9.0-preview. Verified all seven ZIP/staged
  content hashes plus DLL/config equality. DLL SHA256:
  81a4a94dbce3b0c6d5c85d0dc6d7c260e8144468e83b55cd92db20cbb36ae768.
  ZIP SHA256: ae6a4c606f6e7a68d03be6db716e04bca4de58f5db6046bb58779cd338af26c6.
  Raw evidence: dist/review-0.9.0 and dist/response-research-2026-09-17.
- No game was running or launched. No live skill state, hit outcome or contact
  measurement was claimed. No actual user config edit, commit or push occurred.

### Current remaining work

- Restart manually through the new 0.9.0 profile; confirm version/hash, posture
  gap, immediate distant attack warning, grab/sweep/Mikiri labels and cancellation.
  If the skill is not unlocked, set mikiri=false. The optional question about
  the user's skill state was unanswered; the package documents its assumption.
- Investigate the remaining 313 unknown phases without guessing across behavior
  variants, projectile routes or forms. Add representative gameplay evidence.
- Direct native event-dispatch/contact-result hooks and calibrated press timing
  remain optional future work, not part of the simplified incoming-cue contract.

## Session 2026-09-17 - reported tint and overlay rendering isolation

- Inspected the user's 79.15-second `Sekiro 2026-09-17 10-33-30.mp4` recording.
  LOCKED/PARRY and clearance above the lower posture bar are visible. The scene
  looks dim, but a controlled unmodified comparison is absent. Current game PID
  23300 identifies 0.9.0 and the previously staged DLL hash in its startup log.
- Confirmed no intentional full-screen tint drawing or game gamma/HDR setting
  writes. Found incomplete host-state restoration in hudhook 0.9.2's DX11 backend.
- Reproduced the render-target leak on an offscreen D3D11 WARP device, then
  vendored the pinned dependency and patched only its DX11 backend. HUD drawing
  now records a private command list executed with full host-state restoration.
  Empty frames skip GPU work and partial draw commands are discarded on failure.
- The actual backend regression passes for UNORM/sRGB targets: preserved host
  output and aliased resources, NULL bindings, viewport and outside-cue pixels.
  107 Rust tests, formatting and app Clippy passed; offline release build and
  native DLL/ASI loader fixtures passed. Two pre-existing unused `wait_idle`
  warnings remain visible in the now-local dependency.
- Built/staged 0.9.1 separately. DLL SHA256
  `0b5c381db5dc54727dd6e54942e304acc77c78c61b34231a056475db052c9f27`;
  ZIP `1036cf072c1f822a3218b6beab19293287c64e4b1cd2bc922acb4883c22f4204`.
  All package/staged hashes verified. See validation-0.9.1.md and
  dist/review-0.9.1. Old releases preserved; no game/config/save changes,
  game restart, commit or push performed.
- Still required: fully restart with the 0.9.1 profile and compare the same scene
  against an ordinary Steam launch, including performance/fullscreen transitions.
  F8 does not unload the hook. The reproduced bug is fixed; the user's recorded
  tint is not yet confirmed resolved. An optional question about whole-scene tint
  versus the cue's local black panel was unanswered during this work; the color
  report was treated as a whole-scene rendering issue, with cue styling unchanged.

## Session 2026-09-17 - screenshot-reference HUD redesign

- User supplied two stills and a YouTube reference, then explicitly selected the
  upper placement. Direct video fetch was unavailable; reconstruction uses the
  supplied images. Optional controller label question remained unanswered; LB
  follows the image, with L1/RMB supported as saved settings.
- Built 0.10.0's pointed 480x18 top-center rail, half-width response segment,
  moving diamond, center gate, chevrons, small caption/button badge, and original
  red strike art. Removed the large enclosing panel. White/red represents the
  observed active parryable attack phase, not confirmed contact or successful
  deflection. No input automation or contact hook was added.
- Added Top anchor and parry_button configuration, extended width to 640,
  normalized incoming wind-up progress to activation, retained legacy posture
  placement with extra caption clearance. UNKNOWN is dim gray; reduced_flash
  suppresses white/red. Native art also exports assets/ui/strike-emblem.svg.
- 108 Rust tests, app Clippy, formatting, MSVC release build, DLL and ASI smoke
  passed. Shared eight-state gallery was inspected; geometry containment passed
  at 720p, ultrawide, 4K/large scale, minimum scale, reduced flash and posture.
- Staged Mods/SekiroDeflectObserver-0.10.0-preview. All eight content hashes and
  built/staged DLL equality verified. DLL SHA256
  cc323c5f9d51e555fbfbb25f13943831d3b91029021d25da45f3f9625013daaf;
  ZIP ed5ad43d161cdd741255189558ba18287f2a2d68fb945f64ef950d2904f44f02.
- Tested the settings helper on a copied config, then applied top/480/zero offsets
  and LB to actual AppData config through approved execution. Previous settings
  are in cue.toml.before-reference-20260917-180603-7772439.bak beside that file.
- No game restart, injection, game graphics/save edits, commit or push. Existing
  releases preserved. Remaining: restart into 0.10.0 and check actual placement,
  response transitions and appearance. Validation and preview in
  docs/validation-0.10.0.md and docs/images/0.10.0-reference-gallery.png.


## 2026-09-17: 0.11.0 incoming response follow-up

- User prioritizes bosses and missing grabs/Mikiri on smaller enemies. Reviewed
  PID 7844's 0.10.0 render log: 967 UNKNOWN frames in 12 phase identities; the
  stream reached 16 MiB after 846 seconds. No inference of success counts.
- Added optional same-model NPC parameter identity and behavior-variation
  classification, preserving fallback for unavailable/mismatched IDs. Added
  identity to timing/native-capture ownership so changes clear old guidance.
- Restored spear soldier Mikiri, Snake Eyes grabs, ninja warning routes and
  Genichiro visual-effect cases. Separated three thrust activations from the
  spanning counter dummy; excluded non-opponent object contact from timing.
- Generated fallback: 2,112 phases, 53 models, 293 unknown. Specific variations:
  3,730 overlapping records across 78 variations. 198 variant/phase cases that
  overlap old UNKNOWN phases now have parameter-backed responses; not trials.
- Added sparse alerts.csv with transitions/heartbeat, NPC identity, variation
  and activation bounds. Its independent 16 MiB budget continues after the full
  render log fills. Simulation retained 1,159 of 50,234 historical rows.
- 114 Rust checks, 16 incoming Python checks, 11 legacy Python checks, timing
  ledger check, formatting, app Clippy, MSVC release, DLL/ASI loading passed.
  Existing vendor warnings only. Release, staged DLL and eight package content
  hashes agree; details in dist/review-0.11.0/artifact-verification.json.
- Launcher: Mods/SekiroDeflectObserver-0.11.0-preview/launch-observer.cmd.
- DLL: 75d5a8646f604213962ffc52e5fabed61cbd903a2a6410dcdd5255551cf37a24.
- ZIP: 9fa936941b5d3cbc9b667932b21ac0cf142a79b10932bb6cc17a8bd5ac4f154b.
- Sekiro was closed when the fresh identity read was attempted. No response to
  the optional reopen/lock-on request during this build. New runtime identity
  and response behavior still need gameplay checks. Guardian Ape nonstandard
  attack types and unresolved projectile/common dispatches remain unverified.
- No restart, new injection, game/config/save/graphics edits, commit or push.
  Earlier release folders preserved. See docs/validation-0.11.0.md.

## 2026-09-17: isolated 0.12.0 enemy-speed practice prototype

- User requested commit/push of current work, then a new slowdown branch.
  Baseline saved as 4c0df9a and pushed to codex/defensive-cue-posture-hud;
  feat/enemy-attack-slowdown was created afterwards.
- Added session-only F11 toggle, initially off, and practice_speed = 0.8.
  Recognized parry/thrust/sweep phases on the locked enemy use a temporary
  behavior-module animation-speed lease; Wolf/global time remain untouched.
  Grabs, unknown/no-parry phases and legacy mode are excluded in this prototype.
- Original speed is revalidated/restored on attack/context loss; switches restore
  the previous owner first. Failed cleanup blocks new writes; external speed
  changes pause until explicit rearming. No game files/saves were changed.
- Practice status/percentage are visible in the caption and F9, with a bounded
  practice-write transition log. The default-off mode retains the normal HUD.
- 129 Rust tests, Clippy, formatting, release build, DLL and ASI loader smoke
  checks passed. Shared draw mesh/raster checked, package hashes verified and
  previous 0.11.0 artifact preserved. See validation-0.12.0.md for exact hashes.
- Sekiro was closed. Live animation rate, reactions and cleanup still need the
  checklist in enemy-speed-practice.md; synthetic tests are not gameplay proof.

## 2026-09-17: 0.12.1 separates attack facts, alerts and practice

- User requested the proposed separation after an architecture review.
- Added attack.rs for canonical phase/kind/NPC classification, without display
  preferences, HUD decisions or speed policy. Incoming alerts consume these
  facts and apply response toggles/Mikiri fallback only for presentation.
- Practice eligibility now accepts a target/capture time, not a HUD Decision.
  The worker no longer gates it with response toggles or incoming/legacy mode.
  F11, F8 master hiding, focus loss and the existing ownership checks remain.
- Moved the native writer into a private practice adapter; shared LocalMemory
  is read-only. The HUD reports applied status independently of filtered hints,
  including PRACTICE 80% while attack hints are disabled.
- Added raw phase-order/invalid-animation coverage and all 32 combinations of
  response/hint/display mode on each of three supported kinds (96 cases).
  All 131 Rust tests, Clippy, release build, ASI/host-rejection smoke and shared
  mesh bounds checks pass. Disabled-hint raster inspected. Package verified.
- 0.12.1 is staged separately and previous builds are preserved. Feature
  boundaries are documented in feature-boundaries.md. No live game was running;
  this follow-up does not claim gameplay verification of the speed prototype.

## 2026-09-17: prepare the documented 0.12.1 PR

- User requested a PR, push and updated documentation. The current feature branch
  was already pushed through 1f42256; GitHub main still held the 0.6.3 merge.
- Incorporated origin/main into the feature branch. Reviewed merge conflicts and
  retained the tested 0.12.1 implementation; the merge resolution introduces no
  runtime/source changes relative to 1f42256.
- Added 0.12.0/0.12.1 changelog entries, corrected architecture/run-guide claims
  about speed writes, documented F8/F11 and independent alert preferences, and
  added clearly labeled current synthetic visuals. Historical 0.6.3 guidance
  remains available and is labeled as historical.
- The 0.12.1 validation record and local artifact hashes still apply. PR CI is
  separate from those local checks; live gameplay verification remains pending.
