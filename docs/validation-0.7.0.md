# 0.7.0 animation-event tracking

This is a partial implementation of the proposed event-driven architecture.
It implements the animation-batch hook and an ordered timeline-event tracker.
It does **not** implement runtime resolved attack-parameter selection, direct
AttackBehavior dispatch interception, weapon-contact prediction, or hooks for
deflect/block/damage outcomes. Existing response classifications are unchanged.

## Implemented

The native detour observes the completed batch before the engine advances
`ChrAnimModule + 0xec` to `+0xe8` at RVA `0xb5bef0`. The original function is
still invoked exactly once, with the same module argument and return value.
The loaded 171-byte function must hash to
`c8f27b1884982c7d4f6486251dd4f06b41a64e2e4a22bce946278f45d8cf909f`.
The executable must match the existing supported SHA-256 as well.

Provenance: `dist/review-0.6/animation-module-code.bin` begins at RVA 0xb5bc00;
its writer at offset 0x980 matches the separately captured writer at 0xb5c580.
The boundary function is bytes 0x2f0 through 0x39a inclusive. Disassembly shows
one module argument in RCX and writes to +0xf0, +0xec and +0xf8. The initially
considered consumer at 0xb5c730 has a conditional event-processing branch;
capturing before the boundary reset avoids depending on that branch.

Only an independently validated locked target is registered for capture. The
callback reads and rechecks 224 bytes, selects within the completed ten-slot
batch without allocation, and publishes using nonblocking try-locks. It does
not write logs, wait, or invoke additional game functions. A missed capture
expires after 50 ms. Target/module/handle changes, lost lock and death clear
ownership. Empty batches replace the prior attack with a neutral frame;
competing attack tracks suppress guidance rather than picking a guess.

Once installed, stale hook captures suppress guidance instead of falling back
to a potentially inconsistent poll. If installation fails its identity checks,
the ordinary polling preview remains available and the reason is logged.
The hook remains resident until process exit, like the overlay. Restart Sekiro
to switch builds; do not combine observer loading methods.

The existing generated TAE phase table supplies activation/deactivation times.
The event tracker logs each crossed time in order and records track changes,
restarts and ended/cancelled tracks. It does not replay historical crossings
on first observation or carry an interrupted countdown into a different move.
The reader consumes the latest capture, so missed intermediate batches remain
possible under stalls; these logs are not a lossless native event trace.

## Validation and live trial

Rust tests cover a real MinHook detour against an owned native fixture: capture
occurs before the fixture clears its batch, the original function runs and its
return is preserved, and a subsequent empty batch removes the old attack.
Additional tests cover expiry/ownership, competing tracks, ring wrap, ordered
combo crossings, repeated samples, restarts, target changes and cancellation.
These are local automated tests, not an in-game hook validation.

Start `Mods/SekiroDeflectObserver-0.7.0-preview/observer.me3` with Sekiro closed
and Steam running. Confirm 0.7.0 in F9, lock onto an enemy and check:

- Event hook is true and captured batches increase.
- Cue reader reports `event_batch` during valid capture.
- Lock changes, interruptions and death clear old attack timelines.
- `observer-<pid>.events.csv` appears under `%LOCALAPPDATA%/SekiroDeflectObserver`.

Record a short fight with F9 visible. `activation_time_crossed` describes a TAE
time crossing, not verified hitbox dispatch or contact. A reliable parry cue
still requires runtime behavior resolution and contact/result validation.
The game was not running when this build was implemented; live checks remain.

Release checks: 44 Rust tests passed, including the native detour fixture;
formatting, all-target Clippy with warnings denied and optimized Windows build
passed. The restricted startup check could not create its AppData log; rerunning
with approved access passed non-Sekiro-host rejection before any hook installation.
All extracted package manifest entries and the DLL matched the tested output.
ZIP SHA-256: `e616c48d9766d9f805c825fd9e63386f823d66b0c6a959fe0f2c06d95421acf2`.
