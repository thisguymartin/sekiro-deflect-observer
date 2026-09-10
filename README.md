# Sekiro Deflect Observer

A training-tool project intended to show the timing state created by the player's own deflect input in *Sekiro: Shadows Die Twice*.

**Windows overlay proof of concept. Deflect detection is not implemented yet.**

This checkout now contains a native Rust DLL, Windows build scripts, and a me3 launch profile. The panel displays **UNKNOWN** because no game-state reader or verified memory profile exists. It does not turn green when you press deflect.

The handover identified candidate effect `105010`. Its meaning has not been verified here. The original Cheat Engine prototype is optional research material and is not required to build or run this proof of concept.

V1 observes the player's state. Incoming attack prediction belongs to a later V2. The intended tool does not automate inputs or change the deflect window.

## Build and run on Windows

Follow [the Windows instructions](docs/windows.md). With Rust and the Visual Studio C++ build tools installed, run this from the source directory:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build.ps1
```

Extract the ZIP created in `dist`, install [me3](https://github.com/garyttierney/me3/releases), start Steam, and double-click the extracted `observer.me3` while Sekiro is closed.

The build creates `sekiro_deflect_observer.dll`. Its overlay shows that the DLL loaded and that the reader is unavailable. F8 hides or shows the panel. Close the game to stop the observer.

Built packages require no Cheat Engine, Rust, Visual Studio, or Python on the player's PC. The Windows CI workflow also builds a downloadable test artifact. These are engineering test packages, not stable mod releases.

## Test the project

Start with [the native first-launch checklist](docs/windows.md#test-the-first-launch). The older [Cheat Engine research guide](docs/testing.md) applies only if that separate prototype becomes available.

- [Gameplay checklist](tests/manual/gameplay-checklist.md) covers taps, guard, combat, and game transitions.
- [Investigate effect 105010](docs/reverse-engineering.md) explains how to compare observations without assuming the effect means a deflect window.
- [Session report template](tests/compatibility/session-template.md) records the exact build, setup, evidence, and failures.
- [Automated test requirements](tests/README.md) defines the state and history contracts for implementation.
- [Compatibility status](docs/compatibility.md) lists the platforms awaiting validation.
- [Test a native release candidate](docs/release-testing.md) covers installation, removal, packaging, and release evidence.

## Intended design

```text
Sekiro local player state
  -> version-gated memory reader
  -> Unknown / Inactive / Active
  -> timing history
  -> local overlay
```

A failed or stale memory read must produce `Unknown`. Only a valid observation can produce `Inactive` or `Active`. The released observer is intended to operate without network requests.

## Development status

Implemented in this milestone: a Rust DirectX 11 overlay using hudhook, F8 visibility, host-name validation, executable SHA-256 diagnostics, local logs, unit checks, and Windows packaging. See [the architecture](docs/architecture.md).

Not implemented: version-sensitive game-state reading, deflect detection, timing history, metrics, and overlay settings. Executable fingerprinting is diagnostic only; it is not automatic compatibility validation. No game-memory offsets are followed.

Windows is the initial target. Real Sekiro gameplay, Proton, and macOS compatibility remain unverified. [Compatibility status](docs/compatibility.md) records actual evidence.
