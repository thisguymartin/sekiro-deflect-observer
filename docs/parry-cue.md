# Defensive cue timing (0.8.1-preview)

This page describes the legacy estimated timing mode. Since 0.9.0 the default
is [incoming attack responses](incoming-attacks.md), without contact prediction
or a press window. Select `incoming_cues = false` to use the model below.

The historical 0.8.1 request placed a cue above Wolf's bottom posture bar;
0.10.0 now defaults to the user's requested top-center reference design.
READY is preparation; PARRY NOW, DODGE and JUMP describe defensive responses.
Mikiri is not implemented: neither an exact eligible-thrust classifier nor a
validated player-capability observation is available. Enemy recovery does not
produce retaliation or attack-back guidance. No combat rules or inputs change.

## Evidence and reproduction

The regression suite records three synthetic cases that isolate defects before
their fixes:

- T1-A: c1010 / animation 3000 captured at animation 0.650 s, then drawn 20 ms
  later at 1x. Activation is 0.666666687 s. The old sampled clock could still show
  action; bounded projection sees 0.670 and expires it.
- T1-B: a hook capture at monotonic 0 ms reapplied by a poll at 49 ms must be
  stale at 98 ms. Re-stamping it as a new sample incorrectly extended its life.
- T1-C: a changed raw ring sequence with an unchanged animation clock must not
  count as progression. The sequence is not a validated new-attack signal.

Red/green outputs are under `dist/review-0.8.0`; regression sources are
[`tests/timing-regressions.rs`](../tests/timing-regressions.rs) and
[`src/cue.rs`](../src/cue.rs). These reproduce calculation defects, not a measured
miss or a successful gameplay deflect. The Ogre auxiliary-track regression,
current-batch selection, owner checks and ambiguous-track rejection remain.

## Clock and occurrence model

`src/timing.rs` is portable Rust with an injected monotonic `Duration`. An
occurrence belongs to the observed player instance, target animation module,
handle, model and animation ID. Owner/animation switches, clock rewinds, invalid
reads or gaps break continuity. The same animation ID after an observed rewind
is a new occurrence. A restart hidden entirely between reads cannot be identified
reliably; this limitation is not solved by the reader's sequence number.

The engine needs three progressing captures: two rate estimates within 20%,
with capture spacing 1..50 ms and playback rate 0.25..4 animation seconds per
real second. New stops or inconsistent rates suppress action until stable again.
Hook capture time is preserved; a re-delivered hook capture is not a new stop.
A new polling read of an unchanged clock suppresses action immediately and does
not refresh its source age. Polling fallback can consequently flicker or omit
cues between game updates; the completed-batch hook supplies distinct capture
identity without that ambiguity. Projection is at most 25 ms and only within the same
validated occurrence. At 50 ms, required observations expire. A wall timer
alone never advances an attack through a pause, stale read or animation switch.

## Separate intervals and units

All intervals in a decision use animation seconds. Endpoints are half-open:
`start <= animation_time < end`. Let `a` be activation, `r` the stable playback
rate and `L` display latency plus input latency in **real seconds**.

For uncalibrated phases, contact is explicitly the **activation proxy** `[a,a]`.
Default estimated press intervals are `[a - lead - r*L, a - r*L)`, where lead is
0.150 animation seconds for parry and 0.300 for dodge/jump. Configuration can
reduce these estimate leads. Subtracting the same latency shift from both
endpoints moves guidance earlier without widening it. These are not measured
contact or game acceptance windows.

A compatible measured profile stores contact offsets in animation milliseconds
relative to activation, giving `[contact_min,contact_max]`. Its press interval is
`[contact_max - r*early - r*L, contact_min - r*late - r*L)`, with early/late in real
seconds before contact. An empty intersection gives no guidance. A preferred
instant uses the contact midpoint minus the measured preferred lead and latency;
it is used only if it lies inside that valid interval. No profile ships enabled.

READY starts `preparation_ms` (default 650 real ms, converted using `r`) before
the press interval. Preparation, action and expiration have different labels
and shapes. Next-hit preparation/action wins over an older hit's recovery.
Every hit has its own phase ordinal within the occurrence.

A calibrated preferred instant can emit one pulse only on a witnessed crossing
inside the valid press interval. No first-frame catch-up pulse, full-interval
stall catch-up or pulse after expiration is allowed. The 80 ms default visual
envelope is clipped immediately at interval expiry. Reduced flash removes the
brightness/size pulse; the action label and shape remain. Activation-only moves
have no supported preferred instant, so do not emit a fabricated timing pulse.

## Safety and remaining evidence

Fresh validated lock identity has a separate 50 ms observation lifetime from
animation captures. Missing, invalid or stale animation data cancels timing and
pulses but retains a neutral LOCKED indicator while lock reads remain fresh.
Fresh animation progression must be re-established before action resumes.
READY may appear before coarse reach passes; action labels still require reach.
Unverified mapped phases show WATCH without a press interval or pulse.

Target lock, living player/enemy, ownership, current batch and fresh observations
remain required. Out-of-range, away-facing, cancelled, ambiguous and disabled
responses cannot be actionable. Reach uses approximate body bounds, distance,
height and facing; it is not weapon collision geometry. Motion or cancellation
between captures is unobserved until the next capture.

No validated menu/playability flag is available. Existing lock/player/death
checks and clock-stop suppression cover observed invalidation, but complete
menu/loading suppression still requires live evidence and possibly another
validated read. No guessed offset is added. Fixed HUD placement does not require
camera projection; overhead mode does. A known debug camera still invalidates play.

Profiles are bound to exact game/data hashes, model/animation/phase and boundary
values. Named-form profiles stay inactive because runtime form identity is not
observable; only explicit evidence covering the entire exact key may be applied.
See [configuration](configuration.md), [coverage](boss-move-coverage.md), and
[the trial template](../tests/compatibility/cue-trial-template.md).

Trials must distinguish estimated press, draw submission, visible presentation,
actual input, contact and independently confirmed outcome. The DLL observes
only the first two. A draw call, contact spark or candidate effect 105010 alone
is insufficient to label a successful deflect. All shipped moves remain estimated.
