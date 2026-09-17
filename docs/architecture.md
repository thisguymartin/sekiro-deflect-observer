# Native observer architecture (0.12.1-preview)

The DX11 backend is the pinned hudhook 0.9.2 source under `vendor/hudhook` with
the patch documented in `OBSERVER-PATCH.md`. HUD commands are recorded on a
private deferred context and submitted with `ExecuteCommandList(..., TRUE)`;
the D3D runtime restores host context state. Empty draw data does no GPU work.
Partial command lists are discarded on error. Texture updates touch only
renderer-owned resources. The old incomplete manual state backup is removed.
See `tests/dx11-render-isolation.rs` and `docs/validation-0.9.1.md`.

The default alert path is `Engine::incoming` -> `src/incoming.rs` ->
`src/attack.rs`, using `src/incoming_attacks.rs` and the exact evidence in
`docs/incoming-coverage.json`. Raw attack classification does not accept display
preferences. Alert filtering and Mikiri-to-PARRY fallback stay in `incoming.rs`.
It identifies the current/next attack phase and response without reach gating,
rate extrapolation, calibrated contact or press windows. Wind-up/active are
separate from actionable timing. `incoming_cues = false` selects the preceding
timing engine described below. See [incoming responses](incoming-attacks.md).

The project is a Rust Windows x64 DLL that reads game observations and submits
a DX11 HUD through hudhook. Optional F11 practice starts off and temporarily
writes eligible enemy animation speed. Its controller and private native writer
are separate from alert presentation. It does not generate input, alter Wolf's
speed or deflect windows, edit game/save files or use a network service.
See [feature boundaries](feature-boundaries.md) and
[practice behavior and limitations](enemy-speed-practice.md).

```text
DLL initialization -> host + executable/DLL hashes -> hooks + worker
locked target -> current animation batch -> validated capture timestamp
  -> raw attack kind / hit phase -> alert filtering -> shared ImGui renderer
  -> fresh advancing target -> practice policy -> owned speed write / restore
practice applied status -> renderer caption (independent of alert preferences)
worker config reload / atomic save -> bounded settings snapshot
research candidate effects -> separate F9 panel and bounded local CSVs
```

## Ownership and lifecycle

`src/cue.rs` reacquires player, lock manager/selected point, actor and modules,
checks health and ownership, selects only the current ring batch and validates
coarse reach/facing. The Ogre auxiliary track can no longer hide a mapped attack;
competing mapped tracks remain ambiguous and no older ring entry is revived.
Camera reads are independently optional for screen-space placement. Overhead
projection still validates its camera and rejects invalid/offscreen geometry.
The known debug-camera flag suppresses the gameplay observation.

`src/event_hook.rs` observes completed batches before their boundary reset. Its
existing native detour preserves the engine call/return. It is gated by both the
executable and researched function bytes. Snapshots retain their original
monotonic capture time, owner/player identity and generation. Changing owners or
losing lock clears captures. Failed/stale captures never silently become polling
samples while the event hook is active. Without the hook, bounded polling remains
available. Neither path is contact or outcome observation.

`src/timing.rs` owns portable occurrence, rate, phase, interval and pulse decisions.
Worker observations feed it on the existing 8 ms poll cadence; rendering asks for
a decision at its actual monotonic timestamp. Rate stability, 25 ms extrapolation
and the unchanged 50 ms freshness ceiling bound the model. See
[the equations and limitations](parry-cue.md). A raw sequence is retained for
research but does not define an occurrence or progression.

`LiveCue::current_lock` checks only the validated lock read's 50 ms age;
`LiveCue::current` also checks original animation-capture age. The renderer may
retain neutral LOCKED from the former while the timing engine requires the
latter. Animation failure clears press/pulse state without discarding a fresh
lock. Full clear, lock loss, focus/visibility and generation gates still hide it.

The worker reads and publishes before configuration or diagnostic file work.
`src/lifecycle.rs` advances an atomic invalidation generation as soon as loss,
owner/animation change or animation failure is observed, even before the worker
can acquire the cue mutex. F8 hiding advances it immediately, including hide/show
pairs between renders. The renderer holds the short cue-state mutex through
its decision and checks the generation again immediately before draw commands.
No memory traversal or file I/O runs under that draw lock. Already submitted
commands cannot be revoked; visible presentation delay still needs measurement.
Target switching/errors, F8 hiding, lost focus and unsupported geometry clear
pending timing. Dropped settings commands are counted visibly in F9.

## Configuration and rendering

`src/config.rs` parses bounded TOML with whole-file validation and a 64 KiB limit.
The initialization/worker path creates defaults, checks reloads once per second
and performs atomic same-directory saves. Malformed reloads keep the entire last
valid configuration. Hotkeys enqueue changes; the render callback never parses,
hashes or accesses files. `src/windows/diagnostics.rs` publishes config/timing
under one mutex. Profile metadata is validated against exact game and data
identity. Named forms remain inactive without a runtime form discriminator.

`src/layout.rs` computes full cue bounds, a fitted playable viewport, safe margins
and a reserved posture band. The default is configured normalized placement,
not posture detection. `src/windows/cue_draw.rs` is shared with the separate
`examples/cue-layout.rs` synthetic mesh exporter. Label, shape and response color
communicate state; glow/pulse geometry stays inside the full bounds. The offline
example is never used as an unlocked gameplay fallback.

`src/input.rs` accepts fresh focused F6..F11 key-down events; every event continues
to the game. No combat input capture, keyboard hook or synthetic input is added.
F9 is independent of target validity and cannot bypass gameplay lock-on gating.
F11 arms practice for the current process only. F8 hiding disarms practice;
showing the HUD again does not rearm it. Alert response toggles and HUD mode do
not enable or disable slowdown. The worker runs the speed controller outside
the render mutex; no speed writes run in the DX11 or animation-hook callbacks.

## Local evidence

Startup logs record the package version, actual loaded DLL file hash and executable
hash. Cue CSVs record read start/end, owner, animation and reader stage. Render
CSVs record the same epoch, data identity, sampled/projected animation clocks,
source/read age, observation and original-capture IDs, source kind, owner/validity
generations, occurrence/phase, contact proxy/profile, press interval, preferred
instant, state, pulse, HUD mode, surface and viewport. Render rows retain profile
evidence/scope/trial totals and invalidation-to-hidden-submission timestamps.
Those delay fields enable a new trial; they are not a measured live result. Display mode is explicitly
unobserved and must be filled from the trial record; surface size alone cannot
identify exclusive fullscreen versus borderless. Presentation, actual input,
contact and successful-deflect fields remain `unobserved`.

Each CSV stops at 16 MiB; a bounded 512-entry nonblocking queue carries render
records. F9 shows logging status and dropped records. `diagnostic_logging=false`
skips general diagnostic data rows. The separate bounded practice-write audit
remains enabled and records applied/original speed and controller transitions.
Sparse alerts also record practice status. File/log failures never authorize guidance. The separate
`reader.rs`/`samples.rs` candidate-effect history retains its own semantics and
freshness; effect 105010 is not proof of successful deflection.

## API evidence

Context7 is not exposed in this environment. Dependency APIs were checked against
locked upstream source/documentation for serde 1.0.195, toml 0.8.8 and imgui 0.12.
Win32 contracts were checked against official Microsoft documentation for
[GetModuleFileNameW](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulefilenamew),
[GetWindowThreadProcessId](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid)
and [MoveFileExW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw).

Automated checks and renderer images are synthetic evidence. Live timing,
menu/loading behavior, actual posture spacing and accepted defensive outcomes
remain separate tasks in [WORK-STATUS](WORK-STATUS.md).
