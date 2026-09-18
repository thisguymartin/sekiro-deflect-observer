# Compatibility status

The [September 18 recording](screenshots.md) shows the overlay running during
Sekiro combat, including the PARRY rail and 70% practice states. It does not
identify the complete OS, driver, loader, display mode or loaded artifact.
No completed structured compatibility report is checked in.

| Platform | Current status | Evidence |
| --- | --- | --- |
| Windows x64 | Development target; live HUD demonstration, full acceptance open | [Gameplay walkthrough](screenshots.md), [recorded build checks](validation-0.12.4.md) |
| Windows 11 / Windows 10 individually | Exact OS/version coverage unrecorded | The video does not identify the OS build |
| Steam on Linux with standard Proton | Untested | No recorded trial |
| Proton Experimental | Untested | No recorded trial |
| Steam Deck Desktop Mode | Untested | No recorded trial |
| Steam Deck Game Mode | Unsupported until tested | No recorded trial |

The research target is Sekiro PC 1.06. The native observer includes an exact
executable SHA-256 gate:

`637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856`

The animation hook additionally checks the researched function bytes.
A version label alone does not certify every executable with that label.
See [architecture](architecture.md) and [reader evidence](reader-research.md).

## Evidence required for a status change

A result applies to an exact artifact, game executable, OS, driver, display
mode and loader setup. Preserve that combination in a
[compatibility report](../tests/compatibility/README.md).

A supported release requires repeatable state detection, lifecycle handling,
clean installation and clean removal on the claimed setup. Follow the
[native release procedure](release-testing.md).

Builds, synthetic tests and loader support statements do not establish gameplay
compatibility. The new video demonstrates visible behavior in one encounter;
it does not establish cross-platform support, measured slowdown or defensive
timing. A Windows result cannot establish Proton compatibility, and Steam Deck
Desktop Mode does not establish Game Mode compatibility.
