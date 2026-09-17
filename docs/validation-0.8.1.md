# 0.8.1-preview correction - 2026-09-17

This completes the partial local 0.8.1 work found in the checkout. The user
reported an empty/inactive 0.8.0 bar, overlap with Wolf's posture bar and missing
lock feedback, supplied `Sekiro 2026-09-16 11-34-12.mp4` as the older reference,
and requested placement a little above Wolf's posture bar.

## Observed evidence

- The supplied reference is 39.25 seconds, 2560x720. Its extracted 16-second
  frame shows the older dark overhead lane with a readable LOCKED label.
  This establishes appearance, not successful deflect timing or an exact DLL.
- The previously saved `22-14-54` recording review contains a frame with the
  0.8.0 lane overlapping Wolf's posture decoration. Its saved startup log
  identifies 0.8.0 DLL SHA256
  `60756a07633b232c36c1ea33aa89006ff42343cf3efb3f20e798e0b4924d39a8`.
- Saved process-14040 render data has 33,943 submissions. Of 11,248 submissions
  with a target handle, 7,244 report `outside_coarse_reach`, and 1,386 are hidden
  with `missing_or_stale_observation`. These are draw decisions, not frames
  independently confirmed on screen or successful defensive actions.
- The previously identified video time window has 4,658 submissions: 1,797
  hidden, 2,660 neutral, 141 preparation, 48 actionable and 12 expired. It is
  aligned by filename wall clock, not frame synchronization. Thus “never shows
  anything” is not literally true of submissions; feedback was mostly neutral
  or hidden and visibly overlapped the native HUD.
- The latest startup log inspected in the user's local log folder also reports
  0.8.0. A previously built 0.8.1 ZIP existed, but no 0.8.1 Mods folder existed.
  No running Sekiro process was found during this session.

## Correction

The default fixed anchor moves from viewport y=0.87 to 0.81. The reserved posture
band moves from 0.90 to 0.85, the lane becomes 320x14 reference pixels and the
label becomes 30 reference pixels. Dark backing improves contrast. At the
recorded 5120x1440 surface, downscaled to the 2560x720 video, the complete cue
ends near y=594, roughly 24 pixels above the decoration near y=618. This is a
configured fallback based on the recording; it does not detect the posture bar.

READY can warn during a mapped wind-up before coarse reach passes. PARRY NOW,
DODGE and JUMP still require valid timing and reach. WATCH denotes an unverified
mapped phase without a press interval or pulse. No classifications or estimate
widths were expanded.

The completion work separates fresh lock identity from animation freshness.
`LiveCue::current_lock` expires 50 ms after the validated lock read;
`LiveCue::current` additionally checks original animation capture age. Animation
failure/staleness cancels timing but retains neutral LOCKED while the lock is
fresh. Losing lock, full clear, focus/visibility and generation checks still
hide the cue. After failure, action requires fresh stable progression again.
Repeated polling cannot revive an old attack window.

Untouched 0.8.0 layout defaults migrate with an atomic, conflict-checked save.
Customized tuples and unrelated settings remain intact. The user's actual
configuration file was read for diagnosis and was not edited by this session;
its untouched layout tuple is eligible for migration when 0.8.1 loads.

## Verification and artifact identity

Commands used pinned Cargo from `C:/Users/mpati/.cargo/bin`, locked offline
dependencies and target `x86_64-pc-windows-msvc`. Fresh evidence is under
`dist/review-0.8.1-completion/`; preceding partial work remains in
`dist/review-0.8.1/`.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo test --locked --offline --all-targets --target x86_64-pc-windows-msvc` | 97 passed: 71 library, 6 layout, 2 lifecycle, 18 timing |
| `cargo clippy --locked --offline --all-targets --target x86_64-pc-windows-msvc -- -D warnings` | Passed |
| `python scripts/test-attack-timings.py` | 11 passed |
| `python scripts/update-move-coverage.py --check` | Current: 2,161 phases, unchanged response counts |
| `cargo build --release --locked --offline --target x86_64-pc-windows-msvc` | Passed |
| `scripts/test-dll-load.ps1`, with workspace-local log directory | DLL loads; rejects non-game host |
| `scripts/test-asi-loader.ps1 -OutputDirectory dist/review-0.8.1-completion/asi-smoke` | Forwarding, adjacent ASI loading and host rejection passed |
| Shared-renderer 1080p seven-state gallery | Vertex containment passed; image visually inspected |

New regression coverage exercises an animation error after an actionable state,
stale animation versus fresh lock, 50 ms lock expiry, clear/loss, no retained
press/contact/pulse, and three-capture recovery. Existing 0.8.1 tests cover
recorded posture clearance, mapped out-of-reach preparation, unverified thrust
attention, response gates and one-time configuration migration.

Final release DLL SHA256:
`70a2bbbb973edfce9492732b518f8e89bac3605a1590a063342fceefa4957744`.

Final ZIP SHA256:
`cdc3e3186605e1a8f29c4f7fec465092c292e8f33b3de4537d3408a4e29d338a`.

Package: `dist/SekiroDeflectObserver-0.8.1-preview-windows-x64.zip`.
Staged profile: `Mods/SekiroDeflectObserver-0.8.1-preview/observer.me3`.
The earlier partial 0.8.1 ZIP and sidecar were preserved in
`dist/review-0.8.1-completion/previous-candidate/`. Older Mods folders remain.

## Remaining live check

Fully close Sekiro, launch the staged 0.8.1 profile and confirm version/hash in
F9/startup logs. Check that LOCKED appears immediately, the lane clears the
posture decoration, READY moves during a mapped wind-up and unlocking hides it.
Try a normal sword soldier before the Ogre and selected boss attacks. F7 raises
the cue another eight reference pixels per press if desired.

No new gameplay session, successful deflect or contact calibration was performed.
The recorded-frame placement comparison and renderer gallery are synthetic;
they do not validate presentation latency or every resolution/HUD modification.
