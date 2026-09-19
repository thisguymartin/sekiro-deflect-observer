# Configure the observer

The observer stores settings in
`%LOCALAPPDATA%\SekiroDeflectObserver\cue.toml`. It creates the file on first
use and checks it once per second.

Edit and save the file while the game runs to reload settings. If the file is
invalid, the observer keeps the last valid settings and shows the error in F9.
Delete `cue.toml` while the game is closed to restore every default.

The package contains a complete example at [`packaging/cue.toml`](../../packaging/cue.toml).

## Common settings

These settings cover the changes most players need:

```toml
anchor = "top"
offset_x = 0
offset_y = 0
scale = 1.0
width = 480
opacity = 0.95
parry_button = "LB"
reduced_flash = false
practice_speed = 0.9
```

Use F6 and F7 to change `offset_y`. Press F10 to reset both offsets. Press
Shift+F11 to cycle and save the 90%, 80%, 70%, and 60% practice presets.

## Settings reference

| Setting | Default | Accepted value |
| --- | ---: | --- |
| `incoming_cues` | `true` | `true` uses incoming attack labels. `false` uses legacy estimated timing. |
| `mikiri` | `true` | `false` shows PARRY for supported deflectable thrusts. |
| `practice_speed` | `0.9` | `0.5` through `1.0` enemy animation multiplier. |
| `anchor` | `top` | `top`, `posture`, or `overhead`. |
| `parry_button` | `LB` | `LB`, `L1`, or `RMB`. This changes the label only. |
| `offset_x` | `0` | `-480` through `480` reference pixels. |
| `offset_y` | `0` | `-160` through `160` reference pixels. |
| `scale` | `1.0` | `0.5` through `1.5`. |
| `width` | `480` | `160` through `640` reference pixels. |
| `opacity` | `0.95` | `0.2` through `1.0`. |
| `label_size` | `30` | `14` through `30` reference pixels. |
| `safe_margin` | `24` | `8` through `96` reference pixels. |
| `posture_band_top` | `0.85` | `0.75` through `0.98` of the viewport. |
| `posture_gap` | `12` | `8` through `64` reference pixels. |
| `outline_intensity` | `0.8` | `0.0` through `1.0`. |
| `glow_intensity` | `0.35` | `0.0` through `1.0`. |
| `pulse_intensity` | `0.6` | `0.0` through `1.0`. |
| `pulse_duration_ms` | `80` | `16` through `120` real milliseconds. |
| `reduced_flash` | `false` | Disables brightness and size pulses when `true`. |
| `parry`, `dodge`, `jump` | `true` | Hide selected response labels when `false`. |
| `visible` | `true` | Startup state for the gameplay HUD. |
| `diagnostics` | `false` | Startup state for the F9 panel. |
| `diagnostic_logging` | `true` | Enables bounded local diagnostic logs. |

The settings below apply only when `incoming_cues = false`:

| Setting | Default | Accepted value |
| --- | ---: | --- |
| `preparation_ms` | `650` | `350` through `1500` real milliseconds. |
| `display_latency_ms` | `0` | `0` through `120` real milliseconds. |
| `input_latency_ms` | `0` | `0` through `120`; the latency sum cannot exceed `150`. |
| `parry_lead_ms` | `150` | `25` through `150` animation milliseconds. |
| `dodge_lead_ms` | `300` | `25` through `300` animation milliseconds. |
| `jump_lead_ms` | `300` | `25` through `300` animation milliseconds. |

## Colors

Colors use `#RRGGBBAA`. The alpha byte must be at least `80`. The default colors
are:

```toml
[colors]
parry = "#2EF245F2"
dodge = "#FF661FFF"
jump = "#2ECCFFFF"
ready = "#E0E5EBFF"
expired = "#899099CC"
```

## Behavior to remember

- Practice always starts off. Only the selected speed persists.
- F8 saves visibility and disables practice when it hides the HUD.
- F9 saves diagnostics visibility.
- Response toggles affect alerts. They do not change practice eligibility.
- `reduced_flash = true` keeps labels and colors while removing the white rail
  and red emblem transition.
- The observer rejects the complete reload if any setting is unknown or invalid.

Calibration profiles are an advanced research feature. No contact calibrations
ship with the observer. See [calibration profile evidence](../research/calibration-profiles.md)
before adding one.
