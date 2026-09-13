# Windows candidate-effect diagnostics

The DLL observes candidate effect 105010. Its meaning as a deflect window and live layout validity remain unverified. See [reader research](reader-research.md) for provenance, constraints, logging, and gameplay trials.

The 0.4.0 preview adds locked-enemy and animation-history reads, normal camera
projection, and a small overhead ring with an activation-based green estimate.
See [the preview calculation and limits](cue-preview.md). The text panel
described below remains optional research diagnostics and is hidden by default.
Exact player contact prediction and gameplay verification remain.

```text
me3 profile -> Windows x64 DLL
  -> minimal DLL entry point -> initialization thread
  -> host check and executable hash -> local diagnostic log
  -> hash-gated diagnostic worker -> bounded traversal -> sample history and CSV
  -> bounded target/camera reads -> activation estimate and frozen-clock checks
  -> hudhook DirectX 11 render hook -> overhead ring; optional candidate panel
```

## Boundaries

`src/identity.rs` validates the host filename and hashes a stream. `src/reader.rs` gates reads on an exact research hash, reacquires the local player, and performs two bounded effect-list traversals with ownership checks. `src/windows/diagnostics.rs` implements ReadProcessMemory, polling, and capped CSV logging. A matching hash does not prove layout correctness.

`src/cue.rs` implements target/camera reads, latest animation-history selection,
projection, approximate distance/facing filters, and 50 ms sample/clock freshness.
`src/attack_timings.rs` is generated from local TAE JSON and contains a narrow
soldier/general preview table. Hitbox activation is not treated as proven contact.

`src/input.rs` recognizes fresh F8/F9 key-down messages and excludes auto-repeat. It contains no global keyboard hook or synthetic input.

`src/windows.rs` owns DLL startup, local logs, render-hook installation, the panel, and the visibility flag. Its window-message handler forwards messages to the game. The panel accepts no mouse or keyboard interaction.

The DLL entry point schedules initialization and returns. File access, hashing, logging, and renderer setup occur on the initialization thread. The draw callback performs no file access or hashing.

## State and lifecycle

The panel shows candidate presence, absence, or UNKNOWN. `src/samples.rs` handles freshness, transition history, ordering, and gaps. Errors immediately replace known state; observations expire 100 ms after read start. Input never substitutes for candidate observations. The render callback copies a short snapshot under a mutex, with memory reads and sample logging owned by the worker.

F8 hides drawing without unloading the DLL. The proof of concept stays loaded until the game exits. There is no eject hotkey or claim of safe unloading during play. Installation lives in a separate directory referenced by a me3 profile.

Startup logs include the build version and executable hash under `%LOCALAPPDATA%\SekiroDeflectObserver`. Sample CSV files contain read timestamps, candidate state, errors, player addresses and effect IDs; they are capped at 16 MiB per process. Older runs are retained. The observer has no network client or updater. me3 is installed separately and has its own behavior.

## Design decision

A DLL loaded by me3 keeps the user workflow to one launch profile. A separate injector would duplicate loader responsibilities and add another executable to distribute. A simulated deflect display would not establish native integration, so it is not part of this milestone.

The portable reader and bounded transition history are tested with synthetic data. Complete-window metrics are deferred until real observations support the effect's meaning. Their target failure semantics remain in [the testing requirements](../tests/README.md). Repeated traversal cannot create an atomic engine snapshot or fully eliminate pointer reuse races.

## Sources and verification limits

The implementation uses the downloaded hudhook 0.9.2 crate and its DirectX 11 API, rather than the older version numbers in some tutorial examples. See [hudhook](https://github.com/veeenu/hudhook) and [me3 native-DLL profiles](https://github.com/garyttierney/me3/blob/main/docs/configuration-reference.md).

Portable tests validate host checks, hashing, and input-event interpretation. Windows compilation validates the Windows API bindings and DLL link. A Windows load check verifies DLL startup and refusal to install hooks in PowerShell. These checks do not prove correct rendering inside Sekiro. Record that separately using [the Windows checklist](windows.md#test-the-first-launch).

Dependencies and the Rust toolchain are pinned. Byte-identical rebuilds across machines and ZIP timestamps have not been established. A clean build is not a claim of binary reproducibility.
