# Compatibility status

No gameplay compatibility has been validated in this checkout. These statuses describe available evidence, not theoretical loader support.

| Platform | Current status | Evidence |
|---|---|---|
| Windows 11 | Untested; initial native target | None |
| Windows 10 | Untested | None |
| Steam on Linux with a standard Proton release | Untested; planned experimental target | None |
| Proton Experimental | Untested; planned experimental target | None |
| Steam Deck Desktop Mode | Untested; planned experimental target | None |
| Steam Deck Game Mode | Unsupported until tested | None |

The research target is Sekiro PC 1.06. No executable hash allowlist or verified memory profile is included yet. A version label does not certify every executable with that label.

## Evidence required for a status change

A result applies to an exact artifact, game executable, OS, driver, display mode, and loader setup. Preserve that combination in a [compatibility report](../tests/compatibility/README.md).

An experimental result requires an actual run, known limitations, and accessible evidence. A supported release requires repeatable state detection, lifecycle handling, clean installation, and clean removal on the claimed setup. The [native release procedure](release-testing.md) defines the checks.

A successful build, synthetic test, or loader support statement cannot establish gameplay compatibility. A Windows result cannot establish Proton compatibility. A Steam Deck Desktop Mode result cannot establish Game Mode compatibility.
