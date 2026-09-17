# Defensive cue investigation — 2026-09-16

This is a pre-implementation audit. Runtime code, timing intervals and generated
classifications have not changed. The proposed changes require design approval;
see [the draft](superpowers/specs/2026-09-16-defensive-cue-design.md).

## Checkout and evidence identity

The requested audited commit exists:
`2e6e7778bccb518ea17c69e69579d2ca775d9de8` (0.6.3-preview). Actual HEAD is
`a405273ab4ae539be7e59027940e1e4632fa1cb9`, package version 0.7.0-preview.
The initial working tree was clean. Relative to the audited commit, HEAD already
contains 0.6.4 viewport adaptation and 0.7.0 animation-batch capture/event tracking.
Those changes must be preserved. No applicable repository/ancestor AGENTS.md
was found in the locations inspected.

All requested source, script and documentation files were inspected. Two requested
documents did not exist: `docs/WORK-STATUS.md` and `docs/boss-move-coverage.md`.
This session creates them. Generated data was read in full programmatically;
the phase ledger preserves every exact textual Rust phase boundary and references
the original JSON evidence instead of inventing human move names.

SHA-256 values read from disk during this audit:

| Artifact | SHA-256 |
| --- | --- |
| Installed `C:/Program Files (x86)/Steam/steamapps/common/Sekiro/sekiro.exe` | `637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856` |
| Existing release DLL and `Mods/SekiroDeflectObserver-0.7.0-preview/sekiro_deflect_observer.dll` | `8f90adcbb8004a85d6f72bca2330bce26ff33843a87742756ef8eff6d46b15d6` |
| `src/attack_timings.rs` | `3d0c108cb3412fa91f43da0f08a819cd12d792431fc027f0c132854dc0aded3d` |

These are current disk hashes, not proof of the exact DLL previously loaded by
process 2524. Its startup log records version 0.7.0, the matching executable hash
and installation of the animation hook, but no DLL hash. No Sekiro process was
running when inspected. No new live gameplay trial occurred.

`GraphicsConfig.xml` currently selects FULLSCREEN 5120x1440, with a saved window
size of 2560x1440. This is configuration, not measured presentation metadata for
an older recording. Reading it initially failed under restricted access; the
existing approved read permission succeeded. No graphics settings were changed.

## Trace through the existing code

1. `windows/diagnostics.rs`: a worker begins a cycle on a monotonic `Instant`
   epoch, targets an 8 ms poll period, and gives cue memory reads a 10 ms budget.
   `cue::observe_traced` checks the exact executable hash before sensitive reads.
2. `cue.rs`: validate player/world and lock manager, selected point, handle bucket,
   actor/module ownership, target/player HP, model, positions, coarse bounds and
   facing. Recheck owners and lock flags. Lock loss returns no target; animation-
   only failure can keep a neutral target. Camera reads currently remain coupled
   to the target read, even though future fixed placement does not need projection.
3. Polling selects only `[module+0xec,module+0xe8)` in the ten-slot ring, including
   wrap, byte/boundary rechecks, and rejection of distinct competing mapped tracks.
   Ogre's auxiliary 40000 must not hide its attack. Never search an earlier batch.
4. In 0.7.0 `event_hook.rs` observes the completed batch before the researched
   boundary-reset function, calls the original once, and captures a timestamp.
   `attack_events::select_batch` uses the same conservative track selection.
   `event_hook::apply` replaces the polled animation when a matching capture is
   younger than 50 ms. **It does not carry the capture timestamp into `Target`.**
5. `cue::timeline` searches exact `(model, animation)` table rows. An upcoming
   press interval takes priority over earlier recovery. It chooses response by
   exact phase start; the default lead is 150 animation ms for parry, 300 for
   dodge/jump. Eligibility uses generated classifications plus body-distance,
   height and facing filters, not weapon collision geometry. Activation is the
   interval end, not measured contact.
