# 0.6.4 screen adaptation

September 16, 2026: the user's new-game trial loaded 0.6.2 successfully.
Session 15876 recorded 1,666 locked-target samples for model c1012, but all
842 target render submissions were `projection_rejected`. Two earlier 0.6.3
sessions also rejected every target render submission (532 and 18,069).
The renderer was active, and the target reader worked. A new save did not
prevent initialization or target detection.

GraphicsConfig.xml selected fullscreen 5120x1440. The previous projection
rejected any surface aspect differing from the camera aspect by more than
0.03. This is a strong explanation for the missing bar on this setup, but the
old logs did not record the camera aspect or which projection check failed.
Depth and offscreen-anchor rejection cannot be excluded from those logs alone.

The new projection fits a centered camera viewport into each frame's surface:
pillarboxing for wider screens, letterboxing for taller screens. Cue geometry
scales to the fitted viewport, including small windows. It preserves camera
proportions and does not stretch the world-space anchor across black bars.
F9 now reports the rendering result and surface/camera dimensions. New render
logs distinguish invalid projection input, depth rejection and offscreen anchors.

This assumes a centered, aspect-preserving game viewport. A mod that stretches,
crops or offsets the game view needs separate validation. The existing camera
reader's validation remains; narrow native camera lenses below 16:9 still need
their engine FOV conversion researched. A narrow display containing a valid
16:9 camera view is supported. Do not equate arbitrary surface-size support with
every possible camera mode or display modification.

Validation: 38 Rust tests pass, including eight surface sizes from 320x240 to
5120x1440, equal horizontal/vertical pixel scale, centered pillarboxing,
native ultrawide projection and invalid/offscreen rejection. Clippy and the
optimized Windows DLL build pass. The standalone DLL startup check rejects a
non-Sekiro host before installing hooks. Live alignment and successful-deflect
timing remain pending; the game was not running during this investigation.

Launch the extracted `Mods/SekiroDeflectObserver-0.6.4-preview/observer.me3`
with Sekiro fully closed and Steam running. Use only one observer loader.
The distributable is `dist/SekiroDeflectObserver-0.6.4-preview-windows-x64.zip`.
Press F9 to confirm 0.6.4, load the new save and lock onto an enemy.
If the bar is absent, F9's Overlay line now gives a specific placement reason.
The existing 0.6.3 drop-in ZIP is preserved and does not contain this fix.

The actual shared drawing code was rendered offline and visually inspected at
5120x1440 and 1080x1920 (`dist/review-0.6.4/ultrawide` and `portrait`). Both
showed the centered cue with readable labels and separate pointer geometry.
These use synthetic attack states and do not establish live placement accuracy.
All extracted package manifest checksums matched. ZIP SHA-256:
`c3e0a50b95c68ada7d44d191184a2b2b1077bacfbe3a275f61ac069613cddb76`.
