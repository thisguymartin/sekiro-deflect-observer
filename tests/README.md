# Tests and remaining requirements

The Rust code has unit tests for executable-name checks, SHA-256 diagnostics, read-error propagation, and F8 repeat handling. Run them on macOS or Windows with:

```sh
cargo test --locked
```

The DLL and render hooks are Windows-only. These unit tests do not load the DLL, start Sekiro, or prove in-game behavior. Run the complete Windows build checks through [scripts/build.ps1](../scripts/build.ps1), then follow [the native first-launch checklist](../docs/windows.md#test-the-first-launch).

The original Cheat Engine prototype is not required for these checks.

The remaining tables are future reader and history requirements. They do not describe an implemented or passing suite.

## Reader and detector contract

Use synthetic memory and executable metadata. The test runner must not require or attach to a real game process.

| ID | Fixture or operation | Required observable result |
|---|---|---|
| U01 | Supported identity, valid player, complete effect collection without 105010 | `Inactive`. |
| U02 | Supported identity, valid player, valid snapshot containing 105010 | `Active`. |
| U03 | Read failure, exception, missing player, or invalid pointer at any reader stage | `Unknown` with a diagnostic reason. |
| U04 | Missing or unsupported executable identity | `Unknown`; zero version-sensitive state-reader calls. |
| U05 | Cyclic, over-limit, truncated, or inconsistent effect collection | `Unknown`; traversal ends within a defined bound. |
| U06 | Complete, valid empty collection | `Inactive`; distinguish a valid empty collection from a failed head-pointer read. |
| U07 | Effect exists only on a non-player entity or old player instance | Do not report local-player `Active`. Missing current-player data is `Unknown`. |
| U08 | Active or inactive sample followed by read failure | Immediately report `Unknown`; do not retain the previous known state. |
| U09 | Samples stop; advance an injected clock to just before, at, and after the freshness limit | Expire to `Unknown` at the limit. No real-time sleeps are needed. |
| U10 | New process session and a late callback from the old session | Discard the old result. Validate the new identity before reading state. |

## History and metrics contract

Use an injected monotonic clock and explicit sample timestamps. Keep history bounded independently of how often a renderer draws.

| ID | Input sequence or operation | Required observable result |
|---|---|---|
| H01 | Inactive, active, active, inactive with known timestamps | One complete observed interval. Repeated active samples do not add activations. |
| H02 | Inactive, active, unknown, active, inactive | Preserve the unknown gap. Both boundary-incomplete segments stay out of complete-window duration statistics. |
| H03 | Start observation while already active | Mark the start as unknown. Do not infer an input event or include a full-window duration. |
| H04 | Stop, disconnect, or expire while active | Mark the end incomplete. Do not manufacture a completed window. |
| H05 | More samples than capacity; advance beyond history duration | Evict old history, preserve order and gaps, and keep memory bounded. |
| H06 | Interval overlaps the visible history boundary | Clip drawing to the viewport without changing the measured full interval. |
| H07 | Duplicate timestamp or out-of-order sample | No negative duration or corrupted ordering. Document the rejection or coalescing policy. |
| H08 | Long polling gap or clock discontinuity | Mark unavailable coverage according to the freshness policy. Do not bridge the gap as continuous known state. |
| H09 | No complete intervals or only incomplete intervals | Show unavailable duration metrics, with zero complete windows. Avoid division by zero and misleading zero-ms averages. |
| H10 | Several complete intervals with known durations | Correct count, minimum, maximum, and mean. Exclude unknown and incomplete observations. |
| H11 | New process session | Separate histories and reset session metrics according to the documented policy. Do not combine intervals across sessions. |

## Runtime and renderer requirements

When a native implementation exists, test these behaviors through its actual settings and renderer boundaries:

- Draw `Unknown`, `Inactive`, and `Active` distinctly, with text or shape as well as color.
- Draw unknown history gaps without interpolating them into known state.
- Validate history duration, opacity, scale, position, and toggle-key settings at the config boundary.
- Make repeated start and stop operations safe. Release hooks, resources, timers, and hotkeys without duplicate registration.
- Keep the configured visibility policy separate from data freshness. Hiding the overlay cannot make stale data valid.
- Reject unsupported executable profiles before the reader follows version-sensitive pointers.

Add reproducible runner prerequisites, exact commands, and expected exit behavior when the implementation is available. A successful test command must return a nonzero exit status when an assertion fails. Pin build dependencies before enabling release CI.

Use [real gameplay checks](manual/gameplay-checklist.md) for memory-layout and effect-semantic validation. Synthetic tests cannot establish either one.
