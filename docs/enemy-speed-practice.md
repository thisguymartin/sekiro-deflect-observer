# Enemy speed practice (0.12.4 preview)

F11 toggles practice for the current game process. It **starts off every time**.
**Shift+F11** switches through **80%, 90% and 70% enemy speed**, saving the selected
level without changing on/off. At 90%, a remaining second of animation takes
approximately 1.11 real seconds; at 80%, approximately 1.25 real seconds; at
70%, approximately 1.43 real seconds. The cycle is 80% → 90% → 70% → 80%.
Changing the level during an attack restores the original speed before applying
the new multiplier, so reductions do not stack.
With a fresh lock on an eligible attack, `practice_speed = 0.8` multiplies that
enemy's existing animation speed by 80%. Wolf's speed and the global clock are
untouched. A full remaining second of animation takes approximately 1.25 real
seconds at this rate; a 400 ms remaining wind-up becomes approximately 500 ms.
Detection and application happen partway into the animation, so the total move
is not guaranteed to be exactly 25% longer.

The caption appends `80%` after a checked write/readback for that target.
The upper-right crescent-and-katana crest always shows on/off and the selected
speed, even without an enemy lock: gray `OFF 80%` means disabled, gold `ON 80%`
or `ON 90%` / `ON 70%` means armed, and jade means an applied override on a fresh
matching target. The crescent grows fuller from 90% to 80% to 70%. Amber `!` means
unavailable, paused, unsupported or pending cleanup. F9 gives the detailed
status. The displayed percentage is the selected level, even while off or waiting.
The crest follows the playable
viewport and HUD scale/opacity, independently of attack hints and rail offsets.
It hides while unfocused; reduced-flash mode removes its active glow. Turning
F11 off keeps a gray OFF indicator; F8 hides the entire HUD.
`PRACTICE` replaces the neutral LOCKED caption while armed and waiting. F9 shows
the session toggle, configured percentage and controller status. F11 again
disarms; F8 hiding also disarms. Focus loss releases the current speed override,
but leaves the session armed for returning to the game. Restarting starts off.
`practice_speed` accepts 0.5 through 1.0 inclusive; 1.0 performs no slowdown.

The current preview supports recognized PARRY, JUMP and MIKIRI wind-up/active
phases on the locked enemy. It retains slowdown between recognized combo hits
and releases it after the last active phase or an ineligible animation change.
DODGE grabs, NO PARRY, UNKNOWN, absent/stale animation and unfocused/hidden HUD
states do not request slowdown. As of 0.12.1, response toggles, the Mikiri hint
and incoming/legacy HUD mode affect alerts only; slowdown uses raw attack facts.
With attack hints disabled, `PRACTICE 80%` still reports an applied override.
Normal movement and recovery outside those phases are not intentionally slowed.
Other enemies run normally. Grab alerts remain available at normal speed.

The [September 18 gameplay recording](screenshots.md) shows PARRY 70%, the
active-phase emblem and gold/jade/amber moon states. It demonstrates the HUD
in combat; it does not measure the animation-rate change or verify every cleanup
path. Projectile flight, AI timers, sound and other independent systems are not
rescaled. Some otherwise classified attacks can contain such systems. Paired grabs are skipped
because independently changing one participant could desynchronize the action.
Mikiri/deflect reactions still require live checks. This does not widen Wolf's
deflect window or generate controller/keyboard input. It is not a contact
predictor. The HUD follows captured animation time rather than inventing a
longer press window.

## Implementation and evidence

See [feature boundaries](feature-boundaries.md) before changing classification,
presentation or speed policy. Practice does not consume HUD decisions. Its
Windows writer is private to the practice module; the shared reader is read-only.

The pinned primary SekiroTool implementation has `GetSpeed`/`SetSpeed` on the
target character's behavior module:

- [TargetService.cs, commit 1896497](https://github.com/borgCode/SekiroTool/blob/189649781b00ba1f2fddb5d8bbfff7684ff5b647/SekiroTool/Services/TargetService.cs)
- [Offsets.cs, same commit](https://github.com/borgCode/SekiroTool/blob/189649781b00ba1f2fddb5d8bbfff7684ff5b647/SekiroTool/Memory/Offsets.cs)

The control is a float at `ChrIns -> +0x1ff8 modules -> +0x28 ChrBehavior
-> +0xd00 AnimationSpeed`. The locally inspected SDT Cheat Engine table agrees.
These are evidence for the field, not proof that every move retimes correctly.
The exact executable SHA-256 gate is unchanged:
`637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856`.

The worker resolves the handle through the live character bucket and validates
player/actor identities, character ID, NPC resource/ID, animation/behavior/data
modules and health. It rejects Wolf and aliases of Wolf's behavior module. The
original speed must be finite and within 0.25..4.0. It multiplies the saved
original once and records a temporary lease. Repeated samples do not stack the
multiplier. On target switching, the old lease is resolved before a new write.

Cleanup checks the same owner and the last applied value before restoring the
exact original, including non-1.0 speeds. A changed/freed owner is never written
through its old pointer. Read/write failures retain pending cleanup and block
another lease; an external speed change pauses the session until F11 is cycled
off/on. The worker also attempts bounded cleanup on shutdown or unwind.

The Windows backend uses checked four-byte `WriteProcessMemory` and readback.
It does not patch executable instructions, change global time, call game
functions, edit saves or replace game files. The existing animation hook remains
an observer. Reads/rechecks/writes are **not an atomic engine transaction**;
pointer reuse and concurrent engine changes cannot be completely excluded.
Application/restoration run on the reader worker (8 ms requested interval), not
at an exact engine frame boundary. Scheduling/I/O stalls may delay cleanup.

`observer-<pid>.practice.csv` records bounded speed-state transitions with version,
DLL/game hashes, real timestamps, owner, original/applied values and multiplier.
It remains enabled even when general diagnostic logging is disabled, because it
is the audit trail for temporary gameplay writes (16 MiB cap). Render/alert CSVs
also include practice status/percentage. A checked write is not a measured
animation-rate change. Failed reads or writes are visible as `restore_pending`
or `speed_unavailable`; `external_change_paused` requires explicit rearming.

## Live check still needed

Use the separately packaged 0.12.4 preview and fully restart Sekiro. First verify
normal cues with F11 off, then enable on an ordinary sword enemy. Confirm `80%`
appears only during eligible attacks, Wolf stays normal, and movement/recovery
returns to normal. Compare the same move with practice off/on, then check:

1. F11 off during wind-up, F8 hide, focus loss and lock loss.
2. Target switch, enemy death, player death, reload and rest/restart.
3. Boss combos, sweeps and thrusts, including Mikiri/deflect reactions.
4. Grabs/unknowns keep normal speed and retain their existing alert labels.
5. Log original/applied values, cleanup status and measured animation delta per
   real second; note any external-change pause or apparent sound/physics mismatch.
6. Shift+F11 cycles 80% → 90% → 70% → 80% both while off and while armed;
   the selected speed survives restart, but practice restarts off. Confirm
   switching speeds during an attack restores the baseline before reapplying.

Retain the pushed 0.11.0 build as the comparison baseline. The new recording is
visual evidence only; these controlled comparisons and lifecycle checks remain open.
Both builds use the same local config file. If a 0.12.x hotkey save adds
`practice_speed`, remove that setting before running 0.11.0; its older strict
parser does not recognize the new key and otherwise falls back to defaults.
