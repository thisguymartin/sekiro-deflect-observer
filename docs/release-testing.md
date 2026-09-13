# Test a native release candidate

Use this procedure to validate a native release candidate. The Rust DLL includes an experimental incoming-attack timing slider and a separate candidate-effect diagnostic reader. Follow [the Windows instructions](windows.md), [cue scope](cue-preview.md), and [the 0.6 evidence record](validation-0.6.md). Exact contact timing, response accuracy and complete attack coverage remain unvalidated.

Use [the effect investigation guide](reverse-engineering.md) for signal research. Do not advertise a stable release based on the checks below until the evidence exists.

## Prepare the candidate

1. Freeze the source revision, lockfiles, toolchain versions, loader version, build command, and package command.
2. Build from a clean checkout using only documented development dependencies. Preserve the full build and test results.
3. Repeat the build in a second clean environment with the same pinned inputs. Compare the packaged files and ZIP hashes.
4. Investigate differences before claiming byte-for-byte reproducibility. Record reproducible source builds separately from identical binary or archive output.
5. Create an installation manifest listing every project file, destination, required dependency, launch change, and removal action.
6. Record the artifact version and SHA-256. Include the README, install and removal instructions, known limitations, license, and third-party notices.
7. Confirm that the package contains no game executable, extracted game assets, save files, developer credentials, or unlicensed third-party content.

## Test clean Windows installation

1. Use a clean Windows environment with the game installed. Keep the development checkout and toolchains out of the runtime test.
2. Back up the tester's save with the game closed. Record the vanilla launch configuration and game-directory file list.
3. Download the exact candidate package through the intended distribution route. Verify its published SHA-256.
4. Follow the installation README literally, including the selected loader's documented setup. Record every undocumented step as a failure.
5. Launch using the documented user flow. Confirm that installation and play require no compiler, Python, or Cheat Engine.
6. Confirm the detected executable identity and profile before enabling observation. Capture the supported-version display.
7. Run G01 through G21 from the [gameplay checklist](../tests/manual/gameplay-checklist.md). Apply the native requirements to overlay visibility, hotkeys, settings, and lifecycle handling.
8. Repeat the launch after reboot. Confirm that startup does not depend on a development shell or a previously running helper.

Use a physical GPU setup for claims about game rendering and display modes. A virtual-machine smoke test alone does not establish those claims.

## Test failure and recovery

1. Run the synthetic version, pointer, traversal, stale-state, and session-replacement tests in [the automated test requirements](../tests/README.md).
2. Verify that unsupported-version fixtures disable version-sensitive reads and show a clear reason. Do not patch a live executable to manufacture a test case.
3. If a legitimately available unsupported game build can be tested, confirm the same behavior there. Record fixture-only coverage separately.
4. Check missing and invalid configuration, an unavailable render device, and repeated start and stop operations through the implementation's documented test paths.
5. Verify that unknown gaps remain distinct in both the indicator and history. Check that incomplete intervals stay out of complete-window statistics.
6. Inspect runtime logs for crashes, stale callbacks, repeated hook registration, and unbounded resource growth during a long session.

## Test removal and upgrade

1. Close the game and remove only project-owned files identified by the installation manifest.
2. Restore project-specific launch changes. Preserve shared loader files and settings required by other mods.
3. Start Sekiro through its normal launch path. Confirm that the observer is absent, the save loads, and normal controls work.
4. Compare the remaining game-directory files with the baseline. Explain every unexpected change. Save changes from normal gameplay are not evidence of observer writes by themselves.
5. If upgrading from an earlier published artifact is supported, repeat installation over that version and verify settings behavior. Otherwise test the documented remove-then-install path.

## Check offline behavior and package contents

1. Review network-capable dependencies and code paths in the observer and any bundled components.
2. Observe connection attempts during launch, idle, gameplay, settings changes, and shutdown. Record the tool, process attribution, and capture duration.
3. Repeat with network access blocked. Confirm that the observer remains usable.
4. Separate Steam, game, loader, and observer traffic where attribution permits. If a native mod shares the game process, a process-level capture alone cannot prove which component made a request.
5. Record a scan result for the exact release artifact, including scanner version, date, and hash. Explain unresolved findings before release.

One traffic capture or clean scan does not prove universal absence of network behavior or malware. State what was reviewed and exercised. The product requirement remains zero observer network requests.

## Test Proton separately

1. Verify current installation instructions for the selected loader before the first Linux trial. Record the exact instructions used.
2. Use a separate test prefix or backed-up test environment. Preserve existing user prefixes and saves.
3. Record the Linux, Steam, Proton, loader, GPU-driver, display-session, and launch-configuration details in the [session template](../tests/compatibility/session-template.md).
4. Run clean installation, G01 through G21, and removal under a named standard Proton build.
5. Repeat with a recorded Proton Experimental build. Keep the results separate because that channel changes.
6. If hardware is available, repeat in Steam Deck Desktop Mode. Test Game Mode as a separate configuration before claiming it works.
7. Publish only observed results and limitations in the [compatibility table](compatibility.md). Keep Proton experimental until its release criteria have been defined and met.

## Review the release evidence

Leave each item unchecked until a reviewer can follow its evidence link.

- [ ] Effect semantics are documented with recordings, uncertainties, and contradictions.
- [ ] Automatic build detection and safe read failure are tested.
- [ ] Unit tests, clean builds, and packaging checks pass for the frozen revision.
- [ ] Every advertised platform and display mode has an applicable compatibility report.
- [ ] Clean installation and removal work without development tools.
- [ ] Input, toggle, settings, loading, death, restart, and long-session checks pass.
- [ ] Offline behavior, package scan results, and third-party licenses are reviewed.
- [ ] Screenshots and a short demonstration show the exact release candidate working.
- [ ] The package includes its version, checksum, README, credits, and limitations.
- [ ] Release text distinguishes the player-effect research panel from the overhead parry cue. Cue claims name the attacks and conditions actually validated using [the cue requirements](parry-cue.md).
- [ ] Current GitHub and Nexus Mods submission requirements have been checked at publication time, including applicable AI disclosure rules.
- [ ] Any unresolved limitation is visible in the release notes and compatibility table.

This checklist is a project release gate, not a summary of current platform policies. Publishing a pre-release does not make an untested package a validated mod.
