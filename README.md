# Sekiro Deflect Observer

A training tool with a timing slider above Wolf that anticipates selected incoming attacks in *Sekiro: Shadows Die Twice*. The player presses the buttons.

**Windows overhead cue preview. Green timing is an estimate; exact parry timing is not validated.**

For the friend beta, use the [drop-in installation and sharing guide](docs/sharing-beta.md).
It packages the same 0.6.3 observer with an ASI loader: copy two files beside
`sekiro.exe` and launch through Steam. That route does not require me3 and still
needs a live game launch trial. Do not overwrite an existing `dinput8.dll`.

Current design, rendered from the shared overlay code with synthetic attack states:

![0.6.3 offline design preview: READY, PARRY, DODGE and JUMP](docs/images/0.6.3-offline-design.png)

[Gameplay screenshot and image provenance](docs/screenshots.md). The image above
shows appearance, not a successful-deflect or live 0.6.3 gameplay test.

Earlier in-game placement, from the **0.6.0 gameplay trial** (the current design
is shown above):

![Earlier 0.6.0 gameplay showing the neutral LOCKED bar above Wolf](docs/images/0.6.0-gameplay-placement.jpg)

This checkout contains a native Rust DLL, Windows build scripts, and a me3 launch profile. Version 0.6.3-preview refines the supplied reference's design with a larger glowing diamond, longer needle, moving magenta pointer, translucent ribbon wings and outlined English labels. It retains the lower player-relative placement and combo reader improvements. Green **PARRY**, orange **DODGE** for mapped grabs, and blue **JUMP** for mapped sweeps use extracted animation events and attack parameters. Unknown responses stay unverified. The visual update does not change the estimated press intervals or game acceptance windows. See [preview scope, calculation, coverage, and limits](docs/cue-preview.md) and [visual validation](docs/validation-0.6.3.md).

The data contains 2,161 phases across 54 models: 450 green estimates, 39 dodge phases, and 58 jump phases. Chained Ogre and Guardian Ape have selected mappings; this is not every enemy, attack, or form. Stricter response checks remove questionable green prompts from 0.5.0. These counts describe data, not verified gameplay support. See [the validation record](docs/validation-0.6.md).

The 0.6.0 gameplay trial exposed the Ogre's auxiliary animation hiding its attack track. Version 0.6.1 corrects that reader selection using the current engine batch; the corrected cues require another live timing check.

The handover's claim of an earlier Cheat Engine prototype is unconfirmed; the user has never had it. No prototype is required. Public source research provides the candidate layout; live validation remains pending.

The requested first usable version is an overhead **parry-now cue** driven by incoming attack timing. The preview implements placement and an attack-based estimate; predicting actual player contact and verifying successful deflects remain. The optional player-effect panel is a research aid. See [the cue requirements and timing research](docs/parry-cue.md). The tool does not automate inputs or change the deflect window.

## Build and run on Windows

Follow [the Windows instructions](docs/windows.md). With Rust and the Visual Studio C++ build tools installed, run this from the source directory:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build.ps1
```

Extract the ZIP created in `dist`, install [me3](https://github.com/garyttierney/me3/releases), start Steam, and double-click the extracted `observer.me3` while Sekiro is closed.

The build creates `sekiro_deflect_observer.dll`. F6 lowers the bar; F7 raises it (session only). F8 hides or shows the cue; F9 toggles diagnostics. Close Sekiro fully before launching a different package; a running process retains its loaded DLL.

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
Sekiro enemy target, attack timing, and camera
  -> version-gated read-only observations
  -> experimental advance response estimate
  -> timing slider positioned above Wolf
```

A failed or stale observation must suppress the parry cue. The existing diagnostic panel represents read failure as `Unknown`. The released observer is intended to operate without network requests.

## Development status

Implemented: a DirectX 11 overlay, F8 visibility, executable SHA-256 gate, bounded read-only candidate traversal, stale/error handling, diagnostic transition history, timestamped sample logs, tests, and Windows packaging. See [the architecture](docs/architecture.md).

The preview includes bounded locked-target and animation history reads, camera projection, shared animation imports, attack-parameter response classification, and a timing slider above the player. The anchor remains a lowered standing-height approximation rather than an animated head bone. Pending: new placement verification, attack/contact timing and reach geometry, runtime behavior variation and blend handling, and successful-deflect verification. Complete-window statistics and reaction-time scores are not required. A matching executable hash does not establish gameplay correctness.

Local analysis on 2026-09-12 successfully indexed 142 character animation archives and extracted attack event timelines from the installed game. This supplies data for deriving timings automatically rather than asking the player to chart every enemy. See [the measured findings and remaining runtime work](docs/game-file-analysis.md).

Windows is the initial target. Recording 02 shows the older overlay rendering before visible contact sparks in one soldier exchange; it does not validate the new version or exact input timing. Proton and macOS remain unverified. [Compatibility status](docs/compatibility.md) records platform evidence.
