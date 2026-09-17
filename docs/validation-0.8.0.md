# 0.8.0-preview validation — 2026-09-16 execution

This is a **Windows build, deterministic behavior and synthetic-rendering**
record. It does not validate live contact timing, successful defensive inputs,
actual posture-bar spacing or gameplay lifecycle acceptance.

## Source and artifacts

The requested audit commit is `2e6e7778bccb518ea17c69e69579d2ca775d9de8`
(0.6.3-preview). Actual starting HEAD was
`a405273ab4ae539be7e59027940e1e4632fa1cb9` (0.7.0-preview). Implementation is
uncommitted on `codex/defensive-cue-posture-hud`, now package version
0.8.0-preview. Existing releases and historical validation records were preserved.

| Artifact | SHA-256 |
| --- | --- |
| Built, packaged and separately staged 0.8.0 DLL | `60756a07633b232c36c1ea33aa89006ff42343cf3efb3f20e798e0b4924d39a8` |
| `dist/SekiroDeflectObserver-0.8.0-preview-windows-x64.zip` | `33d4b296768653045bf7a4b865194e8549a1a93dfc5bb7ead9bad6cfb1e37253` |
| Existing loader used by isolated ASI fixture | `fa266e3513d02c08a1b808f28c10538a489eaffaa4b0707f7cc1066e71b5afd7` |
| Supported `sekiro.exe`, checked on disk | `637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856` |
| Checked-in generated attack data | `3d0c108cb3412fa91f43da0f08a819cd12d792431fc027f0c132854dc0aded3d` |

The ZIP contains the DLL, `observer.me3`, `launch-observer.cmd`, `cue.toml`,
README, LICENSE, dependency license texts and SHA256SUMS. All seven content hashes
were verified against SHA256SUMS; DLL and config bytes equal the build/default
files. `dist/review-0.8.0/artifacts.json` records the content manifest. A new
`Mods/SekiroDeflectObserver-0.8.0-preview/` folder contains that exact package.
Staging is not a game installation or restart test.

## Environment and exact checks

Windows NT 10.0.26200.0; rustc 1.94.0 (`4a4ef493e`, 2026-03-02), Cargo 1.94.0;
rustup host `x86_64-pc-windows-gnu`, with MSVC target installed; Python 3.12.
Each Cargo shell selected rustup with:

```powershell
$env:PATH = 'C:/Users/mpati/.cargo/bin;' + $env:PATH
```

The original PATH selected standalone Rust 1.87.0. No toolchain or global Git
configuration was changed. Git used the per-call option
`-c safe.directory=C:/Users/mpati/workspace/sekiro-deflect-observer` because
the sandbox account and checkout owner differ.

Raw outputs are under `dist/review-0.8.0/` (local ignored evidence).

| Exact command | Actual result | Raw output |
| --- | --- | --- |
| `cargo fmt --all -- --check` | exit 0 | `fmt.txt`; repeated by `build.txt` and final check |
| `cargo test --locked --offline` | exit 101; GNU `dlltool.exe` missing | `cargo-test-default.txt` |
| `cargo test --locked --offline --target x86_64-pc-windows-msvc` | exit 0; 91 passed: 70 library, 5 layout, 2 lifecycle, 14 timing; 0 failed, 0 doctests | `cargo-test-msvc.txt` |
| `python3 scripts/test-attack-timings.py` | unavailable (`python3` not recognized; recorded as exit 127) | `python3-test.txt` |
| `python scripts/test-attack-timings.py` | exit 0; 11 tests, OK | `python-test.txt` |
| `python scripts/generate-attack-timings.py --input-directory dist/game-analysis-v2` | exit 0; regenerated runtime/JSON bytes unchanged | `task-5-python-verification.txt` |
| `python scripts/update-move-coverage.py --check` | exit 0; exact phase ledger and summary current | `coverage-check.txt` |
| `cargo clippy --locked --offline --all-targets --target x86_64-pc-windows-msvc -- -D warnings` | exit 0 | `integration-clippy-green.txt` |
| `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1` | exit 0; fresh fmt, Clippy, all-target tests, MSVC release, DLL startup and package | `build.txt` |
| `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-asi-loader.ps1` | exit 0; DirectInput forwarding, adjacent ASI load, non-game host rejection | `asi-loader.txt` |
| `cargo run --locked --offline --target x86_64-pc-windows-msvc --example cue-layout -- dist/review-0.8.0/layout/1080-gallery --gallery --display 1920 1080` | exit 0; final shared-renderer gallery, all vertices contained | `layout-final.txt` |
| `python scripts/render-cue-layout.py dist/review-0.8.0/layout/1080-gallery` | exit 0; synthetic PNG | `layout/1080-gallery/cue-layout.png` |
| `git -c safe.directory=C:/Users/mpati/workspace/sekiro-deflect-observer diff --check` | exit 0 | `diff-check.txt`; final handoff check |

