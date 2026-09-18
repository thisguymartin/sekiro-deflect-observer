# Research and evidence

This section records the evidence behind attack classification, timing tables,
memory reads, practice writes, and validation claims. These pages are for
developers and reviewers. They are not installation or gameplay instructions.

## Current evidence

- [Attack classification](attack-classification.md) explains how game parameters
  become PARRY, DODGE, JUMP, MIKIRI, NO PARRY, or UNKNOWN.
- [Timing coverage](timing-coverage.md) describes the optional legacy timing data.
- [Practice implementation and evidence](practice-evidence.md) documents the
  enemy-speed field, ownership rules, audit log, and open live tests.
- [Calibration profiles](calibration-profiles.md) defines the evidence required
  for a contact calibration.
- [Gameplay capture](gameplay-capture.md) records the source and limits of the
  current screenshots.

## Reader research

- [Enemy reader](reader/enemy-reader.md) records target, animation, and camera
  research.
- [Effect reader](reader/effect-reader.md) records the separate effect-105010
  investigation.
- [Game-file analysis](game-file-analysis.md) records extracted archive data.
- [Reverse engineering](reverse-engineering.md) defines the evidence method for
  effect research.

## Generated data

Files under [`data/`](data/) are generated evidence, not hand-edited reference
tables:

- `incoming-coverage.json` supports the default attack classifier.
- `enemy-coverage.json` and `response-coverage.json` support legacy timing.
- `boss-move-phases.csv` is the complete legacy phase ledger.

Run the matching generator with `--check` before changing a generated file.

## Validation history

Version-specific validation records live in [the archive](../archive/README.md).
An archived result applies only to the artifact and evidence named in that file.
