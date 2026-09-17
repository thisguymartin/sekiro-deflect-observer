# Defensive Cue Implementation Plan

> **For agentic workers:** Use superpowers:subagent-driven-development for isolated tasks or superpowers:executing-plans for coupled integration. Steps use checkbox syntax for tracking.

**Goal:** Correct observer timing transport and feedback, add a compact fixed posture HUD and persistent bounded settings, and document actual evidence.

**Architecture:** Existing hash-gated readers publish original capture timestamps and owner generations. A portable state machine decides preparation/action/expiry and per-hit pulses; the shared renderer consumes that decision and validated configuration. The worker owns all config I/O and logging.

**Tech Stack:** Rust 1.94.0, existing hudhook 0.9.2/ImGui/DX11, pinned TOML parser, Python data/preview scripts.

**Spec:** `docs/superpowers/specs/2026-09-16-defensive-cue-design.md`, approved by user with “yes execute”.

## Global constraints

- Native Rust observer; no automated inputs or gameplay writes.
- Preserve current-batch Ogre selection, owner checks, ambiguous-track rejection and quick-combo priority.
- Original source and required observations expire at 50 ms; extrapolation is bounded to 25 ms.
- Activation-only widths remain 150/300/300 animation ms; positive display/input latency moves both endpoints earlier.
- No preferred pulse without compatible calibrated evidence. No Mikiri without thrust and capability evidence.
- Preserve existing release artifacts and historical validation records.
- Work on `codex/defensive-cue-posture-hud`; update WORK-STATUS at each checkpoint.

## Tasks and interfaces

### Task 1: Source timestamps and deterministic timing decisions

Files: modify `src/cue.rs`, `src/attack_events.rs`, `src/event_hook.rs`, `src/lib.rs`; create `src/timing.rs`, `tests/timing-regressions.rs`.

Consumes: current `Target`, `Animation`, `Response`, phase tables.
Produces: `Target.captured_at: Option<Duration>`; `LiveCue` source-age enforcement;
`timing::Engine::observe(at: Duration, target: Option<&Target>)` and
`Engine::decide(now: Duration, target: &Target, config: &Config) -> Decision`.
`Decision` carries state, response, optional phase/interval, evidence, occurrence,
phase ordinal, progress, rate, effective age and pulse envelope/count signal.

- [x] Add and run baseline regression tests before fixes. Example literals:

```rust
state.push(Duration::ZERO, Ok(Some(target_at(0.600, 1))));
state.push(Duration::from_millis(40), Ok(Some(target_at(0.600, 2))));
assert!(!state.advancing(Duration::from_millis(50)));
```

  Test the stale-sample path using an established 1x trace ending at 0.650 s,
  decision 20 ms later: expected not actionable. Test capture=0, poll=49,
  render=98 ms: expected hidden. Save red outputs under `dist/review-0.8.0/`.
- [x] Preserve capture age across hook/worker publication, stop sequence-only
  progress, expose original owner identity and phase ordinal. Replace raw sequence
  continuity with observed animation progression/restarts.
- [x] Implement a three-distinct-capture rate estimator (two rates agree within
  20%, spacings 1–50 ms, rates 0.25–4). Invalidate on rewind/switch/error/gap.
- [x] Implement half-open intervals and calibration sign:

```text
a_now = sample_animation + rate * source_age_seconds
submission_start = press_start - rate * total_latency_ms / 1000
submission_end = press_end - rate * total_latency_ms / 1000
actionable = start <= a_now && a_now < end && all_valid
```

- [x] Test READY/action/expiry, 30 ms earlier latency with unchanged width,
  stop/rewind/rate-change, same-ID restart, combo priority, cancellation, stale
  reads, target switch and skipped preferred interval. Preferred pulse state is
  keyed by occurrence/phase, clipped at expiry; absent profile means no pulse.
- [x] Run `cargo test --locked --offline --target x86_64-pc-windows-msvc` and
  review code plus observable decisions, preserving red/green evidence.

### Task 2: Bounded persistent configuration

