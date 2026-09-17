# Build and run on Windows

The current source candidate is **0.12.1-preview**, with a configured compact HUD
below the enemy's top posture bar, persistent TOML settings and incoming attack
response labels without reach/contact prediction. Optional F11 practice slows
eligible enemy attacks independently of alert preferences and starts off. See
[practice controls and limits](enemy-speed-practice.md). Build it with
`scripts/build.ps1`; that produces the new me3
package. [Current behavior and limits](cue-preview.md), [configuration](configuration.md)
and [check evidence](WORK-STATUS.md) supersede older present-tense cue descriptions.

The 0.6.3 drop-in installation below is retained for users of that **older**
package. It does not gain 0.12.1 behavior without replacing the observer DLL and
fully restarting the process. No new drop-in distribution is claimed here.

## Run a built package

Players need Windows x64 and Sekiro on Steam. No compiler, Rust, Python or
Cheat Engine is needed to run either package. Choose one loading method:

| Package filename | How it starts | Separate loader installation |
| --- | --- | --- |
| `SekiroDeflectObserver-0.6.3-preview-drop-in-windows-x64.zip` | Two files beside `sekiro.exe`; normal Steam launch | None; ASI loader is included |
| `SekiroDeflectObserver-0.12.1-preview-windows-x64.zip` | Double-click the extracted `observer.me3` | Install me3 once |

The beta ZIP is supplied separately by the author. GitHub's **Code → Download ZIP**
downloads source code, not a ready-to-play mod.

### Historical 0.6.3 drop-in installation

1. Extract the **drop-in** ZIP into a temporary folder and read `START-HERE.txt`.
2. Close Sekiro. In Steam, right-click Sekiro → **Manage → Browse local files**.
3. Copy `dinput8.dll` and `sekiro_deflect_observer.asi` into that game folder.
4. Launch through Steam normally, load your save and lock onto an enemy.
5. Look above Wolf for the slider. Press **F9** to check `0.6.3-preview`.

The result should look like this:

```text
Steam/steamapps/common/Sekiro/
  sekiro.exe                     (already installed by Steam)
  dinput8.dll                    (from the drop-in ZIP)
  sekiro_deflect_observer.asi     (from the drop-in ZIP)
```

If `dinput8.dll` already exists, **do not overwrite it**. Another mod may own that
loader. Resolve compatibility first. Keep the complete ZIP and its license
documents when sharing. Do not also launch an observer me3 profile or load
another copy of the observer in the same session.

This startup route passed a standalone ASI loader/DirectInput forwarding check.
A live Sekiro launch through this route and a clean installation on another PC
remain beta checks.

### Current 0.12.1 me3 installation

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

### Current 0.12.1 controls and states

| Control or cue | Meaning |
| --- | --- |
| F6 / F7 | Lower / raise by 8 reference pixels; persist the offset |
| F8 | Show / hide gameplay HUD; persist visibility; hiding disarms practice |
| F9 | Show / hide separate diagnostics and version; persist visibility |
| F10 | Reset horizontal and vertical offsets |
| F11 | Toggle enemy-speed practice for this session; initially off |
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
can appear without a target but cannot bypass gameplay lock gating. The historical
0.6.3 package retains its original overhead controls and appearance.

## Build the package on Windows

Build in a **new checkout** if you want to preserve your existing release files:
`build.ps1` replaces its same-version me3 ZIP in `dist`.

Install these development dependencies once:

