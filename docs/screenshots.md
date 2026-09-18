# Gameplay screenshots and walkthrough

These images come from the author's **September 18, 2026 gameplay recording**,
`Sekiro 2026-09-18 15-29-02.mp4`. They replace synthetic renders as the main
visual introduction. The current source is 0.12.4-preview; the recording's 70%
moon matches that design, but no F9 version or loaded-DLL hash is shown.

## Watch the cue change

![Gameplay sequence showing the green wind-up rail, moving diamond, active-phase emblem and return to PRACTICE](images/gameplay-2026-09-18-demo.gif)

**00:36.800–00:40.400**, normal playback speed. The GIF is reduced to 640 × 360
at 15 fps and has no audio. Use the full-size stills below to read the labels.

## 1. Read the incoming attack

![Wolf facing an enemy with PARRY 70% on a green rail and a jade ON 70% moon](images/gameplay-2026-09-18-hero.jpg)

**00:38.500 — wind-up.** The green rail identifies a parryable phase, and the
white diamond moves toward the center as the enemy's attack progresses.
`LB` is the configured button badge. `PARRY 70%` and the jade moon show that the
practice controller reports an applied override on the locked target.

This is the README's main image: Wolf, the opponent and the actual overlay
appear together. No HUD elements have been added or repositioned.

## 2. Recognize the active phase

![White attack rail and red strike emblem above Wolf during the active parryable phase](images/gameplay-2026-09-18-active.jpg)

**00:39.500 — active phase.** The rail turns white and the red strike emblem
appears at the center. This marks the classified active attack phase. Neither
the emblem nor the combat sparks establish a successful deflect or an exact
button-press time. The observer does not press the button for you.

## 3. Distinguish armed from applied

![Neutral PRACTICE rail and a gold ON 70% moon while Wolf remains locked onto the enemy](images/gameplay-2026-09-18-practice-ready.jpg)

**00:40.000 — waiting.** The rail returns to `PRACTICE`, with no applied
percentage in its caption. The gold `ON 70%` moon says practice is armed with
70% selected; it does not mean slowdown is continuously applied. Eligible
attacks, ordinary movement and recovery can produce different practice states.

## 4. Check an attention state

![Green PARRY rail without an applied percentage, beside an amber ON 70% practice moon and exclamation mark](images/gameplay-2026-09-18-practice-attention.jpg)

**00:47.000 — attention.** The moon is amber with an exclamation mark, and the
rail says `PARRY` without an applied percentage. Attack hints and practice
status are separate. Open **F9** for the controller's detailed reason. The
recording does not show that panel, so the specific cause is unknown.

For all responses and controls, see [HUD behavior](cue-preview.md) and
[enemy-speed practice](enemy-speed-practice.md).

## What this recording establishes

The video shows the overlay drawing during combat, the changing diamond and
rail, the active-phase emblem, and gold/jade/amber practice states at 70%.
It adds live visual evidence to the existing synthetic previews.

It does not establish the exact loaded build, measured enemy animation rate,
unchanged player rate, hotkey persistence, successful restoration after every
lifecycle event, input-to-contact timing, or compatibility across enemies and
platforms. There is no controlled overlay-off/on scene-color comparison.
See [current validation](validation-0.12.4.md) for remaining checks.

## Capture and extraction details

| Property | Value |
| --- | --- |
| Original filename | `Sekiro 2026-09-18 15-29-02.mp4` |
| Duration | Approximately 64.09 seconds |
| Recorded video | 2560 × 720, H.264, approximately 29.84 fps average |
| Original audio | AAC, 48 kHz stereo; not included in the GIF or stills |
| Screenshot crop | `crop=1280:720:640:0` — center game viewport, removing black side margins |
| Still output | 1280 × 720 JPEG; original colors, no retouching or annotations |
| GIF output | Same crop, scaled to 640 × 360, 15 fps, 128-color palette |

Original SHA-256:
`be58f3f4fb2252b1ae8dc4e58bdf39766219c92349e8c49f0039beabaa94d5f0`.
The original video remains outside the repository. Only the selected stills and
short loop are included. Times are seek positions relative to the start of the
recording, not log or input timestamps.

To reproduce a still with FFmpeg, set `$video` to the original local file and
run this from the repository root. Change the timestamp and output name for the
other frames above:

```powershell
$video = 'C:\Users\mpati\Videos\Captures\Sekiro 2026-09-18 15-29-02.mp4'
ffmpeg -n -ss 38.5 -i $video -frames:v 1 -vf 'crop=1280:720:640:0' -q:v 2 docs/images/gameplay-2026-09-18-hero.jpg
```

The loop uses:

```powershell
ffmpeg -n -ss 36.8 -i $video -t 3.6 -filter_complex 'fps=15,crop=1280:720:640:0,scale=640:360:flags=lanczos,split[a][b];[a]palettegen=max_colors=128:stats_mode=diff[p];[b][p]paletteuse=dither=none' -loop 0 docs/images/gameplay-2026-09-18-demo.gif
```

`-n` preserves existing assets. Extract to another output name for a comparison.

## Historical synthetic previews

These are renderer-generated design references, **not gameplay screenshots**.
They remain available for understanding older validation records:

- [0.12.1 stacked HUD/practice gallery](images/archive/0.12.1-practice-gallery.png).
- [0.12.1 practice with attack hints disabled](images/archive/0.12.1-practice-no-hints.png).
- [0.12.2 wind-crest indicator](images/archive/0.12.2-practice-indicator.png), superseded by the moon.
- [0.12.3 moon with two presets](images/archive/0.12.3-practice-moon.png), before the 70% preset.

The [0.12.4 synthetic moon reference](images/0.12.4-practice-moon.png) remains
useful for comparing all three presets and status colors. It is supplementary
to the live images above.
