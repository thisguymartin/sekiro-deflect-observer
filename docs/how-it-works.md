# How one attack becomes a cue

Sekiro Deflect Observer runs inside Sekiro as a Windows DLL. It reads the locked
enemy's animation, classifies the active attack phase, and asks ImGui to draw a
HUD during each DirectX 11 frame.

The shortest model is:

```text
game memory -> validated target -> attack phase -> HUD decision -> draw commands
                                      |
                                      +-> optional practice controller
```

The observer uses animation evidence. It does not observe collision, player
input, or the result of a deflect.

## Capture a completed animation batch

Sekiro processes animation events in batches. `src/event_hook.rs` observes a
completed batch before the game resets its bounds. The hook copies identifiers,
animation times, ownership, and a monotonic capture time. It then calls the
original game function.

The hook is available only for the supported executable and expected function
bytes. If the hook is unavailable, the observer can use bounded polling. Neither
path reports weapon contact.

## Build a target observation

The worker in `src/windows/diagnostics.rs` runs independently from rendering.
It reads the player, lock target, enemy identity, animation modules, health, and
optional camera data. `src/cue.rs` joins those reads into a `Target`.

Every observation carries its owner, capture time, and lifecycle generation.
Lock loss, target replacement, an animation change, or a read failure invalidates
old state. The renderer rejects stale observations instead of displaying an old
cue as if it were current.

## Classify the attack phase

`src/attack.rs` converts the target's model, behavior variation, animation, and
phase into a raw attack kind. The kinds are parryable, grab, sweep, thrust,
unparryable, and unknown.

Generated tables in `src/incoming_attacks.rs` contain the classification data.
The evidence for those rows lives in
[`research/data/incoming-coverage.json`](research/data/incoming-coverage.json).
The classifier preserves unknown and conflicting results instead of guessing.

## Decide what the HUD may show

`src/incoming.rs` applies display preferences to the raw attack kind. It chooses
labels such as PARRY, DODGE, JUMP, and MIKIRI. Turning off a label changes the
presentation, not the underlying attack fact.

The default incoming mode follows the captured wind-up and active phase. It does
not calculate an exact button-press interval. The optional legacy mode uses
`src/timing.rs` to estimate animation rate and a conservative interval from the
older timing tables.

`src/windows/cue_draw.rs` converts the decision into rail, marker, emblem, label,
and practice-status draw commands. `src/layout.rs` keeps those elements inside
the configured viewport and safe area.

## Keep practice separate from alerts

Practice mode consumes the same raw attack facts, but it does not consume HUD
labels. `src/practice.rs` decides whether the current locked target is eligible.
`src/windows/practice.rs` owns the only adapter that can write the enemy animation
speed field.

The controller stores the original value before applying a multiplier. It checks
ownership and the last applied value before restoration. A target change, focus
loss, HUD hide, ineligible animation, or explicit F11 toggle starts cleanup.
Pending cleanup blocks a new speed lease.

This separation allows a player to hide a PARRY hint while practice continues
to use the raw parryable phase. The HUD can report practice status, but it cannot
start or stop a speed write.

## Keep rendering off the read and write paths

The animation hook captures a batch. The worker reads memory, reloads settings,
and runs practice policy. The DX11 callback draws the latest valid snapshot.
These operations do not occur on one shared schedule.

The renderer holds shared state only long enough to produce a decision. It does
not traverse game memory or access files. The worker publishes state before it
handles slower configuration or logging work.

## Treat logs as observations

The observer writes bounded local CSV files for captures, render submissions,
alert transitions, animation events, practice writes, and effect research. A log
row records what that subsystem observed. It does not prove that the player saw
a frame, pressed a button, made contact, or completed a successful deflect.

Each CSV has a size limit. Logging failures never authorize a cue or a practice
write.

## Find the owning module

| Concern | Owner |
| --- | --- |
| Animation and target capture | `src/event_hook.rs`, `src/cue.rs` |
| Raw attack classification | `src/attack.rs` |
| Default alert policy | `src/incoming.rs` |
| Legacy interval estimates | `src/timing.rs` |
| Practice policy and restoration | `src/practice.rs` |
| Native speed writes | `src/windows/practice.rs` |
| HUD drawing | `src/windows/cue_draw.rs` |
| Layout | `src/layout.rs` |
| Configuration | `src/config.rs` |
| Lifecycle invalidation | `src/lifecycle.rs` |

Read [architecture and ownership](development/architecture.md) before changing a
boundary. Use [research and evidence](research/README.md) when a claim depends on
game data or live validation.