6. `LiveCue::push` stores the worker's read-start time. Its progress fingerprint
   contains handle, animation ID, raw sequence and current animation time. Any
   change in that tuple refreshes `progressed`, including a sequence-only change.
   `current`/`advancing` expire at 50 ms but do not advance the animation clock.
7. `windows.rs::render` takes `epoch.elapsed()`, copies a current target under a
   short mutex lock, then releases the lock. `cue_draw::draw` projects Wolf's root
   plus 1.55 world units and calls `timeline` on the unchanged sampled clock.
   Geometry uses the fitted camera viewport added in 0.6.4.
8. `RenderSample` records the callback time, sampled animation clock, estimated
   phase and submission decision via a bounded nonblocking queue. Neither DXGI
   visible presentation nor actual input/contact/outcome timestamps are measured.
   F6/F7/F8/F9 are focused, fresh-key, pass-through UI controls; they are not
   measurements of the player's defensive input.

Consequently sampling/read/publish delay affects the age gates but not the
interval calculation. Presentation/input latency is absent. The hook path has
two independent age gates using different timestamps. Sequence identity is used
as progress in `LiveCue` and as continuity in `attack_events::Tracker`, without
establishing its occurrence semantics. There is no per-hit visual pulse state.

## Reproduction specifications — synthetic, not yet executed regressions

These precise cases are candidates for the required failing tests after design
approval. They are code-derived failure hypotheses, **not completed T1 evidence**.
No green interval may be changed before the corresponding case is run red.

| Case | Input trace | Current code-derived outcome | Required result |
| --- | --- | --- | --- |
| T1-A: delayed sample | c1010/3000, phase `[0.666666687,0.800000012)`, in reach; valid 1x history ending at animation 0.650 at monotonic 1000 ms; render at 1020 ms | Sample age 20 ms passes; `timeline` still uses 0.650 and reports parry | At projected animation 0.670 the interval is expired; no action/pulse |
| T1-B: capture age lost | Valid hook capture at 0 ms, accepted by poll at 49 ms; worker stalls; render at 98 ms | Hook was younger than 50 ms at acceptance; copied target is younger than 50 ms relative to poll, despite capture age 98 ms | Original capture age exceeds ceiling; hide/suppress immediately |
| T1-C: frozen clock with changed sequence | Push same handle/ID/time 0.600 at 0 and 40 ms, changing sequence only; evaluate at 50 ms | `progressed` refreshes to 40 ms, so `advancing` is true | Raw sequence alone cannot establish positive progression; no action |
| T2-A: latency sign | Supported stable 1x synthetic contact/press profile; increase measured total latency by 30 ms | No calibration path exists | Both interval endpoints shift 30 real ms earlier; width unchanged |
| T3-A: skipped interval | Valid per-hit synthetic profile; render before its start, next render after end | No pulse mechanism exists | Zero pulses, including after recovery/reacquisition |
| T3-B: repeated/combo hits | Distinct observed same-ID restart and multi-phase animation; multiple renders per interval | No occurrence/phase pulse deduplication exists | One pulse per eligible hit at most; next READY unaffected by prior decoration |

Additional tests must cover half-open boundaries, speed changes, stop/rewind,
first observation mid-attack, repeated identical captures, cancellations,
out-of-reach/away-facing input, ambiguous batches, ownership changes, errors and
the exact 50 ms freshness boundary. Existing tests do not cover the first three
cases as written; their passing result does not disprove these hypotheses.

## Existing live log review

Read-only sources are under
`C:/Users/mpati/AppData/Local/SekiroDeflectObserver/`:

| File | SHA-256 |
| --- | --- |
| `observer-2524.log` | `eace40465fd729c76842baf1fd54c55fc7e7afe414f1591a776cddae5e781010` |
| `observer-2524.cue.csv` | `3bc2f6a5f7f3115e252eda008f188c3413b00469392e60f991d290f9b918d354` |
| `observer-2524.render.csv` | `c3c25c1ca3e96eddde4c5ff51e289b315a7cdd013353dd45d51e55a144fb3f75` |
| `observer-2524.events.csv` | `259c7b3d9036e42957c7535721c7ed4355e7d57da8a781b0293b20dd566e84f7` |

