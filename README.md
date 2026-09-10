# Sekiro Deflect Observer

A planned training tool that shows the timing state created by the player's own deflect input in *Sekiro: Shadows Die Twice*.

**Experimental research. No playable release is included in this checkout.**

The project handover describes a Cheat Engine and Lua prototype targeting Sekiro PC 1.06 and candidate special effect `105010`. Its source, tests, and gameplay recordings were not present when these docs were written. The effect's meaning has not been verified here.

V1 observes the player's state. Incoming attack prediction belongs to a later V2. The intended tool does not automate inputs or change the deflect window.

## Start testing

Start with [Test the research prototype](docs/testing.md). It walks through preparation, a first gameplay session, and recording results.

- [Gameplay checklist](tests/manual/gameplay-checklist.md) covers taps, guard, combat, and game transitions.
- [Investigate effect 105010](docs/reverse-engineering.md) explains how to compare observations without assuming the effect means a deflect window.
- [Session report template](tests/compatibility/session-template.md) records the exact build, setup, evidence, and failures.
- [Automated test requirements](tests/README.md) defines the state and history contracts for implementation.
- [Compatibility status](docs/compatibility.md) lists the platforms awaiting validation.
- [Test a native release candidate](docs/release-testing.md) covers installation, removal, packaging, and release evidence.

## Intended design

```text
Sekiro local player state
  -> version-gated memory reader
  -> Unknown / Inactive / Active
  -> timing history
  -> local overlay
```

A failed or stale memory read must produce `Unknown`. Only a valid observation can produce `Inactive` or `Active`. The released observer is intended to operate without network requests.

## Development status

The current checkout contains testing documentation. There is no native runtime, selected loader, build command, automated test runner, release package, or validated installation procedure yet.

Preserve the original prototype and its tests under [prototypes/cheat-engine](prototypes/cheat-engine/README.md) before changing them. Fill in its actual startup, shutdown, and test commands there. Do not infer offsets from an executable's displayed version alone.

Windows is the initial target. Proton is a future experimental target. Neither has been validated in this checkout.
