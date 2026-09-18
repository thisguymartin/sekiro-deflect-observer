# 0.12.4-preview validation

Shift+F11 now cycles 80% → 90% → 70% → 80%, saving the selected speed.
F11 remains on/off. The existing badge shows OFF/ON and the selected percentage;
the 70% crescent is fuller than 80% and 90%.

![Enlarged synthetic preset preview](../../images/0.12.4-practice-moon.png)

All 136 Windows MSVC tests pass. The existing persistence regression now checks
90%, 70% and the return to 80%; the speed-controller regression switches through
all three presets against a 1.25 baseline and verifies exact restoration without
stacking. Selecting any preset while disabled performs no new speed write.
Formatting, Clippy, the Windows release build and the isolated ASI loader smoke
check passed. All extracted files in `Mods/SekiroDeflectObserver-0.12.4-preview`
matched the packaged checksums. Vendored hudhook retains two existing warnings.

The 70% badge uses the same renderer and percentage source as the existing
presets. Synthetic geometry checks cover OFF, armed, applied and attention states,
720p and ultrawide sizes, and hidden/reduced-flash modes. These are the previously
recorded build checks; the documentation refresh did not rebuild the DLL.

```powershell
cargo run --locked --offline --example cue-layout --target x86_64-pc-windows-msvc -- dist/review-0.12.4/layout/armed-0.7 --indicator-only --indicator armed --speed 0.7
python scripts/render-cue-layout.py dist/review-0.12.4/layout/armed-0.7
```

## September 18 gameplay recording

The author's `Sekiro 2026-09-18 15-29-02.mp4` now supplies the README's main
image and short gameplay loop. The [walkthrough](../../gameplay/walkthrough.md) records source
hash, extraction settings and full-size frames:

| Time | Visible result |
| --- | --- |
| 00:38.500 | Green PARRY 70% rail, approaching diamond, jade ON 70% moon. |
| 00:39.500 | White rail and red strike emblem during the active parryable phase. |
| 00:40.000 | Neutral PRACTICE caption and gold ON 70% moon. |
| 00:47.000 | PARRY without an applied percentage; amber moon with attention mark. |

This establishes live visual behavior in the recorded encounter. The 70% badge
is consistent with this preview's design; F9, startup hashes and matching
practice logs are not supplied in the clip, so the exact loaded artifact and
the amber state's cause are unconfirmed.

Still open: measured off/on animation-rate comparisons; Wolf's unchanged rate;
all three presets and hotkey persistence; restoration after focus/lock loss,
target changes, death and reload; Mikiri/deflect reactions; other enemies and
display setups. The video is not an input/contact timing trial or a controlled
comparison for the earlier scene-tint report. Follow the
[practice checklist](../../research/practice-evidence.md#live-checks-still-required).