The build script uses `cargo fetch --locked` and does not claim a fully offline
build; the separately recorded locked offline MSVC suite passed. It also runs
`cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings`,
`cargo test --locked --all-targets --target x86_64-pc-windows-msvc`, and
`cargo build --release --locked --target x86_64-pc-windows-msvc`.
Redirected native stderr appears as PowerShell `NativeCommandError` text in some
logs; command exit codes and complete test summaries above are the results.

The build startup fixture's LOCALAPPDATA was isolated to
`dist/review-0.8.0/startup-local-app-data/`. The ASI fixture uses its own
`asi-smoke/local-app-data/`. Its process-30580 log identifies the exact DLL hash
above and rejects the non-Sekiro host before installing rendering hooks. Neither
fixture starts Sekiro, exercises its swapchain, or presses gameplay buttons.

## Regression evidence and review

- **T1-A:** A sampled 0.650 animation-second clock with a 20 ms render delay
  incorrectly retained action beyond the 0.666666687 endpoint. The baseline
  failed `t1-a-red.txt`; projection-aware half-open decisions pass.
- **T1-B:** Capture at 0 ms, repoll at 49 ms, render at 98 ms falsely appeared
  fresh. `t1-b-red.txt` fails before retaining the original hook capture age.
- **T1-C:** Raw sequence change with a stopped animation incorrectly counted
  as progression. `t1-c-red.txt` fails before progression/occurrence fixes.
  This was not observed in the historical 41,509 same-track pairs.
- Additional saved failing cases cover state separation, sequence-based event
  restarts, polling pauses, camera independence, invalidation before publication,
  rapid F8 hide/show, failed hook-capture attribution, mixed-projectile responses,
  configuration conflicts, and a mutated incorrect HUD anchor.
- Final tests cover half-open boundaries, latency sign and unchanged width,
  speed changes, stops/rewinds, repeated IDs, per-hit combo pulses, cancellations,
  lock switches/reacquisition, stale/bad reads, skipped intervals, reduced flash,
  all response cases, and source/capture ownership. Existing Ogre current-batch,
  ambiguity, stale-ring and native detour fixture tests remain green.
- Configuration tests cover default-file creation, invalid initial/reload
  retention, bounded numbers/colors/profiles, invariant override rejection,
  persistence/reset and external edits before and during temporary-file writes.
  Atomic replacement prevents partial files. The final fingerprint recheck
  narrows, but cannot remove, a noncooperating editor's recheck-to-rename race.

Independent scoped reviews of configuration/layout and timing/runtime were
completed. Findings were fixed and re-reviewed; final closure specifically
checked capture provenance and all 15 event CSV columns. Render CSV has 52
columns and cue CSV 20. Unknown input/contact/outcome/presentation values remain
unobserved; they are not populated from activation crossings or effect 105010.
Review notes remain under `.superpowers/sdd/2026-09-16-defensive-cue/`.

## Synthetic geometry versus live acceptance

The [shared-renderer matrix](cue-preview.md#actual-synthetic-layout-checks)
checks 720p, 1080p, 1440p, 4K, 3440x1440, 5120x1440, letterboxing, alternate
camera aspect, scale extremes and missing-camera fallback. Geometry tests also
cover inset viewports and suppress undersized/portrait fits. Every emitted
vertex must fit the declared full cue bounds and viewport. Representative 720p,
gallery and ultrawide images were inspected. The checked-in image is explicitly
synthetic. These checks do not detect Wolf's actual posture bar or establish
windowed/borderless/UI scaling behavior.

During this session Sekiro PID 11400 was observed using the existing
`Mods/SekiroDeflectObserver-0.7.0-preview/sekiro_deflect_observer.dll` through me3.
That disk DLL hashes to
`8f90adcbb8004a85d6f72bca2330bce26ff33843a87742756ef8eff6d46b15d6`;
the old startup log identifies 0.7.0 but does not record the loaded DLL hash.
The running process was left untouched. It is not a trial of the new candidate.

**Live acceptance is open.** A full manual exit/restart using the new profile,
hash-confirmed startup, and the [gameplay checklist](../tests/manual/gameplay-checklist.md)
are required. Begin with the ordinary soldier, then Ogre's auxiliary-track case,
then representative Ape responses and combos. Use the
[trial template](../tests/compatibility/cue-trial-template.md) to record encounter,
form, exact model/animation/phase, ages/clocks, frame rate/display mode,
distance/angle/modifiers, input/contact/presentation observations, both successes
and failures, and log/video references. Measure new observation-to-hidden
submission and separately visible-hide delay; the historical n=6 delay range
does not validate 0.8.0.

All shipped phases remain estimates: 2,161 extracted / 54 models / 450 parry /
39 dodge / 58 jump / 1,614 unverified; zero complete calibrated or gameplay-
validated per-move records. Exact contact, preferred-time calibration, runtime
form/behavior applicability, immediate menu/playability observation, weapon
collision geometry and Mikiri capability/mappings remain unresolved.
See [the continuing work record](WORK-STATUS.md#remaining-work).