The cue log contains 240,231 rows and reaches the 16 MiB cap. It includes 41,660
`event_batch`, 315 `event_batch_waiting_or_stale`, 190,067 lock-disabled, 2,661
target-dead and 136 player-dead rows. The capped cue log is not a complete account
of later render rows. Among 41,509 adjacent rows with the same nonempty handle,
model and nonnegative animation ID, there were **zero sequence changes**. Thus
T1-C is only a synthetic robustness case; no sequence-only freeze was observed.

The render log contains 83 parry submissions. The event log records 574 starts,
574 ends/cancellations, 101 activation crossings and 54 deactivation crossings.
None is a count of successful deflects or unique human moves.

One exactly identifiable **submission**, not a calibrated timing trial:

| Field | Observed value / limitation |
| --- | --- |
| Version / executable hash | 0.7.0-preview / executable hash above |
| Loaded DLL hash | Missing from session log; current disk hash cannot substitute |
| Render epoch | `1789590194355633` Unix microseconds in CSV header |
| Target / animation | handle `1000403f`, model c1020, animation 3000 |
| Phase | activation 0.600000, deactivation 0.800000 animation seconds (CSV precision) |
| Animation clock / raw sequence | 0.466704 seconds / 22972 |
| Reported sample age | 4.08 ms from worker read start; original hook capture age absent |
| Render timestamp / frame | 395465489 microseconds after epoch / frame 23006 |
| Decision | parry, estimated press submitted true |
| Display mode / actual presentation | Not recorded for this trial |
| Actual input / contact / successful deflect | Not recorded or independently confirmed |

All six observed transitions from a target row to `lock_disabled` in the available
cue segment had `lock_disabled` in the first render row at or after read completion:

| Observation end (us) | Delay to next hidden submission (ms) |
| ---: | ---: |
| 240630480 | 12.884 |
| 262498639 | 13.362 |
| 485174789 | 16.599 |
| 502847465 | 0.376 |
| 706956149 | 1.549 |
| 1114022103 | 15.434 |

Range 0.376–16.599 ms, n=6. This is observed read-completion-to-callback delay,
not physical lock-input-to-hide, publish-to-hide or visible presentation latency.
It is not a guarantee against a snapshot-copy/render race or future stalls.

Reproduce the comparison by skipping `#` headers, parsing both CSVs with
`csv.DictReader`, selecting cue row pairs where the earlier row has a handle and
the later stage is `lock_disabled`, and binary-searching the first render
`at_us >= cue.end_us`. Compare its status and subtract timestamps. Do not use
render rows after the capped cue segment to infer missing observation events.

## Checks performed on the unchanged runtime baseline

Use `$env:PATH = 'C:/Users/mpati/.cargo/bin;' + $env:PATH` for each development
shell. The original PATH selected standalone Cargo/Rust 1.87.0 and bypassed
rustup. Rustup's pinned versions are Cargo 1.94.0 (`85eff7c80`) and rustc 1.94.0
(`4a4ef493e`). Its active host is GNU; the Windows MSVC target is installed.

| Exact command | Actual result |
| --- | --- |
| `cargo fmt --all -- --check` | Exit 0 |
| `cargo test --locked --offline` | Exit 101: `error calling dlltool 'dlltool.exe': program not found`; GNU target dependencies fail |
| `cargo test --locked --offline --target x86_64-pc-windows-msvc` | Exit 0; 44 tests passed, 0 failed; 0 doctests |
| `python3 scripts/test-attack-timings.py` | Command unavailable: `python3` is not recognized |
| `python scripts/test-attack-timings.py` | Python 3.12; exit 0; 8 tests passed |
| `git -c safe.directory=C:/Users/mpati/workspace/sekiro-deflect-observer diff --check` | Exit 0 before edits; repeated for final documentation changes |

