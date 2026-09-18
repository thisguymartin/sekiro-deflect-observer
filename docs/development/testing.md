# Run the test suites

Run the complete automated suite on Windows with the pinned toolchain:

```powershell
cargo test --locked --offline --all-targets --target x86_64-pc-windows-msvc
```

Run the full build and package checks with:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build.ps1
```

## What the tests cover

| Test file | Contract |
| --- | --- |
| `tests/incoming-regressions.rs` | Attack classification and alert preferences. |
| `tests/practice-regressions.rs` | Eligibility, ownership, restoration, and preference independence. |
| `tests/timing-regressions.rs` | Legacy occurrence, rate, interval, and pulse decisions. |
| `tests/layout-regressions.rs` | Viewport fitting and complete HUD bounds. |
| `tests/lifecycle-regressions.rs` | Cross-thread invalidation and stale-state rejection. |
| `tests/dx11-render-isolation.rs` | Host DX11 bindings and pixels outside the HUD. |

The DX11 isolation test uses a windowless WARP device. It exercises the vendored
production backend, but it does not load Sekiro or prove visible in-game output.

## Check generated data

Run these commands from the repository root:

```powershell
python scripts/generate-incoming-attacks.py --check
python scripts/test-incoming-attacks.py
python scripts/test-attack-timings.py
python scripts/update-move-coverage.py --check
python scripts/check-doc-links.py
```

The game-data generators may require the local research inputs documented in
[research](../research/README.md). A normal Rust build uses the checked-in output.

## Test in the game

Automated tests cannot establish memory-layout correctness, visible presentation,
contact timing, successful deflects, or cleanup across every game transition.

Use the [gameplay checklist](../../tests/manual/gameplay-checklist.md) and save
results under [`tests/compatibility`](../../tests/compatibility/README.md). Record
the exact artifact, executable hash, Windows version, driver, loader, display
mode, settings, enemy, move, and evidence files.

Use the [release procedure](release.md) for a package that will be shared.
