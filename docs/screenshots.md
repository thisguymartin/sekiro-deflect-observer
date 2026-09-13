# Versioned visuals for the beta

## Current 0.6.3 design - offline preview

![0.6.3 offline rendering of READY, PARRY, DODGE and JUMP](images/0.6.3-offline-design.png)

This image is produced by the actual shared ImGui drawing code using synthetic
attack states. It shows the current filled diamond, needle, pointer, ribbon and
English action labels. It is not a screenshot of successful gameplay inputs.
No AI-generated replacement scene or promotional video thumbnail is used.

Reproduce:

```powershell
cargo run --locked --offline --example cue-layout --target x86_64-pc-windows-msvc -- dist/review-0.6.3/gallery --gallery
python scripts/render-cue-layout.py dist/review-0.6.3/gallery
```

The compact export retains a 16:9 camera/display. An initial tall-canvas attempt
was rejected by the projection checks; the export now reports empty geometry
explicitly instead of traversing an empty ImGui draw-list buffer.

## Earlier 0.6.0 gameplay - placement context

![Earlier 0.6.0 gameplay recording showing the neutral LOCKED bar above Wolf](images/0.6.0-gameplay-placement.jpg)

Unmodified frame image from the existing `gameplay-review-03.mp4` trial, previously
saved as `dist/game-analysis/gameplay-review-03-placement.jpg`. This demonstrates
the earlier overlay's in-game placement and neutral LOCKED state. It predates
the current visual design and does not verify 0.6.3 or drop-in startup.

Sekiro was closed when this gallery was prepared, so no new 0.6.3 live screenshot
was captured. Replace or supplement this historical image with a version-verified
gameplay capture after the next trial, retaining this provenance for reviewers.
