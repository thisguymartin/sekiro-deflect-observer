# 0.10.0-preview UI validation

Reference: the user's two supplied 2026-09-17 screenshots and
https://www.youtube.com/watch?v=bejKX5vw2-Y. Direct video retrieval was unavailable;
the visual reconstruction is based on the supplied stills. The user explicitly
requested the upper placement. The LB badge follows the reference; L1/RMB can
be configured. The optional button-label question had no response during work.

## Visual behavior

- Top center at `(0.5, 0.16)` of the fitted playable viewport, 480x18 reference
  pixel lane, slim pointed ends, dark left trail, colored right segment.
- White diamond moves toward the center gate during the observed wind-up.
- Active parryable attack phase: white segment and red strike emblem. No contact,
  successful deflect, player button press, or input-window event is claimed.
- Other responses keep their hues; UNKNOWN uses dim gray and a caption.
- Reduced-flash suppresses the white/red transition. Neutral remains LOCKED.
- SVG exported from the same native vector geometry, not an extracted game/video
  asset. No raster or new GPU texture dependencies. The broad dark panel is gone.
- Legacy lower-posture mode remains supported, shifted to y=0.78 to clear the
  native posture region with the new below-rail caption.

## Validation

- 108 Rust tests passed (71 unit, 1 DX11 isolation, 10 incoming, 6 layout,
  2 lifecycle, 18 timing). New regression checks center arrival at activation
  and separate active-phase progress without contact/press/pulse claims.
- App Clippy with -D warnings passed. Two existing unused-method warnings in
  vendored hudhook remain. Root formatting checked before packaging.
- Shared renderer generated an eight-state gallery and contained every mesh
  vertex in its declared local bounds. Additional active-art checks passed at
  720p, 5120x1440 pillarboxed, 4K/scale 1.5, minimum scale, reduced-flash and
  optional lower-posture placement.
- Gallery: `docs/images/0.10.0-reference-gallery.png`; icon:
  `assets/ui/strike-emblem.svg`; raw mesh/atlas evidence under
  `dist/review-0.10.0/layout`.

These are synthetic rendering and native offscreen checks. Restart into this
candidate to verify native boss-HUD clearance, response transitions and color in
gameplay. No confirmed collision/outcome trigger has been added.

## Release and saved placement

- Offline MSVC release build, native DLL host rejection and isolated ASI loader
  smoke passed. All eight staged content hashes and DLL equality verified.
- Launcher: `Mods/SekiroDeflectObserver-0.10.0-preview/launch-observer.cmd`.
- DLL SHA256: `cc323c5f9d51e555fbfbb25f13943831d3b91029021d25da45f3f9625013daaf`.
- ZIP SHA256: `ed5ad43d161cdd741255189558ba18287f2a2d68fb945f64ef950d2904f44f02`.
- `scripts/use-reference-ui.ps1` was checked on a copied configuration: only
  anchor/width/offsets/button label changed, and the backup matched original bytes.
  Then it applied the user's requested top placement to the actual AppData file.
- Backup: `%LOCALAPPDATA%/SekiroDeflectObserver/cue.toml.before-reference-20260917-180603-7772439.bak`.
- No game graphics settings, installation or saves were changed. No game process
  was restarted, no new DLL was injected into a running game, and no commit or
  push was made. Earlier release folders and packages remain preserved.
