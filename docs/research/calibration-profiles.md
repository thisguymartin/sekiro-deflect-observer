# Calibration profile evidence

Calibration profiles apply only to the optional legacy timing mode. No contact
calibrations ship with the observer.

A profile can narrow guidance for one exact model, animation, phase, executable,
and generated-data identity. It cannot change an attack response, create a
missing capability, or bypass a hash check.

## Required fields

```toml
[[profiles]]
model = 1010
animation = 3000
phase = 0
activation_s = 0.666666687
end_s = 0.800000012
response = "parry"
game_sha256 = "637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856"
data_sha256 = "e428d48ab2e7bb6e99e6687362506dd589b585c0bfdda130a9ec53b2911aa987"
evidence = "SYNTHETIC EXAMPLE - replace with stable log and video references"
encounter = "SYNTHETIC EXAMPLE - replace with the tested encounter"
form = "model-animation-phase"
contact_min_ms = -10.0
contact_max_ms = 20.0
early_ms = 100.0
late_ms = 50.0
preferred_ms = 75.0
trials = 3
successes = 2
failures = 1
```

The numbers above demonstrate syntax only. They are not gameplay evidence.

## Validation rules

- `trials` must be greater than zero and equal `successes + failures`.
- `evidence`, `encounter`, and `form` must not be empty.
- Contact bounds must be ordered and remain within `-500` through `500`
  animation milliseconds.
- The press interval requires `0 <= late_ms < early_ms <= 300`.
- `preferred_ms`, when present, must satisfy
  `late_ms < preferred_ms <= early_ms`.
- At nominal speed, the contact span must be narrower than the press interval.
- The executable and generated-data hashes must match at runtime.

The runtime cannot observe form-specific applicability. The value
`model-animation-phase` claims that the evidence covers every runtime variant
with the same key. Other form names remain inactive until the runtime can
observe that form.

A valid profile can still produce an empty conservative interval at another
animation rate. The runtime suppresses that phase instead of widening the
interval.

Use the [cue trial template](../../tests/compatibility/cue-trial-template.md) to
record the observations before adding a profile.
