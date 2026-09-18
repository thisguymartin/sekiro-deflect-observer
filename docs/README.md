# Documentation

Current source: **0.12.4-preview**. Start with the
[project README](../README.md) for the gameplay introduction and quick install.

## Play and configure

- [Gameplay screenshots and walkthrough](screenshots.md) — frames and a short
  animation from the September 18 recording, with timestamps and explanations.
- [Build and run on Windows](windows.md) — install, controls, removal and troubleshooting.
- [Configuration](configuration.md) — defaults, bounds, placement and reload behavior.
- [HUD behavior](cue-preview.md) — response labels, animation phases and practice status.
- [Enemy-speed practice](enemy-speed-practice.md) — F11, speed presets and limitations.
- [Incoming attack classification](incoming-attacks.md) — data sources and unresolved moves.

## Evidence and development

- [Current validation](validation-0.12.4.md) — recorded build checks and the new video evidence.
- [Compatibility status](compatibility.md) and [release testing](release-testing.md).
- [Architecture](architecture.md) and [feature ownership](feature-boundaries.md).
- [Legacy timing model](parry-cue.md) and [timing coverage](boss-move-coverage.md).
- [Gameplay checklist](../tests/manual/gameplay-checklist.md) and
  [compatibility reports](../tests/compatibility/README.md).

## Historical references

Versioned validation pages retain the results and limitations recorded for that
version. The [visual archive](screenshots.md#historical-synthetic-previews) keeps
earlier synthetic designs separate from current gameplay screenshots.

The [0.6.3 sharing guide](sharing-beta.md) and [Nexus listing draft](nexus-beta-draft.md)
describe the older drop-in package. The [Cheat Engine testing guide](testing.md)
describes an earlier external prototype. They are not current install instructions.

[Reader research](reader-research.md), [enemy reader research](enemy-reader-research.md),
[game-file analysis](game-file-analysis.md) and [reverse engineering](reverse-engineering.md)
document research evidence and open questions. Earlier decisions remain in the
[changelog](../CHANGELOG.md) and Git history.
