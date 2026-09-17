# 0.9.1-preview rendering-isolation validation

## Report and evidence

The user reported an overlay/tint taking over the game and supplied
`Sekiro 2026-09-17 10-33-30.mp4` (79.15 s, 2560x720, Game DVR).
Clip SHA256: `f7d1e46709ecc8cd7caf75a5881784bf90ef200c8b5722cd5af926dd861e124e`.
Sampled frames are in `dist/review-0.9.1/clip`. The frames show the compact
LOCKED/PARRY cue above the native lower posture bar. The scene is visually dim;
there is no same-scene unmodified reference establishing the source of the tint.
The current process 23300 log identifies the 0.9.0 DLL
`81a4a94dbce3b0c6d5c85d0dc6d7c260e8144468e83b55cd92db20cbb36ae768`.

Source inspection found hudhook 0.9.2's DX11 backup omitted output targets,
skipped restoration of several NULL bindings, and did not account for resources
implicitly unbound by binding a render target. The new offscreen WARP regression
failed against the original backend with `host render target leaked` and passed
with the patched production backend. This confirms a renderer bug, not that it
is the sole cause of the user's particular recorded tint.

## Change

The vendored pinned backend uses a private D3D11 deferred context. The command
list is executed with full host-state restoration, as specified by Microsoft's
[ExecuteCommandList contract](https://learn.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11devicecontext-executecommandlist).
Empty frames are skipped. Partial draw commands are discarded on draw failure.
Private bindings are cleared and command lists released before the next frame.
No game/HDR/gamma settings or cue geometry/attack classification changed.

## Automated checks

- 107 Rust tests passed: 71 unit, 1 actual DX11 backend, 9 incoming, 6 layout,
  2 lifecycle and 18 legacy timing.
- The DX11 check uses WARP without a window, game process or injection. Both
  UNORM and sRGB targets preserve host output, PS/VS aliases, NULL texture,
  sampler, projection and input-layout bindings, and viewport values. It checks
  repeated frames, an empty frame, visible drawing, and unchanged pixels outside
  the cue region.
- `cargo fmt --all -- --check` and Clippy with `-D warnings` passed for the app.
  The local dependency exposes two existing unused `wait_idle` warnings.
- Offline MSVC release build, native DLL host-rejection check, and isolated
  DirectInput/ASI forwarding-and-load check passed. The game was not launched.
- Every packaged/staged content hash and staged/release DLL equality verified.

## Artifacts

- Launcher: `Mods/SekiroDeflectObserver-0.9.1-preview/launch-observer.cmd`.
- Archive: `dist/SekiroDeflectObserver-0.9.1-preview-windows-x64.zip`.
- DLL SHA256: `0b5c381db5dc54727dd6e54942e304acc77c78c61b34231a056475db052c9f27`.
- ZIP SHA256: `1036cf072c1f822a3218b6beab19293287c64e4b1cd2bc922acb4883c22f4204`.
- Per-file checks: `dist/review-0.9.1/artifact-verification.json`.

Prior release folders remain available. No user configuration, graphics settings,
installation, or saves were modified. No commit or push was made.

## Remaining live validation

Fully exit Sekiro and launch the staged 0.9.1 profile. Compare the same scene
with a normal Steam launch, and verify lock, cue colors, posture gap, fullscreen
transitions and performance. F8 only hides drawings and does not unload hooks.
The previously running process was not stopped, injected into or replaced.
No visual cure or gameplay validation is claimed until this comparison occurs.