1. [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with **Desktop development with C++**, the MSVC x64 compiler and Windows SDK.
2. Rust through [rustup](https://rust-lang.org/tools/install/), using the MSVC toolchain.
3. Git if cloning the repository; downloading the preview branch as source also works.

The 0.12.1 candidate is committed and pushed on `feat/enemy-attack-slowdown`.
Select that branch to build this preview while its PR is open; older branches
and packages retain their earlier behavior. See [the source-state record](WORK-STATUS.md)
and [validated artifacts](validation-0.12.1.md). Open **Developer PowerShell for
Visual Studio** in this source folder, then run:

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

### Run your compiled DLL with the drop-in loader

Obtain the existing friend ZIP for its bundled `dinput8.dll`. In a separate test
folder, keep that loader with your compiled DLL renamed to `.asi`:

```powershell
New-Item -ItemType Directory -Path .\dist\local-drop-in-test -Force
Copy-Item -LiteralPath .\target\x86_64-pc-windows-msvc\release\sekiro_deflect_observer.dll -Destination .\dist\local-drop-in-test\sekiro_deflect_observer.asi
```

Copy `dinput8.dll` from the extracted friend ZIP into that test folder, then
follow the drop-in installation steps using those two files. Close Sekiro before
replacing your earlier observer ASI, and restart after each build. Changing the
extension loads the same compiled code; it does not convert or change the DLL.

This is a developer test folder. If sharing a changed build, give it a new version,
run the checks, and include current checksums and the project/loader license
notices. Do not present it as the original hash-verified 0.6.3 ZIP.

### Reproduce the existing drop-in ZIP

`scripts/package-drop-in.py` is pinned to the **original tested 0.6.3 release**.
It does not package arbitrary newly compiled DLLs. A new clone plus
`build.ps1` alone is not sufficient for that script.

In a separate restore folder, supply:

- The original `SekiroDeflectObserver-0.6.3-preview-windows-x64.zip` in `dist`, matching the hash in the script.
- The preserved `loader-research-cache.zip` from the release backup, extracted into the source root. It restores `dist/drop-in-research` with the pinned loader and license inputs.
- Python 3 for the packaging script, plus the build prerequisites for the native loader check.

Then run from that source root:

```powershell
Expand-Archive -LiteralPath .\dist\SekiroDeflectObserver-0.6.3-preview-windows-x64.zip -DestinationPath .\Mods\SekiroDeflectObserver-0.6.3-preview
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-asi-loader.ps1 -ObserverDll .\Mods\SekiroDeflectObserver-0.6.3-preview\sekiro_deflect_observer.dll -OutputDirectory .\dist\review-0.6.3-restored-loader
python .\scripts\package-drop-in.py
```

The destination Mods folder and drop-in output ZIP should not already exist.
Keep any earlier outputs elsewhere instead of replacing them. The packaging
script checks pinned input hashes, includes licenses and verifies its ZIP entries.
It refuses to overwrite an existing drop-in ZIP or checksum sidecar. See
[the sharing record](sharing-beta.md) for input provenance and published local hashes.

### Check a downloaded package

For the friend ZIP:

```powershell
Get-FileHash -LiteralPath .\SekiroDeflectObserver-0.6.3-preview-drop-in-windows-x64.zip -Algorithm SHA256
Get-Content -LiteralPath .\SekiroDeflectObserver-0.6.3-preview-drop-in-windows-x64.zip.sha256
```

Compare the hash values without regard to letter case. Matching hashes verify
the file bytes, not gameplay accuracy.

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

**Drop-in:** close Sekiro and remove `sekiro_deflect_observer.asi` from the game
folder. Remove the supplied `dinput8.dll` only if no other mod uses it. While
those files remain installed, starting through Steam loads the observer.

**me3:** close Sekiro and start normally through Steam without the observer
profile. Remove the extracted observer folder if you no longer need it. Keep
shared me3 files used by other mods.

F8 hides the display and disarms practice; it does not unload the DLL. Game executables and
save files do not need to be replaced for either installation method.

## Troubleshoot

| Problem | Action |
| --- | --- |
| `dinput8.dll` already exists | Do not overwrite it; identify the existing loader before combining mods. |
| No overlay with drop-in files | Check both runtime files are beside `sekiro.exe`, restart fully, lock on, then try F8/F9. |
| No overlay with me3 | Extract the ZIP fully and keep the DLL next to `observer.me3`; inspect launch output and logs. |
| Bar is in the wrong position | In 0.12.1, use F6/F7 or the bounded anchor/offset/safe-area settings in cue.toml. Fixed placement uses a configured posture band, not automatic detection. F10 resets offsets. |
| LOCKED during attacks | The target may be detected while the move has no eligible timing instruction. Use F9, enemy/move details and a clip. Older versions may show UNVERIFIED. |
| Unsupported build / stale or failed reads | Record the F9 reason; the observer suppresses guidance when data is unavailable. |
| `cargo` is not recognized | Install Rust and open a new terminal. |
| Missing `link.exe` / Windows SDK | Install the C++ workload and run from Developer PowerShell. |
| Missing target `core` or `std` | Run `rustup target add x86_64-pc-windows-msvc` from the source root. |
| Dependency download fails | Preserve the error, restore network access and retry the locked build. |
| Formatting, Clippy or tests fail | Fix the reported error before calling the new build verified. |
| Drop-in packaging reports unexpected bytes | Use the preserved original inputs; it intentionally rejects a different ZIP, DLL or loader. |
| Packaging reports an existing output | Keep the previous package and use a separate restore/output workspace. |
| `me3` is not recognized | Install it for the me3 variant, reopen the terminal, or double-click the profile. |
| Game crashes after installing | Close it and remove this observer using the instructions above; preserve logs for diagnosis. |

Logs: `%LOCALAPPDATA%\SekiroDeflectObserver`. Share relevant error text, the
observer version, enemy/move and a short clip. A render-submission CSV row is not
proof of a visible frame or successful deflect. Do not upload game executables,
save files or memory dumps with a routine bug report.

## Current source checks and isolated loader trial

Select rustup's pinned 1.94.0 Cargo. On the development machine the standalone
1.87 Cargo earlier in PATH bypassed rust-toolchain.toml; use:

```powershell
$env:PATH = 'C:/Users/mpati/.cargo/bin;' + $env:PATH
cargo test --locked --offline --target x86_64-pc-windows-msvc
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-asi-loader.ps1 -OutputDirectory dist/review-0.12.1/asi-smoke
```

The ASI smoke script defaults to the **current release DLL**. The command above
copies it as `.asi` into isolated `dist/review-0.12.1/asi-smoke`. It uses the preserved research loader
at `dist/drop-in-research/loader/dinput8.dll`; if that local input is missing, the
loader check is blocked. It never starts Sekiro or captures input. Optional
`-ObserverDll` and `-OutputDirectory` select the DLL and a workspace-dist output.
A passing non-game smoke check is not live game installation/launch evidence.
The old `package-drop-in.py` remains pinned to 0.6.3 and must not be used to label
a newly compiled 0.12.1 DLL as that tested package.

Settings live in `%LOCALAPPDATA%/SekiroDeflectObserver/cue.toml`; a copy ships in
the package. All defaults/ranges and reload/reset behavior are in
[configuration](configuration.md). The renderer contains no file I/O. F9 works
without a target; it cannot enable an unlocked gameplay cue. Focus loss hides
gameplay guidance; regained focus still needs fresh observations.

Live acceptance requires 720p/1080p/1440p/4K, ultrawide and letterboxed layouts,
actual UI scaling, windowed/borderless/fullscreen, resize, focus, lock changes,
death, loading, menus, save reload, full restart and removal trials. Synthetic
layout images do not validate those game modes. See [the manual checklist](../tests/manual/gameplay-checklist.md).
