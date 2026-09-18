# Defensive cue configuration

The observer reads `%LOCALAPPDATA%/SekiroDeflectObserver/cue.toml` on its worker thread. Start with [`packaging/cue.toml`](../packaging/cue.toml). On first use, a missing file is created atomically from safe defaults. Missing settings use those defaults; unknown settings and invalid values reject the whole reload. An existing malformed file is never overwritten automatically. A rejected reload keeps the complete last valid configuration and reports a concise reason in diagnostics and the local log.

The worker checks the file once per second. F8 queues a persisted visibility change and F9 queues a persisted diagnostics change. F6 and F7 queue an 8-reference-pixel vertical placement adjustment and persist it from the worker. One focused F10 press resets `offset_x` and `offset_y` to zero. Delete `cue.toml` while the game is closed to reset every setting.

Writes use a temporary file in the same directory, flush it, and atomically replace `cue.toml`. A hotkey save checks the source fingerprint before preparing the temporary file and checks it again immediately before replacement. If either check sees an external change, the save reports a conflict, retains the external file, and removes its temporary file. This two-phase check narrows the race window but is not a filesystem transaction with a noncooperating editor: finish and save external edits before using configuration hotkeys, then allow the worker to reload them. Malformed external content is never replaced by a hotkey save.

## Settings

0.8.1 migrates the untouched 0.8.0 default layout tuple (posture anchor, zero
offsets, scale 1, width 240, label 20, band 0.90, gap 12) to its historical
320-wide, 30-pixel-label layout with band 0.85.
The migration uses the same atomic, conflict-checked save. Customized tuples and
unrelated timing, visibility and calibration settings are preserved.

| Setting | Default | Inclusive bounds / meaning |
| --- | ---: | --- |
| `incoming_cues` | true | Incoming response display; false selects legacy estimated timing |
| `mikiri` | true | Mikiri hint assumes unlocked skill; false shows PARRY for classified thrusts |
| `practice_speed` | 0.8 | 0.5..1.0 enemy animation multiplier; F11 enables for current session only |
| `anchor` | `top` | `top`, `posture` or `overhead`; existing explicit anchors are preserved |
| `parry_button` | `LB` | `LB`, `L1`, or `RMB`; label only, not binding detection |
| `offset_x` | 0 | -480..480 reference pixels |
| `offset_y` | 0 | -160..160 reference pixels |
| `scale` | 1.0 | 0.5..1.5 |
| `width` | 480 | 160..640 reference pixels |
| `opacity` | 0.95 | 0.2..1.0 |
| `label_size` | 30 | 14..30 reference pixels |
| `safe_margin` | 24 | 8..96 reference pixels |
| `posture_band_top` | 0.85 | 0.75..0.98 of the viewport |
| `posture_gap` | 12 | 8..64 reference pixels |
| `outline_intensity` | 0.8 | 0..1 |
| `glow_intensity` | 0.35 | 0..1 |
| `pulse_intensity` | 0.6 | 0..1 |
| `pulse_duration_ms` | 80 | 16..120 real milliseconds |
| `preparation_ms` | 650 | 350..1500 real milliseconds before the press interval |
| `display_latency_ms` | 0 | 0..120 real milliseconds |
| `input_latency_ms` | 0 | 0..120 real milliseconds; latency sum at most 150 |
| `parry_lead_ms` | 150 | 25..150 animation milliseconds |
| `dodge_lead_ms` | 300 | 25..300 animation milliseconds |
| `jump_lead_ms` | 300 | 25..300 animation milliseconds |
| `reduced_flash` | false | Boolean; disable brightness/size pulses when true |
| `parry`, `dodge`, `jump` | true each | Boolean response enables; cannot add eligibility |
| `visible` | true | Boolean gameplay HUD visibility; F8 persists changes |
| `diagnostics` | false | Boolean research-panel visibility; F9 persists changes |
| `diagnostic_logging` | true | Boolean bounded local logging |
| `profiles` | `[]` | No contact calibrations ship; compatible profiles only |

`reduced_flash = true` disables the white rail/red strike emblem transition,
retaining the response color, caption and marker. `parry`, `dodge`, and `jump`
restrict displayed responses. These toggles, `mikiri` and `incoming_cues` do not
affect practice eligibility. `visible`, `diagnostics`, and `diagnostic_logging`
set the startup state for F8, F9, and bounded local logging.

Practice always starts off and has no persisted enable flag. F11 toggles it;
Shift+F11 switches and saves the 80% / 90% / 70% speed presets without changing on/off
state. The crescent icon shows OFF/ON plus the selected percentage, including
without a target. F8 hiding disarms it. At 1.0 the controller performs no slowdown. General log
disable does not disable the bounded practice-write audit trail. See
[enemy speed controls and limitations](enemy-speed-practice.md).

Top mode places the center gate at `(0.5, 0.16)` of the fitted playable viewport.
Its complete art bounds stay below the upper 9.5% HUD band. This is configured
placement, not detection of the native UI. The optional lower posture anchor
uses `(0.5, 0.78)` to make room for the new caption beneath the rail.

Colors live under `[colors]` and use exactly `#RRGGBBAA`. The alpha byte must be at least `80`; label contrast is enforced independently from configurable fill opacity. The defaults are parry `#2EF245F2`, dodge `#FF661FFF`, jump `#2ECCFFFF`, ready `#E0E5EBFF`, and expired `#899099CC`.

Incoming mode ignores timing leads, latency, calibration profiles and timing
pulses. These settings apply only with `incoming_cues = false`. Mikiri and
NO PARRY have fixed yellow/red accents. Mikiri does not read skill unlocks.

## Calibration profiles

Profiles calibrate guidance for one exact checked-in parry, dodge, or jump phase. Unverified responses cannot have profiles. Profiles never change a defensive response, establish a missing capability, or bypass executable/data hash checks. Contact offsets are animation milliseconds relative to activation. `early_ms`, `late_ms`, and `preferred_ms` are real milliseconds before contact. The accepted interval requires `0 <= late_ms < early_ms <= 300`; `preferred_ms`, when present, must satisfy `late_ms < preferred_ms <= early_ms`. Contact bounds must be ordered and each lie within -500..500 animation milliseconds. At nominal 1x speed, the contact span must be narrower than `early_ms - late_ms` so the conservative press intersection is nonempty. A valid profile can still have an empty intersection at another observed playback rate; the runtime suppresses that phase rather than widening it.

The following is **synthetic syntax only**, with invented example trial numbers; it is not evidence and must not be used as a gameplay calibration. Replace every observation and evidence field with measured results before enabling a profile. Remove `profiles = []` before adding a real profile table:

```toml
[[profiles]]
model = 1010
animation = 3000
phase = 0
activation_s = 0.666666687
end_s = 0.800000012
response = "parry"
game_sha256 = "637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856"
data_sha256 = "3d0c108cb3412fa91f43da0f08a819cd12d792431fc027f0c132854dc0aded3d"
evidence = "SYNTHETIC EXAMPLE - replace with actual logs/video"
encounter = "training soldier"
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

`trials` must be nonzero and equal `successes + failures`. Evidence, encounter, and form must be nonempty. Stored evidence identifies the calibration source; the observer does not claim its contents were independently gameplay-validated.

The runtime cannot currently observe form-specific applicability. `form = "model-animation-phase"` states that the supplied evidence covers every runtime variant sharing that exact model, animation, and phase key. Other nonempty form names may be stored, but runtime guidance keeps those profiles inactive until that form can be observed; profiles are never borrowed across forms.
