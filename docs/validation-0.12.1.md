# 0.12.1-preview validation — 2026-09-17

Follow-up to `f189efe` on `feat/enemy-attack-slowdown`: attack facts, alert
preferences and practice eligibility are separate code paths. No attack mappings
or speed offsets were changed. F8 master shutdown and F11 arming are preserved.

## Completed

- All-target MSVC tests: **131 passed** (74 unit, 1 native DX11 isolation,
  14 incoming, 6 layout, 2 lifecycle, 16 practice, 18 legacy timing).
- New independence regression: all 32 combinations of parry/dodge/jump/Mikiri
  hints and incoming/legacy mode for each of parryable, thrust and sweep attacks
  (96 cases). The same speed override stays applied once, without stacking or
  restoration when preferences change. Explicit disarming restores normal speed.
- Raw classification keeps thrust identity despite the HUD's PARRY fallback;
  combo order and invalid/ended animation checks pass. Unknown/grab/unparryable
  exclusions now use real classified animation samples instead of edited HUD
  decisions.
- Clippy with `-D warnings`, formatting and release build pass. The vendored
  hudhook dependency retains its two existing unused `wait_idle` warnings.
- Isolated DirectInput/ASI loader smoke passes: forwarding, adjacent DLL loading
  and rejection of a non-Sekiro host before hooking. This does not exercise a
  live speed write.
- Shared ImGui mesh passes bounds checks for the practice gallery and disabled
  hints at 1080p scales 1.0/0.5. The disabled-hint raster was inspected:
  `PRACTICE 80%` appears without the hidden attack label or attack-colored rail.
- All eight packaged payload files match ZIP entries, extracted files and
  checksums. Release/staged DLLs agree; the previous 0.12.0 DLL is unchanged.

Commands used:

```powershell
cargo test --locked --offline --all-targets --target x86_64-pc-windows-msvc
cargo clippy --locked --offline --all-targets --target x86_64-pc-windows-msvc -- -D warnings
cargo fmt --all -- --check
cargo build --release --locked --offline --target x86_64-pc-windows-msvc
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-asi-loader.ps1 -OutputDirectory dist/review-0.12.1/asi-smoke
```

## Artifacts

- DLL SHA-256: `b6eaab4c4f0ceb5028a5f576bf9762d024f8bea234344bec2002fea464c6fdda`
- ZIP SHA-256: `0cf274595a7b76338e7f07b03efc04617d4cd97ae90974567363e9950f360dff`
- ZIP: `dist/SekiroDeflectObserver-0.12.1-preview-windows-x64.zip`
- Launcher: `Mods/SekiroDeflectObserver-0.12.1-preview/launch-observer.cmd`
- Verification: `dist/review-0.12.1/artifact-verification.json`
- Disabled-hint raster: `dist/review-0.12.1/layout/no-hints/cue-layout.png`

Sekiro was not running during verification. These tests prove the checked code
paths and synthetic behavior, not live slowdown, reactions or cleanup. Refer to
[feature boundaries](feature-boundaries.md) for future edits and
[practice limitations](enemy-speed-practice.md) for the remaining gameplay trial.