Files: create `src/config.rs`, `packaging/cue.toml`, `docs/configuration.md`;
modify only dependency entries in `Cargo.toml`/`Cargo.lock` and add `pub mod config`
to `src/lib.rs` for test compilation. Coordinate these small shared-file edits.

Produces `Config: Clone + Debug + PartialEq + Default` with public fields:

```rust
// Each float is f32. Colors are [f32; 4].
anchor: AnchorMode, // Posture or Overhead
offset_x, offset_y, scale, width, opacity, label_size, safe_margin,
posture_band_top, posture_gap, outline_intensity, glow_intensity,
pulse_intensity, pulse_duration_ms, preparation_ms,
display_latency_ms, input_latency_ms, parry_lead_ms, dodge_lead_ms, jump_lead_ms,
reduced_flash: bool, parry: bool, dodge: bool, jump: bool,
visible: bool, diagnostics: bool, diagnostic_logging: bool,
colors: Colors, // parry,dodge,jump,ready,expired: [f32;4]
profiles: Vec<MoveProfile>,
```

`MoveProfile` exact fields: model/animation i32, phase u32, activation_s/end_s f32,
response `cue::Response`, game_sha256/data_sha256/evidence/encounter/form strings,
contact_min_ms/contact_max_ms f32 offsets from activation, early_ms/late_ms f32
real-ms before contact, preferred_ms Option<f32> real-ms before contact,
trials/successes/failures u32. Contact metadata is not a classifier override.
Store supports `open(path: PathBuf) -> Store`, `current() -> &Config`,
`reload() -> Result<bool,String>`, `adjust_placement(delta:f32) -> Result<(),String>`,
`reset_placement() -> Result<(),String>`, `diagnostic() -> Option<&str>`.
`Config::parse(&str) -> Result<Config,String>` is independently testable.

- [x] Write failing tests exercising parse/default use, invalid reload preserving
  the entire old config, bounds, NaN/infinity/colors/intervals, settings that try
  to weaken invariants, atomic persistence and external edit conflicts.
- [x] Use the approved spec's defaults/ranges exactly. Narrowable response leads
  default 150/300/300, bounded 25..150 / 25..300 / 25..300 animation ms.
  No lock-only/hash/freshness override key exists. Unknown keys reject reload.
- [x] Validate profiles against `reader::RESEARCH_HASH`, SHA-256 of checked-in
  data, exact existing phase and response, nonempty evidence/encounter/form and
  trials=successes+failures >0. Bound contact offsets -500..500 animation ms,
  0<=late<early<=300 real ms, and optional preferred between late/early. Store
  evidence without claiming its content has been independently gameplay-validated.
- [x] Implement worker-callable store, max file 64 KiB, same-directory temporary
  file+flush+atomic replacement. Never replace malformed external content during
  a hotkey persist; reload/validate or report conflict. Preserve last valid config.
- [x] Ship commented defaults and exact reload/reset/hotkey and calibration docs.
  Use upstream API documentation because Context7 is absent. Run config tests
  and provide red/green output plus a scoped review report.

### Task 3: Compact shared renderer and layout checks

Files: create `src/layout.rs`; replace focused drawing in `src/windows/cue_draw.rs`;
update `examples/cue-layout.rs` and optional `scripts/render-cue-layout.py` inputs.
Consumes `Config`, `Decision`, current target and viewport. Produces portable
`layout::bounds(display, aspect, config) -> Option<Bounds>` and shared rendering.

- [x] Add failing geometry tests for 720p/1080p/1440p/4K, ultrawide, letterboxing,
  inset/small viewports, label sizes, full glow bounds and posture gap. Expected
  neutral 1080p lane center x=960; lower bound <=972-12 with default settings.
- [x] Fixed anchor `(0.5,0.87)` in fitted viewport; clamp full bounds to margins
  and configured posture band, suppress impossible fits. Optional overhead uses
  existing checked camera projection and identical timing decisions.
- [x] Shared draw consumes decision rather than recomputing timing. Hollow READY,
  filled actionable with PARRY NOW/DODGE/JUMP, neutral expired EXPIRED; neutral LOCKED
  for unknown. All brightness/size effects fit inside bounds. No diagnostic text.