Local raw outputs: `dist/review-2026-09-16-design/fmt.txt`,
`cargo-test-default.txt`, `cargo-test-msvc.txt` and `python-test.txt`. PowerShell
marks ordinary redirected native stderr as `NativeCommandError` text; the exit
codes and test summaries above distinguish this from test failures. The missing
`python3` command failed before creating its intended redirected output file.

No new release DLL/package, loader exercise, shared-renderer layout, installation,
removal or live gameplay validation was attempted before design approval. Existing
44 tests include a native detour fixture, which is not a Sekiro integration test.

## Workflow and blockers

The actual Superpowers files were found under
`C:/Users/mpati/.codex/.tmp/plugins/plugins/superpowers/skills/`, outside the
initial installed-skill catalog/cache search. Read: using-superpowers (including
Codex tool adaptation), systematic-debugging, brainstorming, writing-plans,
test-driven-development and verification-before-completion. Applied investigation,
design and evidence-verification stages; planning/TDD implementation remain next.

Brainstorming explicitly says: "Do NOT ... take any implementation action until
you have told your human partner what you intend and they have approved it."
The draft specification is supplied now to make that approval concrete. No
Context7 tool is available; official upstream documentation will be used before
library API changes. No unavailable tool or guessed installation path is assumed.

Exact contact, preferred press timing, successful outcomes, immediate menu state,
Mikiri capability and actual posture geometry still lack the required evidence.
The design records how to continue without mislabeling these as implemented or
validated. Historical validation records remain unchanged.

## Execution follow-up (2026-09-16, approved design)

The user approved implementation with "yes execute". The implementation branch
is `codex/defensive-cue-posture-hud`, based on actual HEAD `a405273`, now version
0.8.0-preview in the working tree. This does not change the older evidence above.

The exact T1-A/B/C synthetic defects were made to fail before their fixes. Saved
outputs are `dist/review-0.8.0/t1-a-red.txt`, `t1-b-red.txt`, `t1-c-red.txt`, and
subsequent green/full test logs in that directory. T1-A failed by showing action
at animation 0.670 after the 0.666666687 endpoint. T1-B failed by making a capture
at 0 ms appear fresh when repolled at 49 ms and rendered at 98 ms. T1-C failed
because a sequence-only change kept a frozen clock advancing. These are labeled
synthetic; none claims a measured failed input or successful game deflect.

`src/timing.rs` now projects only a stable validated animation occurrence, carries
original capture age, separates prepare/press/contact/preferred/pulse concepts,
and clips pulse/labels at interval expiry. Configuration latency moves both
endpoints earlier without increasing their width. Tests include every response,
boundaries, 2x playback, stop/rewind, repeated IDs, independent combo hits, skipped
render intervals, loss/switch/reacquisition, exclusions and pulse count. No exact
contact offset or preferred press profile was manufactured for shipped data.

Review additionally reproduced sequence-based false event starts, a newly polled
pause retaining action, loss observed before mutex publication, and a hide/show
pair between renders retaining old eligibility. The event owner key now includes
player/module/generation; `src/lifecycle.rs` supplies the immediate invalidation
generation checked before render submission. Cue reads/publication precede config
and log I/O. New capture/read IDs, monotonic source/read/publication/render times,
calibration references and hidden-submission delay fields support the next trial.
The last check cannot revoke commands already submitted; presentation remains
unobserved and separate from the submission timestamp.

A later process inspection found Sekiro PID 11400 running the **old**
`Mods/SekiroDeflectObserver-0.7.0-preview/sekiro_deflect_observer.dll` through me3.
Its startup log identifies 0.7.0 and the existing executable hash. The process was
left untouched; it is not evidence for 0.8.0 behavior. A full restart with the
new candidate and manual trials are required. The final build/hash/check record
is in [WORK-STATUS](WORK-STATUS.md) and [0.8 validation](validation-0.8.0.md).
