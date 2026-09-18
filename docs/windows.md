# Build and run on Windows

The current source candidate is **0.12.1-preview**, with a configured compact HUD
below the enemy's top posture bar, persistent TOML settings and incoming attack
response labels without reach/contact prediction. Optional F11 practice slows
eligible enemy attacks independently of alert preferences and starts off. See
[practice controls and limits](enemy-speed-practice.md). Build it with
`scripts/build.ps1`; that produces the new me3
package. [Current behavior and limits](cue-preview.md), [configuration](configuration.md),
and [validation](validation-0.12.1.md) supersede older cue descriptions.

## Run a built package

Players need Windows x64 and Sekiro on Steam. No compiler, Rust, Python or
Cheat Engine is needed to run the package. Install me3 once, then launch the
observer profile from its extracted folder.

The beta ZIP is supplied separately by the author. GitHub's **Code → Download ZIP**
downloads source code, not a ready-to-play mod.

### Install the 0.12.1 me3 package

1. Extract the **me3** ZIP into its own folder, such as `C:\Mods\SekiroDeflectObserver`.
2. Install me3 using its [official Windows release](https://github.com/garyttierney/me3/releases).
3. Keep Steam running and close Sekiro.
4. Double-click `observer.me3`, or run `launch-observer.cmd` from that folder.
5. Load your save, lock onto an enemy and check the HUD below the enemy's top posture bar.
   Existing configurations need `anchor = "top"`, `width = 480` and zero offsets.

For a terminal launch, run this inside the extracted package folder:

```powershell
me3 launch --game sekiro --profile .\observer.me3
```

Keep the DLL and profile together. Remove the drop-in observer from the game
folder before switching to this method. A running game retains its loaded DLL
until it exits completely.

### Current 0.12.4 controls and states

| Control or cue | Meaning |
| --- | --- |
| F6 / F7 | Lower / raise by 8 reference pixels; persist the offset |
| F8 | Show / hide gameplay HUD; persist visibility; hiding disarms practice |
| F9 | Show / hide separate diagnostics and version; persist visibility |
| F10 | Reset horizontal and vertical offsets |
| F11 | Toggle enemy-speed practice for this session; initially off |
| Shift+F11 | Switch and save 80% / 90% / 70% enemy speed without changing on/off |
| Crescent OFF / ON + percentage | Gray off; gold armed; jade applied; amber ! needs attention |
| PRACTICE / PRACTICE 80% | Armed and waiting / applied slowdown with attack hints disabled |
| PARRY / DODGE / JUMP / MIKIRI | Incoming response; marker approaches center during wind-up |
| NO PARRY / UNKNOWN | Deflection disabled / response unresolved |
| READY, hollow lane | Legacy timing mode only: prepare for the selected hit |
| Green PARRY NOW, filled lane | Estimated or compatible calibrated defensive press interval |
| Orange DODGE | Mapped incoming grab; no safe direction is predicted |
| Blue JUMP | Mapped low sweep |
| Gray EXPIRED | Press interval ended; no actionable pulse remains |
| LOCKED | Fresh target without an active timing cue |
| WATCH | Known attack with unverified response; no button instruction |

All hotkeys require a focused fresh press and pass through. Lane fill follows
animation progression. Default mode labels the move and response, with no
press interval. A preferred pulse exists only in calibrated legacy timing mode. Default incoming mode
shows response types without reach/contact prediction. F9 research
can appear without a target but cannot bypass gameplay lock gating.

## Build the package on Windows

Build in a **new checkout** if you want to preserve your existing release files:
`build.ps1` replaces its same-version me3 ZIP in `dist`.

Install these development dependencies once:

1. [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with **Desktop development with C++**, the MSVC x64 compiler and Windows SDK.
2. Rust through [rustup](https://rust-lang.org/tools/install/), using the MSVC toolchain.
3. Git if cloning the repository; downloading a release source archive also works.

Build a tagged release or the default branch after its checks pass. See
[current validation](validation-0.12.1.md). Open **Developer PowerShell for Visual
Studio** in this source folder, then run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build.ps1
```

The project pins Rust **1.94.0**, rustfmt, Clippy and the Windows x64 target in
`rust-toolchain.toml`. The first build needs network access for Rust and locked
dependencies. Generated timing tables are included in the source; compiling
does not require extracting or copying game archives.

The script checks formatting, runs Clippy and Rust tests, builds the optimized
DLL, verifies that the DLL rejects a non-game host, and creates:

```text
target/x86_64-pc-windows-msvc/release/sekiro_deflect_observer.dll
dist/SekiroDeflectObserver-0.12.1-preview-windows-x64.zip
dist/SekiroDeflectObserver-0.12.1-preview-windows-x64.zip.sha256
```

The ZIP from this command is the **me3 variant**. Follow the me3 instructions
above to run it. The DLL uses the static MSVC runtime. Python is not needed
for this build; the optional analysis and drop-in packaging tools use Python.

## Build with GitHub Actions

The repository's `Windows proof of concept` workflow runs `scripts/build.ps1`.
Its `SekiroDeflectObserver-windows-x64` artifact contains the me3 package and
checksum, with 14-day retention configured in this repository.

Open the workflow run for the source commit you want, wait for successful
completion, and download that artifact. Extract the outer artifact ZIP first
to find the actual mod ZIP. This workflow does not currently build the drop-in
variant and does not publish a permanent GitHub or Nexus release.

## Test the first launch

These are checks to perform, not claims that every condition has passed:

- [ ] Sekiro starts through the chosen loader, with only one observer copy.
- [ ] F9 displays the expected version and explains unsupported-build/read errors.
- [ ] Locking onto a living enemy shows the cue or an F9 diagnostic reason; unlocking hides gameplay guidance.
- [ ] F6/F7 placement, F8/F9 visibility and F10 reset work once per focused fresh key press; settings survive a full restart.
- [ ] F11 practice starts off; enabling it affects only eligible enemy phases. Changing alert hints/HUD mode does not change slowdown; F8 hiding disarms it. Complete the [practice checklist](enemy-speed-practice.md#live-check-still-needed).
- [ ] Malformed config reload preserves the last valid settings and reports the reason.
- [ ] Top HUD bounds stay below the enemy's native posture bar at your resolution/UI scale; record the gap. Check lower-posture mode separately if selected.
- [ ] Movement, camera, menus, controller input and normal attacks remain usable.
- [ ] The overlay survives loading, death, menus and returning to the title screen.
- [ ] Supported attack cues, combos and uncertain responses behave as documented.
- [ ] Closing and relaunching unloads/reloads the expected version.

Record the game, Windows, GPU driver and loader version. Keep a short clip and
use the [session report template](../tests/compatibility/session-template.md).
Exact contact and successful-deflect timing need gameplay evidence.

## Stop and uninstall

Close Sekiro and start normally through Steam without the observer
profile. Remove the extracted observer folder if you no longer need it. Keep
shared me3 files used by other mods.

F8 hides the display and disarms practice; it does not unload the DLL. Game executables and
save files do not need to be replaced for the me3 installation.

## Troubleshoot

| Problem | Action |
| --- | --- |
| No overlay with me3 | Extract the ZIP fully and keep the DLL next to `observer.me3`; inspect launch output and logs. |
| Bar is in the wrong position | In 0.12.1, use F6/F7 or the bounded anchor/offset/safe-area settings in cue.toml. Fixed placement uses a configured posture band, not automatic detection. F10 resets offsets. |
| LOCKED during attacks | The target may be detected while the move has no eligible timing instruction. Use F9, enemy/move details and a clip. Older versions may show UNVERIFIED. |
| Unsupported build / stale or failed reads | Record the F9 reason; the observer suppresses guidance when data is unavailable. |
| `cargo` is not recognized | Install Rust and open a new terminal. |
| Missing `link.exe` / Windows SDK | Install the C++ workload and run from Developer PowerShell. |
| Missing target `core` or `std` | Run `rustup target add x86_64-pc-windows-msvc` from the source root. |
| Dependency download fails | Preserve the error, restore network access and retry the locked build. |
| Formatting, Clippy or tests fail | Fix the reported error before calling the new build verified. |
| `me3` is not recognized | Install it for the me3 variant, reopen the terminal, or double-click the profile. |
| Game crashes after installing | Close it and remove this observer using the instructions above; preserve logs for diagnosis. |

Logs: `%LOCALAPPDATA%\SekiroDeflectObserver`. Share relevant error text, the
observer version, enemy/move and a short clip. A render-submission CSV row is not
proof of a visible frame or successful deflect. Do not upload game executables,
save files or memory dumps with a routine bug report.

## Current source checks

Use the pinned toolchain through rustup. If another Cargo installation appears
earlier in `PATH`, prepend rustup's default bin directory:

```powershell
$env:PATH = "$env:USERPROFILE/.cargo/bin;$env:PATH"
cargo test --locked --offline --target x86_64-pc-windows-msvc
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1
```

Settings live in `%LOCALAPPDATA%/SekiroDeflectObserver/cue.toml`; a copy ships in
the package. All defaults/ranges and reload/reset behavior are in
[configuration](configuration.md). The renderer contains no file I/O. F9 works
without a target; it cannot enable an unlocked gameplay cue. Focus loss hides
gameplay guidance; regained focus still needs fresh observations.

Live acceptance requires 720p/1080p/1440p/4K, ultrawide and letterboxed layouts,
actual UI scaling, windowed/borderless/fullscreen, resize, focus, lock changes,
death, loading, menus, save reload, full restart and removal trials. Synthetic
layout images do not validate those game modes. See [the manual checklist](../tests/manual/gameplay-checklist.md).
