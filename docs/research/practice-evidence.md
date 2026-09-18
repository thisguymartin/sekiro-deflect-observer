# Practice implementation and evidence

Practice mode temporarily writes the locked enemy's animation-speed field. It
starts off and requires an explicit F11 press each session.

## Memory field

The pinned SekiroTool implementation exposes `GetSpeed` and `SetSpeed` on the
target character's behavior module:

- [TargetService.cs at commit 1896497](https://github.com/borgCode/SekiroTool/blob/189649781b00ba1f2fddb5d8bbfff7684ff5b647/SekiroTool/Services/TargetService.cs)
- [Offsets.cs at commit 1896497](https://github.com/borgCode/SekiroTool/blob/189649781b00ba1f2fddb5d8bbfff7684ff5b647/SekiroTool/Memory/Offsets.cs)

The field is a float at `ChrIns -> +0x1ff8 modules -> +0x28 ChrBehavior ->
+0xd00 AnimationSpeed`. The supported executable SHA-256 is:

```text
637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856
```

This evidence identifies the field. It does not prove that every Sekiro move
retimes correctly.

## Ownership and restoration

The worker validates the player, target, character ID, NPC identity, animation
module, behavior module, data module, and health. It rejects Wolf and aliases of
Wolf's behavior module.

The controller accepts a finite original speed from `0.25` through `4.0`. It
stores that value before applying one multiplier. Repeated samples do not stack
the multiplier.

Before restoration, the controller checks the same owner and the last applied
value. It never writes through a released owner pointer. A read or write failure
keeps cleanup pending and blocks a new lease. An external speed change pauses
practice until the player cycles F11 off and on.

The Windows adapter uses checked four-byte `WriteProcessMemory` followed by a
readback. It does not patch executable instructions, edit saves, replace game
files, change the global clock, or generate input. Reads and writes are not an
atomic game-engine transaction.

## Audit log

`observer-<pid>.practice.csv` records controller transitions, timestamps, target
ownership, original and applied values, and the selected multiplier. The file
stops at 16 MiB. It remains enabled when general diagnostic logging is disabled.

A checked write proves that the process returned the requested float. It does
not prove the measured animation-rate change or a gameplay outcome.

## Live checks still required

Run these checks with a packaged 0.12.4-preview build and a complete game
restart:

1. Compare the same ordinary sword attack with practice off and on.
2. Confirm that Wolf remains at normal speed.
3. Disable practice during wind-up and verify restoration.
4. Check F8 hide, focus loss, lock loss, target switch, enemy death, player
   death, reload, rest, and restart.
5. Check boss combos, sweeps, thrusts, Mikiri reactions, and deflect reactions.
6. Confirm that grabs and unknown attacks remain at normal speed.
7. Cycle 80%, 90%, 70%, and 80% while off and while armed.
8. Measure animation progress per real second and retain the practice audit log.

The current [gameplay recording](../gameplay/walkthrough.md) shows practice HUD
states. It does not complete these controlled comparisons.
