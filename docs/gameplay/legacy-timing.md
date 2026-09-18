# Use legacy estimated timing

The default HUD identifies attack phases. Set `incoming_cues = false` only if
you want the older mode that estimates a defensive timing interval.

Legacy mode is less broadly useful than the default. It covers fewer attacks,
depends on reach and animation-rate estimates, and has no shipped contact
calibrations.

## Read the states

| State | Meaning |
| --- | --- |
| READY | A supported attack is approaching its estimated interval. |
| PARRY NOW | The estimated parry interval is active. |
| DODGE | The estimated dodge interval is active. No direction is selected. |
| JUMP | The estimated jump interval is active. |
| WATCH | The move is detected without an actionable interval. |
| EXPIRED | The estimated interval has passed. |

The mode does not support Mikiri guidance. It never generates input, changes
Sekiro's rules, or confirms that a defensive action succeeded.

## Understand the estimate

The timing engine tracks one continuous attack occurrence. A target change,
owner change, animation change, backwards animation clock, invalid read, or
large observation gap ends that occurrence.

The engine compares two clocks:

- Real monotonic time measures observation age and configured latency.
- Animation time tracks the enemy's move and scales with its measured playback
  rate.

The engine projects the captured animation clock forward by at most 25 real
milliseconds. It rejects unstable rates and observations older than 50
milliseconds.

Without a calibration profile, the configured lead defines the interval before
the attack phase activates. That activation is an animation event boundary. It
is not measured weapon contact.

## Know when guidance is suppressed

Legacy guidance requires:

- A supported executable and current generated data.
- A fresh lock and animation capture.
- A supported attack phase.
- Stable animation progression.
- A valid reach and facing estimate.
- An interval that remains nonempty after latency and calibration limits.

Missing or conflicting evidence removes the actionable state. The engine does
not extend an expired interval or reuse an older attack.

## Configure the mode

Set the mode and timing values in `cue.toml`:

```toml
incoming_cues = false
preparation_ms = 650
display_latency_ms = 0
input_latency_ms = 0
parry_lead_ms = 150
dodge_lead_ms = 300
jump_lead_ms = 300
```

Display and input latency move the interval earlier. They do not widen it. Read
[the configuration reference](../users/configuration.md) for accepted bounds.

No calibration profiles ship with the observer. A profile requires exact game
and data hashes, one model, animation and phase key, measured contact bounds,
trial counts, outcomes, and evidence links. Read
[calibration profile evidence](../research/calibration-profiles.md) before adding
one.

## Review the evidence

The legacy tables contain activation estimates, not validated press times. Their
coverage and unresolved phases are recorded in
[timing coverage](../research/timing-coverage.md). The regression suite is in
[`tests/timing-regressions.rs`](../../tests/timing-regressions.rs).

Gameplay trials must record animation capture, draw submission, visible
presentation, manual input, contact, and confirmed outcome separately. Use the
[cue trial template](../../tests/compatibility/cue-trial-template.md).
