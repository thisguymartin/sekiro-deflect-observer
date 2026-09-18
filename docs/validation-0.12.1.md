# 0.12.1-preview validation

The 0.12.1 change separates attack facts, alert preferences, and practice
eligibility. It does not change attack mappings or speed offsets. F11 still arms
practice for the current process. F8 still hides the HUD and disarms practice.

## Automated checks

- The previously recorded all-target Windows MSVC run passed 131 tests. The two
  new catalog-selection regressions pass locally and raise the expected total to
  133 on the next Windows run.
- The practice regression covers all 32 alert-preference and display-mode
  combinations for parryable, thrust, and sweep attacks. Preference changes do
  not restore, stack, or replace an active speed lease.
- Raw classification keeps thrust identity when the HUD falls back to PARRY.
- Clippy with `-D warnings`, formatting, and the release build passed. Vendored
  hudhook retains two existing unused-method warnings.
- The isolated ASI loader check passed for DirectInput forwarding, adjacent DLL
  loading, and non-Sekiro host rejection.
- Shared ImGui geometry passed bounds checks for the practice gallery and the
  disabled-hints render at 1080p scales 1.0 and 0.5.
- Packaged payloads matched the extracted files and checksums.

Run the source checks from Developer PowerShell:

```powershell
cargo test --locked --offline --all-targets --target x86_64-pc-windows-msvc
cargo clippy --locked --offline --all-targets --target x86_64-pc-windows-msvc -- -D warnings
cargo fmt --all -- --check
cargo build --release --locked --offline --target x86_64-pc-windows-msvc
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-asi-loader.ps1 -OutputDirectory dist/review-0.12.1/asi-smoke
```

## Limits

Sekiro was not running during these checks. The checks cover source behavior,
synthetic rendering, packaging, and host rejection. They do not prove live HUD
placement, enemy slowdown, Wolf's unchanged speed, defensive reactions, or
cleanup after game lifecycle changes.

See [practice limits](enemy-speed-practice.md), [feature ownership](feature-boundaries.md),
and the [gameplay checklist](../tests/manual/gameplay-checklist.md).
