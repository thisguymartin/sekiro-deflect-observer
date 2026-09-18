# Gameplay capture record

The current README and gameplay walkthrough use frames from
`Sekiro 2026-09-18 15-29-02.mp4`.

| Property | Value |
| --- | --- |
| Duration | Approximately 64.09 seconds. |
| Video | 2560 by 720, H.264, approximately 29.84 frames per second. |
| Audio | AAC, 48 kHz stereo. Audio is not included in the GIF or stills. |
| Screenshot crop | `crop=1280:720:640:0`. |
| Still output | 1280 by 720 JPEG with no retouching or annotations. |
| GIF output | 640 by 360, 15 frames per second, 128-color palette. |

Original SHA-256:

```text
be58f3f4fb2252b1ae8dc4e58bdf39766219c92349e8c49f0039beabaa94d5f0
```

The original video remains outside the repository. The repository contains
selected stills and a short silent loop. Times are seek positions in the video,
not input or log timestamps.

## Extract a still

Set `$video` to the original file. Change the timestamp and output name for
another frame.

```powershell
$video = 'C:\Users\mpati\Videos\Captures\Sekiro 2026-09-18 15-29-02.mp4'
ffmpeg -n -ss 38.5 -i $video -frames:v 1 -vf 'crop=1280:720:640:0' -q:v 2 docs/images/gameplay-2026-09-18-hero.jpg
```

The loop uses:

```powershell
ffmpeg -n -ss 36.8 -i $video -t 3.6 -filter_complex 'fps=15,crop=1280:720:640:0,scale=640:360:flags=lanczos,split[a][b];[a]palettegen=max_colors=128:stats_mode=diff[p];[b][p]paletteuse=dither=none' -loop 0 docs/images/gameplay-2026-09-18-demo.gif
```

`-n` preserves existing assets.

## Evidence limits

The recording proves that the HUD drew during combat and changed through the
captured states. It does not identify the loaded DLL hash, measure animation
speed, align manual input with contact, or establish a successful deflect.

Synthetic historical images remain under [`docs/images/archive`](../images/archive/).
They are design references, not gameplay screenshots.
