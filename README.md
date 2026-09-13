# Sekiro Deflect Observer

A training tool with a timing slider above Wolf that anticipates selected incoming attacks in *Sekiro: Shadows Die Twice*. The player presses the buttons.

**Windows overhead cue preview. Green timing is an estimate; exact parry timing is not validated.**

**Quick links:** [Install and run](#install-and-run-on-windows) · [Build from source](#build-from-source) · [More images](docs/screenshots.md) · [Troubleshooting](docs/windows.md#troubleshoot)

For the friend beta, use the [drop-in installation and sharing guide](docs/sharing-beta.md).
It packages the same 0.6.3 observer with an ASI loader: copy two files beside
`sekiro.exe` and launch through Steam. That route does not require me3 and still
needs a live game launch trial. Do not overwrite an existing `dinput8.dll`.

Current design, rendered from the shared overlay code with synthetic attack states:

![0.6.3 offline design preview: READY, PARRY, DODGE and JUMP](docs/images/0.6.3-offline-design.png)

| PARRY - current offline detail | DODGE - current offline detail | JUMP - current offline detail |
| --- | --- | --- |
| ![Green PARRY close-up](docs/images/0.6.3-parry-detail.png) | ![Orange DODGE close-up](docs/images/0.6.3-dodge-detail.png) | ![Blue JUMP close-up](docs/images/0.6.3-jump-detail.png) |

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

## Install and run on Windows

Your friend needs **Windows x64 and Sekiro on Steam**. They do not need to build
anything or install Rust, Python, Visual Studio, Cheat Engine or me3 for the
drop-in package.

1. Get `SekiroDeflectObserver-0.6.3-preview-drop-in-windows-x64.zip` from the person sharing the beta and extract it.
2. Close Sekiro. In Steam, right-click Sekiro → **Manage → Browse local files**.
3. Copy `dinput8.dll` and `sekiro_deflect_observer.asi` beside `sekiro.exe`.
4. Launch normally through Steam, load a save and lock onto an enemy. Press **F9** to check the observer version.

If `dinput8.dll` already exists, **do not overwrite it**; another mod's loader
needs a compatibility check first. Use one observer loading method per session.
The ZIP's `START-HERE.txt` repeats these instructions and explains the colors.
This drop-in route has passed a standalone loader check; live Sekiro startup
and installation on another PC remain beta checks.

| Key | Action |
| --- | --- |
| F6 / F7 | Lower / raise the bar for this session |
| F8 | Show / hide the overlay |
| F9 | Show / hide diagnostics and version |

To uninstall, close Sekiro and remove `sekiro_deflect_observer.asi`. Remove the
supplied `dinput8.dll` only if no other mod uses it. With the drop-in files still
installed, a normal Steam launch loads the observer.

The separate `SekiroDeflectObserver-0.6.3-preview-windows-x64.zip` uses me3.
Extract that variant into its own folder, install [me3](https://github.com/garyttierney/me3/releases),
keep Steam running and double-click `observer.me3` with Sekiro closed.
See [the complete Windows guide](docs/windows.md) for both methods and troubleshooting.

<a id="build-and-run-on-windows"></a>

## Build from source

These steps are for development. Install **Rust through rustup** and
**Visual Studio Build Tools with Desktop development with C++ and the Windows SDK**.
Open Developer PowerShell for Visual Studio in this source folder, then run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build.ps1
```

The script runs formatting, Clippy, Rust tests, an optimized Windows build and
the DLL startup check, then creates:

```text
target/x86_64-pc-windows-msvc/release/sekiro_deflect_observer.dll
dist/SekiroDeflectObserver-0.6.3-preview-windows-x64.zip
dist/SekiroDeflectObserver-0.6.3-preview-windows-x64.zip.sha256
```

This build produces the **me3 package**. To run your compiled DLL with the
drop-in loader instead, copy it into a separate test folder as
`sekiro_deflect_observer.asi`, alongside the `dinput8.dll` from the friend ZIP,
then follow the drop-in installation steps. Restart Sekiro after every rebuild.

The [step-by-step build and packaging guide](docs/windows.md#build-the-package-on-windows)
covers getting the source, prerequisites, exact commands and the pinned inputs
needed to reproduce the existing drop-in ZIP. `scripts/package-drop-in.py`
repackages that specific tested release; it is not a general fresh-build packager.
Keep older ZIPs before building again: `build.ps1` replaces its same-version me3 ZIP.
The checked-in generated timings are sufficient to compile; game archives are not needed.

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
