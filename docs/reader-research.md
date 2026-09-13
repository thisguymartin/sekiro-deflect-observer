# Candidate reader: 0.3.0-dev

This is a diagnostic implementation, not a validated deflect-window detector.
It observes candidate effect 105010 without writing gameplay memory. No Cheat
Engine installation or old prototype is required.

## Build and evidence

The user confirmed Sekiro launched and the 0.2.0-dev panel rendered on
2026-09-10. The local startup log `observer-18940.log` recorded executable SHA-256
`637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856`.
F8, combat, lifecycle, and performance were not reported as tested.
The old package remains in `dist` and `Mods/SekiroDeflectObserver`.
Its DLL SHA-256 is
`22a911ce8cc757c2f38e5cc6e04353f68fd9f58d87c45acdb3297ee4b78579ed`.

The 0.3.0-dev build passed formatting, Clippy with warnings denied, 17 unit
tests, release compilation, and the non-Sekiro DLL startup/host rejection test
on 2026-09-10. The first startup check in the restricted session could not
write the AppData log; the full build passed with the required access.
The new ZIP and every extracted file were checked against their SHA-256 lists.
ZIP SHA-256: `51d660e02806ae550d4687bb5a49563a736e7f81acf85d5d149f7999542f1f23`.
The extracted test package is in `Mods/SekiroDeflectObserver-0.3.0-research`.
No in-game test of this new reader has been performed or reported yet.

The research reader accepts only that exact hash. This is an identity gate,
not evidence that its layout or semantics have passed gameplay validation.
The offsets below are documented for PC 1.06; the relationship between that
layout and this local executable still needs local verification. No additional
hashes should be accepted solely on the basis of an identical version label.

## Source provenance

Sources were inspected as text, not executed or bundled. The implementation is
new Rust code using these documented layout facts:

- [ElaDiDu practice table, revision 328065da6dc3c5c9138cd31030791804e8e14e4e](https://github.com/ElaDiDu/Sekiro-Practice-CT/blob/328065da6dc3c5c9138cd31030791804e8e14e4e/Ela_Sekiro_Table.CT): WorldChrMan RVA `0x3d7a1e0`; local player at `+0x88`; effect manager at player `+0x11d0`; list head at manager `+0x8`; signed 32-bit ID at node `+0x58`; next pointer at `+0x70`; null terminates. See its "Print Active Effects" entry.
- [borgCode SekiroTool offsets, revision 189649781b00ba1f2fddb5d8bbfff7684ff5b647](https://github.com/borgCode/SekiroTool/blob/189649781b00ba1f2fddb5d8bbfff7684ff5b647/SekiroTool/Memory/Offsets.cs): independently agrees on the 1.06 WorldChrMan RVA, local-player slot, and effect-manager offset. Agreement is corroboration, not proof of independent discovery.
- [SekiroResurrection SpEffectParam reference](https://github.com/SekiroResurrection/modding-wiki/wiki/SpEffectParam): describes 105010 as "Just guard judgment". A parameter name is a research lead, not a measurement of contact acceptance or exact engine timing.

## Reader behavior

`reader.rs` checks the hash before following any version-sensitive pointer.
It reacquires WorldChrMan, its local-player slot, and that player's manager on
each sample. It traverses the complete list even if 105010 is found, rejecting
cycles, unreadable bytes, invalid pointers, and lists longer than 256 nodes.
Two complete traversals must agree on node addresses, IDs, and links; the owner
chain is checked between and after traversals.

The Windows adapter uses ReadProcessMemory on the current process. It never
constructs a Rust reference to game memory or calls a game function. A 10 ms
budget is checked around each read. This bounds subsequent work, not the OS
call's scheduling latency. Repeated checks cannot provide an atomic snapshot
or fully detect freed-and-reused pointers (ABA). Local-player slot ownership
has not yet been independently confirmed on the user's build.

The worker targets an 8 ms period using a monotonic clock. Actual gaps and read
durations are recorded; Windows scheduling may produce longer intervals.
The display expires observations 100 ms after read start. Errors replace the
previous state immediately. F8 hides drawing while sampling continues.

The overlay shows PRESENT, ABSENT, or UNKNOWN, plus the five latest observed
transitions. History is limited to 128 transitions per process. Long gaps and
player changes insert UNKNOWN boundaries. This is diagnostic history, not
completed-window durations, successful deflect counts, or input counts.

## Logs and testing

Each run writes `observer-PID.samples.csv` under
`%LOCALAPPDATA%\SekiroDeflectObserver`. Columns are read start/end in microseconds
since reader startup, state, error reason, local-player address, and observed
effect IDs separated by semicolons. Files have a 16 MiB limit per process; the
overlay reports when logging stops. Sampling continues at the cap. Buffers
flush every second, so an abrupt process exit can lose the last buffered rows.
Older runs are retained, so disk usage across runs is not globally bounded.

After launching the new research package, load a save and perform these trials:

1. Stand idle for five seconds.
2. Tap guard five times, with two seconds between taps.
3. Hold guard for five seconds, release, then repeat several rapid taps.
4. Against the same attack, compare a visible deflect, a held-guard block, and
   an unguarded hit. Record video with game audio to classify contact independently.
5. Return to title, reload, die/respawn, and toggle F8. Check normal controls.

Record observations using `tests/manual/gameplay-checklist.md`. A green flash
is not sufficient validation. If UNKNOWN persists, preserve the reason and
sample log; do not loosen pointer checks or guess alternative offsets.

## Remaining milestones

- Verify executable metadata/layout and local-player provenance on the local build.
- Perform the gameplay and lifecycle trials above; examine contradictory results.
- Only after that, decide whether the signal supports a deflect-window label.
- Add complete interval bounds and duration statistics once the observation
  semantics are supported. Do not average across missing samples or player changes.

Synthetic tests and a successful DLL build cannot complete these live trials.
