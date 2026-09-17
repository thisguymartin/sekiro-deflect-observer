# 0.12.0-preview validation — 2026-09-17

The 0.11.0 baseline was committed as `4c0df9a` and pushed to
`origin/codex/defensive-cue-posture-hud` before this work. Optional enemy-speed
practice is isolated on `feat/enemy-attack-slowdown`.

## Completed

- `cargo test --locked --offline --all-targets --target x86_64-pc-windows-msvc`:
  **129 passed** (74 unit, 1 native DX11 isolation, 13 incoming, 6 layout,
  2 lifecycle, 15 practice, 18 legacy timing).
- `cargo clippy --locked --offline --all-targets --target x86_64-pc-windows-msvc
  -- -D warnings`: passed. The vendored hudhook dependency still emits its two
  pre-existing unused `wait_idle` warnings; the application has no Clippy errors.
- Formatting and MSVC release build: passed using the pinned Rust toolchain.
- DLL startup smoke: non-Sekiro PowerShell host rejected before installing hooks.
- DirectInput/ASI smoke: forwarding, adjacent ASI loading and host rejection pass
  in the isolated `dist/review-0.12.0/asi-smoke` fixture.
- Actual shared ImGui draw mesh: practice gallery, 1080p at scale 0.5,
  1440p at scale 1.5, and 540p Mikiri at scale 1.0 remain inside declared bounds.
  Gallery raster visually inspected: eligible labels append `80%`; excluded
  grabs, no-parry and unknown labels do not. At 360p with scale 0.5, layout
  suppresses the undersized cue; that run produces no geometry, not a pass for
  visible rendering at that unsupported combination.
- Eight packaged files match their checksums, ZIP entries and extracted copies.
  The packaged DLL matches the release build. The 0.11.0 staged DLL remains
  unchanged at `75d5a8646f604213962ffc52e5fabed61cbd903a2a6410dcdd5255551cf37a24`.

The practice tests cover disabled/unsupported zero reads and writes; original
speed restoration; no repeated multiplier stacking; death/context loss; exact
target switching order; owner replacement; Wolf/behavior alias rejection;
invalid floats/rates; failed cleanup retry; external writer conflicts;
unsupported/stale attack exclusion; and measured-clock HUD progression without
fabricating an input window. These use synthetic memory and clocks.

## Artifacts

- DLL SHA-256: `933c51c982a1bf48ef37097596d1f0b06353f55115375d042a3763ba166befb1`
- ZIP SHA-256: `f2a52be70a1947dc8483c0f5a5296964afc167760f0123cdb26e9349722a59d1`
- ZIP: `dist/SekiroDeflectObserver-0.12.0-preview-windows-x64.zip`
- Launcher: `Mods/SekiroDeflectObserver-0.12.0-preview/launch-observer.cmd`
- File verification: `dist/review-0.12.0/artifact-verification.json`
- Synthetic gallery: `dist/review-0.12.0/layout/practice/cue-layout.png`

## Not verified

Sekiro was closed during this implementation. No live memory write, measured
enemy slowdown, unchanged Wolf animation rate, paired Mikiri reaction, boss
combo, grab transition, projectile behavior or live cleanup result is claimed.
The native smoke tests exercise loading/host rejection, not game-speed writes.
Per-enemy animation speed uses an independently documented field; the worker's
rechecks are not an atomic engine transaction. See the
[live check procedure and limitations](enemy-speed-practice.md).
