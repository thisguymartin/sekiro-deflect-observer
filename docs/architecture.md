# Windows overlay bootstrap

The first native milestone proves that our DLL can be built, loaded, drawn, hidden, and stopped with the game. It does not inspect deflect state.

```text
me3 profile -> Windows x64 DLL
  -> minimal DLL entry point -> initialization thread
  -> host check and executable hash -> local diagnostic log
  -> hudhook DirectX 11 render hook -> unavailable-reader panel
```

## Boundaries

`src/identity.rs` validates the host filename and hashes a stream. A filename check and a hash do not establish supported gameplay offsets. The current DLL has no offset table or version-sensitive memory reader.

`src/input.rs` recognizes the first F8 key-down message and excludes auto-repeat. It contains no global keyboard hook or synthetic input.

`src/windows.rs` owns DLL startup, local logs, render-hook installation, the panel, and the visibility flag. Its window-message handler forwards messages to the game. The panel accepts no mouse or keyboard interaction.

The DLL entry point schedules initialization and returns. File access, hashing, logging, and renderer setup occur on the initialization thread. The draw callback performs no file access or hashing.

## State and lifecycle

The panel always says `UNKNOWN` and explains that the reader is not implemented. No timer, button, or fabricated event can make it claim an active deflect.

F8 hides drawing without unloading the DLL. The proof of concept stays loaded until the game exits. There is no eject hotkey or claim of safe unloading during play. Installation lives in a separate directory referenced by a me3 profile.

Startup logs include the build version and executable hash under `%LOCALAPPDATA%\SekiroDeflectObserver`. They contain no raw game memory. The observer has no network client or updater. me3 is installed separately and has its own behavior.

## Design decision

A DLL loaded by me3 keeps the user workflow to one launch profile. A separate injector would duplicate loader responsibilities and add another executable to distribute. A simulated deflect display would not establish native integration, so it is not part of this milestone.

The portable code is limited to behavior used by this DLL. A detector, bounded history, and metrics will be added when a real reader can produce justified observations. Their required failure semantics remain in [the testing requirements](../tests/README.md).

The design review had reduced provider coverage. The configured Claude lanes were unauthenticated and Grok was unavailable. A native Codex lane and the parent considered the DLL-only design. This is not a completed multi-provider review.

## Sources and verification limits

The implementation uses the downloaded hudhook 0.9.2 crate and its DirectX 11 API, rather than the older version numbers in some tutorial examples. See [hudhook](https://github.com/veeenu/hudhook) and [me3 native-DLL profiles](https://github.com/garyttierney/me3/blob/main/docs/configuration-reference.md).

Portable tests validate host checks, hashing, and input-event interpretation. Windows compilation validates the Windows API bindings and DLL link. Neither proves correct rendering inside Sekiro. Record that separately using [the Windows checklist](windows.md#test-the-first-launch).

Dependencies and the Rust toolchain are pinned. Byte-identical rebuilds across machines and ZIP timestamps have not been established. A clean build is not a claim of binary reproducibility.
