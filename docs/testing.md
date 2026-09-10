# Test the research prototype

Use this guide to collect evidence about the candidate player effect `105010` on Sekiro PC 1.06. Start with the external Cheat Engine window described in the handover.

For the native Rust proof of concept included in this repository, use [the Windows build and run instructions](windows.md). It does not depend on the original prototype. This document covers the separate, older Cheat Engine research path.

**Current blocker:** the prototype files and their operating instructions are missing from this checkout. You can prepare a report now. Activation and gameplay testing require the [prototype import](../prototypes/cheat-engine/README.md).

## Prepare a session

1. Obtain the original prototype from its author. Record its origin and revision.
2. Have a developer inspect its Lua, table entries, dependencies, and startup actions. Confirm that the observer does not write gameplay memory, patch instructions, automate inputs, or make network requests.
3. Fill in the prototype's activation and stop instructions. Confirm how its timers and window are cleaned up. A window closing is not proof that observation stopped.
4. Use a Windows machine with a legitimately installed copy of Sekiro. Record the OS build, GPU, driver, display mode, and input device.
5. Close the game and back up your own save before testing death, transitions, or installation changes. Keep the backup outside the game and mod directories.
6. Disable other gameplay mods, trainers, and timing changes for the baseline. Record any remaining loader or overlay.
7. Copy the [session report template](../tests/compatibility/session-template.md) to a report named with the UTC date, platform, and revision. Keep it under `tests/compatibility/` if you plan to share it.
8. Record every installed test file and its destination. This list becomes the removal checklist.

Keep save files, game binaries, raw memory dumps, and private desktop content out of shared reports. Share measurements and relevant clips instead.

## Identify the executable and prototype

1. Record the version shown by Sekiro and the executable's version metadata, if available in Windows file properties. Record missing metadata as missing.
2. In PowerShell, run these commands. Enter the full paths without surrounding quotes when prompted.

```powershell
$sekiroTestExe = Read-Host 'Full path to sekiro.exe'
Get-FileHash -LiteralPath $sekiroTestExe -Algorithm SHA256 -ErrorAction Stop
$observerTestFile = Read-Host 'Full path to the prototype table'
Get-FileHash -LiteralPath $observerTestFile -Algorithm SHA256 -ErrorAction Stop
```

3. Copy each hash into the report. Repeat the file-hash command for companion Lua files that affect behavior.
4. Compare the executable identity with the prototype's documented research build and offset provenance. If compatibility is unknown, stop before enabling version-sensitive reads. Mark the session `Blocked` and request build identification work.

These commands read the files and report their hashes. Their syntax follows the current [Microsoft Get-FileHash documentation](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.utility/get-filehash). They have not been executed against a Windows game installation in this checkout.

A hash identifies the bytes tested. It does not prove compatibility. A displayed `1.06` alone is not an offset allowlist. Record manual identification separately from automatic version detection. The latter remains a release requirement.

## Start and capture a baseline

1. Launch Sekiro without the observer. Pick a safe location with no enemy contact.
2. Set up a recording that captures gameplay, the observer window's intended location, and an independent view of the input. A camera on the controller is one option.
3. Prefer 60 FPS or higher. Record the actual capture rate, game frame rate, dropped frames, and any capture delay you know about.
4. Record 30 seconds of idle gameplay, five isolated guard taps, and one five-second guard hold with the observer off.
5. Return to the same location and settings for the observer-on run.

For a first research session, a windowed or borderless layout may make the external window easier to capture. Record the mode. An external window visible beside the game does not establish native fullscreen overlay support.

## Attach and start the observer

1. Start the reviewed Cheat Engine version and select the running `sekiro.exe` in its process selector. Record the selected process ID.
2. Open the reviewed table. Keep Lua execution prompts enabled. Execute only the script whose startup behavior was inspected.
3. Use the exact activation action recorded in the [prototype instructions](../prototypes/cheat-engine/README.md). Do not enable unrelated table entries.
4. Confirm that the timing window opens and samples update. Record its actual labels, colors, polling interval, and error output.
5. Keep the display visible during 30 seconds of idle gameplay. If it reports unavailable data, resolve that before interpreting any inactive state.

Cheat Engine documents its table script under **Table > Show Cheat Table Lua Script** and its Lua Engine under **Memory View > Tools > Lua Engine**. Its default table-load prompt depends on settings. Use these locations for inspected script access and error output, following the [official Lua guide](https://wiki.cheatengine.org/index.php?title=Lua_Basics).

Do not assume that green means a validated deflect window. During research, describe it as the displayed active state or observed candidate effect.

## Run the gameplay checks

1. Run G01 through G04 in the [gameplay checklist](../tests/manual/gameplay-checklist.md) first. They cover idle, isolated taps, held guard, and repeated presses.
2. Save the clips and fill in the report before starting combat. Investigate contradictions using [the effect analysis procedure](reverse-engineering.md).
3. Run G05 through G08 with the same enemy and repeatable attack where possible. Judge combat outcomes from the game, independently of the observer.
4. Run G09 through G16 to test transitions, restart, and shutdown. Save evidence before restarting either process.
5. Run G17 through G21 for input, display, performance, and removal checks that apply to the prototype.
6. Record every result as `Pass`, `Fail`, `Blocked`, `Not run`, or `Not applicable`. Include a reason for the last three.

The repeat counts in the checklist are starting points for finding contradictions. They are not a statistical guarantee. Keep failed and ambiguous trials in the report.

## Stop and record the result

1. Use the documented stop action. Check that samples and timers stop and that normal controls still work.
2. Close Cheat Engine and relaunch the game normally for the removal check.
3. Complete the case results, evidence references, and unresolved questions in the report.
4. Separate detector correctness, effect semantics, and display behavior in your conclusion. Passing one does not establish the others.
5. Add a result link to the [compatibility records](../tests/compatibility/README.md). Keep unsupported platform claims out of the README.

If the game crashes, freezes, or the display stays active after reads stop, end that trial. Preserve the exact error and last valid observation. Do not repair the symptom by mapping failed reads to `Inactive` or editing offsets until the display looks plausible.

## Resolve common testing problems

| Symptom | Next action |
|---|---|
| No prototype or activation instructions | Complete the prototype import before running gameplay checks. |
| Game process cannot be selected | Confirm the game is running and record the exact error. Do not guess a process or change privileges without diagnosing the failure. |
| Executable identity is not documented | Stop version-sensitive reads and record `Blocked`. |
| Window is always unavailable | Capture read errors and the last successful reader stage, if exposed. Do not convert unavailable to inactive. |
| Window is always active or inactive | Recheck process identity, player ownership, and read validity with a developer. Use the isolated-tap test before combat. |
| Window disappears in fullscreen | Record a display failure for that mode. Repeat in a visible mode to collect state evidence separately. |
| No logs or sample timestamps | Use video for coarse observations. Mark timing accuracy and read-path diagnosis as blocked where video cannot answer them. |
| The stop action is unknown | Close Cheat Engine, end the run, and document the missing cleanup instructions before repeating activation. |
