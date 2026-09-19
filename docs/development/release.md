# Test a release candidate

Use this procedure before sharing a native package. Automated tests are required,
but they do not prove in-game behavior.

## Prepare the artifact

1. Freeze the source revision, lockfiles, Rust toolchain, loader version, and
   package version.
2. Build from a clean checkout with `scripts/build.ps1`.
3. Preserve the build and test output.
4. Record the package and DLL SHA-256 values.
5. Confirm that the ZIP contains the DLL, me3 profile, launcher, README, license,
   notices, and checksum.
6. Confirm that the ZIP contains no game executable, extracted game assets,
   save files, credentials, or private evidence.

Repeat the build in a second clean environment when you claim reproducible
output. Investigate differences before claiming byte-for-byte equality.

## Test a clean installation

1. Use a Windows system with no development checkout in the runtime path.
2. Record the Windows version, GPU, driver, display mode, Sekiro executable hash,
   loader version, other mods, and baseline game-directory files.
3. Download the candidate through the intended distribution path and verify its
   checksum.
4. Follow the packaged instructions without using undocumented steps.
5. Confirm the loaded observer version and DLL hash in F9 and the startup log.
6. Run the [gameplay checklist](../../tests/manual/gameplay-checklist.md).
7. Save a [compatibility report](../../tests/compatibility/README.md).
8. Restart Windows and repeat the documented launch.

Use a physical GPU for claims about in-game rendering and display modes. A
windowless test or virtual machine does not establish those claims.

## Test failure and recovery

Check these cases:

- Unsupported executable identity.
- Missing, malformed, and externally changed configuration.
- Lock loss, target switch, focus loss, F8 hide, death, loading, rest, and quit
  to title.
- Practice disable and restoration during an eligible phase.
- Repeated start and stop operations.
- Long-session log limits and queue drops.

Unknown or stale data must remove actionable guidance. A logging failure must
not enable a cue or practice write.

## Test removal and upgrade

1. Close Sekiro.
2. Remove only project-owned files from the extracted package.
3. Preserve shared loader files used by other mods.
4. Start Sekiro normally and confirm that the observer is absent.
5. Compare the game directory with the baseline.
6. Test the documented remove-then-install path for an upgrade.

## Check offline behavior and package contents

Review network-capable code and dependencies. Observe connection attempts during
launch, gameplay, settings changes, and shutdown. Repeat with network access
blocked. Attribute Steam, Sekiro, loader, and observer traffic separately where
the tooling permits it.

Record a malware scan for the exact artifact, including the scanner, date, and
hash. One clean scan does not prove universal absence of malware.

## Record acceptance evidence

Use the [cue trial template](../../tests/compatibility/cue-trial-template.md) for
attack timing and response claims. Keep these timestamps and outcomes separate:

- Animation capture.
- HUD decision.
- Draw submission.
- Visible presentation.
- Manual input.
- Contact.
- Confirmed gameplay outcome.

Effect 105010 is not a confirmed deflect. A draw row is not a visible frame.
A checked practice write is not a measured animation rate.

Update [compatibility status](../users/compatibility.md) only after a structured
report supports the claim. Preserve versioned results in
[the validation archive](../archive/README.md).
