# Build and run the Windows proof of concept

This version observes candidate effect 105010 and displays CANDIDATE PRESENT, CANDIDATE ABSENT, or UNKNOWN. Deflect-window semantics and live layout validation remain pending. See [reader research and trials](reader-research.md).

Use it to test DLL loading, rendering, F8, normal controls, and startup diagnostics. It is an engineering proof of concept, not a deflect trainer release.

## Run a built package

You need Windows x64, the Steam version of Sekiro, and [me3](https://github.com/garyttierney/me3/releases). You do not need Cheat Engine or development tools to run the ZIP.

1. Obtain the `SekiroDeflectObserver-0.3.0-dev-windows-x64.zip` produced by this project's build. If you downloaded a GitHub Actions artifact, extract its outer ZIP first to find the package and checksum.
2. Extract the package into a separate folder, such as `C:\Mods\SekiroDeflectObserver`. Keep the DLL and `observer.me3` together.
3. Install me3 using its Windows installer. The [me3 setup guide](https://github.com/garyttierney/me3/blob/main/docs/index.md) documents the loader's prerequisites.
4. Start Steam. Close Sekiro if it is already running.
5. Double-click `observer.me3`. Alternatively, run `launch-observer.cmd` from the extracted package.
6. Look for the Sekiro Deflect Observer panel in the top-left corner. Load a save and check candidate status. If UNKNOWN persists, record its reason and preserve the sample log. If it does not appear, use the troubleshooting steps below.
7. Press F8 to hide the panel, then press F8 again to show it.

For a terminal launch, run this from the extracted package directory:

```powershell
me3 launch --game sekiro --profile .\observer.me3
```

The profile uses me3's documented native-DLL entry. Relative DLL paths are stored with the profile. See the [configuration reference](https://github.com/garyttierney/me3/blob/main/docs/configuration-reference.md).

The observer DLL makes no network requests. me3 and Steam are separate dependencies with their own behavior. This version adds rendering hooks and reads version-sensitive gameplay state for research. It does not write gameplay state.

## Build the package on Windows

Install these development dependencies once:

1. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with **Desktop development with C++**. Include the MSVC x64 compiler and Windows SDK.
2. Install Rust using the Windows installer from [rustup](https://rust-lang.org/tools/install/). Choose the MSVC toolchain.
3. Open a new **Developer PowerShell for VS 2022** terminal so the build tools are available.
4. Copy or clone this source checkout onto the Windows PC. Keep `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `src`, `scripts`, and `packaging` together, including the `.cargo` directory.
5. Change to the project root, then run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build.ps1
```

The execution-policy option applies to that script process. It does not change the machine's saved policy. Review the script before running it.

The toolchain file selects Rust 1.94.0, the required formatting and lint components, and the `x86_64-pc-windows-msvc` standard library. The first build downloads the toolchain and locked dependencies. These downloads are development activity, not observer runtime traffic.

The build script runs formatting, Clippy, unit tests, and a release build. It then loads the DLL in a separate PowerShell process and checks that the DLL rejects this non-Sekiro host before installing hooks. Finally, it creates:

```text
dist/
  SekiroDeflectObserver-0.3.0-dev-windows-x64.zip
  SekiroDeflectObserver-0.3.0-dev-windows-x64.zip.sha256
```

The ZIP contains the DLL, launcher, me3 profile, README, project license, dependency license texts, and file checksums. The DLL uses the static MSVC runtime. No loader or game files are bundled.

Check the ZIP's checksum with PowerShell:

```powershell
Get-FileHash .\dist\SekiroDeflectObserver-0.3.0-dev-windows-x64.zip -Algorithm SHA256
Get-Content .\dist\SekiroDeflectObserver-0.3.0-dev-windows-x64.zip.sha256
```

Compare the hash values without regard to letter case. A matching checksum checks the downloaded bytes, not gameplay correctness. See [Microsoft's hash command documentation](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.utility/get-filehash).

## Build with GitHub Actions

1. Push the source branch containing `.github/workflows/windows.yml` to your GitHub repository.
2. Open **Actions > Windows proof of concept** and select the run for that exact commit.
3. Wait for the build job to pass.
4. Download the `SekiroDeflectObserver-windows-x64` artifact at the bottom of the run page.
5. Extract the artifact and follow the built-package instructions above.

The workflow also supports **Run workflow** once it is available on the default branch. Artifacts expire after 14 days. They are test downloads, not stable GitHub Releases. GitHub documents this process in [Downloading workflow artifacts](https://docs.github.com/en/actions/managing-workflow-runs-and-deployments/managing-workflow-runs/downloading-workflow-artifacts).

## Test the first launch

Record the exact Windows, GPU-driver, me3, and game versions. Start in windowed or borderless mode and test fullscreen separately.

- [ ] The game starts using `observer.me3`.
- [ ] The panel appears and says the observer is loaded.
- [ ] The panel shows candidate status or explains why reads are unavailable.
- [ ] F8 hides and shows the panel once per press. Holding F8 does not flicker it.
- [ ] Movement, camera, attacks, guard, menus, and controller input still work.
- [ ] Moving the pointer over the panel does not steal gameplay input.
- [ ] Returning from another application does not leave the hotkey stuck.
- [ ] The panel survives opening a menu, loading a save, and returning to title without crashing.
- [ ] A local startup log exists under `%LOCALAPPDATA%\SekiroDeflectObserver`.
- [ ] Closing the game stops the observer. A normal Steam launch does not load this profile.

These are manual checks to perform, not results already observed. The [session report template](../tests/compatibility/session-template.md) can record the evidence. Deflect-semantic checks remain blocked until a reader exists.

## Stop and uninstall

F8 changes visibility only. Close Sekiro to unload the proof of concept. Do not use a DLL ejector to unload it while the game is running.

For a normal game session, launch Sekiro through Steam without the observer profile. To remove the observer, close the game and delete its extracted folder. You may also delete `%LOCALAPPDATA%\SekiroDeflectObserver` after collecting any logs you need.

Keep shared me3 files used by other mods. This profile does not require copying the observer into the game directory or changing Steam launch options.

## Troubleshoot

| Problem | Action |
|---|---|
| `cargo` is not recognized | Install Rust, then open a new terminal. |
| `E0463`: cannot find crate for `core` or `std` | From the project root, run `rustup target add x86_64-pc-windows-msvc`, then rerun the build. Targets belong to the active toolchain; installing one for `stable` does not install it for this project's pinned Rust version. |
| `link.exe` or the Windows SDK is missing | Install the C++ workload and run the build from Developer PowerShell. |
| A dependency cannot download | Record the exact download error. Restore normal network access and rerun the same locked build. |
| Formatting, Clippy, or tests fail | Keep the full command output. Do not skip checks to call the package verified. |
| Packaging reports missing license text | Review that dependency and update the package procedure before distributing it. |
| `me3` is not recognized | Open a new terminal after installation, or double-click the `.me3` file. |
| The game is already running | Exit it normally and launch the profile again. |
| The panel does not appear | Check the observer log folder and me3 output. Confirm the DLL is next to the profile and the ZIP was fully extracted. |
| A log exists but drawing never starts | Record the hook error, driver, display mode, and other overlays. Retry in windowed mode with unrelated overlays disabled. |
| The panel stays amber when pressing deflect | Record the UNKNOWN reason and sample log; unsupported hashes and failed or stale reads remain unavailable. |
| The game crashes | Start it normally without the profile. Preserve logs and the exact build for diagnosis. |

Share relevant log text and hardware/software versions with a report. Do not upload the game executable, saves, or memory dumps.