- [x] Extend offline example with synthetic state/profile and size controls; emit
  measured bounds and use same draw code. Generate labelled synthetic previews,
  inspect images, verify states/labels and full geometry inside playable viewport.

### Task 4: Worker/render integration, lifecycle and diagnostics

Files: `src/windows.rs`, `src/windows/diagnostics.rs`, `src/input.rs`,
`src/event_hook.rs`, `scripts/build.ps1`, `scripts/package.ps1`, `Cargo.toml`.
Consumes interfaces from Tasks 1–3. Produces versioned 0.8.0-preview DLL/package.

- [x] Regression tests for focus/fresh key filtering, reset, snapshot invalidation
  and stale capture lifecycle. Add a validity generation checked at submission;
  loss/switch clears engine state before diagnostic/file work.
- [x] Worker owns config open/reload (1 s) and queued F6/F7/F10 persistence. Render
  only reads validated snapshots. F8 remains visibility, F9 diagnostics; no lock
  bypass. Fixed placement may use configured 16:9 fallback independently of
  projection. Missing required timing observations still suppress action.
- [x] Log exact executable/DLL/data hashes at startup and original capture time,
  poll start/end, render time, effective age, owner/occurrence/phase, state, rate,
  calibration evidence and surface/viewport/mode metadata in bounded logs.
  Distinguish estimated press and draw submission from unavailable presentation,
  actual input/contact/confirmed outcome. Preserve candidate-effect diagnostics.
- [x] Update version without replacing prior-version artifacts. Execute build,
  host-rejection and loader checks. Record unavailable live checks explicitly.

### Task 5: Coverage, user documentation and final verification

Files: `scripts/generate-attack-timings.py`, `scripts/attack_responses.py`,
`scripts/test-attack-timings.py`, coverage files, README, cue/parry/architecture/
Windows/release/manual docs, CHANGELOG and WORK-STATUS.

- [x] Test classifier exclusions against mixed projectiles, uncertain imports,
  unknown routing and every response; fix only demonstrated unsafe classifications.
  Generate exact ledger via a checked-in script and add a check mode.
- [x] Recompute and verify all counts/provenance. Retain unresolved forms and
  zero calibrated/validated status unless new actual trials establish otherwise.
- [x] Update current behavior docs, preserve historical records. Add trial fields
  and remaining evidence for menu/Mikiri/contact/layout limitations.
- [x] Run fmt, locked offline tests, Python tests, diff check, Windows Clippy and
  build script; inspect fresh DLL/shared-renderer evidence and review complete diff.
- [x] Append status with implemented/tested/synthetic/gameplay split, hashes,
  commands, blockers and concrete next actions; print Remaining work before final.

## Execution ledger

User approved the design and execution. Work happens in the requested checkout
on a local feature branch; no worktree copy is needed for user-owned evidence.
Task 1 and Task 2 share only module registration/dependency compilation; config
agent owns its named files, root coordinates registration and all integration.
Tasks 3–4 consume the same decision/config interfaces; no renderer timing fork.
No runtime preferred pulse is claimed for baseline activation-only mappings.
No plan step depends on guessed menu/Mikiri/contact offsets.

- [x] Task 1: timing/source regression and implementation
- [x] Task 2: configuration and scoped review
- [x] Task 3: layout/shared renderer
- [x] Task 4: Windows integration/build
- [x] Task 5: coverage/docs/final review

## Execution outcome - 2026-09-16

All five implementation/check/documentation tasks above were executed. This
checklist marks code and automated evidence work, not the unresolved gameplay
acceptance criteria. The portable Engine API evolved to pass validated settings
and profiles directly; the renderer consumes the resulting Decision. EXPIRED
was selected as the non-actionable expiry label.

91 Windows MSVC Rust tests, 11 Python tests, fmt, Clippy, native DLL/ASI fixtures,
package checksums and synthetic layout checks passed. Default GNU tests and the
python3 command remain environment-blocked; explicit MSVC/python alternatives
passed. No complete live 0.8.0 trial or contact calibration exists. See
[WORK-STATUS](../../WORK-STATUS.md) and [validation](../../validation-0.8.0.md) for
exact results, artifact hashes, source state and detailed remaining work.
